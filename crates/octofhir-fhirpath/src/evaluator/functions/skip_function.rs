//! Skip function implementation
//!
//! The skip function returns a collection containing all but the first num items in the input collection.
//! Syntax: collection.skip(num)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Skip function evaluator
pub struct SkipFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SkipFunctionEvaluator {
    /// Create a new skip function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "skip".to_string(),
                description: "Returns a collection containing all but the first num items in the input collection.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "num".to_string(),
                            parameter_type: vec!["Integer".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Number of items to skip from the beginning".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
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
impl FunctionEvaluator for SkipFunctionEvaluator {
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
                "skip function requires exactly one argument (num)".to_string(),
            ));
        }

        let num_expr = &args[0];

        // Evaluate the num argument
        let num_result = evaluator.evaluate(num_expr, context).await?;

        if num_result.value.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "skip argument cannot be empty".to_string(),
            ));
        }

        if num_result.value.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "skip argument must be a single value".to_string(),
            ));
        }

        let skip_value = &num_result.value[0];
        let skip_num = match skip_value {
            FhirPathValue::Integer(n, _, _) => *n,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    "skip argument must be an integer".to_string(),
                ));
            }
        };

        // If num <= 0, return the input collection as is
        if skip_num <= 0 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(input),
            });
        }

        // Skip the first 'num' items
        let result_items: Vec<FhirPathValue> = input.into_iter().skip(skip_num as usize).collect();
        Ok(EvaluationResult {
            value: crate::core::Collection::from(result_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
