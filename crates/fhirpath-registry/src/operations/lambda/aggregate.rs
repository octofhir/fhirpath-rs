// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Aggregate function implementation - reduces collection to single value

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    lambda::{ExpressionEvaluator, LambdaContextBuilder, LambdaFunction},
    metadata::{MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Aggregate function - reduces collection to single value using accumulator
#[derive(Debug, Clone)]
pub struct AggregateFunction;

impl Default for AggregateFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AggregateFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("aggregate", OperationType::Function)
            .description("Reduces a collection to a single value using an accumulator and aggregation expression")
            .returns(TypeConstraint::Any)
            .example("children().aggregate($total + $this.value, 0)")
            .example("Bundle.entry.aggregate($count + 1, 0)")
            .example("name.given.aggregate($result + ' ' + $this, '')")
            .build()
    }

    /// Evaluate the aggregation expression in lambda context with $this and $total variables
    async fn apply_aggregation(
        accumulator: &FhirPathValue,
        current_item: &FhirPathValue,
        index: usize,
        expression: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        // Create lambda context using LambdaContextBuilder
        let lambda_context = LambdaContextBuilder::new(context)
            .with_this(current_item.clone())
            .with_total(accumulator.clone())
            .with_index(index as i64)
            .with_input(current_item.clone())
            .build();

        // Evaluate the expression in the lambda context
        let result = evaluator
            .evaluate_expression(expression, &lambda_context)
            .await;

        // Debug logging to understand what's happening
        #[cfg(debug_assertions)]
        {
            log::debug!(
                "Aggregate: $this={current_item:?}, $total={accumulator:?}, $index={index}, expr result={result:?}"
            );
        }

        result
    }
}

#[async_trait]
impl FhirPathOperation for AggregateFunction {
    fn identifier(&self) -> &str {
        "aggregate"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AggregateFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle pre-evaluated arguments (regular function mode)
        // This is a simplified version that works with pre-evaluated values
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: FhirPathOperation::identifier(self).to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // For regular function mode, we can't properly handle lambda expressions
        // This is a limitation of the pre-evaluation approach
        // The aggregation expression should be a simple operation identifier
        let _aggregation_op = &args[0];

        let initial_value = if args.len() == 2 {
            args[1].clone()
        } else {
            FhirPathValue::Empty
        };

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Simple aggregation without lambda expressions
                // This is a basic implementation for compatibility
                let mut accumulator = initial_value;

                for item in items.iter() {
                    // For regular mode, we can only do basic operations
                    // This is why lambda mode is preferred for aggregate
                    match item {
                        FhirPathValue::Integer(n) => {
                            match &accumulator {
                                FhirPathValue::Integer(acc) => {
                                    accumulator = FhirPathValue::Integer(acc + n);
                                }
                                FhirPathValue::Empty => {
                                    accumulator = FhirPathValue::Integer(*n);
                                }
                                _ => {
                                    // Can't aggregate mixed types in simple mode
                                    return Err(FhirPathError::EvaluationError {
                                        message: "Cannot aggregate mixed types in regular function mode. Use lambda mode for complex aggregations.".to_string()
                                    });
                                }
                            }
                        }
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                                message: "Regular aggregate function only supports integer aggregation. Use lambda mode for complex aggregations.".to_string()
                            });
                        }
                    }
                }

                Ok(accumulator)
            }
            single_item => {
                // For single item, return the initial value combined with the item
                match (single_item, &initial_value) {
                    (FhirPathValue::Integer(n), FhirPathValue::Integer(init)) => {
                        Ok(FhirPathValue::Integer(init + n))
                    }
                    (item, FhirPathValue::Empty) => Ok(item.clone()),
                    _ => Ok(initial_value),
                }
            }
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Lambda functions don't support sync evaluation
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl LambdaFunction for AggregateFunction {
    fn identifier(&self) -> &str {
        "aggregate"
    }

    async fn evaluate_lambda(
        &self,
        expressions: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if expressions.is_empty() || expressions.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 2,
                actual: expressions.len(),
            });
        }

        let aggregation_expr = &expressions[0];

        // Evaluate initial value if provided
        let initial_value = if expressions.len() == 2 {
            evaluator
                .evaluate_expression(&expressions[1], context)
                .await?
        } else {
            FhirPathValue::Empty
        };

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut accumulator = initial_value;

                for (index, item) in items.iter().enumerate() {
                    // Apply aggregation expression in lambda context
                    accumulator = Self::apply_aggregation(
                        &accumulator,
                        item,
                        index,
                        aggregation_expr,
                        context,
                        evaluator,
                    )
                    .await?;
                }

                Ok(accumulator)
            }
            single_item => {
                // For single item, apply aggregation once
                Self::apply_aggregation(
                    &initial_value,
                    single_item,
                    0,
                    aggregation_expr,
                    context,
                    evaluator,
                )
                .await
            }
        }
    }

    fn expected_expression_count(&self) -> usize {
        2 // Expression + initial value (initial value is optional but we expect 2 max)
    }

    fn validate_expressions(&self, expressions: &[ExpressionNode]) -> Result<()> {
        if expressions.is_empty() || expressions.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 2,
                actual: expressions.len(),
            });
        }
        Ok(())
    }
}
