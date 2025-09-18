//! All function implementation
//!
//! The all function returns true if for every element in the input collection, criteria evaluates to true.
//! Otherwise, the result is false. If the input collection is empty, the result is true.
//! Syntax: collection.all(criteria)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// All function evaluator
pub struct AllFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AllFunctionEvaluator {
    /// Create a new all function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "all".to_string(),
                description: "Returns true if for every element in the input collection, criteria evaluates to true. Otherwise, the result is false. If the input collection is empty, the result is true.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "criteria".to_string(),
                            parameter_type: vec!["Boolean".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Boolean expression used to test each item".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Boolean".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                empty_propagation: EmptyPropagation::NoPropagation, // Returns true for empty collections
                deterministic: true,
                category: FunctionCategory::Existence,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for AllFunctionEvaluator {
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
                "all function requires exactly one argument (criteria expression)".to_string(),
            ));
        }

        // If the input collection is empty, the result is true per spec
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(true)),
            });
        }

        let criteria_expr = &args[0];

        // Evaluate the criteria for each element in the input collection
        for (index, item) in input.iter().enumerate() {
            // Create single-element collection for this item (focus)
            let single_item_collection = vec![item.clone()];

            // Create nested context for this iteration
            let mut iteration_context = EvaluationContext::new(
                crate::core::Collection::from(single_item_collection.clone()),
                context.model_provider().clone(),
                context.terminology_provider().clone(),
                context.trace_provider(),
            )
            .await;

            // Set lambda variables: $this = single item, $index = current index, $total = input length
            iteration_context.set_system_this(item.clone());
            iteration_context.set_system_index(index as i64);
            iteration_context.set_system_total(input.len() as i64);

            // Evaluate criteria expression in the iteration context
            let result = evaluator
                .evaluate(criteria_expr, &iteration_context)
                .await?;

            // Check if the result is truthy
            // If the result is empty or contains a falsy value, return false
            if result.value.is_empty() {
                return Ok(EvaluationResult {
                    value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                });
            }

            // Check if result is truthy using the same logic as where function
            if !is_truthy(&result.value) {
                return Ok(EvaluationResult {
                    value: crate::core::Collection::single(FhirPathValue::boolean(false)),
                });
            }
        }

        // All criteria evaluations were true
        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(true)),
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
