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
use fhirpath_cli::cli::{Cli, Commands, RegistryCommands, RegistryShowTarget, RegistryTarget};
use octofhir_fhir_model::HttpTerminologyProvider;
use octofhir_fhir_model::provider::FhirVersion;
use octofhir_fhirpath::core::trace::create_cli_provider;
use octofhir_fhirpath::evaluator::FhirPathEngine;
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use octofhir_fhirpath::{self, create_function_registry};
use octofhir_fhirschema::create_validation_provider_from_embedded;
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::process;
use std::sync::Arc;
use std::time::Instant;

/// Create a shared EmbeddedModelProvider instance for all commands
async fn create_shared_model_provider() -> anyhow::Result<Arc<EmbeddedModelProvider>> {
    let provider = EmbeddedModelProvider::new(FhirVersion::R4);
    Ok(Arc::new(provider))
}

/// Create FhirPathEngine with the shared model provider
async fn create_fhirpath_engine(
    model_provider: Arc<EmbeddedModelProvider>,
) -> anyhow::Result<FhirPathEngine> {
    let registry = Arc::new(create_function_registry());
    let engine = FhirPathEngine::new(registry, model_provider.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create FhirPath engine: {}", e))?;

    let trace_provider = create_cli_provider();
    let mut engine = engine.with_trace_provider(trace_provider);

    // Create and wire in the FhirSchemaValidationProvider from the existing EmbeddedModelProvider
    // This reuses the already initialized provider and provides out-of-the-box support for conformsTo()
    if let Ok(validation_provider) = create_validation_provider_from_embedded(
        model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>,
    )
    .await
    {
        engine = engine.with_validation_provider(validation_provider);
    }

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
            eprintln!("❌ Failed to initialize FHIR schema: {e}");
            process::exit(1);
        }
    };

    // Create formatter factory
    let formatter_factory = FormatterFactory::new(cli.no_color);
    let _formatter = formatter_factory.create_formatter(cli.output_format.clone());

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
            analyze,
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
                analyze,
                &merged_cli,
                &*merged_formatter,
                &shared_model_provider,
            )
            .await;
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

            handle_validate(
                expression,
                &merged_cli,
                &*merged_formatter,
                &shared_model_provider,
            )
            .await;
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
        Commands::Registry { ref command } => {
            handle_registry(command, &cli).await;
        }
        Commands::Server {
            port,
            ref storage,
            ref host,
            cors_all,
            max_body_size,
            timeout,
            rate_limit,
        } => {
            handle_server(
                port,
                storage.clone(),
                host.clone(),
                cors_all,
                max_body_size,
                timeout,
                rate_limit,
                &cli,
            )
            .await;
        } // Commands::Tui {
          //     ref input,
          //     ref variables,
          //     ref config,
          //     ref theme,
          //     no_mouse,
          //     no_syntax_highlighting,
          //     no_auto_completion,
          //     performance_monitoring,
          //     check_terminal,
          // } => {
          //     handle_tui(
          //         input.as_deref(),
          //         variables,
          //         config.as_deref(),
          //         theme,
          //         no_mouse,
          //         no_syntax_highlighting,
          //         no_auto_completion,
          //         performance_monitoring,
          //         check_terminal,
          //         &cli,
          //     )
          //     .await;
          // }
    }
}

