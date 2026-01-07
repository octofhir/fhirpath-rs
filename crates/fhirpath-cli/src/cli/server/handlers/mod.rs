use crate::cli::ast::{convert_ast_to_lab_format, extract_resource_type};
use crate::cli::server::context::{ContextEvaluationOutcome, evaluate_context_items};
use crate::cli::server::error::{ServerError, ServerResult};
use crate::cli::server::models::{
    EvaluationResultSet, EvaluationTiming, OperationOutcome, ParametersResource,
};
use crate::cli::server::registry::ServerRegistry;
use crate::cli::server::response::{ParseDebugInfo, ResponseMetadata, build_success_response};
use crate::cli::server::results::evaluate_expression_for_contexts;
use crate::cli::server::version::ServerFhirVersion;
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use octofhir_fhir_model::{ModelProvider, TypeInfo};
use octofhir_fhirpath::diagnostics::DiagnosticSeverity;
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode, parse_with_semantic_analysis};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::Instant;
use tracing::error;

/// Analysis request structure
#[derive(Debug, Deserialize)]
pub struct AnalysisRequest {
    /// FHIRPath expression to analyze
    pub expression: String,
    /// Optional resource type for context
    #[serde(rename = "resourceType")]
    pub resource_type: Option<String>,
    /// Analysis options
    #[serde(default)]
    pub options: AnalysisOptions,
}

/// Analysis options
#[derive(Debug, Default, Deserialize)]
pub struct AnalysisOptions {
    /// Only validate, don't perform full type inference
    #[serde(default)]
    pub validate_only: bool,
    /// Include detailed type information
    #[serde(default = "default_true")]
    pub include_type_info: bool,
    /// Include AST debug information
    #[serde(default)]
    pub include_ast: bool,
}

fn default_true() -> bool {
    true
}

/// Analysis response structure
#[derive(Debug, Serialize)]
pub struct AnalysisResponse {
    /// Whether analysis was successful
    pub success: bool,
    /// Parsed expression text
    pub expression: String,
    /// Inferred return type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    /// Type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_info: Option<JsonValue>,
    /// Diagnostics (errors and warnings)
    pub diagnostics: Vec<DiagnosticInfo>,
    /// AST representation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<JsonValue>,
    /// Analysis timing
    pub timing_ms: f64,
}

/// Diagnostic information
#[derive(Debug, Serialize)]
pub struct DiagnosticInfo {
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<DiagnosticLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Location information for diagnostics
#[derive(Debug, Serialize)]
pub struct DiagnosticLocation {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

pub async fn health_handler(
    State(registry): State<ServerRegistry>,
) -> ServerResult<Json<JsonValue>> {
    let versions: Vec<_> = ServerFhirVersion::all()
        .iter()
        .map(|v| v.as_str().to_string())
        .collect();

    let payload = serde_json::json!({
        "status": "ok",
        "versions": versions,
        "engines": registry.version_count(),
    });

    Ok(Json(payload))
}

pub async fn version_handler() -> Result<Json<JsonValue>, ServerError> {
    let payload = serde_json::json!({
        "service": "octofhir-fhirpath-server",
        "version": env!("CARGO_PKG_VERSION"),
        "build": {
            "commit": option_env!("OCTOFHIR_BUILD_COMMIT").unwrap_or("unknown"),
            "date": option_env!("OCTOFHIR_BUILD_DATE").unwrap_or("unknown"),
        }
    });

    Ok(Json(payload))
}

/// Analysis endpoint for R4
pub async fn analyze_r4_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<AnalysisRequest>,
) -> impl IntoResponse {
    handle_analysis(&registry, ServerFhirVersion::R4, request).await
}

/// Analysis endpoint for R4B
pub async fn analyze_r4b_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<AnalysisRequest>,
) -> impl IntoResponse {
    handle_analysis(&registry, ServerFhirVersion::R4B, request).await
}

/// Analysis endpoint for R5
pub async fn analyze_r5_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<AnalysisRequest>,
) -> impl IntoResponse {
    handle_analysis(&registry, ServerFhirVersion::R5, request).await
}

/// Analysis endpoint for R6
pub async fn analyze_r6_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<AnalysisRequest>,
) -> impl IntoResponse {
    handle_analysis(&registry, ServerFhirVersion::R6, request).await
}

