// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Simple CLI for FHIRPath evaluation
//!
//! A command-line interface for evaluating FHIRPath expressions against FHIR resources.

use clap::Parser;
use fhirpath_cli::FhirSchemaModelProvider;
use octofhir_fhirpath::evaluator::FhirPathEngine;
use octofhir_fhirpath::create_standard_registry;
use octofhir_fhirpath::parser::{ParsingMode, ParseResult, parse_with_mode};
use std::sync::Arc;
use fhirpath_cli::cli::output::{
    EvaluationOutput, FormatterFactory, OutputMetadata, ParseOutput,
};
use fhirpath_cli::cli::{Cli, Commands};
use octofhir_fhirpath;
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::process;
use std::time::Instant;

/// Merge global and subcommand output options, with subcommand taking precedence
fn merge_output_options(
    global: &fhirpath_cli::cli::Cli,
    subcommand_format: Option<fhirpath_cli::cli::output::OutputFormat>,
    subcommand_no_color: bool,
    subcommand_quiet: bool,
    subcommand_verbose: bool,
) -> (fhirpath_cli::cli::output::OutputFormat, bool, bool, bool) {
    let format = subcommand_format.unwrap_or(global.output_format.clone());
    let no_color = subcommand_no_color || global.no_color;
    let quiet = subcommand_quiet || global.quiet;
    let verbose = subcommand_verbose || global.verbose;
    
    (format, no_color, quiet, verbose)
}

/// Convert diagnostic code string to ErrorCode
fn parse_error_code(code_str: &str) -> octofhir_fhirpath::core::error_code::ErrorCode {
    use octofhir_fhirpath::core::error_code::*;
    
    match code_str {
        "FP0001" => FP0001,
        "FP0002" => FP0002,
        "FP0003" => FP0003,
        "FP0004" => FP0004,
        "FP0005" => FP0005,
        "FP0006" => FP0006,
        "FP0007" => FP0007,
        "FP0008" => FP0008,
        "FP0009" => FP0009,
        "FP0010" => FP0010,
        "FP0051" => FP0051,
        "FP0052" => FP0052,
        "FP0053" => FP0053,
        "FP0054" => FP0054,
        "FP0055" => FP0055,
        "FP0056" => FP0056,
        "FP0057" => FP0057,
        "FP0058" => FP0058,
        "FP0059" => FP0059,
        "FP0060" => FP0060,
        // Extract numeric code from string like "FP0123" or fallback to FP0001
        _ => {
            if code_str.starts_with("FP") && code_str.len() == 6 {
                if let Ok(num) = code_str[2..].parse::<u16>() {
                    ErrorCode::new(num)
                } else {
                    FP0001
                }
            } else if let Ok(num) = code_str.parse::<u16>() {
                ErrorCode::new(num)
            } else {
                FP0001 // Fallback to generic parse error
            }
        }
    }
}

/// Convert FhirPathError to AriadneDiagnostic with proper error code extraction
fn fhirpath_error_to_ariadne(error: &octofhir_fhirpath::core::FhirPathError, span: std::ops::Range<usize>) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};
    
    AriadneDiagnostic {
        severity: DiagnosticSeverity::Error,
        error_code: error.error_code().clone(),  // Use the actual error code from FhirPathError
        message: error.to_string(),
        span,
        help: None,
        note: None,
        related: Vec::new(),
    }
}

/// Convert parser Diagnostic to AriadneDiagnostic for proper span information
fn convert_diagnostic_to_ariadne(diagnostic: &octofhir_fhirpath::diagnostics::Diagnostic) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::diagnostics::AriadneDiagnostic;
    use std::ops::Range;
    
    // Convert location to span
    let span: Range<usize> = if let Some(location) = &diagnostic.location {
        location.offset..(location.offset + location.length)
    } else {
        0..0  // Fallback to zero span if no location
    };
    
    // Extract proper error code from diagnostic
    let error_code = parse_error_code(&diagnostic.code.code);
    
    AriadneDiagnostic {
        severity: diagnostic.severity.clone(),
        error_code,
        message: diagnostic.message.clone(),
        span,
        help: None,  // TODO: Extract from diagnostic when available
        note: None,  // TODO: Extract from diagnostic when available 
        related: Vec::new(),  // TODO: Convert related diagnostics
    }
}

