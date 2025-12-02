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

//! Handler for the analyze command

use crate::EmbeddedModelProvider;
use crate::cli::context::CliContext;
use crate::cli::diagnostics::CliDiagnosticHandler;
use octofhir_fhir_model::TypeInfo;
use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use std::io::{stderr, stdout};
use std::sync::Arc;

/// Handle the analyze command
pub async fn handle_analyze(
    expression: &str,
    _variables: &[String],
    _validate_only: bool,
    _no_inference: bool,
    context: &CliContext,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    let parse_result = parse_with_mode(expression, ParsingMode::Analysis);

    let mut all_diagnostics: Vec<octofhir_fhirpath::diagnostics::AriadneDiagnostic> = Vec::new();

    // Collect parser diagnostics
    if !parse_result.diagnostics.is_empty() {
        let parser_diagnostics: Vec<_> = parse_result
            .diagnostics
            .iter()
            .map(convert_diagnostic_to_ariadne)
            .collect();
        all_diagnostics.extend(parser_diagnostics);
    }

    // Run static analysis if parsing succeeded
    if parse_result.success && parse_result.ast.is_some() {
        let mut analyzer = StaticAnalyzer::new(
            model_provider.clone() as Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>
        );

        let inferred_type = TypeInfo {
            type_name: "Resource".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("FHIR".to_string()),
            name: Some("Resource".to_string()),
        };

        let analysis_context = AnalysisContext::new(inferred_type)
            .with_deep_analysis()
            .with_optimization_suggestions(true)
            .with_max_suggestions(10);

        let analysis_result = analyzer
            .analyze_expression(expression, analysis_context)
            .await;

        all_diagnostics.extend(analysis_result.diagnostics);

        // Show statistics in verbose mode
        if context.verbose {
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

            if !analysis_result.suggestions.is_empty() {
                eprintln!("💡 Suggestions:");
                for suggestion in &analysis_result.suggestions {
                    eprintln!("   {}: {}", suggestion.suggestion_type, suggestion.message);
                }
            }
        }
    }

    // Sort and deduplicate diagnostics
    all_diagnostics.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then(a.error_code.code.cmp(&b.error_code.code))
            .then(a.message.cmp(&b.message))
    });

    all_diagnostics.dedup_by(|a, b| {
        a.message == b.message && a.error_code == b.error_code && a.span == b.span
    });

    // Report diagnostics
    if context.output_format != crate::cli::output::OutputFormat::Json {
        handler
            .report_diagnostics(&all_diagnostics, source_id, &mut stderr())
            .unwrap_or_default();
    } else {
        handler
            .report_diagnostics(&all_diagnostics, source_id, &mut stdout())
            .unwrap_or_default();
    }

    // Exit with error if there were any errors
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

fn convert_diagnostic_to_ariadne(
    diagnostic: &octofhir_fhirpath::diagnostics::Diagnostic,
) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::core::error_code::{FP0001, FP0002};
    use octofhir_fhirpath::diagnostics::AriadneDiagnostic;

    let span = if let Some(location) = &diagnostic.location {
        location.offset..(location.offset + location.length)
    } else {
        0..0
    };

    let error_code = match diagnostic.code.code.as_str() {
        "FP0001" => FP0001,
        "FP0002" => FP0002,
        _ => FP0001,
    };

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