/// Handle analysis request
async fn handle_analysis(
    registry: &ServerRegistry,
    version: ServerFhirVersion,
    request: AnalysisRequest,
) -> Result<Json<AnalysisResponse>, ServerError> {
    let start = Instant::now();

    // Get model provider for the requested FHIR version
    let model_provider = match registry.get_model_provider(version) {
        Some(provider) => provider,
        None => {
            return Err(ServerError::NotSupported(format!(
                "FHIR version {} not available",
                version
            )));
        }
    };

    // Parse the expression
    let parse_result = parse_with_mode(&request.expression, ParsingMode::Analysis);

    let mut diagnostics = Vec::new();

    // Convert parse diagnostics
    for diag in &parse_result.diagnostics {
        diagnostics.push(DiagnosticInfo {
            severity: format!("{:?}", diag.severity),
            message: diag.message.clone(),
            location: diag.location.as_ref().map(|loc| DiagnosticLocation {
                line: loc.line,
                column: loc.column,
                offset: loc.offset,
                length: loc.length,
            }),
            code: Some(diag.code.code.clone()),
        });
    }

    // Check for parse errors
    let has_errors = parse_result
        .diagnostics
        .iter()
        .any(|d| matches!(d.severity, DiagnosticSeverity::Error));

    if has_errors {
        return Ok(Json(AnalysisResponse {
            success: false,
            expression: request.expression,
            return_type: None,
            type_info: None,
            diagnostics,
            ast: None,
            timing_ms: start.elapsed().as_secs_f64() * 1000.0,
        }));
    }

    let ast = match &parse_result.ast {
        Some(ast) => ast,
        None => {
            return Ok(Json(AnalysisResponse {
                success: false,
                expression: request.expression,
                return_type: None,
                type_info: None,
                diagnostics,
                ast: None,
                timing_ms: start.elapsed().as_secs_f64() * 1000.0,
            }));
        }
    };

    // Perform semantic analysis
    let resource_type_info: Option<TypeInfo> =
        if let Some(ref resource_type) = request.resource_type {
            match model_provider.get_type(resource_type).await {
                Ok(Some(info)) => Some(info),
                Ok(None) => {
                    diagnostics.push(DiagnosticInfo {
                        severity: "Warning".to_string(),
                        message: format!("Resource type '{}' not found in model", resource_type),
                        location: None,
                        code: None,
                    });
                    None
                }
                Err(e) => {
                    return Err(ServerError::Model(e));
                }
            }
        } else {
            None
        };

    let semantic_result = parse_with_semantic_analysis(
        &request.expression,
        model_provider.clone(),
        resource_type_info.clone(),
    )
    .await;

    // Add semantic diagnostics
    for diag in &semantic_result.analysis.diagnostics {
        diagnostics.push(DiagnosticInfo {
            severity: format!("{:?}", diag.severity),
            message: diag.message.clone(),
            location: diag.location.as_ref().map(|loc| DiagnosticLocation {
                line: loc.line,
                column: loc.column,
                offset: loc.offset,
                length: loc.length,
            }),
            code: Some(diag.code.code.clone()),
        });
    }

    // Build AST if requested
    let ast_json = if request.options.include_ast {
        let engine_guard = registry
            .get_evaluation_engine(version)
            .ok_or_else(|| {
                ServerError::NotSupported(format!("FHIR version {} not available", version))
            })?
            .lock_owned()
            .await;

        let lab_ast = convert_ast_to_lab_format(
            ast,
            Some(engine_guard.get_function_registry().as_ref()),
            Some(model_provider.as_ref()),
        );
        Some(serde_json::to_value(lab_ast).unwrap_or(serde_json::json!({})))
    } else {
        None
    };

    // Build type info if requested
    let type_info = if request.options.include_type_info {
        semantic_result.analysis.root_type.as_ref().map(|ti| {
            serde_json::json!({
                "type_name": ti.type_name,
                "singleton": ti.singleton,
                "is_empty": ti.is_empty,
                "namespace": ti.namespace,
                "name": ti.name,
            })
        })
    } else {
        None
    };

    let return_type = semantic_result
        .analysis
        .root_type
        .as_ref()
        .map(|ti| ti.type_name.clone());

    Ok(Json(AnalysisResponse {
        success: semantic_result.analysis.success,
        expression: request.expression,
        return_type,
        type_info,
        diagnostics,
        ast: ast_json,
        timing_ms: start.elapsed().as_secs_f64() * 1000.0,
    }))
}

