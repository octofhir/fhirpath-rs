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

//! FHIRPath CLI

use clap::Parser;
use fhirpath_cli::EmbeddedModelProvider;
use fhirpath_cli::cli::output::{EvaluationOutput, FormatterFactory, OutputMetadata, ParseOutput};
use fhirpath_cli::cli::{Cli, Commands};
use octofhir_fhirpath;
use octofhir_fhirpath::create_standard_registry;
use octofhir_fhirpath::evaluator::FhirPathEngine;
use octofhir_fhirpath::parser::{ParseResult, ParsingMode, parse_with_mode};
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::process;
use std::sync::Arc;
use std::time::Instant;

/// Create a shared EmbeddedModelProvider instance for all commands
async fn create_shared_model_provider() -> anyhow::Result<Arc<EmbeddedModelProvider>> {
    let provider = EmbeddedModelProvider::r4()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize FHIR schema: {}", e))?;
    Ok(Arc::new(provider))
}

/// Create FhirPathEngine with the shared model provider
async fn create_fhirpath_engine(
    model_provider: Arc<EmbeddedModelProvider>,
) -> anyhow::Result<FhirPathEngine> {
    let registry = Arc::new(create_standard_registry().await);
    let fhir_version = std::env::var("FHIRPATH_FHIR_VERSION").unwrap_or_else(|_| "r4".to_string());
    let engine = FhirPathEngine::new_with_fhir_version(registry, model_provider, &fhir_version)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create FhirPath engine: {}", e))?;
    Ok(engine)
}

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
fn fhirpath_error_to_ariadne(
    error: &octofhir_fhirpath::core::FhirPathError,
    span: std::ops::Range<usize>,
) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};

    AriadneDiagnostic {
        severity: DiagnosticSeverity::Error,
        error_code: error.error_code().clone(), // Use the actual error code from FhirPathError
        message: error.to_string(),
        span,
        help: None,
        note: None,
        related: Vec::new(),
    }
}

/// Convert parser Diagnostic to AriadneDiagnostic for proper span information
fn convert_diagnostic_to_ariadne(
    diagnostic: &octofhir_fhirpath::diagnostics::Diagnostic,
) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::diagnostics::AriadneDiagnostic;
    use std::ops::Range;

    // Convert location to span
    let span: Range<usize> = if let Some(location) = &diagnostic.location {
        location.offset..(location.offset + location.length)
    } else {
        0..0 // Fallback to zero span if no location
    };

    // Extract proper error code from diagnostic
    let error_code = parse_error_code(&diagnostic.code.code);

    AriadneDiagnostic {
        severity: diagnostic.severity.clone(),
        error_code,
        message: diagnostic.message.clone(),
        span,
        help: None,          // TODO: Extract from diagnostic when available
        note: None,          // TODO: Extract from diagnostic when available
        related: Vec::new(), // TODO: Convert related diagnostics
    }
}

