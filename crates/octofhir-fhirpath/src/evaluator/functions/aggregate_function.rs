//! Aggregate function implementation
//!
//! The aggregate function reduces a collection to a single value by iteratively applying
//! an aggregator expression. It provides $this (current item), $index (0-based), and
//! $total (accumulated result) variables.
//! Syntax: collection.aggregate(aggregator [, init])

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Aggregate function evaluator
pub struct AggregateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AggregateFunctionEvaluator {
    /// Create a new aggregate function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "aggregate".to_string(),
                description:
                    "Reduces a collection to a single value using an aggregator expression"
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "aggregator".to_string(),
                            parameter_type: vec!["Expression".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Expression that combines $total and $this".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "init".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Initial value for the aggregation".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(2),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Aggregate,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for AggregateFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "aggregate function requires an aggregator expression".to_string(),
            ));
        }

        if args.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "aggregate function takes at most 2 arguments".to_string(),
            ));
        }

        let aggregator_expr = &args[0];
        let init_expr = args.get(1); // optional init value

        // Evaluate init value if provided, otherwise start with empty collection
        let mut total = if let Some(init_expr) = init_expr {
            let init_result = evaluator.evaluate(init_expr, context).await?;
            init_result.value.iter().cloned().collect()
        } else {
            vec![]
        };

        // If input is empty and init is provided, return the init value
        if input.is_empty() && init_expr.is_some() {
            return Ok(EvaluationResult {
                value: Collection::from(total),
            });
        }

        // If input is empty and no init, return empty
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // For each item in the input collection, evaluate the aggregator expression
        for (index, item) in input.iter().enumerate() {
            // Clone the context to avoid mutation issues
            let mut iteration_context = context.clone();

            // Set the special aggregate variables
            iteration_context.set_user_variable("$this".to_string(), item.clone())?;
            iteration_context.set_user_variable("$index".to_string(), FhirPathValue::integer(index as i64))?;

            // Set $total - this is the accumulated value from previous iterations
            if total.len() == 1 {
                iteration_context.set_user_variable("$total".to_string(), total[0].clone())?;
            } else if total.len() > 1 {
                // Multiple values in total - use the first one (shouldn't happen normally)
                iteration_context.set_user_variable("$total".to_string(), total[0].clone())?;
            }
            // If total is empty, we don't set $total variable - it will be undefined

            // Evaluate the aggregator expression in this context
            let result = evaluator.evaluate(aggregator_expr, &iteration_context).await?;

            // Update total with the result - this becomes the new accumulator value
            total = result.value.iter().cloned().collect();
        }

        Ok(EvaluationResult {
            value: Collection::from(total),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
