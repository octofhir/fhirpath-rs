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
use octofhir_fhir_model::TypeInfo;
use octofhir_fhirpath::diagnostics::DiagnosticSeverity;
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode, parse_with_semantic_analysis};
use serde_json::Value as JsonValue;
use std::time::Instant;
use tracing::error;

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

async fn handle_request(
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
    if let Some(resource) = resource_type {
        if trimmed.starts_with(resource) {
            let next_char = trimmed.chars().nth(resource.len());
            return next_char
                .map(|c| c == '.' || c == '[' || c.is_whitespace())
                .unwrap_or(true);
        }
    }
    false
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