/// Create FHIRPath engine with FhirSchemaModelProvider
async fn create_fhirpath_engine_with_schema_provider() -> octofhir_fhirpath::Result<FhirPathEngine> {
    let registry = create_standard_registry().await;
    let model_provider = Arc::new(FhirSchemaModelProvider::r4().await.map_err(|e| {
        octofhir_fhirpath::FhirPathError::model_error(
            octofhir_fhirpath::core::error_code::FP0001,
            format!("Failed to create FHIR R4 schema model provider: {}", e),
        )
    })?);
    
    Ok(FhirPathEngine::new(Arc::new(registry), model_provider))
}

#[tokio::main]
async fn main() {
    // Setup human-panic for better error messages
    human_panic::setup_panic!();

    let cli = Cli::parse();

    // Create formatter factory
    let formatter_factory = FormatterFactory::new(cli.no_color);
    let formatter = formatter_factory.create_formatter(cli.output_format.clone());

    match cli.command {
        Commands::Evaluate {
            ref expression,
            ref input,
            ref variables,
            pretty,
            ref output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let (merged_format, merged_no_color, merged_quiet, merged_verbose) =
                merge_output_options(&cli, output_format.clone(), no_color, quiet, verbose);
            
            // Create temporary CLI with merged options for this command
            let mut merged_cli = cli.clone();
            merged_cli.output_format = merged_format.clone();
            merged_cli.no_color = merged_no_color;
            merged_cli.quiet = merged_quiet;
            merged_cli.verbose = merged_verbose;
            
            // Create formatter with merged options
            let merged_formatter_factory = FormatterFactory::new(merged_no_color);
            let merged_formatter = merged_formatter_factory.create_formatter(merged_format);
            
            handle_evaluate(
                expression,
                input.as_deref(),
                variables,
                pretty,
                &merged_cli,
                &*merged_formatter,
            )
            .await;
        }
        Commands::Parse { 
            ref expression, 
            ref output_format, 
            no_color, 
            quiet, 
            verbose 
        } => {
            let (merged_format, merged_no_color, merged_quiet, merged_verbose) =
                merge_output_options(&cli, output_format.clone(), no_color, quiet, verbose);
            
            let mut merged_cli = cli.clone();
            merged_cli.output_format = merged_format.clone();
            merged_cli.no_color = merged_no_color;
            merged_cli.quiet = merged_quiet;
            merged_cli.verbose = merged_verbose;
            
            let merged_formatter_factory = FormatterFactory::new(merged_no_color);
            let merged_formatter = merged_formatter_factory.create_formatter(merged_format);
            
            handle_parse(expression, &merged_cli, &*merged_formatter);
        }
        Commands::Validate { 
            ref expression, 
            ref output_format, 
            no_color, 
            quiet, 
            verbose 
        } => {
            let (merged_format, merged_no_color, merged_quiet, merged_verbose) =
                merge_output_options(&cli, output_format.clone(), no_color, quiet, verbose);
            
            let mut merged_cli = cli.clone();
            merged_cli.output_format = merged_format.clone();
            merged_cli.no_color = merged_no_color;
            merged_cli.quiet = merged_quiet;
            merged_cli.verbose = merged_verbose;
            
            // Create formatter with merged options
            let merged_formatter_factory = FormatterFactory::new(merged_no_color);
            let merged_formatter = merged_formatter_factory.create_formatter(merged_format);
            
            handle_validate(expression, &merged_cli, &*merged_formatter);
        }
        Commands::Analyze {
            ref expression,
            ref variables,
            validate_only,
            no_inference,
            ref output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let (merged_format, merged_no_color, merged_quiet, merged_verbose) =
                merge_output_options(&cli, output_format.clone(), no_color, quiet, verbose);
            
            let mut merged_cli = cli.clone();
            merged_cli.output_format = merged_format.clone();
            merged_cli.no_color = merged_no_color;
            merged_cli.quiet = merged_quiet;
            merged_cli.verbose = merged_verbose;
            
            // Create formatter with merged options
            let merged_formatter_factory = FormatterFactory::new(merged_no_color);
            let merged_formatter = merged_formatter_factory.create_formatter(merged_format);
            
            handle_analyze(
                expression,
                variables,
                validate_only,
                no_inference,
                &merged_cli,
                &*merged_formatter,
            )
            .await;
        }
        Commands::Docs { ref error_code } => {
            handle_docs(error_code, &cli);
        } // Commands::Repl {
          //     ref input,
          //     ref variables,
          //     ref history_file,
          //     history_size,
          // } => {
          //     handle_repl(
          //         input.as_deref(),
          //         variables,
          //         history_file.as_deref(),
          //         history_size,
          //         &cli,
          //     )
          //     .await;
          // }
          // TODO: Re-enable server handling after fixing dependencies
          // Commands::Server {
          //     port,
          //     ref storage,
          //     ref host,
          //     cors_all,
          // } => {
          //     handle_server(port, storage.clone(), host.clone(), cors_all, &cli).await;
          // }
    }
}

