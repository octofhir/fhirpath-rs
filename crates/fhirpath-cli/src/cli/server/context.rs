use crate::cli::server::error::ServerResult;
use crate::cli::server::models::{
    ContextEvaluationInfo, ContextItem, ParsedServerRequest, PathSegment, fhir_value_to_json,
    path_segments_to_string,
};
use octofhir_fhirpath::FhirPathEngine;
use octofhir_fhirpath::core::trace::SharedTraceProvider;
use octofhir_fhirpath::core::{Collection, FhirPathValue};
use octofhir_fhirpath::evaluator::EvaluationContext;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::debug;

pub struct ContextEvaluationOutcome {
    pub items: Vec<ContextItem>,
    pub info: ContextEvaluationInfo,
    pub duration: Duration,
}

/// Evaluate the optional context expression and produce contextual items for downstream evaluation.
pub async fn evaluate_context_items(
    engine: &FhirPathEngine,
    request: &ParsedServerRequest,
    trace_provider: Option<SharedTraceProvider>,
) -> ServerResult<ContextEvaluationOutcome> {
    let model_provider = engine.get_model_provider();
    let terminology_provider = engine.get_terminology_provider();
    let validation_provider = engine.get_validation_provider();

    let root_value = match FhirPathValue::resource_with_model_provider(
        request.resource.clone(),
        Some(model_provider.clone()),
    )
    .await
    {
        Ok(value) => value,
        Err(_) => FhirPathValue::resource(request.resource.clone()),
    };

    let input_collection = Collection::single(root_value.clone());

    let evaluation_context = EvaluationContext::new(
        input_collection,
        model_provider.clone(),
        terminology_provider.clone(),
        validation_provider.clone(),
        trace_provider.clone(),
    );

    initialise_variables(&evaluation_context, &model_provider, &request.variables).await?;

    let resource_type = request
        .resource
        .get("resourceType")
        .and_then(|v| v.as_str())
        .unwrap_or("Resource")
        .to_string();

    if let Some(expr) = request.context.as_deref() {
        let start = Instant::now();
        let evaluation = engine
            .evaluate_with_metadata(expr, &evaluation_context)
            .await?;
        let elapsed = start.elapsed();
        debug!("Context expression '{}' evaluated in {:?}", expr, elapsed);

        let mut used_paths: Vec<Vec<PathSegment>> = Vec::new();
        let mut items = Vec::new();
        let collection = evaluation.result.value;
        for (index, value) in collection.iter().cloned().enumerate() {
            let (segments, path_string) = infer_path_for_value(
                &request.resource,
                &resource_type,
                Some(expr),
                &value,
                index,
                &mut used_paths,
            );
            items.push(ContextItem {
                value,
                path: path_string,
                path_segments: segments,
                index,
            });
        }

        let info = ContextEvaluationInfo {
            context_expression: Some(expr.to_string()),
            context_item_count: items.len(),
            context_success: true,
        };

        Ok(ContextEvaluationOutcome {
            items,
            info,
            duration: elapsed,
        })
    } else {
        let item = ContextItem {
            value: root_value,
            path: Some(resource_type.clone()),
            path_segments: Vec::new(),
            index: 0,
        };

        let info = ContextEvaluationInfo {
            context_expression: None,
            context_item_count: 1,
            context_success: true,
        };

        Ok(ContextEvaluationOutcome {
            items: vec![item],
            info,
            duration: Duration::ZERO,
        })
    }
}

pub(crate) async fn initialise_variables(
    eval_context: &EvaluationContext,
    model_provider: &Arc<dyn octofhir_fhir_model::ModelProvider + Send + Sync>,
    variables: &[crate::cli::server::models::Parameter],
) -> ServerResult<()> {
    for parameter in variables {
        let value = parameter
            .to_fhirpath_value(Some(model_provider.clone()))
            .await?;
        eval_context.set_variable(parameter.name.clone(), value);
    }
    Ok(())
}

fn infer_path_for_value(
    resource: &JsonValue,
    resource_type: &str,
    context_expr: Option<&str>,
    value: &FhirPathValue,
    index: usize,
    used: &mut Vec<Vec<PathSegment>>,
) -> (Vec<PathSegment>, Option<String>) {
    let target_json = fhir_value_to_json(value.clone());
    let mut path_segments = find_first_path(resource, &target_json, used);

    if path_segments.is_none()
        && matches!(value, FhirPathValue::Resource(_, _, _))
        && &target_json == resource
    {
        path_segments = Some(Vec::new());
    }

    if let Some(segments) = path_segments.clone() {
        let path_string = Some(path_segments_to_string(resource_type, &segments));
        used.push(segments.clone());
        (segments, path_string)
    } else {
        let fallback = build_fallback_path(resource_type, context_expr, index);
        (Vec::new(), Some(fallback))
    }
}

fn build_fallback_path(resource_type: &str, context_expr: Option<&str>, index: usize) -> String {
    let mut base = match context_expr {
        Some(expr) if expr.starts_with('%') => expr.to_string(),
        Some(expr) if expr.starts_with(resource_type) => expr.to_string(),
        Some(expr) if expr.starts_with('.') => format!("{}{}", resource_type, expr),
        Some("") => resource_type.to_string(),
        Some(expr) => format!("{}.{}", resource_type, expr),
        None => resource_type.to_string(),
    };
    base.push('[');
    base.push_str(&index.to_string());
    base.push(']');
    base
}

fn find_first_path(
    node: &JsonValue,
    target: &JsonValue,
    used: &mut Vec<Vec<PathSegment>>,
) -> Option<Vec<PathSegment>> {
    let mut current = Vec::new();
    find_first_path_inner(node, target, &mut current, used)
}

fn find_first_path_inner(
    node: &JsonValue,
    target: &JsonValue,
    current: &mut Vec<PathSegment>,
    used: &mut Vec<Vec<PathSegment>>,
) -> Option<Vec<PathSegment>> {
    if node == target && !used.iter().any(|p| p == current) {
        return Some(current.clone());
    }

    match node {
        JsonValue::Object(map) => {
            for (key, value) in map {
                current.push(PathSegment::Property(key.clone()));
                if let Some(found) = find_first_path_inner(value, target, current, used) {
                    return Some(found);
                }
                current.pop();
            }
        }
        JsonValue::Array(items) => {
            for (idx, value) in items.iter().enumerate() {
                current.push(PathSegment::Index(idx));
                if let Some(found) = find_first_path_inner(value, target, current, used) {
                    return Some(found);
                }
                current.pop();
            }
        }
        _ => {}
    }

    None
}
