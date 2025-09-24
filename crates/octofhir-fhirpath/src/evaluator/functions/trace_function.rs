//! Trace function implementation

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use serde_json::Value as JsonValue;
use stacker::maybe_grow;
use std::sync::Arc;
use tracing::debug;

pub struct TraceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TraceFunctionEvaluator {
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "trace".to_string(),
                description: "Logs the input collection and returns it unchanged".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "name".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Name for the trace output".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "selector".to_string(),
                            parameter_type: vec!["Collection".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Optional expression to evaluate for each item"
                                .to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(2),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: false, // trace has side effects
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

const TRACE_MAX_DEPTH: usize = 8;
const TRACE_MAX_ITEMS: usize = 25;
const TRACE_MAX_STRING_LENGTH: usize = 4096;
const TRACE_MAX_LITERAL_LENGTH: usize = 256;
const TRACE_MAX_RESOURCE_FIELDS: usize = 10;
const TRACE_STACK_RED_ZONE: usize = 64 * 1024;
const TRACE_STACK_GROW: usize = 8 * 1024 * 1024;

fn format_trace_value(value: &FhirPathValue, depth: usize) -> String {
    maybe_grow(TRACE_STACK_RED_ZONE, TRACE_STACK_GROW, || {
        format_trace_value_inner(value, depth)
    })
}

fn format_trace_value_inner(value: &FhirPathValue, depth: usize) -> String {
    if depth >= TRACE_MAX_DEPTH {
        return "[max depth]".to_string();
    }

    let formatted = match value {
        FhirPathValue::Collection(collection) => format_trace_collection(collection, depth + 1),
        FhirPathValue::Boolean(b, ..) => b.to_string(),
        FhirPathValue::Integer(i, ..) => i.to_string(),
        FhirPathValue::Decimal(d, ..) => d.to_string(),
        FhirPathValue::String(s, ..) => {
            let truncated = truncate_literal_str(s);
            format!("'{}'", truncated.replace('\'', "\\'"))
        }
        FhirPathValue::Date(d, ..) => format!("@{d}"),
        FhirPathValue::DateTime(dt, ..) => format!("@{dt}"),
        FhirPathValue::Time(t, ..) => format!("@T{t}"),
        FhirPathValue::Quantity { value, unit, .. } => {
            if let Some(unit) = unit {
                let unit_fmt = truncate_literal_str(unit);
                format!("{} '{}'", value, unit_fmt)
            } else {
                value.to_string()
            }
        }
        FhirPathValue::Resource(json, ..) => format_trace_resource(json, depth + 1),
        FhirPathValue::Empty => "{}".to_string(),
    };

    truncate_trace_string(formatted)
}

fn format_trace_collection(collection: &Collection, depth: usize) -> String {
    maybe_grow(TRACE_STACK_RED_ZONE, TRACE_STACK_GROW, || {
        if depth >= TRACE_MAX_DEPTH {
            return "Collection[...]".to_string();
        }

        if collection.is_empty() {
            return "{}".to_string();
        }

        if depth > 0 {
            return format!("Collection(len={})", collection.len());
        }

        let mut parts = Vec::new();
        let len = collection.len();
        for (index, item) in collection.iter().enumerate() {
            if index >= TRACE_MAX_ITEMS {
                let remaining = len.saturating_sub(TRACE_MAX_ITEMS);
                if remaining > 0 {
                    parts.push(format!("... (+{remaining} more)"));
                } else {
                    parts.push("...".to_string());
                }
                break;
            }

            parts.push(format_trace_value(item, depth));
        }

        format!("Collection[{}]", parts.join(", "))
    })
}

fn truncate_trace_string(input: String) -> String {
    truncate_to_length(input, TRACE_MAX_STRING_LENGTH)
}

fn truncate_literal_str(value: &str) -> String {
    truncate_to_length(value.to_string(), TRACE_MAX_LITERAL_LENGTH)
}

fn truncate_to_length(mut input: String, limit: usize) -> String {
    if input.len() <= limit {
        return input;
    }

    let mut cut_index = limit;
    while cut_index > 0 && !input.is_char_boundary(cut_index) {
        cut_index -= 1;
    }

    input.truncate(cut_index);
    input.push_str("...[truncated]");
    input
}

fn format_trace_resource(json: &JsonValue, depth: usize) -> String {
    if depth >= TRACE_MAX_DEPTH {
        return "Resource[...]".to_string();
    }

    if let Some(obj) = json.as_object() {
        let mut descriptor = obj
            .get("resourceType")
            .and_then(|v| v.as_str())
            .unwrap_or("Resource")
            .to_string();

        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
            descriptor.push('#');
            let truncated_id = truncate_literal_str(id);
            descriptor.push_str(&truncated_id);
        }

        let mut field_names: Vec<String> = Vec::new();
        let mut total_fields = 0usize;
        for key in obj.keys() {
            if key == "resourceType" || key == "id" {
                continue;
            }
            total_fields += 1;
            if field_names.len() < TRACE_MAX_RESOURCE_FIELDS {
                field_names.push(key.to_string());
            }
        }

        if total_fields > TRACE_MAX_RESOURCE_FIELDS {
            field_names.push("...".to_string());
        }

        if field_names.is_empty() {
            descriptor
        } else {
            format!("{}{{{}}}", descriptor, field_names.join(", "))
        }
    } else if let Some(array) = json.as_array() {
        format!("Array(len={})", array.len())
    } else {
        json.to_string()
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for TraceFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "trace function requires at least one argument (name parameter)".to_string(),
            ));
        }

        // Evaluate the name parameter
        let name_result = evaluator
            .evaluate(
                &args[0],
                &context.create_child_context(crate::core::Collection::empty()),
            )
            .await?;

        let name = if let Some(name_value) = name_result.value.first() {
            match name_value {
                FhirPathValue::String(s, _, _) => s.clone(),
                _ => {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        "trace name parameter must be a string".to_string(),
                    ));
                }
            }
        } else {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "trace name parameter is required".to_string(),
            ));
        };

        // Get trace provider from context
        let trace_provider = context.trace_provider();

        debug!(
            trace_name = %name,
            input_len = input.len(),
            selector = (args.len() == 2),
            "trace() invoked"
        );

        // If there's a second parameter (selector), evaluate it for each item
        if args.len() == 2 {
            for (index, item) in input.iter().enumerate() {
                let item_context =
                    context.create_child_context(crate::core::Collection::single(item.clone()));
                item_context.set_variable("$this".to_string(), item.clone());
                item_context
                    .set_variable("$index".to_string(), FhirPathValue::integer(index as i64));
                item_context.set_variable(
                    "$total".to_string(),
                    FhirPathValue::integer(input.len() as i64),
                );
                let selector_result = evaluator.evaluate(&args[1], &item_context).await?;

                // Format the trace message with the selector result
                let selector_str = format_trace_collection(&selector_result.value, 0);

                // Use trace provider if available, otherwise fall back to eprintln
                if let Some(provider) = trace_provider {
                    provider.trace(&name, index, &selector_str);
                } else {
                    eprintln!("TRACE[{name}][{index}]: {selector_str}");
                }
            }
        } else {
            // Simple trace without selector
            for (index, item) in input.iter().enumerate() {
                let item_str = format_trace_value(item, 0);

                // Use trace provider if available, otherwise fall back to eprintln
                if let Some(provider) = trace_provider {
                    provider.trace(&name, index, &item_str);
                } else {
                    eprintln!("TRACE[{name}][{index}]: {item_str}");
                }
            }
        }

        // Return the input collection unchanged
        Ok(EvaluationResult {
            value: crate::core::Collection::from_values(input),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deep_collection(depth: usize) -> FhirPathValue {
        let mut value = FhirPathValue::integer(1);
        for _ in 0..depth {
            value = FhirPathValue::Collection(Collection::from_values(vec![value]));
        }
        value
    }

    #[test]
    fn format_trace_value_limits_depth() {
        let value = deep_collection(16);
        let formatted = format_trace_value(&value, 0);
        assert!(formatted.contains("[max depth]") || formatted.contains("Collection[..."));
    }

    #[test]
    fn format_trace_collection_limits_items() {
        let items = (0..30).map(FhirPathValue::integer).collect::<Vec<_>>();
        let collection = Collection::from_values(items);
        let formatted = format_trace_collection(&collection, 0);
        assert!(formatted.contains("... (+5 more)"));
    }

    #[test]
    fn truncate_trace_string_adds_suffix() {
        let source = "a".repeat(TRACE_MAX_STRING_LENGTH + 16);
        let truncated = truncate_trace_string(source);
        assert!(truncated.ends_with("...[truncated]"));
        assert!(truncated.len() <= TRACE_MAX_STRING_LENGTH + "...[truncated]".len());
    }

    #[test]
    fn format_trace_resource_limits_fields() {
        let mut obj = serde_json::Map::new();
        obj.insert(
            "resourceType".to_string(),
            JsonValue::String("Patient".to_string()),
        );
        obj.insert("id".to_string(), JsonValue::String("example".to_string()));
        obj.insert("name".to_string(), JsonValue::Array(vec![]));
        obj.insert(
            "birthDate".to_string(),
            JsonValue::String("1970-01-01".to_string()),
        );
        let json = JsonValue::Object(obj);

        let formatted = format_trace_resource(&json, 0);
        assert!(formatted.contains("Patient#example"));
    }
}