pub async fn fhirpath_lab_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<ParametersResource>,
) -> impl IntoResponse {
    let version = ServerFhirVersion::R4; // default when not specified explicitly
    match handle_request(&registry, version, request).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn fhirpath_lab_r4_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<ParametersResource>,
) -> impl IntoResponse {
    match handle_request(&registry, ServerFhirVersion::R4, request).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn fhirpath_lab_r4b_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<ParametersResource>,
) -> impl IntoResponse {
    match handle_request(&registry, ServerFhirVersion::R4B, request).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn fhirpath_lab_r5_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<ParametersResource>,
) -> impl IntoResponse {
    match handle_request(&registry, ServerFhirVersion::R5, request).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn fhirpath_lab_r6_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<ParametersResource>,
) -> impl IntoResponse {
    match handle_request(&registry, ServerFhirVersion::R6, request).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

pub(crate) async fn handle_request(
    registry: &ServerRegistry,
    version: ServerFhirVersion,
    request: ParametersResource,
) -> Result<Json<JsonValue>, ServerError> {
    let parsed_request = match request.parse_request() {
        Ok(parsed) => parsed,
        Err(error) => {
            return Ok(json_outcome(OperationOutcome::error(
                "invalid",
                &error.to_string(),
                None,
            )));
        }
    };

    let engine_arc = match registry.get_evaluation_engine(version) {
        Some(engine) => engine,
        None => {
            return Ok(json_outcome(OperationOutcome::error(
                "not-supported",
                &format!("FHIR version {} not available", version),
                None,
            )));
        }
    };

    let engine_guard = engine_arc.lock_owned().await;
    let model_provider = engine_guard.get_model_provider();

    let parse_start = Instant::now();
    let parse_result = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);

    if let Some(outcome) =
        parse_error_outcome(&parsed_request.expression, &parse_result.diagnostics)
    {
        return Ok(json_outcome(outcome));
    }

    let ast = match &parse_result.ast {
        Some(ast) => ast,
        None => {
            return Ok(json_outcome(OperationOutcome::error(
                "invalid",
                "Parsing failed without AST",
                None,
            )));
        }
    };

    let lab_ast = convert_ast_to_lab_format(
        ast,
        Some(engine_guard.get_function_registry().as_ref()),
        Some(model_provider.as_ref()),
    );

    let resource_type = extract_resource_type(&parsed_request.resource);
    let expression_has_explicit_resource =
        expression_has_explicit_resource_head(&parsed_request.expression, resource_type.as_deref());
    let parse_time = parse_start.elapsed();

    let analysis_start = Instant::now();
    let resource_type_info: Option<TypeInfo> = if let Some(ref name) = resource_type {
        match model_provider.get_type(name).await {
            Ok(Some(info)) => Some(info),
            Ok(None) => None,
            Err(error) => {
                return Err(ServerError::Model(error));
            }
        }
    } else {
        None
    };

    let mut context_semantic_root: Option<TypeInfo> = None;
    if let Some(context_expr) = parsed_request.context.as_deref() {
        let context_semantic = parse_with_semantic_analysis(
            context_expr,
            model_provider.clone(),
            resource_type_info.clone(),
        )
        .await;

        // Context expression semantic analysis issues are now embedded in response
        // instead of returning as errors
        context_semantic_root = context_semantic.analysis.root_type.clone();
    }

    let context_evaluation = evaluate_context_items(&engine_guard, &parsed_request, None).await?;
    let ContextEvaluationOutcome {
        items: context_items,
        info: context_info,
        duration: context_duration,
    } = context_evaluation;

    let mut context_type = if expression_has_explicit_resource {
        resource_type_info.clone()
    } else if let Some(ref root) = context_semantic_root {
        Some(root.clone())
    } else if let Some(first_item) = context_items.first() {
        Some(first_item.value.type_info().clone())
    } else {
        resource_type_info.clone()
    };

    if context_type.is_none() {
        context_type = resource_type_info.clone();
    }

    let mut semantic_result = parse_with_semantic_analysis(
        &parsed_request.expression,
        model_provider.clone(),
        context_type,
    )
    .await;

    if !semantic_result.analysis.success && expression_has_explicit_resource {
        let fallback =
            parse_with_semantic_analysis(&parsed_request.expression, model_provider.clone(), None)
                .await;

        if fallback.analysis.success {
            semantic_result = fallback;
        }
    }

    // Store semantic diagnostics for embedding in response instead of returning error
    let semantic_diagnostics = semantic_result.analysis.diagnostics.clone();

    let expected_return_type = semantic_result
        .analysis
        .root_type
        .as_ref()
        .map(|type_info| type_info.type_name.clone());

    let analysis_time = analysis_start.elapsed();

    let evaluation_outcome =
        evaluate_expression_for_contexts(&engine_guard, &parsed_request, &context_items).await?;

    let timing = EvaluationTiming {
        parse: parse_time + analysis_time,
        evaluation: context_duration + evaluation_outcome.evaluation_time,
        total: parse_time + analysis_time + context_duration + evaluation_outcome.evaluation_time,
    };

    let evaluation = EvaluationResultSet {
        context_info,
        contexts: evaluation_outcome.contexts,
        timing,
    };

    let parse_debug_tree =
        serde_json::to_string_pretty(&lab_ast).unwrap_or_else(|_| "{}".to_string());
    let parse_debug = ParseDebugInfo {
        summary: format!(
            "{} : {}",
            parsed_request.expression,
            expected_return_type
                .clone()
                .unwrap_or_else(|| "unknown".to_string())
        ),
        tree: parse_debug_tree,
    };

    let evaluator_label = evaluator_label(version);
    let metadata = ResponseMetadata {
        evaluator_label: &evaluator_label,
        expected_return_type,
        parse_debug: &parse_debug,
        semantic_diagnostics: &semantic_diagnostics,
    };

    let response = build_success_response(&parsed_request, &evaluation, metadata);
    let json = serde_json::to_value(response)?;
    Ok(Json(json))
}