#[tokio::main]
async fn main() {
    // Setup human-panic for better error messages
    human_panic::setup_panic!();

    let cli = Cli::parse();

    // Create shared model provider for all commands
    let shared_model_provider = match create_shared_model_provider().await {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("❌ Failed to initialize FHIR schema: {}", e);
            process::exit(1);
        }
    };

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
                &shared_model_provider,
            )
            .await;
        }
        Commands::Parse {
            ref expression,
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

            let merged_formatter_factory = FormatterFactory::new(merged_no_color);
            let merged_formatter = merged_formatter_factory.create_formatter(merged_format);

            handle_parse(expression, &merged_cli, &*merged_formatter);
        }
        Commands::Validate {
            ref expression,
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
                &shared_model_provider,
            )
            .await;
        }
        Commands::Docs { ref error_code } => {
            handle_docs(error_code, &cli);
        }
        Commands::Repl {
            ref input,
            ref variables,
            ref history_file,
            history_size,
        } => {
            handle_repl(
                input.as_deref(),
                variables,
                history_file.as_deref(),
                history_size,
                &cli,
            )
            .await;
        }
        Commands::Server {
            port,
            ref storage,
            ref host,
            cors_all,
            max_body_size,
            timeout,
            rate_limit,
            no_ui,
        } => {
            handle_server(
                port,
                storage.clone(),
                host.clone(),
                cors_all,
                max_body_size,
                timeout,
                rate_limit,
                no_ui,
                &cli,
            )
            .await;
        }
        Commands::Tui {
            ref input,
            ref variables,
            ref config,
            ref theme,
            no_mouse,
            no_syntax_highlighting,
            no_auto_completion,
            performance_monitoring,
            check_terminal,
        } => {
            handle_tui(
                input.as_deref(),
                variables,
                config.as_deref(),
                theme,
                no_mouse,
                no_syntax_highlighting,
                no_auto_completion,
                performance_monitoring,
                check_terminal,
                &cli,
            )
            .await;
        }
    }
}

async fn handle_evaluate(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    _pretty: bool,
    cli: &Cli,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    model_provider: &Arc<EmbeddedModelProvider>,
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

                handler
                    .report_diagnostic(&diagnostic, source_id, &mut stderr())
                    .unwrap_or_default();
                process::exit(1);
            }
        }
    };

    // Create FHIRPath engine with shared model provider
    let mut engine = match create_fhirpath_engine(model_provider.clone()).await {
        Ok(engine) => engine,
        Err(e) => {
            let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());

            // Create AriadneDiagnostic using proper error code from anyhow::Error
            let diagnostic = handler.create_diagnostic_from_error(
                octofhir_fhirpath::core::error_code::FP0001,
                format!("Failed to create FHIRPath engine: {}", e),
                0..expression.len(),
                None,
            );
            handler
                .report_diagnostic(&diagnostic, source_id, &mut stderr())
                .unwrap_or_default();
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
            let ariadne_diagnostics: Vec<_> = parse_result
                .diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler
                .report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr())
                .unwrap_or_default();
        }

        // Collect all error codes and positions from diagnostics
        let error_details: Vec<String> = parse_result
            .diagnostics
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
                let first_error_code = error_details[0]
                    .split(" at ")
                    .next()
                    .unwrap_or(&error_details[0]);
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
            Ok(fhir_path_value) => EvaluationOutput::from_fhir_path_value(
                fhir_path_value,
                expression.to_string(),
                execution_time,
            ),
            Err(e) => {
                // Report diagnostic for evaluation error
                let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
                let source_id =
                    handler.add_source("expression".to_string(), expression.to_string());

                // Create AriadneDiagnostic using proper error code from FhirPathError
                let diagnostic = fhirpath_error_to_ariadne(&e, 0..expression.len());
                if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
                    handler
                        .report_diagnostic(&diagnostic, source_id, &mut stderr())
                        .unwrap_or_default();
                }

                EvaluationOutput {
                    success: false,
                    result: None,
                    error: Some(e),
                    expression: expression.to_string(),
                    execution_time,
                    metadata: OutputMetadata::default(),
                }
            }
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

            handler
                .report_diagnostic(&diagnostic, source_id, &mut stderr())
                .unwrap_or_default();
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
            let ariadne_diagnostics: Vec<_> = parse_result
                .diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler
                .report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr())
                .unwrap_or_default();
        }

        // Collect all error codes and positions from diagnostics
        let error_details: Vec<String> = parse_result
            .diagnostics
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
                None,
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

            handler
                .report_diagnostic(&diagnostic, source_id, &mut stderr())
                .unwrap_or_default();
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
            let ariadne_diagnostics: Vec<_> = parse_result
                .diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler
                .report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr())
                .unwrap_or_default();
        }

        // Collect all error codes and positions from diagnostics
        let error_details: Vec<String> = parse_result
            .diagnostics
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
                let first_error_code = error_details[0]
                    .split(" at ")
                    .next()
                    .unwrap_or(&error_details[0]);
                parse_error_code(first_error_code)
            },
            &error_message,
            expression,
            None,
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

            handler
                .report_diagnostic(&diagnostic, source_id, &mut stderr())
                .unwrap_or_default();
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
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    // Always use comprehensive multi-error display
    handle_analyze_multi_error(expression, cli, formatter, model_provider).await;
}

