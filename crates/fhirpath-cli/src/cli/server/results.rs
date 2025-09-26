use crate::cli::server::context::initialise_variables;
use crate::cli::server::error::ServerResult;
use crate::cli::server::models::{
    ContextItem, ContextualResult, EvaluationResultItem, ParsedServerRequest, PathSegment,
    TraceOutput, TracePart, fhir_value_to_json, path_segments_to_string,
};
use crate::cli::server::trace::ServerApiTraceProvider;
use octofhir_fhirpath::FhirPathEngine;
use octofhir_fhirpath::core::trace::SharedTraceProvider;
use octofhir_fhirpath::core::{Collection, FhirPathValue, TraceProvider};
use octofhir_fhirpath::evaluator::EvaluationContext;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use std::time::Instant;

pub struct ExpressionEvaluationOutcome {
    pub contexts: Vec<ContextualResult>,
    pub evaluation_time: std::time::Duration,
}

pub async fn evaluate_expression_for_contexts(
    engine: &FhirPathEngine,
    request: &ParsedServerRequest,
    context_items: &[ContextItem],
) -> ServerResult<ExpressionEvaluationOutcome> {
    let model_provider = engine.get_model_provider();
    let terminology_provider = engine.get_terminology_provider();
    let validation_provider = engine.get_validation_provider();

    let resource_type = request
        .resource
        .get("resourceType")
        .and_then(|v| v.as_str())
        .unwrap_or("Resource")
        .to_string();

    let evaluation_start = Instant::now();
    let mut output_contexts = Vec::new();

    for context_item in context_items {
        let trace_provider_impl = Arc::new(ServerApiTraceProvider::new());
        let trace_shared: SharedTraceProvider = trace_provider_impl.clone();

        let collection = Collection::single(context_item.value.clone());
        let evaluation_context = EvaluationContext::new(
            collection,
            model_provider.clone(),
            terminology_provider.clone(),
            validation_provider.clone(),
            Some(trace_shared),
        )
        .await;

        initialise_variables(&evaluation_context, &model_provider, &request.variables).await?;

        let evaluation = engine
            .evaluate_with_metadata(&request.expression, &evaluation_context)
            .await?;

        let mut used_paths: Vec<Vec<PathSegment>> = Vec::new();
        let context_json = fhir_value_to_json(context_item.value.clone());
        let mut result_items = Vec::new();

        for (index, value) in evaluation.result.value.iter().cloned().enumerate() {
            let (segments, path_string) = infer_result_path(
                &context_json,
                context_item,
                &resource_type,
                &value,
                index,
                &mut used_paths,
            );

            result_items.push(EvaluationResultItem {
                datatype: value.display_type_name(),
                value,
                path: path_string,
                path_segments: segments,
                index,
            });
        }

        let traces = build_trace_output(&trace_provider_impl);

        output_contexts.push(ContextualResult {
            context: context_item.clone(),
            results: result_items,
            traces,
        });
    }

    let evaluation_time = evaluation_start.elapsed();

    Ok(ExpressionEvaluationOutcome {
        contexts: output_contexts,
        evaluation_time,
    })
}

fn infer_result_path(
    context_json: &JsonValue,
    context_item: &ContextItem,
    resource_type: &str,
    value: &FhirPathValue,
    index: usize,
    used: &mut Vec<Vec<PathSegment>>,
) -> (Vec<PathSegment>, Option<String>) {
    let target_json = fhir_value_to_json(value.clone());
    let mut local_segments = find_path_within_context(context_json, &target_json, used);

    if local_segments.is_none()
        && matches!(value, FhirPathValue::Resource(_, _, _))
        && &target_json == context_json
    {
        local_segments = Some(Vec::new());
    }

    if let Some(mut relative_segments) = local_segments {
        used.push(relative_segments.clone());
        let mut full_segments = context_item.path_segments.clone();
        full_segments.append(&mut relative_segments);
        let path_string = Some(path_segments_to_string(resource_type, &full_segments));
        (full_segments, path_string)
    } else {
        let base = context_item.path.as_deref().unwrap_or(resource_type);
        let fallback = format!("{}#{}", base, index);
        (context_item.path_segments.clone(), Some(fallback))
    }
}

fn find_path_within_context(
    node: &JsonValue,
    target: &JsonValue,
    used: &mut Vec<Vec<PathSegment>>,
) -> Option<Vec<PathSegment>> {
    let mut current = Vec::new();
    find_path_inner(node, target, &mut current, used)
}

fn find_path_inner(
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
                if let Some(found) = find_path_inner(value, target, current, used) {
                    return Some(found);
                }
                current.pop();
            }
        }
        JsonValue::Array(items) => {
            for (idx, value) in items.iter().enumerate() {
                current.push(PathSegment::Index(idx));
                if let Some(found) = find_path_inner(value, target, current, used) {
                    return Some(found);
                }
                current.pop();
            }
        }
        _ => {}
    }

    None
}

fn build_trace_output(provider: &Arc<ServerApiTraceProvider>) -> Vec<TraceOutput> {
    let mut outputs = Vec::new();

    for entry in provider.collect_structured_traces() {
        let parts = entry
            .values
            .into_iter()
            .map(|value| TracePart {
                datatype: classify_trace_value(&value),
                value,
            })
            .collect();
        outputs.push(TraceOutput {
            name: entry.name,
            parts,
        });
    }

    let simple_lines = provider.collect_traces();
    if !simple_lines.is_empty() {
        outputs.push(TraceOutput {
            name: "log".to_string(),
            parts: simple_lines
                .into_iter()
                .map(|line| TracePart {
                    datatype: "string".to_string(),
                    value: JsonValue::String(line),
                })
                .collect(),
        });
    }

    outputs
}

fn classify_trace_value(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(_) => "boolean".to_string(),
        JsonValue::Number(n) => {
            if n.is_i64() {
                "integer".to_string()
            } else {
                "decimal".to_string()
            }
        }
        JsonValue::String(_) => "string".to_string(),
        JsonValue::Array(_) => "Collection".to_string(),
        JsonValue::Object(map) => map
            .get("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("Element")
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath::core::TraceProvider;
    use serde_json::json;

    #[test]
    fn build_trace_output_includes_structured_and_simple_traces() {
        let provider = Arc::new(ServerApiTraceProvider::new());
        provider.add_structured_trace(
            "stage",
            vec![json!({"resourceType": "Patient"}), json!(true)],
        );
        provider.trace_simple("execution", "step completed");

        let traces = build_trace_output(&provider);

        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].name, "stage");
        assert_eq!(traces[0].parts.len(), 2);
        assert_eq!(traces[0].parts[0].datatype, "Patient");
        assert_eq!(traces[0].parts[1].datatype, "boolean");

        assert_eq!(traces[1].name, "log");
        assert_eq!(traces[1].parts.len(), 1);
        assert_eq!(
            traces[1].parts[0].value,
            JsonValue::String("TRACE[execution]: step completed".to_string())
        );
    }
}