fn expression_has_explicit_resource_head(expression: &str, resource_type: Option<&str>) -> bool {
    let trimmed = expression.trim_start();
    if let Some(resource) = resource_type
        && trimmed.starts_with(resource)
    {
        let next_char = trimmed.chars().nth(resource.len());
        return next_char
            .map(|c| c == '.' || c == '[' || c.is_whitespace())
            .unwrap_or(true);
    }
    false
}

fn evaluator_label(version: ServerFhirVersion) -> String {
    format!(
        "octofhir-fhirpath-{} ({})",
        env!("CARGO_PKG_VERSION"),
        version.as_str()
    )
}

fn parse_error_outcome(
    expression: &str,
    diagnostics: &[octofhir_fhirpath::diagnostics::Diagnostic],
) -> Option<OperationOutcome> {
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|diag| matches!(diag.severity, DiagnosticSeverity::Error))
        .collect();

    if errors.is_empty() {
        return None;
    }

    let summary = errors
        .iter()
        .map(|diag| {
            if let Some(location) = &diag.location {
                format!("{} at {}:{}", diag.message, location.line, location.column)
            } else {
                diag.message.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("; ");

    Some(OperationOutcome::error(
        "invalid",
        &format!("Failed to parse expression: {}", expression),
        Some(summary),
    ))
}

fn json_outcome(outcome: OperationOutcome) -> Json<JsonValue> {
    Json(serde_json::to_value(outcome).unwrap_or_else(|error| {
        error!("Failed to serialize OperationOutcome: {}", error);
        serde_json::json!({"resourceType": "OperationOutcome", "issue": []})
    }))
}

#[cfg(test)]
mod tests {
    use super::expression_has_explicit_resource_head;

    #[test]
    fn detects_matching_resource_head() {
        assert!(expression_has_explicit_resource_head(
            "Patient.name",
            Some("Patient")
        ));
        assert!(expression_has_explicit_resource_head(
            "Observation.code.system",
            Some("Observation")
        ));
        assert!(expression_has_explicit_resource_head(
            "Patient ",
            Some("Patient")
        ));
    }

    #[test]
    fn rejects_non_matching_or_missing_head() {
        assert!(!expression_has_explicit_resource_head(
            "name.given",
            Some("Patient")
        ));
        assert!(!expression_has_explicit_resource_head(
            "Patient.name",
            Some("Observation")
        ));
        assert!(!expression_has_explicit_resource_head(
            "%context.name",
            Some("Patient")
        ));
    }
}