async fn handle_repl(
    input: Option<&str>,
    variables: &[String],
    history_file: Option<&str>,
    history_size: usize,
    cli: &Cli,
) {
    use fhirpath_cli::cli::repl::{ReplConfig, start_repl};
    use serde_json::Value as JsonValue;
    use std::path::PathBuf;

    // Create FhirSchemaModelProvider
    let model_provider = match fhirpath_cli::EmbeddedModelProvider::r4().await {
        Ok(provider) => std::sync::Arc::new(provider),
        Err(e) => {
            eprintln!("Failed to create FHIR schema model provider: {}", e);
            return;
        }
    };

    // Parse initial variables
    let mut initial_variables = Vec::new();
    for var in variables {
        if let Some((name, value)) = var.split_once('=') {
            initial_variables.push((name.to_string(), value.to_string()));
        } else {
            eprintln!(
                "Warning: Invalid variable format '{}'. Expected name=value",
                var
            );
        }
    }

    // Load initial resource if provided
    let initial_resource = if let Some(input_path) = input {
        match std::fs::read_to_string(input_path) {
            Ok(content) => match serde_json::from_str::<JsonValue>(&content) {
                Ok(json) => Some(json),
                Err(e) => {
                    eprintln!("Warning: Failed to parse JSON from '{}': {}", input_path, e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read file '{}': {}", input_path, e);
                None
            }
        }
    } else {
        None
    };

    // Create REPL configuration
    let config = ReplConfig {
        prompt: "fhirpath> ".to_string(),
        history_size,
        auto_save_history: true,
        color_output: !cli.no_color,
        show_types: false,
        history_file: history_file.map(PathBuf::from),
    };

    // Start REPL
    if let Err(e) = start_repl(model_provider, config, initial_resource, initial_variables).await {
        eprintln!("REPL error: {}", e);
        std::process::exit(1);
    }
}

/// Handle analyze command with unified diagnostics system (like evaluate command)
async fn handle_analyze_multi_error(
    expression: &str,
    cli: &Cli,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    use fhirpath_cli::cli::diagnostics::CliDiagnosticHandler;
    use octofhir_fhirpath::analyzer::StaticAnalyzer;
    use std::io::{stderr, stdout};

    // Create diagnostic handler for unified error reporting
    let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    // First parse the expression with proper diagnostics (same as evaluate command)
    let parse_result = parse_with_mode(expression, ParsingMode::Analysis);

    let mut all_diagnostics: Vec<octofhir_fhirpath::diagnostics::AriadneDiagnostic> = Vec::new();

    // Collect parser diagnostics first
    if !parse_result.diagnostics.is_empty() {
        let parser_diagnostics: Vec<_> = parse_result
            .diagnostics
            .iter()
            .map(convert_diagnostic_to_ariadne)
            .collect();
        all_diagnostics.extend(parser_diagnostics);
    }

    if parse_result.success && parse_result.ast.is_some() {
        // Phase 2: Run static analysis with shared model provider (UNIFIED)
        let registry = std::sync::Arc::new(octofhir_fhirpath::create_standard_registry().await);
        let analyzer = StaticAnalyzer::new(registry, model_provider.clone());

        // Run the analysis on the parsed AST
        match analyzer.analyze(parse_result.ast.as_ref().unwrap()).await {
            Ok(analysis_result) => {
                // Add static analysis diagnostics (Ariadne diagnostics are already in the result)
                let mut static_diagnostics = analysis_result.ariadne_diagnostics.clone();

                // Fix missing spans by calculating them from expression text
                for diagnostic in &mut static_diagnostics {
                    if diagnostic.span == (0..0) {
                        if let Some(span) =
                            calculate_span_from_message(&diagnostic.message, expression)
                        {
                            diagnostic.span = span;
                        }
                    }
                }

                all_diagnostics.extend(static_diagnostics);
            }
            Err(e) => {
                // Convert analysis error to diagnostic
                let error_diagnostic = handler.create_diagnostic_from_error(
                    octofhir_fhirpath::core::error_code::FP0001,
                    format!("Static analysis failed: {}", e),
                    0..expression.len(),
                    None,
                );
                all_diagnostics.push(error_diagnostic);
            }
        }
    }

    // Sort diagnostics before deduplication to ensure consistent ordering
    all_diagnostics.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then(a.error_code.code.cmp(&b.error_code.code))
            .then(a.message.cmp(&b.message))
    });

    // Deduplicate diagnostics based on message, error code, and span
    all_diagnostics.dedup_by(|a, b| {
        a.message == b.message && a.error_code == b.error_code && a.span == b.span
    });

    // Report unified diagnostics (same pattern as evaluate command)
    if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
        handler
            .report_diagnostics(&all_diagnostics, source_id, &mut stderr())
            .unwrap_or_default();
    } else {
        // JSON format: report to stdout
        handler
            .report_diagnostics(&all_diagnostics, source_id, &mut stdout())
            .unwrap_or_default();
    }

    // Exit with error code if there were any errors
    let has_errors = all_diagnostics.iter().any(|d| {
        matches!(
            d.severity,
            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
        )
    });

    if has_errors {
        std::process::exit(1);
    }
}

