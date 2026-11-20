//! All function implementation
//!
//! The all function returns true if for every element in the input collection, criteria evaluates to true.
//! Otherwise, the result is false. If the input collection is empty, the result is true.
//! Syntax: collection.all(criteria)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// All function evaluator
pub struct AllFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AllFunctionEvaluator {
    /// Create a new all function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
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
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::None,
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
impl LazyFunctionEvaluator for AllFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        self.evaluate_borrowed(input, context, &args, evaluator)
            .await
    }

    async fn evaluate_borrowed(
        &self,
        input: Collection,
        context: &EvaluationContext,
        args: &[ExpressionNode],
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

        let context = crate::evaluator::lambda_hoisting::hoist_into(
            context,
            std::slice::from_ref(criteria_expr),
            input.len(),
            &evaluator,
        )
        .await?;
        let context = Arc::new(context);

        // Evaluate the criteria for each element in the input collection
        for (index, item) in input.iter().enumerate() {
            let iteration_context = context.iteration_context(item, index, Some(input.len()));

            // Evaluate criteria expression in the iteration context
            let result = evaluator
                .evaluate(criteria_expr, &iteration_context)
                .await?;

            // Check if the result is truthy
            // If the result is empty, treat it as false per FHIRPath specification for all()
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
