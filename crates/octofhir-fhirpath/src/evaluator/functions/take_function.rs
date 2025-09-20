//! Take function implementation
//!
//! The take function returns a collection containing the first num items in the input collection,
//! or less if there are less than num items.
//! Syntax: collection.take(num)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Take function evaluator
pub struct TakeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TakeFunctionEvaluator {
    /// Create a new take function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "take".to_string(),
                description: "Returns a collection containing the first num items in the input collection, or less if there are less than num items. If num is less than or equal to 0, or if the input collection is empty, take returns an empty collection.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "num".to_string(),
                            parameter_type: vec!["Integer".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Number of items to take from the beginning".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Subsetting,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for TakeFunctionEvaluator {
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
                "take function requires exactly one argument (num)".to_string(),
            ));
        }

        // If input is empty, return empty collection
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let num_expr = &args[0];

        // Evaluate the num argument
        let num_result = evaluator.evaluate(num_expr, context).await?;

        if num_result.value.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "take argument cannot be empty".to_string(),
            ));
        }

        if num_result.value.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "take argument must be a single value".to_string(),
            ));
        }

        let take_value = &num_result.value[0];
        let take_num = match take_value {
            FhirPathValue::Integer(n, _, _) => *n,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "take argument must be an integer".to_string(),
                ));
            }
        };

        // If num is less than or equal to 0, return empty collection
        if take_num <= 0 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // Return the first num items (or all items if num is greater than length)
        let result_items: Vec<FhirPathValue> = input.into_iter().take(take_num as usize).collect();
        Ok(EvaluationResult {
            value: crate::core::Collection::from(result_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