/// Handle docs command - open documentation for error codes
fn handle_docs(error_code: &str, cli: &Cli) {
    use colored::Colorize;
    use octofhir_fhirpath::core::error_code::ErrorCode;
    use std::process::Command;

    // Parse error code - handle both "FP0001" and "1" formats
    let code_num = if error_code.starts_with("FP") || error_code.starts_with("fp") {
        error_code[2..].parse::<u16>()
    } else {
        error_code.parse::<u16>()
    };

    match code_num {
        Ok(num) => {
            let error_code_obj = ErrorCode::new(num);
            let error_info = error_code_obj.info();
            let docs_url = error_code_obj.docs_url();

            // Display error information in terminal in Rust-like style
            if cli.output_format != fhirpath_cli::cli::output::OutputFormat::Json {
                if !cli.no_color
                    && !std::env::var("NO_COLOR").is_ok()
                    && !std::env::var("FHIRPATH_NO_COLOR").is_ok()
                {
                    // Colored output (Rust-like style)
                    println!(
                        "{}: {}",
                        format!("error[{}]", error_code_obj.code_str()).red().bold(),
                        error_info.title.bold()
                    );

                    println!("\n{}", "Description:".cyan().bold());
                    println!("  {}", error_info.description);

                    println!("\n{}", "Help:".cyan().bold());
                    println!("  {}", error_info.help);

                    println!("\n{}", "Category:".cyan().bold());
                    println!("  {:?} errors", error_code_obj.category());

                    println!(
                        "\n{} {}",
                        "Online documentation:".green().bold(),
                        docs_url.underline().blue()
                    );
                } else {
                    // Non-colored output
                    println!("error[{}]: {}", error_code_obj.code_str(), error_info.title);

                    println!("\nDescription:");
                    println!("  {}", error_info.description);

                    println!("\nHelp:");
                    println!("  {}", error_info.help);

                    println!("\nCategory:");
                    println!("  {:?} errors", error_code_obj.category());

                    println!("\nOnline documentation: {}", docs_url);
                }

                // Ask if user wants to open browser
                println!("\nWould you like to open the online documentation? [y/N]");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    let input = input.trim().to_lowercase();
                    if input == "y" || input == "yes" {
                        open_browser(&docs_url);
                    }
                }
            } else {
                // JSON output format
                use serde_json::json;
                let json_output = json!({
                    "error_code": error_code_obj.code_str(),
                    "title": error_info.title,
                    "description": error_info.description,
                    "help": error_info.help,
                    "category": format!("{:?}", error_code_obj.category()),
                    "docs_url": docs_url
                });
                println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
            }
        }
        Err(_) => {
            if !cli.no_color
                && !std::env::var("NO_COLOR").is_ok()
                && !std::env::var("FHIRPATH_NO_COLOR").is_ok()
            {
                eprintln!(
                    "{}: Invalid error code format: '{}'",
                    "error".red().bold(),
                    error_code
                );
                eprintln!("{}: Expected format: FP0001 or 1", "help".cyan().bold());
            } else {
                eprintln!("error: Invalid error code format: '{}'", error_code);
                eprintln!("help: Expected format: FP0001 or 1");
            }
            process::exit(1);
        }
    }
}