async fn handle_evaluate(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    _pretty: bool,
    cli: &Cli,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
) {
    use fhirpath_cli::cli::diagnostics::CliDiagnosticHandler;
    use std::io::stderr;

    // Get resource data
    let resource_data = if let Some(input_str) = input {
        // Check if input is a file path or JSON string
        if input_str.starts_with('{') || input_str.starts_with('[') || input_str.trim().is_empty() {
            // Treat as JSON string directly
            input_str.to_string()
        } else {
            // Treat as file path
            match fs::read_to_string(input_str) {
                Ok(content) => content,
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("Error reading file {}: {}", input_str, e);
                    }
                    process::exit(1);
                }
            }
        }
    } else {
        // No input provided - read from stdin

        use std::io::{self, Read};
        let mut stdin_content = String::new();
        match io::stdin().read_to_string(&mut stdin_content) {
            Ok(_) => stdin_content,
            Err(e) => {
                if !cli.quiet {
                    eprintln!("Error reading from stdin: {}", e);
                }
                process::exit(1);
            }
        }
    };

    // Handle empty input case
    let resource: JsonValue = if resource_data.trim().is_empty() {
        // Use empty object for empty input
        parse_json("{}").unwrap_or_default()
    } else {
        // Parse JSON resource
        match parse_json(&resource_data) {
            Ok(json) => json,
            Err(e) => {
                let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
                let source_id = handler.add_source("resource".to_string(), resource_data.clone());
                
                let diagnostic = handler.create_diagnostic_from_error(
                    octofhir_fhirpath::core::error_code::FP0001,
                    format!("Invalid JSON resource: {}", e),
                    0..resource_data.len(),
                    Some("Ensure the resource is valid JSON format".to_string()),
                );
                
                handler.report_diagnostic(&diagnostic, source_id, &mut stderr()).unwrap_or_default();
                process::exit(1);
            }
        }
    };

    // Create FHIRPath engine with FhirSchemaModelProvider (R4 by default)
    let engine = match create_fhirpath_engine_with_schema_provider().await {
        Ok(engine) => engine,
        Err(e) => {
            let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());
            
            // Create AriadneDiagnostic using proper error code from FhirPathError
            let diagnostic = fhirpath_error_to_ariadne(&e, 0..expression.len());
            handler.report_diagnostic(&diagnostic, source_id, &mut stderr()).unwrap_or_default();
            process::exit(1);
        }
    };

    // Engine initialized successfully - now start timing for actual execution
    let start_time = Instant::now();

    // Parse initial variables from command line
    let mut initial_variables = std::collections::HashMap::new();
    for var_spec in variables {
        if let Some((name, value_str)) = var_spec.split_once('=') {
            // Try to parse value as JSON first
            let value = match parse_json::<JsonValue>(value_str) {
                Ok(json_value) => octofhir_fhirpath::FhirPathValue::resource(json_value),
                Err(_) => {
                    // If JSON parsing fails, treat as string
                    octofhir_fhirpath::FhirPathValue::String(value_str.to_string().into())
                }
            };
            initial_variables.insert(name.to_string(), value);
        } else {
            eprintln!(
                "⚠️ Invalid variable format {}, expected 'name=value'",
                var_spec
            );
        }
    }

    // Convert variables to correct HashMap type
    let variables: std::collections::HashMap<String, octofhir_fhirpath::FhirPathValue> =
        initial_variables.into_iter().collect();

    // Create evaluation context with the resource
    let context_collection =
        octofhir_fhirpath::Collection::single(octofhir_fhirpath::FhirPathValue::resource(resource));
    let mut eval_context = octofhir_fhirpath::EvaluationContext::new(context_collection);
    if !variables.is_empty() {
        for (name, value) in variables {
            eval_context.set_variable(name, value);
        }
    }

    // First parse the expression to get proper diagnostics with span information
    let parse_result = parse_with_mode(expression, ParsingMode::Analysis);
    
    let output = if !parse_result.success {
        // Parse failed - show detailed diagnostics with proper spans
        let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
        let source_id = handler.add_source("expression".to_string(), expression.to_string());
        
        // Report all diagnostics as a unified report (with proper spans)
        if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
            let ariadne_diagnostics: Vec<_> = parse_result.diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler.report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr()).unwrap_or_default();
        }
        
        // Collect all error codes and positions from diagnostics  
        let error_details: Vec<String> = parse_result.diagnostics
            .iter()
            .map(|d| {
                if let Some(location) = &d.location {
                    format!("{} at {}:{}", d.code.code, location.line, location.column)
                } else {
                    d.code.code.clone()
                }
            })
            .collect();
        
        let error_message = if error_details.is_empty() {
            "Parse failed".to_string()
        } else {
            format!("{}: Parse failed", error_details.join(", "))
        };
        
        // Create error with all collected error codes instead of using parse_result.into_result()
        let error = octofhir_fhirpath::core::FhirPathError::parse_error(
            if error_details.is_empty() { 
                octofhir_fhirpath::core::error_code::FP0001 
            } else {
                // Use first error code as the primary error for the ErrorCode type - extract just the code part
                let first_error_code = error_details[0].split(" at ").next().unwrap_or(&error_details[0]);
                parse_error_code(first_error_code)
            },
            &error_message,
            expression,
            None,
        );
        
        let execution_time = start_time.elapsed();
        EvaluationOutput {
            success: false,
            result: None,
            error: Some(error),
            expression: expression.to_string(),
            execution_time,
            metadata: OutputMetadata::default(),
        }
    } else {
        // Parse successful - now evaluate using the AST
        let ast = parse_result.ast.unwrap();
        let result = engine.evaluate_ast(&ast, &eval_context).await;
        
        let execution_time = start_time.elapsed();
        match result {
            Ok(collection) => EvaluationOutput {
                success: true,
                result: Some(collection),
                error: None,
                expression: expression.to_string(),
                execution_time,
                metadata: OutputMetadata {
                    cache_hits: 0, // TODO: Implement cache hit tracking
                    ast_nodes: 0,   // TODO: Track AST nodes
                    memory_used: 0, // TODO: Track memory usage
                },
            },
            Err(e) => {
                // Report diagnostic for evaluation error
                let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
                let source_id = handler.add_source("expression".to_string(), expression.to_string());
                
                // Create AriadneDiagnostic using proper error code from FhirPathError
                let diagnostic = fhirpath_error_to_ariadne(&e, 0..expression.len());
                if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
                    handler.report_diagnostic(&diagnostic, source_id, &mut stderr()).unwrap_or_default();
                }
                
                EvaluationOutput {
                    success: false,
                    result: None,
                    error: Some(e),
                    expression: expression.to_string(),
                    execution_time,
                    metadata: OutputMetadata::default(),
                }
            },
        }
    };

    match formatter.format_evaluation(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            // Create diagnostic handler for error reporting
            let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());
            
            let diagnostic = handler.create_diagnostic_from_error(
                octofhir_fhirpath::core::error_code::FP0001,
                format!("Error formatting output: {}", e),
                0..expression.len(),
                Some("Check output format configuration".to_string()),
            );
            
            handler.report_diagnostic(&diagnostic, source_id, &mut stderr()).unwrap_or_default();
            process::exit(1);
        }
    }
}

