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

//! Handler for the evaluate command

use crate::EmbeddedModelProvider;
use crate::cli::context::{CliContext, EngineBuilder};
use crate::cli::diagnostics::CliDiagnosticHandler;
use crate::cli::output::{EvaluationOutput, OutputMetadata};
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use serde_json::{Value as JsonValue, from_str as parse_json};
use std::fs;
use std::io::{Read, stderr};
use std::process;
use std::sync::Arc;
use std::time::Instant;

/// Handle the evaluate command
pub async fn handle_evaluate(
    expression: &str,
    input: Option<&str>,
    variables: &[String],
    _pretty: bool,
    analyze: bool,
    context: &CliContext,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    // Get resource data with smart detection
    let resource_data = load_resource_input(input, context);

    // Handle empty input case
    let resource: JsonValue = if resource_data.trim().is_empty() {
        parse_json("{}").unwrap_or_default()
    } else {
        match parse_json(&resource_data) {
            Ok(json) => json,
            Err(e) => {
                report_json_parse_error(&resource_data, e, expression, context);
                process::exit(1);
            }
        }
    };

    // Create FHIRPath engine
    let mut engine = match EngineBuilder::new()
        .with_model_provider(model_provider.clone())
        .build()
        .await
    {
        Ok(engine) => engine,
        Err(e) => {
            report_engine_creation_error(expression, e, context);
            process::exit(1);
        }
    };

    // Start timing for actual execution
    let start_time = Instant::now();

    // Parse initial variables from command line
    let parsed_variables = parse_variables(variables);

    // Create Collection with proper resource typing
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
            if !context.quiet {
                eprintln!("⚠️ Warning: Failed to properly type resource, using fallback: {e}");
            }
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
        None,
    );

    // Add variables if provided
    if !parsed_variables.is_empty() {
        for (name, value) in parsed_variables {
            eval_context.set_variable(name.to_string(), value);
        }
    }

    // Parse and evaluate the expression
    let output =
        evaluate_expression(expression, &mut engine, &eval_context, start_time, context).await;

    // Run static analysis if requested
    if analyze {
        run_static_analysis(expression, model_provider, context).await;
    }

    // Format and display output
    let formatter = context.create_formatter();
    match formatter.format_evaluation(&output) {
        Ok(formatted) => {
            println!("{formatted}");

            // Show profiling information if requested
            if context.profile && !context.quiet {
                use crate::cli::profiler::PerformanceReport;

                // Create a simple profiling report
                let phases = vec![crate::cli::profiler::ProfilePhase {
                    name: "Total Execution".to_string(),
                    duration: output.execution_time,
                    percentage: 100.0,
                }];

                let report = PerformanceReport {
                    total_duration: output.execution_time,
                    phases,
                };

                eprintln!("{}", report.format_table());
            }

            if !output.success {
                process::exit(1);
            }
        }
        Err(e) => {
            report_format_error(expression, e, context);
            process::exit(1);
        }
    }
}

/// Load resource input from file, stdin, or literal JSON
fn load_resource_input(input: Option<&str>, context: &CliContext) -> String {
    if let Some(input_str) = input {
        // Check if input is a file path or JSON string
        if input_str.starts_with('{') || input_str.starts_with('[') || input_str.trim().is_empty() {
            // Treat as JSON string directly
            input_str.to_string()
        } else {
            // Treat as file path
            match fs::read_to_string(input_str) {
                Ok(content) => content,
                Err(e) => {
                    if !context.quiet {
                        eprintln!("Error reading file {input_str}: {e}");
                    }
                    process::exit(1);
                }
            }
        }
    } else {
        // No input provided - read from stdin
        let mut stdin_content = String::new();
        match std::io::stdin().read_to_string(&mut stdin_content) {
            Ok(_) => stdin_content,
            Err(e) => {
                if !context.quiet {
                    eprintln!("Error reading from stdin: {e}");
                }
                process::exit(1);
            }
        }
    }
}

/// Parse variables from command line format
fn parse_variables(variables: &[String]) -> Vec<(String, octofhir_fhirpath::FhirPathValue)> {
    let mut parsed = Vec::new();
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
            parsed.push((name.to_string(), value));
        } else {
            eprintln!("⚠️ Invalid variable format {var_spec}, expected 'name=value'");
        }
    }
    parsed
}