fn open_browser(url: &str) {
    use std::process::Command;

    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "start", url]).status()
    } else {
        // Linux/Unix
        Command::new("xdg-open").arg(url).status()
    };

    match result {
        Ok(_) => {
            println!("Opened documentation in your default browser.");
        }
        Err(e) => {
            eprintln!("Failed to open browser: {}. Please visit: {}", e, url);
        }
    }
}

async fn handle_server(
    port: u16,
    storage: std::path::PathBuf,
    host: String,
    cors_all: bool,
    max_body_size: u64,
    timeout: u64,
    rate_limit: u32,
    no_ui: bool,
    _cli: &Cli,
) {
    use fhirpath_cli::cli::server::{config::ServerConfig, start_server};

    let config = ServerConfig {
        port,
        host: host.parse().unwrap_or([127, 0, 0, 1].into()),
        cors_all,
        max_body_size_mb: max_body_size,
        timeout_seconds: timeout,
        rate_limit_per_minute: rate_limit,
        trace_config: fhirpath_cli::cli::server::config::TraceConfig::Server, // Server mode for API
    };

    if let Err(e) = start_server(config).await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }
}

/// Calculate span from diagnostic message by finding tokens in expression
fn calculate_span_from_message(message: &str, expression: &str) -> Option<std::ops::Range<usize>> {
    // Extract resource type or property name from message
    if message.contains("Unknown resource type: '") {
        // Extract resource type name from message like "Unknown resource type: 'Pat'"
        let start_marker = "Unknown resource type: '";
        let end_marker = "'";
        if let Some(start) = message.find(start_marker) {
            let name_start = start + start_marker.len();
            if let Some(name_end) = message[name_start..].find(end_marker) {
                let resource_name = &message[name_start..name_start + name_end];
                // Find this resource name in the expression
                if let Some(pos) = expression.find(resource_name) {
                    return Some(pos..pos + resource_name.len());
                }
            }
        }
    } else if message.contains("Cannot validate property '") {
        // Extract property name from message like "Cannot validate property 'name' on unknown type"
        let start_marker = "Cannot validate property '";
        let end_marker = "'";
        if let Some(start) = message.find(start_marker) {
            let name_start = start + start_marker.len();
            if let Some(name_end) = message[name_start..].find(end_marker) {
                let property_name = &message[name_start..name_start + name_end];
                // Find this property name in the expression (usually after a dot)
                if let Some(pos) = expression.find(&format!(".{}", property_name)) {
                    return Some(pos + 1..pos + 1 + property_name.len());
                }
            }
        }
    }

    None
}