fn handle_parse(
    expression: &str,
    cli: &Cli,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
) {
    use fhirpath_cli::cli::diagnostics::CliDiagnosticHandler;
    use std::io::stderr;
    let parse_result = parse_with_mode(expression, ParsingMode::Analysis);
    
    let output = if parse_result.success {
        ParseOutput {
            success: true,
            ast: parse_result.ast,
            error: None,
            expression: expression.to_string(),
            metadata: OutputMetadata {
                cache_hits: 0,
                ast_nodes: 1, // TODO: Count AST nodes properly
                memory_used: 0,
            },
        }
    } else {
        // Report rich diagnostics with proper spans
        let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
        let source_id = handler.add_source("expression".to_string(), expression.to_string());
        
        // Report all diagnostics from the parser (with proper spans)
        if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
            let ariadne_diagnostics: Vec<_> = parse_result.diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler.report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr()).unwrap_or_default();
        }
        
        // Collect all error codes and positions from diagnostics
        let error_details: Vec<String> = parse_result.diagnostics
            .iter()
            .map(|d| {
                if let Some(location) = &d.location {
                    format!("{} at {}:{}", d.code.code, location.line, location.column)
                } else {
                    d.code.code.clone()
                }
            })
            .collect();
        
        let error_message = if error_details.is_empty() {
            "Parse failed".to_string()
        } else {
            format!("{}: Parse failed", error_details.join(", "))
        };
        
        // Convert first error to FhirPathError for output structure
        let error = parse_result.into_result().err().unwrap_or_else(|| {
            octofhir_fhirpath::core::FhirPathError::parse_error(
                octofhir_fhirpath::core::error_code::FP0001,
                &error_message,
                expression,
                None
            )
        });
        
        ParseOutput {
            success: false,
            ast: None,
            error: Some(error.into()),
            expression: expression.to_string(),
            metadata: OutputMetadata::default(),
        }
    };

    match formatter.format_parse(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            // Create diagnostic handler for error reporting
            let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());
            
            let diagnostic = handler.create_diagnostic_from_error(
                octofhir_fhirpath::core::error_code::FP0001,
                format!("Error formatting parse output: {}", e),
                0..expression.len(),
                Some("Check output format configuration".to_string()),
            );
            
            handler.report_diagnostic(&diagnostic, source_id, &mut stderr()).unwrap_or_default();
            process::exit(1);
        }
    }
}

