//! Iif (conditional) function implementation
//!
//! The iif function returns one of two values based on a boolean condition.
//! Syntax: iif(condition, trueResult, falseResult)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Iif (conditional) function evaluator
pub struct IifFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl IifFunctionEvaluator {
    /// Create a new iif function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "iif".to_string(),
                description: "Returns one of two values based on a boolean condition".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "condition".to_string(),
                            parameter_type: vec!["Expression".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Boolean condition to evaluate".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "trueResult".to_string(),
                            parameter_type: vec!["Expression".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Value to return if condition is true".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "falseResult".to_string(),
                            parameter_type: vec!["Expression".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Value to return if condition is false".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 2,
                    max_params: Some(3),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Custom,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Convert a FHIRPath value to boolean for condition evaluation
    fn to_boolean(value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Boolean(b, _, _) => *b,
            FhirPathValue::Integer(i, _, _) => *i != 0,
            FhirPathValue::Decimal(d, _, _) => !d.is_zero(),
            FhirPathValue::String(s, _, _) => !s.is_empty(),
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for IifFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() < 2 || args.len() > 3 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "iif function requires 2 or 3 arguments (condition, trueResult [, falseResult])"
                    .to_string(),
            ));
        }

        // Check that input collection has at most one item
        if input.len() > 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "iif function can only be called on a singleton collection",
            ));
        }

        let condition_expr = &args[0];
        let true_expr = &args[1];
        let false_expr = args.get(2);

        // Create a child context for evaluation that sets input to the function receiver
        // and $this to the receiver item when singleton
        let evaluation_context = {
            let child_context =
                context.create_child_context(crate::core::Collection::from(input.clone()));
            if input.len() == 1 {
                child_context.set_variable("$this".to_string(), input[0].clone());
            }
            child_context
        };

        // Evaluate the condition
        let condition_result = evaluator
            .evaluate(condition_expr, &evaluation_context)
            .await?;
        let condition_values: Vec<FhirPathValue> = condition_result.value.iter().cloned().collect();

        // Validate condition type - should be boolean in strict mode
        if !condition_values.is_empty() && condition_values.len() == 1 {
            match &condition_values[0] {
                FhirPathValue::Boolean(_, _, _) => {
                    // Valid boolean condition, continue
                }
                _ => {
                    // Non-boolean condition - this should be an error in strict mode
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "iif function condition must be boolean",
                    ));
                }
            }
        }

        // Determine if condition is true
        let is_true = if condition_values.is_empty() {
            false // Empty collection is falsy
        } else if condition_values.len() == 1 {
            Self::to_boolean(&condition_values[0])
        } else {
            true // Non-empty collection with multiple values is truthy
        };

        // Return the appropriate result
        if is_true {
            // Evaluate and return true expression
            let result = evaluator.evaluate(true_expr, &evaluation_context).await?;
            Ok(result)
        } else if let Some(false_expr) = false_expr {
            // Evaluate and return false expression
            let result = evaluator.evaluate(false_expr, &evaluation_context).await?;
            Ok(result)
        } else {
            // No false expression provided, return empty
            Ok(EvaluationResult {
                value: Collection::empty(),
            })
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