/// Handle the TUI command
async fn handle_tui(
    input: Option<&str>,
    variables: &[String],
    config_path: Option<&str>,
    theme: &str,
    no_mouse: bool,
    no_syntax_highlighting: bool,
    no_auto_completion: bool,
    performance_monitoring: bool,
    check_terminal: bool,
    _cli: &Cli,
) {
    use fhirpath_cli::tui::{TuiConfig, check_terminal_capabilities, start_tui};
    use serde_json::Value as JsonValue;

    // Check terminal capabilities if requested
    if check_terminal {
        match check_terminal_capabilities() {
            Ok(_) => {
                println!("✅ Terminal capabilities check passed");
                println!("   - Minimum size requirement met");
                println!("   - Color support available");
                return;
            }
            Err(e) => {
                eprintln!("❌ Terminal capabilities check failed: {}", e);
                eprintln!("   Consider using a larger terminal or different terminal emulator");
                process::exit(1);
            }
        }
    }

    // Create FhirSchemaModelProvider
    let model_provider = match fhirpath_cli::EmbeddedModelProvider::r4().await {
        Ok(provider) => std::sync::Arc::new(provider),
        Err(e) => {
            eprintln!("Failed to create FHIR schema model provider: {}", e);
            return;
        }
    };

    // Load configuration
    let mut config = if let Some(config_path) = config_path {
        match TuiConfig::load_from_file(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to load config from {}: {}", config_path, e);
                eprintln!("Using default configuration");
                TuiConfig::default()
            }
        }
    } else {
        match TuiConfig::load_with_fallbacks() {
            Ok(config) => config,
            Err(_) => TuiConfig::default(),
        }
    };

    // Apply command-line overrides
    if let Err(e) = config.set_theme(theme) {
        eprintln!("Warning: {}", e);
        eprintln!("Using default theme");
    }

    if no_mouse {
        config.set_feature("mouse_support", false).ok();
    }

    if no_syntax_highlighting {
        config.set_feature("syntax_highlighting", false).ok();
    }

    if no_auto_completion {
        config.set_feature("auto_completion", false).ok();
    }

    if performance_monitoring {
        config.set_feature("performance_monitoring", true).ok();
    }

    // Parse initial variables
    let mut initial_variables = Vec::new();
    for var in variables {
        if let Some((name, value)) = var.split_once('=') {
            initial_variables.push((name.to_string(), value.to_string()));
        } else {
            eprintln!(
                "Warning: Invalid variable format '{}', expected 'name=value'",
                var
            );
        }
    }

    // Load initial resource if provided
    let initial_resource = if let Some(input_path) = input {
        match load_resource_from_input(input_path) {
            Ok(resource) => Some(resource),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load resource from '{}': {}",
                    input_path, e
                );
                None
            }
        }
    } else {
        None
    };

    // Show startup information
    if !config.ui_preferences.show_performance_info {
        println!(
            "🎨 Starting FHIRPath TUI with {} theme",
            config.theme.metadata.name
        );
        if config.features.syntax_highlighting {
            println!("✨ Syntax highlighting enabled");
        }
        if config.features.auto_completion {
            println!("🔮 Auto-completion enabled");
        }
        if config.features.performance_monitoring {
            println!("📊 Performance monitoring enabled");
        }
        println!("Press F1 for help, Esc to quit\n");
    }

    // Start the TUI
    if let Err(e) = start_tui(model_provider, config, initial_resource, initial_variables).await {
        eprintln!("TUI error: {}", e);
        process::exit(1);
    }
}

/// Load a resource from input (file path or JSON string)
fn load_resource_from_input(input: &str) -> anyhow::Result<JsonValue> {
    use anyhow::Context;

    if input.starts_with('{') || input.starts_with('[') {
        // Input looks like JSON, try to parse directly
        serde_json::from_str(input).context("Failed to parse input as JSON")
    } else {
        // Input is likely a file path
        let content = std::fs::read_to_string(input).context("Failed to read input file")?;
        serde_json::from_str(&content).context("Failed to parse file content as JSON")
    }
}