fn handle_validate(
    expression: &str,
    cli: &Cli,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
) {
    use fhirpath_cli::cli::diagnostics::CliDiagnosticHandler;
    use std::io::stderr;
    // Validate is basically the same as parse but focuses on success/failure
    let parse_result = parse_with_mode(expression, ParsingMode::Analysis);
    
    let output = if parse_result.success {
        ParseOutput {
            success: true,
            ast: parse_result.ast,
            error: None,
            expression: expression.to_string(),
            metadata: OutputMetadata {
                cache_hits: 0,
                ast_nodes: 1,
                memory_used: 0,
            },
        }
    } else {
        // Report rich diagnostics with proper spans
        let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
        let source_id = handler.add_source("expression".to_string(), expression.to_string());
        
        // Report all diagnostics from the parser (with proper spans)
        if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
            let ariadne_diagnostics: Vec<_> = parse_result.diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler.report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr()).unwrap_or_default();
        }
        
        // Collect all error codes and positions from diagnostics
        let error_details: Vec<String> = parse_result.diagnostics
            .iter()
            .map(|d| {
                if let Some(location) = &d.location {
                    format!("{} at {}:{}", d.code.code, location.line, location.column)
                } else {
                    d.code.code.clone()
                }
            })
            .collect();
        
        let error_message = if error_details.is_empty() {
            "Validation failed".to_string()
        } else {
            format!("{}: Validation failed", error_details.join(", "))
        };
        
        // Create error with all collected error codes instead of using parse_result.into_result()
        let error = octofhir_fhirpath::core::FhirPathError::parse_error(
            if error_details.is_empty() { 
                octofhir_fhirpath::core::error_code::FP0001 
            } else {
                // Use first error code as the primary error for the ErrorCode type - extract just the code part
                let first_error_code = error_details[0].split(" at ").next().unwrap_or(&error_details[0]);
                parse_error_code(first_error_code)
            },
            &error_message,
            expression,
            None
        );
        
        ParseOutput {
            success: false,
            ast: None,
            error: Some(error.into()),
            expression: expression.to_string(),
            metadata: OutputMetadata::default(),
        }
    };

    match formatter.format_parse(&output) {
        Ok(formatted) => {
            println!("{}", formatted);
            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            // Create diagnostic handler for error reporting
            let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());
            
            let diagnostic = handler.create_diagnostic_from_error(
                octofhir_fhirpath::core::error_code::FP0001,
                format!("Error formatting validation output: {}", e),
                0..expression.len(),
                Some("Check output format configuration".to_string()),
            );
            
            handler.report_diagnostic(&diagnostic, source_id, &mut stderr()).unwrap_or_default();
            process::exit(1);
        }
    }
}

