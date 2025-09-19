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
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Aggregate function evaluator
pub struct AggregateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AggregateFunctionEvaluator {
    /// Create a new aggregate function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
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
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
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
impl LazyFunctionEvaluator for AggregateFunctionEvaluator {
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

        // If input is empty, return empty regardless of init
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Handle initialization
        let mut total;
        let start_index;

        if let Some(init_expr) = init_expr {
            // Init value provided - evaluate it and start from index 0
            let init_result = evaluator.evaluate(init_expr, context).await?;
            total = init_result.value.iter().cloned().collect();
            start_index = 0;
        } else {
            // No init value - use first element as init and start from index 1
            total = vec![input[0].clone()];
            start_index = 1;
        }

        // For each item in the input collection (starting from appropriate index)
        for (index, item) in input.iter().enumerate().skip(start_index) {
            // Prepare total value for this iteration
            let total_value = if total.len() == 1 {
                total[0].clone()
            } else if total.len() > 1 {
                FhirPathValue::Collection(Collection::from(total.clone()))
            } else {
                FhirPathValue::Collection(Collection::empty())
            };

            let mut child_context = context.create_child_context(Collection::single(item.clone()));
            child_context.set_variable("$this".to_string(), item.clone());
            child_context.set_variable("$index".to_string(), FhirPathValue::integer(index as i64));
            child_context.set_variable("$total".to_string(), total_value);

            // Evaluate the aggregator expression in child context
            let result = evaluator.evaluate(aggregator_expr, &child_context).await?;

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
