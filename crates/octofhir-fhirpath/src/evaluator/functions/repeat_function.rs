//! Repeat function implementation
//!
//! The repeat function applies an expression repeatedly to a collection until no new items are added.
//! Syntax: collection.repeat(expression)

use std::collections::HashSet;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Repeat function evaluator
pub struct RepeatFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl RepeatFunctionEvaluator {
    /// Create a new repeat function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "repeat".to_string(),
                description: "Applies an expression repeatedly until no new items are added"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "expression".to_string(),
                        parameter_type: vec!["Expression".to_string()],
                        optional: false,
                        is_expression: true,
                        description: "The expression to apply repeatedly".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Aggregate,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for RepeatFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("repeat function expects 1 argument, got {}", args.len()),
            ));
        }

        let repeat_expr = &args[0];
        let mut result_values = Vec::new();
        let mut seen = HashSet::new();
        let mut current_items = input;

        // Add initial items to result and track them to avoid infinite loops
        // repeat() returns all items reachable by repeatedly applying the expression
        for item in &current_items {
            let item_key = self.create_item_hash(item);
            if seen.insert(item_key) {
                result_values.push(item.clone());
            }
        }

        // Maximum iterations to prevent infinite loops
        const MAX_ITERATIONS: usize = 1000;

        // Maximum total items to prevent excessive memory usage
        const MAX_ITEMS: usize = 10000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!("repeat function exceeded maximum iterations ({}) - potential infinite loop", MAX_ITERATIONS),
                ));
            }

            if result_values.len() > MAX_ITEMS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!("repeat function exceeded maximum items ({}) - preventing excessive memory usage", MAX_ITEMS),
                ));
            }

            let mut new_items = Vec::new();

            // Apply the repeat expression to each current item
            for item in &current_items {
                // Create a proper context with this item as the input
                let single_item_collection = vec![item.clone()];
                let item_context = EvaluationContext::new(
                    crate::core::Collection::from(single_item_collection),
                    context.model_provider().clone(),
                    context.terminology_provider().clone(),
                    context.trace_provider(),
                )
                .await;

                let item_result = evaluator.evaluate(repeat_expr, &item_context).await?;
                for new_item in item_result.value.into_iter() {
                    let item_key = self.create_item_hash(&new_item);
                    if seen.insert(item_key) {
                        new_items.push(new_item.clone());
                        result_values.push(new_item);
                    }
                }
            }

            // If no new items were found, we're done
            if new_items.is_empty() {
                break;
            }

            current_items = new_items;
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(result_values),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl RepeatFunctionEvaluator {
    /// Create a better hash for item deduplication
    fn create_item_hash(&self, item: &FhirPathValue) -> String {
        match item {
            FhirPathValue::String(s, _, _) => format!("str:{}", s),
            FhirPathValue::Integer(i, _, _) => format!("int:{}", i),
            FhirPathValue::Decimal(d, _, _) => format!("dec:{}", d),
            FhirPathValue::Boolean(b, _, _) => format!("bool:{}", b),
            FhirPathValue::Date(d, _, _) => format!("date:{:?}", d),
            FhirPathValue::DateTime(dt, _, _) => format!("datetime:{:?}", dt),
            FhirPathValue::Time(t, _, _) => format!("time:{:?}", t),
            FhirPathValue::Resource(json, type_info, _) => {
                // For resources, create hash based on resource type and id if available
                let resource_type = json.get("resourceType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let id = json.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("no-id");
                format!("resource:{}:{}", resource_type, id)
            }
            FhirPathValue::Quantity { value, unit, .. } => {
                let unit_str = unit.as_deref().unwrap_or("no-unit");
                format!("quantity:{}:{}", value, unit_str)
            }
            FhirPathValue::Collection(collection) => {
                format!("collection:{}", collection.len())
            }
            FhirPathValue::Empty => "empty".to_string(),
        }
    }
}