async fn handle_evaluate(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    _pretty: bool,
    analyze: bool,
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
                        eprintln!("Error reading file {input_str}: {e}");
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
                    eprintln!("Error reading from stdin: {e}");
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
                    format!("Invalid JSON resource: {e}"),
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
                format!("Failed to create FHIRPath engine: {e}"),
                0..expression.len(),
                None,
            );
            handler
                .report_diagnostic(&diagnostic, source_id, &mut stderr())
                .unwrap_or_default();
            process::exit(1);
        }
    };

    // Attach default terminology provider (tx.fhir.org) matching model provider's FHIR version
    let tx_path = match model_provider.get_fhir_version().await {
        Ok(octofhir_fhir_model::provider::FhirVersion::R4) => "r4",
        Ok(octofhir_fhir_model::provider::FhirVersion::R4B) => "r4b",
        Ok(octofhir_fhir_model::provider::FhirVersion::R5) => "r5",
        Ok(octofhir_fhir_model::provider::FhirVersion::R6) => "r6",
        _ => "r4",
    };
    let tx_url = format!("https://tx.fhir.org/{tx_path}");
    if let Ok(tx) = HttpTerminologyProvider::new(tx_url) {
        let tx_arc: Arc<dyn octofhir_fhir_model::terminology::TerminologyProvider> = Arc::new(tx);
        engine = engine.with_terminology_provider(tx_arc);
    }

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
                    octofhir_fhirpath::FhirPathValue::string(value_str.to_string())
                }
            };
            initial_variables.insert(name.to_string(), value);
        } else {
            eprintln!("⚠️ Invalid variable format {var_spec}, expected 'name=value'");
        }
    }

    // Convert variables to correct HashMap type
    let variables: std::collections::HashMap<String, octofhir_fhirpath::FhirPathValue> =
        initial_variables.into_iter().collect();

    // Create Collection with proper resource typing using ModelProvider
    let model_provider_arc =
        model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>;

    let context_collection = match octofhir_fhirpath::Collection::from_json_resource(
        resource.clone(),
        Some(model_provider_arc.clone()),
    )
    .await
    {
        Ok(collection) => collection,
        Err(e) => {
            eprintln!("⚠️ Warning: Failed to properly type resource, using fallback: {e}");
            octofhir_fhirpath::Collection::single(octofhir_fhirpath::FhirPathValue::resource(
                resource,
            ))
        }
    };
    let eval_context = octofhir_fhirpath::EvaluationContext::new(
        context_collection,
        model_provider_arc,
        engine.get_terminology_provider(),
        engine.get_validation_provider(),
        None, // trace_provider
    )
    .await;

    // Add variables if provided - support multiple variables
    if !variables.is_empty() {
        for (name, value) in variables {
            eval_context.set_variable(name.to_string(), value);
        }
    }

    // First parse the expression to get proper diagnostics with span information
    let parse_result = parse_with_mode(expression, ParsingMode::Fast);

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
            result_with_metadata: None,
            error: Some(error),
            expression: expression.to_string(),
            execution_time,
            metadata: OutputMetadata::default(),
        }
    } else {
        // Parse successful - now evaluate with metadata for rich debugging info
        let result = engine
            .evaluate_with_metadata(expression, &eval_context)
            .await;

        let execution_time = start_time.elapsed();
        match result {
            Ok(eval_result_with_metadata) => {
                // Convert Collection to CollectionWithMetadata for rich CLI output
                let collection_with_metadata =
                    octofhir_fhirpath::core::CollectionWithMetadata::from(
                        eval_result_with_metadata.result.value.clone(),
                    );

                EvaluationOutput {
                    success: true,
                    result: Some(eval_result_with_metadata.result.value.clone()),
                    result_with_metadata: Some(collection_with_metadata),
                    error: None,
                    expression: expression.to_string(),
                    execution_time,
                    metadata: OutputMetadata::default(),
                }
            }
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
                    result_with_metadata: None,
                    error: Some(e),
                    expression: expression.to_string(),
                    execution_time,
                    metadata: OutputMetadata::default(),
                }
            }
        }
    };

    // If analyze flag is set, run static analysis alongside evaluation
    if analyze {
        use octofhir_fhir_model::TypeInfo;
        use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};

        let mut analyzer = StaticAnalyzer::new(
            model_provider.clone() as Arc<dyn octofhir_fhir_model::ModelProvider>
        );

        // Extract resource type from expression or use default
        let inferred_type =
            extract_resource_type_from_expression(expression).unwrap_or_else(|| TypeInfo {
                type_name: "Resource".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some("Resource".to_string()),
            });

        let context = AnalysisContext::new(inferred_type)
            .with_deep_analysis()
            .with_optimization_suggestions(true)
            .with_max_suggestions(5);

        // Run static analysis
        let analysis_result = analyzer.analyze_expression(expression, context).await;

        // Show analysis results if verbose or if there are significant issues
        if cli.verbose
            || analysis_result.statistics.errors_found > 0
            || !analysis_result.suggestions.is_empty()
        {
            eprintln!("🔍 Static Analysis Results:");
            eprintln!(
                "   Errors: {}, Warnings: {}",
                analysis_result.statistics.errors_found, analysis_result.statistics.warnings_found
            );

            if !analysis_result.suggestions.is_empty() {
                eprintln!("💡 Suggestions:");
                for suggestion in &analysis_result.suggestions {
                    eprintln!("   {}: {}", suggestion.suggestion_type, suggestion.message);
                }
            }

            // Report diagnostics if there are any
            if !analysis_result.diagnostics.is_empty() {
                let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
                let source_id =
                    handler.add_source("expression".to_string(), expression.to_string());

                handler
                    .report_diagnostics(&analysis_result.diagnostics, source_id, &mut stderr())
                    .unwrap_or_default();
            }
        }
    }

    match formatter.format_evaluation(&output) {
        Ok(formatted) => {
            println!("{formatted}");
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
                format!("Error formatting output: {e}"),
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

#[allow(dead_code)]
fn handle_parse(
    expression: &str,
    cli: &Cli,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
) {
    use fhirpath_cli::cli::diagnostics::CliDiagnosticHandler;
    use std::io::stderr;
    let parse_result = parse_with_mode(expression, ParsingMode::Fast);

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
            error: Some(error),
            expression: expression.to_string(),
            metadata: OutputMetadata::default(),
        }
    };

    match formatter.format_parse(&output) {
        Ok(formatted) => {
            println!("{formatted}");
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
                format!("Error formatting parse output: {e}"),
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

async fn handle_validate(
    expression: &str,
    cli: &Cli,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    use fhirpath_cli::cli::diagnostics::CliDiagnosticHandler;
    use octofhir_fhir_model::TypeInfo;
    use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};
    use std::io::stderr;

    // Create diagnostic handler for unified error reporting
    let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    // First parse the expression
    let parse_result = parse_with_mode(expression, ParsingMode::Fast);

    let mut all_diagnostics: Vec<octofhir_fhirpath::diagnostics::AriadneDiagnostic> = Vec::new();
    let mut has_errors = false;

    // Collect parser diagnostics first
    if !parse_result.diagnostics.is_empty() {
        let parser_diagnostics: Vec<_> = parse_result
            .diagnostics
            .iter()
            .map(convert_diagnostic_to_ariadne)
            .collect();

        has_errors = parse_result.diagnostics.iter().any(|d| {
            matches!(
                d.severity,
                octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
            )
        });

        all_diagnostics.extend(parser_diagnostics);
    }

    // If parsing succeeded, run static analysis for enhanced validation
    if parse_result.success && parse_result.ast.is_some() {
        let mut analyzer = StaticAnalyzer::new(
            model_provider.clone() as Arc<dyn octofhir_fhir_model::ModelProvider>
        );

        // Extract resource type from expression or use default
        let inferred_type =
            extract_resource_type_from_expression(expression).unwrap_or_else(|| TypeInfo {
                type_name: "Resource".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some("Resource".to_string()),
            });

        let context = AnalysisContext::new(inferred_type)
            .with_deep_analysis()
            .with_optimization_suggestions(false); // Keep validation focused

        // Run static analysis
        let analysis_result = analyzer.analyze_expression(expression, context).await;

        // Check for errors in analysis results
        let analysis_has_errors = analysis_result.statistics.errors_found > 0;
        has_errors = has_errors || analysis_has_errors;

        // Add analysis diagnostics
        all_diagnostics.extend(analysis_result.diagnostics);
    }

    if has_errors || !all_diagnostics.is_empty() {
        // Report rich diagnostics
        handler
            .report_diagnostics(&all_diagnostics, source_id, &mut stderr())
            .unwrap_or_default();

        // Exit with error code if there are errors
        if has_errors {
            process::exit(1);
        }
    } else {
        // Validation successful - show success in quiet mode-aware way
        if !cli.quiet {
            println!("✅ Validation successful");
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

    // Create shared model provider matching the pattern from other commands
    let shared_model_provider = match create_shared_model_provider().await {
        Ok(provider) => provider,
        Err(e) => {
            eprintln!("❌ Failed to initialize FHIR schema: {e}");
            process::exit(1);
        }
    };

    // Parse initial variables
    let mut initial_variables = Vec::new();
    for var in variables {
        if let Some((name, value)) = var.split_once('=') {
            initial_variables.push((name.to_string(), value.to_string()));
        } else {
            eprintln!("Warning: Invalid variable format '{var}'. Expected name=value");
        }
    }

    // Load initial resource if provided
    let initial_resource = if let Some(input_path) = input {
        match std::fs::read_to_string(input_path) {
            Ok(content) => match serde_json::from_str::<JsonValue>(&content) {
                Ok(json) => Some(json),
                Err(e) => {
                    eprintln!("Warning: Failed to parse JSON from '{input_path}': {e}");
                    None
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read file '{input_path}': {e}");
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

    // Start REPL with shared model provider (same as other commands)
    let model_provider_arc =
        shared_model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>;
    if let Err(e) = start_repl(
        model_provider_arc,
        config,
        initial_resource,
        initial_variables,
    )
    .await
    {
        eprintln!("REPL error: {e}");
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
    use octofhir_fhir_model::TypeInfo;
    use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};
    use std::io::{stderr, stdout};

    // Create diagnostic handler for unified error reporting
    let mut handler = CliDiagnosticHandler::new(cli.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    // First parse the expression with analysis mode for better error recovery
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
        // Phase 2: Run static analysis with shared model provider
        let mut analyzer = StaticAnalyzer::new(
            model_provider.clone() as Arc<dyn octofhir_fhir_model::ModelProvider>
        );

        // Extract resource type from expression or use default
        let inferred_type =
            extract_resource_type_from_expression(expression).unwrap_or_else(|| TypeInfo {
                type_name: "Resource".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some("Resource".to_string()),
            });

        let context = AnalysisContext::new(inferred_type)
            .with_deep_analysis()
            .with_optimization_suggestions(true)
            .with_max_suggestions(10);

        // Run comprehensive static analysis
        let analysis_result = analyzer.analyze_expression(expression, context).await;

        // Add analysis diagnostics to our collection
        all_diagnostics.extend(analysis_result.diagnostics);

        // If verbose mode, show analysis statistics and suggestions
        if cli.verbose {
            eprintln!("📊 Analysis Statistics:");
            eprintln!(
                "   Total expressions: {}",
                analysis_result.statistics.total_expressions
            );
            eprintln!(
                "   Errors found: {}",
                analysis_result.statistics.errors_found
            );
            eprintln!(
                "   Warnings found: {}",
                analysis_result.statistics.warnings_found
            );
            eprintln!(
                "   Analysis time: {}μs",
                analysis_result
                    .statistics
                    .performance_metrics
                    .total_analysis_time
            );

            if !analysis_result.suggestions.is_empty() {
                eprintln!("💡 Suggestions:");
                for suggestion in &analysis_result.suggestions {
                    eprintln!("   {}: {}", suggestion.suggestion_type, suggestion.message);
                }
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
                    && std::env::var("NO_COLOR").is_err()
                    && std::env::var("FHIRPATH_NO_COLOR").is_err()
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

                    println!("\nOnline documentation: {docs_url}");
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
                && std::env::var("NO_COLOR").is_err()
                && std::env::var("FHIRPATH_NO_COLOR").is_err()
            {
                eprintln!(
                    "{}: Invalid error code format: '{}'",
                    "error".red().bold(),
                    error_code
                );
                eprintln!("{}: Expected format: FP0001 or 1", "help".cyan().bold());
            } else {
                eprintln!("error: Invalid error code format: '{error_code}'");
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
        Command::new("cmd").args(["/C", "start", url]).status()
    } else {
        // Linux/Unix
        Command::new("xdg-open").arg(url).status()
    };

    match result {
        Ok(_) => {
            println!("Opened documentation in your default browser.");
        }
        Err(e) => {
            eprintln!("Failed to open browser: {e}. Please visit: {url}");
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_server(
    port: u16,
    _storage: std::path::PathBuf,
    host: String,
    cors_all: bool,
    max_body_size: u64,
    _timeout: u64,
    _rate_limit: u32,
    _cli: &Cli,
) {
    use fhirpath_cli::cli::server::{config::ServerConfig, start_server};

    let config = ServerConfig {
        port,
        host: host
            .parse()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        cors_all,
        max_body_size_mb: max_body_size,
        timeout_seconds: _timeout,
        rate_limit_per_minute: _rate_limit,
        trace_config: fhirpath_cli::cli::server::config::TraceConfig::Server,
    };

    if let Err(e) = start_server(config).await {
        eprintln!("❌ Server error: {e}");
        std::process::exit(1);
    }
}

/// Extract resource type from FHIRPath expression automatically
fn extract_resource_type_from_expression(
    expression: &str,
) -> Option<octofhir_fhir_model::TypeInfo> {
    // Parse common FHIR resource patterns from expressions
    let trimmed = expression.trim();

    // Check if expression starts with a known FHIR resource type
    let known_resources = [
        "Patient",
        "Observation",
        "Condition",
        "Procedure",
        "MedicationRequest",
        "DiagnosticReport",
        "Encounter",
        "Practitioner",
        "Organization",
        "Location",
        "Device",
        "Medication",
        "Substance",
        "AllergyIntolerance",
        "CarePlan",
        "Goal",
        "ServiceRequest",
        "Specimen",
        "ImagingStudy",
        "DocumentReference",
        "Bundle",
        "Composition",
        "ValueSet",
        "CodeSystem",
        "ConceptMap",
        "StructureDefinition",
        "CapabilityStatement",
        "OperationDefinition",
        "SearchParameter",
        "CompartmentDefinition",
        "ImplementationGuide",
        "NamingSystem",
        "TerminologyCapabilities",
        "MessageDefinition",
        "EventDefinition",
        "PlanDefinition",
        "ActivityDefinition",
        "Questionnaire",
        "QuestionnaireResponse",
        "List",
        "Library",
    ];

    // Look for resource types at the start of the expression
    for resource in &known_resources {
        if trimmed.starts_with(resource) {
            // Check if it's followed by a dot, bracket, or end of string
            let resource_len = resource.len();
            if trimmed.len() == resource_len
                || trimmed
                    .chars()
                    .nth(resource_len)
                    .is_some_and(|c| c == '.' || c == '[' || c.is_whitespace())
            {
                return Some(octofhir_fhir_model::TypeInfo {
                    type_name: resource.to_string(),
                    singleton: Some(true),
                    is_empty: Some(false),
                    namespace: Some("FHIR".to_string()),
                    name: Some(resource.to_string()),
                });
            }
        }
    }

    // If no specific resource found, return None to use default
    None
}

/// Calculate span from diagnostic message by finding tokens in expression
#[allow(dead_code)]
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
                if let Some(pos) = expression.find(&format!(".{property_name}")) {
                    return Some(pos + 1..pos + 1 + property_name.len());
                }
            }
        }
    }

    None
}

/// Handle registry commands for functions and operators
async fn handle_registry(command: &RegistryCommands, cli: &Cli) {
    use fhirpath_cli::cli::output::FormatterFactory;

    match command {
        RegistryCommands::List {
            target,
            category,
            search,
            output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let (merged_format, merged_no_color, merged_quiet, _merged_verbose) =
                merge_output_options(cli, output_format.clone(), *no_color, *quiet, *verbose);

            let formatter_factory = FormatterFactory::new(merged_no_color);
            let formatter = formatter_factory.create_formatter(merged_format);

            match target {
                RegistryTarget::Functions => {
                    handle_registry_list_functions(category, search, &*formatter, merged_quiet)
                        .await;
                }
                RegistryTarget::Operators => {
                    handle_registry_list_operators(category, search, &*formatter, merged_quiet)
                        .await;
                }
            }
        }
        RegistryCommands::Show {
            name,
            target,
            output_format,
            no_color,
            quiet,
            verbose,
        } => {
            let (merged_format, merged_no_color, merged_quiet, _merged_verbose) =
                merge_output_options(cli, output_format.clone(), *no_color, *quiet, *verbose);

            let formatter_factory = FormatterFactory::new(merged_no_color);
            let formatter = formatter_factory.create_formatter(merged_format);

            handle_registry_show(name, target, &*formatter, merged_quiet).await;
        }
    }
}

async fn handle_registry_list_functions(
    category: &Option<String>,
    search: &Option<String>,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    quiet: bool,
) {
    use octofhir_fhirpath::evaluator::function_registry::{
        FunctionCategory, create_function_registry,
    };

    let registry = create_function_registry();
    let mut functions: Vec<_> = registry.all_metadata().iter().collect();

    // Filter by category if provided
    if let Some(cat_filter) = category {
        let category_enum = match cat_filter.to_lowercase().as_str() {
            "existence" => Some(FunctionCategory::Existence),
            "filtering" | "projection" => Some(FunctionCategory::FilteringProjection),
            "subsetting" => Some(FunctionCategory::Subsetting),
            "combining" => Some(FunctionCategory::Combining),
            "conversion" => Some(FunctionCategory::Conversion),
            "logic" => Some(FunctionCategory::Logic),
            "string" => Some(FunctionCategory::StringManipulation),
            "math" => Some(FunctionCategory::Math),
            "tree" | "navigation" => Some(FunctionCategory::TreeNavigation),
            "utility" => Some(FunctionCategory::Utility),
            "terminology" => Some(FunctionCategory::Terminology),
            "types" => Some(FunctionCategory::Types),
            "aggregate" => Some(FunctionCategory::Aggregate),
            "cda" => Some(FunctionCategory::CDA),
            _ => {
                if !quiet {
                    eprintln!(
                        "Unknown category '{cat_filter}'. Available categories: existence, filtering, subsetting, combining, conversion, logic, string, math, tree, utility, terminology, types, aggregate, cda"
                    );
                }
                return;
            }
        };

        if let Some(cat) = category_enum {
            functions.retain(|(_, metadata)| metadata.category == cat);
        }
    }

    // Filter by search pattern if provided
    if let Some(search_pattern) = search {
        let pattern = search_pattern.to_lowercase();
        functions.retain(|(name, metadata)| {
            name.to_lowercase().contains(&pattern)
                || metadata.description.to_lowercase().contains(&pattern)
                || metadata.name.to_lowercase().contains(&pattern)
                || format!("{:?}", metadata.category)
                    .to_lowercase()
                    .contains(&pattern)
                || metadata
                    .signature
                    .input_type
                    .to_lowercase()
                    .contains(&pattern)
                || metadata
                    .signature
                    .return_type
                    .to_lowercase()
                    .contains(&pattern)
                || metadata.signature.parameters.iter().any(|p| {
                    p.name.to_lowercase().contains(&pattern)
                        || p.description.to_lowercase().contains(&pattern)
                        || p.parameter_type
                            .iter()
                            .any(|t| t.to_lowercase().contains(&pattern))
                })
        });
    }

    // Sort by name
    functions.sort_by(|(a, _), (b, _)| a.cmp(b));

    if functions.is_empty() {
        if !quiet {
            println!("No functions found matching the criteria");
        }
        return;
    }

    // Output functions based on format
    {
        // Pretty/table format - show summary list
        if !quiet {
            println!("📋 FHIRPath Functions ({} found)", functions.len());
            println!("{}", "=".repeat(50));
        }

        for (name, metadata) in functions {
            let category_str = format!("{:?}", metadata.category);
            let param_count = metadata.signature.parameters.len();
            let param_info = if param_count == 0 {
                "no params".to_string()
            } else if metadata.signature.max_params.is_none() {
                format!("{}+ params", metadata.signature.min_params)
            } else {
                format!(
                    "{}-{} params",
                    metadata.signature.min_params,
                    metadata.signature.max_params.unwrap_or(0)
                )
            };

            println!(
                "🔧 {:<20} | {:<15} | {:<15} | {}",
                name,
                category_str,
                param_info,
                metadata.description.chars().take(40).collect::<String>()
            );
        }

        if !quiet {
            println!("\n💡 Use 'registry show <function_name>' for detailed information");
        }
    }
}

async fn handle_registry_list_operators(
    _category: &Option<String>,
    search: &Option<String>,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    quiet: bool,
) {
    use octofhir_fhirpath::evaluator::operator_registry::create_standard_operator_registry;

    let registry = create_standard_operator_registry();
    let mut operators: Vec<_> = registry.all_metadata().iter().collect();

    // Filter by search pattern if provided
    if let Some(search_pattern) = search {
        let pattern = search_pattern.to_lowercase();
        operators.retain(|(name, metadata)| {
            name.to_lowercase().contains(&pattern)
                || metadata.description.to_lowercase().contains(&pattern)
                || metadata.name.to_lowercase().contains(&pattern)
                || format!("{:?}", metadata.associativity)
                    .to_lowercase()
                    .contains(&pattern)
                || format!("{:?}", metadata.empty_propagation)
                    .to_lowercase()
                    .contains(&pattern)
                || format!("{:?}", metadata.signature.signature.return_type)
                    .to_lowercase()
                    .contains(&pattern)
                || metadata
                    .signature
                    .signature
                    .parameters
                    .iter()
                    .any(|t| format!("{t:?}").to_lowercase().contains(&pattern))
                || metadata.signature.overloads.iter().any(|overload| {
                    format!("{:?}", overload.return_type)
                        .to_lowercase()
                        .contains(&pattern)
                        || overload
                            .parameters
                            .iter()
                            .any(|t| format!("{t:?}").to_lowercase().contains(&pattern))
                })
        });
    }

    // Sort by name
    operators.sort_by(|(a, _), (b, _)| a.cmp(b));

    if operators.is_empty() {
        if !quiet {
            println!("No operators found matching the criteria");
        }
        return;
    }

    // Output operators based on format
    {
        // Pretty/table format - show summary list
        if !quiet {
            println!("🔧 FHIRPath Operators ({} found)", operators.len());
            println!("{}", "=".repeat(50));
        }

        for (name, metadata) in operators {
            let precedence = metadata.precedence;
            let assoc = format!("{:?}", metadata.associativity);

            println!(
                "⚙️  {:<15} | P:{:<2} | {:<5} | {}",
                name,
                precedence,
                assoc,
                metadata.description.chars().take(50).collect::<String>()
            );
        }

        if !quiet {
            println!("\n💡 Use 'registry show <operator_name>' for detailed information");
        }
    }
}

async fn handle_registry_show(
    name: &str,
    target: &RegistryShowTarget,
    formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    quiet: bool,
) {
    match target {
        RegistryShowTarget::Auto => {
            // Try to find in functions first, then operators
            if !try_show_function(name, formatter, quiet).await {
                try_show_operator(name, formatter, quiet).await;
            }
        }
        RegistryShowTarget::Function => {
            try_show_function(name, formatter, quiet).await;
        }
        RegistryShowTarget::Operator => {
            try_show_operator(name, formatter, quiet).await;
        }
    }
}

async fn try_show_function(
    name: &str,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    _quiet: bool,
) -> bool {
    use octofhir_fhirpath::evaluator::function_registry::create_function_registry;

    let registry = create_function_registry();

    if let Some(metadata) = registry.get_metadata(name) {
        // Function found - display detailed information
        println!("🔧 Function: {}", metadata.name);
        println!("{}", "=".repeat(60));
        println!("📝 Description: {}", metadata.description);
        println!("📂 Category: {:?}", metadata.category);
        println!("🔢 Input Type: {}", metadata.signature.input_type);
        println!("🎯 Return Type: {}", metadata.signature.return_type);
        println!("⚡ Deterministic: {}", metadata.deterministic);
        println!("📋 Empty Propagation: {:?}", metadata.empty_propagation);

        if !metadata.signature.parameters.is_empty() {
            println!("\n📥 Parameters:");
            for (i, param) in metadata.signature.parameters.iter().enumerate() {
                let optional = if param.optional { " (optional)" } else { "" };
                let expr = if param.is_expression {
                    " [expression]"
                } else {
                    ""
                };
                println!(
                    "  {}: {} - {}{}{}",
                    i + 1,
                    param.name,
                    param.parameter_type.join(" | "),
                    optional,
                    expr
                );
                if !param.description.is_empty() {
                    println!("     {}", param.description);
                }
            }
        }

        println!(
            "\n🎛️  Signature: {}({})",
            metadata.name,
            metadata
                .signature
                .parameters
                .iter()
                .map(|p| {
                    let name = &p.name;
                    let types = p.parameter_type.join("|");
                    if p.optional {
                        format!("[{name}: {types}]")
                    } else {
                        format!("{name}: {types}")
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        );

        true
    } else {
        false
    }
}

async fn try_show_operator(
    name: &str,
    _formatter: &dyn fhirpath_cli::cli::output::OutputFormatter,
    quiet: bool,
) -> bool {
    use octofhir_fhirpath::evaluator::operator_registry::create_standard_operator_registry;

    let registry = create_standard_operator_registry();

    if let Some(metadata) = registry.get_metadata(name) {
        // Operator found - display detailed information
        println!("⚙️  Operator: {}", metadata.name);
        println!("{}", "=".repeat(60));
        println!("📝 Description: {}", metadata.description);
        println!(
            "🎯 Precedence: {} (higher = evaluated first)",
            metadata.precedence
        );
        println!("↔️  Associativity: {:?}", metadata.associativity);
        println!("⚡ Deterministic: {}", metadata.deterministic);
        println!("📋 Empty Propagation: {:?}", metadata.empty_propagation);

        println!("\n🎛️  Primary Signature:");
        println!(
            "  Input Types: {}",
            metadata
                .signature
                .signature
                .parameters
                .iter()
                .map(|t| format!("{t:?}"))
                .collect::<Vec<_>>()
                .join(" × ")
        );
        println!(
            "  Return Type: {:?}",
            metadata.signature.signature.return_type
        );

        if !metadata.signature.overloads.is_empty() {
            println!("\n🔄 Overloaded Signatures:");
            for (i, overload) in metadata.signature.overloads.iter().enumerate() {
                println!(
                    "  {}: {} → {:?}",
                    i + 1,
                    overload
                        .parameters
                        .iter()
                        .map(|t| format!("{t:?}"))
                        .collect::<Vec<_>>()
                        .join(" × "),
                    overload.return_type
                );
            }
        }

        true
    } else {
        if !quiet {
            eprintln!("Operator '{name}' not found");
        }
        false
    }
}

// Handle the TUI command
/* async fn handle_tui(
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

    // Create ModelProvider
    let model_provider = std::sync::Arc::new(fhirpath_cli::EmbeddedModelProvider::new(FhirVersion::R4));

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
*/