async fn handle_analyze(
    expression: &str,
    _variables: &[String],
    _validate_only: bool,
    _no_inference: bool,
    cli: &Cli,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
) {
    // Always use comprehensive multi-error display
    handle_analyze_multi_error(expression, cli, formatter).await;
}

// TODO: Re-enable after repl module is implemented
// async fn handle_repl(
//     input: Option<&str>,
//     variables: &[String],
//     history_file: Option<&str>,
//     history_size: usize,
//     cli: &Cli,
// ) {
//     // REPL functionality implementation pending
//     eprintln!("REPL functionality is currently disabled");
// }

/// Handle analyze command with comprehensive multi-error display (default mode)
async fn handle_analyze_multi_error(
    expression: &str,
    cli: &Cli,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
) {
    use fhirpath_cli::cli::{diagnostics::CliDiagnosticHandler, output::OutputFormat};
    use octofhir_fhirpath::parser::analysis_integration::ComprehensiveAnalyzer;
    use std::io::{stderr, stdout};

    // Create diagnostic handler for this analysis
    let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());

    // Add source for better error reporting
    let _source_id = handler.add_source("expression".to_string(), expression.to_string());

    handler
        .info(
            &format!("Analyzing FHIRPath expression: {}", expression),
            &mut std::io::stderr(),
        )
        .unwrap_or_default();

    // Show progress phases
    handler
        .show_analysis_progress("Phase 1: Parsing with error recovery", &mut stderr())
        .unwrap_or_default();

    // Run comprehensive analysis
    let mut analyzer = ComprehensiveAnalyzer::new();
    let analysis_result = analyzer.analyze(expression, "expression".to_string());

    handler
        .show_analysis_progress("Phase 2: Static analysis", &mut stderr())
        .unwrap_or_default();
    handler
        .show_analysis_progress("Phase 3: Generating diagnostics", &mut stderr())
        .unwrap_or_default();

    // Report all results with beautiful formatting
    if cli.output_format == OutputFormat::Json {
        // JSON output goes to stdout
        handler
            .report_analysis_result(&analysis_result, &mut stdout())
            .unwrap_or_default();
    } else {
        // Other formats show diagnostics on stderr
        handler
            .report_analysis_result(&analysis_result, &mut stderr())
            .unwrap_or_default();
    }

    // Show completion status
    handler
        .show_analysis_completion(&analysis_result.diagnostics.statistics, &mut stderr())
        .unwrap_or_default();

    // Exit with appropriate code
    if analysis_result.diagnostics.statistics.error_count > 0 {
        process::exit(1);
    }
}

/// Handle docs command - open documentation for error codes
fn handle_docs(error_code: &str, cli: &Cli) {
    use std::process::Command;
    use octofhir_fhirpath::core::error_code::ErrorCode;
    
    // Parse error code - handle both "FP0001" and "1" formats
    let code_num = if error_code.starts_with("FP") || error_code.starts_with("fp") {
        error_code[2..].parse::<u16>()
    } else {
        error_code.parse::<u16>()
    };
    
    match code_num {
        Ok(num) => {
            let error_code = ErrorCode::new(num);
            let url = error_code.docs_url();
            
            println!("Opening documentation for {}: {}", error_code.code_str(), url);
            
            // Try to open the URL in the user's default browser
            let result = if cfg!(target_os = "macos") {
                Command::new("open").arg(&url).status()
            } else if cfg!(target_os = "windows") {
                Command::new("cmd").args(&["/C", "start", &url]).status()
            } else {
                // Linux/Unix
                Command::new("xdg-open").arg(&url).status()
            };
            
            match result {
                Ok(_) => {
                    if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
                        println!("✅ Documentation opened in your default browser");
                    }
                }
                Err(e) => {
                    if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
                        println!("❌ Failed to open browser: {}", e);
                        println!("You can manually visit: {}", url);
                    } else {
                        eprintln!("{{\"error\": \"Failed to open browser\", \"url\": \"{}\"}}", url);
                    }
                    process::exit(1);
                }
            }
        }
        Err(_) => {
            if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
                println!("❌ Invalid error code format: {}", error_code);
                println!("Expected formats: FP0001, fp0055, 1, 55");
            } else {
                eprintln!("{{\"error\": \"Invalid error code format\", \"provided\": \"{}\"}}", error_code);
            }
            process::exit(1);
        }
    }
}
