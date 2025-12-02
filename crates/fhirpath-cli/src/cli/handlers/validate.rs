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

//! Handler for the validate command

use crate::EmbeddedModelProvider;
use crate::cli::context::CliContext;
use crate::cli::diagnostics::CliDiagnosticHandler;
use octofhir_fhir_model::TypeInfo;
use octofhir_fhirpath::analyzer::{AnalysisContext, StaticAnalyzer};
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use std::io::stderr;
use std::process;
use std::sync::Arc;

/// Handle the validate command
pub async fn handle_validate(
    expression: &str,
    context: &CliContext,
    model_provider: &Arc<EmbeddedModelProvider>,
) {
    let mut handler = CliDiagnosticHandler::new(context.output_format.clone());
    let source_id = handler.add_source("expression".to_string(), expression.to_string());

    let parse_result = parse_with_mode(expression, ParsingMode::Fast);

    let mut all_diagnostics: Vec<octofhir_fhirpath::diagnostics::AriadneDiagnostic> = Vec::new();
    let mut has_errors = false;

    // Collect parser diagnostics
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

    // Run static analysis if parsing succeeded
    if parse_result.success && parse_result.ast.is_some() {
        let mut analyzer = StaticAnalyzer::new(
            model_provider.clone() as Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>
        );

        let inferred_type = extract_resource_type(expression, model_provider)
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
            .with_optimization_suggestions(false);

        let analysis_result = analyzer
            .analyze_expression(expression, analysis_context)
            .await;

        let analysis_has_errors = analysis_result.statistics.errors_found > 0;
        has_errors = has_errors || analysis_has_errors;

        all_diagnostics.extend(analysis_result.diagnostics);
    }

    if has_errors || !all_diagnostics.is_empty() {
        handler
            .report_diagnostics(&all_diagnostics, source_id, &mut stderr())
            .unwrap_or_default();

        if has_errors {
            process::exit(1);
        }
    } else if !context.quiet {
        println!("✅ Validation successful");
    }
}

/// Extract resource type from FHIRPath expression by checking if the first
/// identifier is a valid FHIR resource type using the model provider.
/// This automatically works with all FHIR resource types without hardcoding.
async fn extract_resource_type(
    expression: &str,
    model_provider: &Arc<EmbeddedModelProvider>,
) -> Option<TypeInfo> {
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

fn convert_diagnostic_to_ariadne(
    diagnostic: &octofhir_fhirpath::diagnostics::Diagnostic,
) -> octofhir_fhirpath::diagnostics::AriadneDiagnostic {
    use octofhir_fhirpath::core::error_code::{FP0001, FP0002, FP0003};
    use octofhir_fhirpath::diagnostics::AriadneDiagnostic;
    use std::ops::Range;

    let span: Range<usize> = if let Some(location) = &diagnostic.location {
        location.offset..(location.offset + location.length)
    } else {
        0..0
    };

    let error_code = match diagnostic.code.code.as_str() {
        "FP0001" => FP0001,
        "FP0002" => FP0002,
        "FP0003" => FP0003,
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
