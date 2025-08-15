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

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint}};
use crate::lambda::{LambdaFunction, ExpressionEvaluator};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_ast::ExpressionNode;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Aggregate function - reduces collection to single value using accumulator
#[derive(Debug, Clone)]
pub struct AggregateFunction;

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
        expression: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        // Create a new context with $this and $total variables
        let mut lambda_context = context.clone();
        lambda_context.set_variable("$this".to_string(), current_item.clone());
        lambda_context.set_variable("$total".to_string(), accumulator.clone());
        
        // Set the current item as the focus
        let lambda_context = lambda_context.with_focus(current_item.clone());
        
        // Evaluate the expression in the lambda context
        evaluator.evaluate_expression(expression, &lambda_context).await
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
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            AggregateFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        // This should not be called for lambda functions - they should use evaluate_lambda
        Err(FhirPathError::EvaluationError { 
            message: "AggregateFunction should be called via evaluate_lambda, not evaluate".to_string() 
        })
    }

    fn try_evaluate_sync(&self, _args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
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
        if expressions.len() < 1 || expressions.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: LambdaFunction::identifier(self).to_string(), 
                expected: 2, 
                actual: expressions.len() 
            });
        }

        let aggregation_expr = &expressions[0];
        
        // Evaluate initial value if provided
        let initial_value = if expressions.len() == 2 {
            evaluator.evaluate_expression(&expressions[1], context).await?
        } else {
            FhirPathValue::Empty
        };

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut accumulator = initial_value;

                for item in items.iter() {
                    // Apply aggregation expression in lambda context
                    accumulator = Self::apply_aggregation(&accumulator, item, aggregation_expr, context, evaluator).await?;
                }

                Ok(accumulator)
            },
            single_item => {
                // For single item, apply aggregation once
                Self::apply_aggregation(&initial_value, single_item, aggregation_expr, context, evaluator).await
            }
        }
    }

    fn expected_expression_count(&self) -> usize {
        2 // Expression + initial value (initial value is optional but we expect 2 max)
    }

    fn validate_expressions(&self, expressions: &[ExpressionNode]) -> Result<()> {
        if expressions.len() < 1 || expressions.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 2,
                actual: expressions.len(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aggregate_function_sum() {
        let func = AggregateFunction::new();

        let numbers = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let collection = FhirPathValue::collection(numbers);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());

        let args = vec![FhirPathValue::String("sum".into()), FhirPathValue::Integer(0)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(6));
    }

    #[tokio::test]
    async fn test_aggregate_function_concat() {
        let func = AggregateFunction::new();

        let strings = vec![
            FhirPathValue::String("Hello".into()),
            FhirPathValue::String(" "),
            FhirPathValue::String("World".into()),
        ];
        let collection = FhirPathValue::collection(strings);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());

        let args = vec![FhirPathValue::String("concat".into()), FhirPathValue::String("".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Hello World".into()));
    }

    #[tokio::test]
    async fn test_aggregate_function_count() {
        let func = AggregateFunction::new();

        let items = vec![
            FhirPathValue::String("item1".into()),
            FhirPathValue::String("item2".into()),
            FhirPathValue::String("item3".into()),
        ];
        let collection = FhirPathValue::collection(items);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());

        let args = vec![FhirPathValue::String("count".into()), FhirPathValue::Integer(0)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));
    }

    #[tokio::test]
    async fn test_aggregate_function_mixed_numbers() {
        let func = AggregateFunction::new();

        let numbers = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Decimal(2.5),
            FhirPathValue::Integer(3),
        ];
        let collection = FhirPathValue::collection(numbers);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());

        let args = vec![FhirPathValue::String("sum".into()), FhirPathValue::Integer(0)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Decimal(6.5));
    }

    #[tokio::test]
    async fn test_aggregate_function_single_item() {
        let func = AggregateFunction::new();

        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::Integer(42), registry, model_provider);
        let args = vec![FhirPathValue::String("sum".into()), FhirPathValue::Integer(0)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[tokio::test]
    async fn test_aggregate_function_empty_collection() {
        let func = AggregateFunction::new();

        let collection = FhirPathValue::collection(vec![]);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());

        let args = vec![FhirPathValue::String("sum".into()), FhirPathValue::Integer(10)];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(10)); // Returns initial value
    }

    #[tokio::test]
    async fn test_aggregate_function_sync() {
        let func = AggregateFunction::new();
        
        let numbers = vec![
            FhirPathValue::Integer(5),
            FhirPathValue::Integer(10),
        ];
        let collection = FhirPathValue::collection(numbers);
        let ctx = EvaluationContext::new(collection, std::collections::HashMap::new());
        
        let args = vec![FhirPathValue::String("sum".into()), FhirPathValue::Integer(0)];
        let result = func.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Integer(15));
    }

    #[tokio::test]
    async fn test_aggregate_function_invalid_args() {
        let func = AggregateFunction::new();
        let ctx = EvaluationContext::new(FhirPathValue::Empty, std::collections::HashMap::new());

        // No arguments
        let result = func.evaluate(&[], &ctx).await;
        assert!(result.is_err());

        // Too many arguments
        let args = vec![
            FhirPathValue::String("sum".into()),
            FhirPathValue::Integer(0),
            FhirPathValue::String("extra".into()),
        ];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}