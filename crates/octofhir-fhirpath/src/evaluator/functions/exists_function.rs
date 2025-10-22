//! Exists function implementation

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use std::sync::Arc;

pub struct ExistsFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ExistsFunctionEvaluator {
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "exists".to_string(),
                description: "Returns true if the collection has any elements (optionally filtered by criteria)".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "criteria".to_string(),
                        parameter_type: vec!["Boolean".to_string()],
                        optional: true,
                        is_expression: true,
                        description: "Optional boolean expression used to filter items".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Custom,
                empty_propagation: EmptyPropagation::Custom,
                deterministic: true,
                category: FunctionCategory::Existence,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for ExistsFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        // If no criteria provided, just check if input is not empty
        if args.is_empty() {
            let result = !input.is_empty();
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(result)),
            });
        }

        // If input is empty, result is false regardless of criteria
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // Evaluate criteria for each element and return true if any match
        let criteria_expr = &args[0];

        for (index, item) in input.iter().enumerate() {
            // Create single-element collection for this item (focus)
            let single_item_collection = vec![item.clone()];

            // Create nested context for this iteration
            let iteration_context = EvaluationContext::new(
                crate::core::Collection::from(single_item_collection.clone()),
                context.model_provider().clone(),
                context.terminology_provider().cloned(),
                context.validation_provider().cloned(),
                context.trace_provider().cloned(),
            );

            // Set lambda variables: $this = single item, $index = current index
            iteration_context.set_variable("$this".to_string(), item.clone());
            iteration_context
                .set_variable("$index".to_string(), FhirPathValue::integer(index as i64));

            // Evaluate criteria expression in the iteration context
            let result = evaluator
                .evaluate(criteria_expr, &iteration_context)
                .await?;

            // Check if result is truthy - if so, return true immediately
            if is_truthy(&result.value) {
                return Ok(EvaluationResult {
                    value: crate::core::Collection::single(FhirPathValue::boolean(true)),
                });
            }
        }

        // None of the items matched the criteria
        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(false)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Helper function to determine if a collection is truthy
fn is_truthy(values: &crate::core::Collection) -> bool {
    if values.is_empty() {
        return false;
    }

    for value in values.iter() {
        match value {
            FhirPathValue::Boolean(b, _, _) => {
                if !b {
                    return false;
                }
            }
            _ => return false, // Non-boolean values are falsy in boolean context
        }
    }

    true
}