/// Evaluate FHIRPath expression and return output
async fn evaluate_expression(
    expression: &str,
    engine: &mut octofhir_fhirpath::evaluator::FhirPathEngine,
    eval_context: &octofhir_fhirpath::EvaluationContext,
    start_time: Instant,
    context: &CliContext,
) -> EvaluationOutput {
    // First parse the expression to get proper diagnostics
    let parse_result = parse_with_mode(expression, ParsingMode::Fast);

    if !parse_result.success {
        // Parse failed - show diagnostics
        let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
        let source_id = handler.add_source("expression".to_string(), expression.to_string());

        // Report all diagnostics
        if context.output_format != crate::cli::output::OutputFormat::Json {
            let ariadne_diagnostics: Vec<_> = parse_result
                .diagnostics
                .iter()
                .map(convert_diagnostic_to_ariadne)
                .collect();
            handler
                .report_diagnostics(&ariadne_diagnostics, source_id, &mut stderr())
                .unwrap_or_default();
        }

        // Create error output
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

        let error = octofhir_fhirpath::core::FhirPathError::parse_error(
            if error_details.is_empty() {
                octofhir_fhirpath::core::error_code::FP0001
            } else {
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
        return EvaluationOutput {
            success: false,
            result: None,
            result_with_metadata: None,
            error: Some(error),
            expression: expression.to_string(),
            execution_time,
            metadata: OutputMetadata::default(),
        };
    }

    // Parse successful - evaluate
    let result = engine
        .evaluate_with_metadata(expression, eval_context)
        .await;

    let execution_time = start_time.elapsed();
    match result {
        Ok(eval_result_with_metadata) => {
            let collection_with_metadata = octofhir_fhirpath::core::CollectionWithMetadata::from(
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
            let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());

            let diagnostic = fhirpath_error_to_ariadne(&e, 0..expression.len());
            if context.output_format != crate::cli::output::OutputFormat::Json {
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
}

/// Run static analysis on the expression
async fn run_static_analysis(
    expression: &str,
    model_provider: &Arc<EmbeddedModelProvider>,
    context: &CliContext,
) {
    use octofhir_fhir_model::TypeInfo;
    use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};

    let mut analyzer = StaticAnalyzer::new(
        model_provider.clone() as Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>
    );

    let inferred_type = extract_resource_type_from_expression(expression, model_provider)
        .await
        .unwrap_or_else(|| TypeInfo {
            type_name: "Resource".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("FHIR".to_string()),
            name: Some("Resource".to_string()),
        });

    let analysis_context = AnalysisContext::new(inferred_type)
        .with_deep_analysis()
        .with_optimization_suggestions(true)
        .with_max_suggestions(5);

    let analysis_result = analyzer
        .analyze_expression(expression, analysis_context)
        .await;

    // Show analysis results if verbose or if there are issues
    if context.verbose
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

        if !analysis_result.diagnostics.is_empty() {
            let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
            let source_id = handler.add_source("expression".to_string(), expression.to_string());

            handler
                .report_diagnostics(&analysis_result.diagnostics, source_id, &mut stderr())
                .unwrap_or_default();
        }
    }
}

/// Extract resource type from FHIRPath expression by checking if the first
/// identifier is a valid FHIR resource type using the model provider.
/// This automatically works with all FHIR resource types without hardcoding.
async fn extract_resource_type_from_expression(
    expression: &str,
    model_provider: &Arc<EmbeddedModelProvider>,
) -> Option<octofhir_fhir_model::TypeInfo> {
    let trimmed = expression.trim();

    // Extract the first identifier (must start with uppercase to be a resource type)
    let first_identifier = trimmed
        .split(|c: char| c == '.' || c == '[' || c.is_whitespace() || c == '(')
        .next()?;

    // Check if it starts with uppercase (FHIR resource types are PascalCase)
    if first_identifier.is_empty()
        || !first_identifier.chars().next()?.is_uppercase()
        || !first_identifier.chars().all(|c| c.is_alphanumeric())
    {
        return None;
    }

    // Verify it's a real FHIR resource type by querying the model provider
    let model_provider_arc =
        model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>;
    if let Ok(Some(type_info)) = model_provider_arc.get_type(first_identifier).await {
        return Some(type_info);
    }

    None
}

// Error reporting helpers

fn report_json_parse_error(
    resource_data: &str,
    error: serde_json::Error,
    _expression: &str,
    context: &CliContext,
) {
    let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
    let source_id = handler.add_source("resource".to_string(), resource_data.to_string());

    let diagnostic = handler.create_diagnostic_from_error(
        octofhir_fhirpath::core::error_code::FP0001,
        format!("Invalid JSON resource: {error}"),
        0..resource_data.len(),
        Some("Ensure the resource is valid JSON format".to_string()),
    );

    handler
        .report_diagnostic(&diagnostic, source_id, &mut stderr())
        .unwrap_or_default();
}

fn report_engine_creation_error(expression: &str, error: anyhow::Error, context: &CliContext) {
    let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    let diagnostic = handler.create_diagnostic_from_error(
        octofhir_fhirpath::core::error_code::FP0001,
        format!("Failed to create FHIRPath engine: {error}"),
        0..expression.len(),
        None,
    );

    handler
        .report_diagnostic(&diagnostic, source_id, &mut stderr())
        .unwrap_or_default();
}

fn report_format_error(
    expression: &str,
    error: crate::cli::output::FormatError,
    context: &CliContext,
) {
    let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    let diagnostic = handler.create_diagnostic_from_error(
        octofhir_fhirpath::core::error_code::FP0001,
        format!("Error formatting output: {error}"),
        0..expression.len(),
        Some("Check output format configuration".to_string()),
    );

    handler
        .report_diagnostic(&diagnostic, source_id, &mut stderr())
        .unwrap_or_default();
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
                FP0001
            }
        }
    }
}

/// Convert parser Diagnostic to AriadneDiagnostic
fn convert_diagnostic_to_ariadne(
    diagnostic: &octofhir_fhirpath::diagnostics::Diagnostic,
) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::diagnostics::AriadneDiagnostic;
    use std::ops::Range;

    let span: Range<usize> = if let Some(location) = &diagnostic.location {
        location.offset..(location.offset + location.length)
    } else {
        0..0
    };

    let error_code = parse_error_code(&diagnostic.code.code);

    AriadneDiagnostic {
        severity: diagnostic.severity.clone(),
        error_code,
        message: diagnostic.message.clone(),
        span,
        help: None,
        note: None,
        related: Vec::new(),
    }
}

/// Convert FhirPathError to AriadneDiagnostic
fn fhirpath_error_to_ariadne(
    error: &octofhir_fhirpath::core::FhirPathError,
    span: std::ops::Range<usize>,
) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};

    AriadneDiagnostic {
        severity: DiagnosticSeverity::Error,
        error_code: error.error_code().clone(),
        message: error.to_string(),
        span,
        help: None,
        note: None,
        related: Vec::new(),
    }
}
