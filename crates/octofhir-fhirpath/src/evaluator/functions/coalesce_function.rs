//! Coalesce function implementation
//!
//! The coalesce function returns the first non-empty collection from arguments.
//! Short-circuit evaluation (later arguments not evaluated if earlier ones are non-empty).
//! Useful for providing fallback options.
//! Syntax: coalesce(value1, value2, ..., valueN)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Coalesce function evaluator
pub struct CoalesceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl CoalesceFunctionEvaluator {
    /// Create a new coalesce function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "coalesce".to_string(),
                description: "Returns first non-empty collection from arguments. Short-circuit evaluation (later arguments not evaluated if earlier ones are non-empty). Useful for providing fallback options.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "value".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Value expressions to evaluate in order".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: None, // Unlimited parameters
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Combining,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for CoalesceFunctionEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "coalesce function requires at least one argument".to_string(),
            ));
        }

        // Evaluate each argument in order with short-circuit evaluation
        for arg_expr in &args {
            // Evaluate the argument expression
            let result = evaluator.evaluate(arg_expr, context).await?;

            // If result is non-empty, return it immediately (short-circuit)
            if !result.value.is_empty() {
                return Ok(EvaluationResult {
                    value: result.value,
                });
            }
        }

        // All arguments evaluated to empty, return empty collection
        Ok(EvaluationResult {
            value: crate::core::Collection::empty(),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
