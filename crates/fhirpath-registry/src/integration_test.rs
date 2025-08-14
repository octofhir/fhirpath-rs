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

//! Integration tests demonstrating lambda function functionality

#[cfg(test)]
mod tests {
    use crate::unified_implementations::aggregates::EnhancedAggregateFunction;
    use crate::expression_argument::{ExpressionArgument, VariableScope};
    use crate::function::{EvaluationContext, FunctionError, FunctionResult};
    use crate::lambda_function::{LambdaEvaluationContext, LambdaFhirPathFunction};
    use octofhir_fhirpath_ast::{BinaryOpData, BinaryOperator, ExpressionNode};
    use octofhir_fhirpath_model::FhirPathValue;
    use std::str::FromStr;

    /// Test lambda evaluator that handles basic variable substitution and arithmetic
    struct TestLambdaEvaluator;

    impl TestLambdaEvaluator {
        /// Evaluate an expression with variable scope
        async fn evaluate(
            &self,
            expr: &ExpressionNode,
            scope: &VariableScope,
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            match expr {
                ExpressionNode::Variable(name) => {
                    scope
                        .get_owned(name)
                        .ok_or_else(|| FunctionError::EvaluationError {
                            name: "test_evaluator".to_string(),
                            message: format!("Variable '{}' not found", name),
                        })
                }

                ExpressionNode::BinaryOp(data) => {
                    let left_val = self.evaluate(&data.left, scope, _context).await?;
                    let right_val = self.evaluate(&data.right, scope, _context).await?;

                    match data.op {
                        BinaryOperator::Add => self.add_values(&left_val, &right_val),
                        BinaryOperator::Subtract => self.subtract_values(&left_val, &right_val),
                        BinaryOperator::Multiply => self.multiply_values(&left_val, &right_val),
                        _ => Err(FunctionError::EvaluationError {
                            name: "test_evaluator".to_string(),
                            message: format!("Unsupported operator: {:?}", data.op),
                        }),
                    }
                }

                ExpressionNode::Literal(literal) => match literal {
                    octofhir_fhirpath_ast::LiteralValue::Integer(n) => {
                        Ok(FhirPathValue::Integer(*n))
                    }
                    octofhir_fhirpath_ast::LiteralValue::Decimal(d) => {
                        match rust_decimal::Decimal::from_str(d) {
                            Ok(decimal) => Ok(FhirPathValue::Decimal(decimal)),
                            Err(_) => Err(FunctionError::EvaluationError {
                                name: "test_evaluator".to_string(),
                                message: format!("Invalid decimal: {}", d),
                            }),
                        }
                    }
                    octofhir_fhirpath_ast::LiteralValue::String(s) => {
                        Ok(FhirPathValue::String(s.clone().into()))
                    }
                    octofhir_fhirpath_ast::LiteralValue::Boolean(b) => {
                        Ok(FhirPathValue::Boolean(*b))
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: "test_evaluator".to_string(),
                        message: "Unsupported literal type".to_string(),
                    }),
                },

                _ => Err(FunctionError::EvaluationError {
                    name: "test_evaluator".to_string(),
                    message: "Unsupported expression type".to_string(),
                }),
            }
        }

        fn add_values(
            &self,
            left: &FhirPathValue,
            right: &FhirPathValue,
        ) -> FunctionResult<FhirPathValue> {
            use octofhir_fhirpath_model::FhirPathValue::*;

            match (left, right) {
                (Empty, val) | (val, Empty) => Ok(val.clone()),
                (Integer(l), Integer(r)) => Ok(Integer(l + r)),
                (Decimal(l), Decimal(r)) => Ok(Decimal(*l + *r)),
                (Integer(l), Decimal(r)) => Ok(Decimal(rust_decimal::Decimal::from(*l) + *r)),
                (Decimal(l), Integer(r)) => Ok(Decimal(*l + rust_decimal::Decimal::from(*r))),
                _ => Err(FunctionError::EvaluationError {
                    name: "test_evaluator".to_string(),
                    message: format!("Cannot add {} and {}", left.type_name(), right.type_name()),
                }),
            }
        }

        fn subtract_values(
            &self,
            left: &FhirPathValue,
            right: &FhirPathValue,
        ) -> FunctionResult<FhirPathValue> {
            use octofhir_fhirpath_model::FhirPathValue::*;

            match (left, right) {
                (Integer(l), Integer(r)) => Ok(Integer(l - r)),
                (Decimal(l), Decimal(r)) => Ok(Decimal(*l - *r)),
                (Integer(l), Decimal(r)) => Ok(Decimal(rust_decimal::Decimal::from(*l) - *r)),
                (Decimal(l), Integer(r)) => Ok(Decimal(*l - rust_decimal::Decimal::from(*r))),
                _ => Err(FunctionError::EvaluationError {
                    name: "test_evaluator".to_string(),
                    message: format!(
                        "Cannot subtract {} from {}",
                        right.type_name(),
                        left.type_name()
                    ),
                }),
            }
        }

        fn multiply_values(
            &self,
            left: &FhirPathValue,
            right: &FhirPathValue,
        ) -> FunctionResult<FhirPathValue> {
            use octofhir_fhirpath_model::FhirPathValue::*;

            match (left, right) {
                (Empty, _) | (_, Empty) => Ok(Empty),
                (Integer(l), Integer(r)) => Ok(Integer(l * r)),
                (Decimal(l), Decimal(r)) => Ok(Decimal(*l * *r)),
                (Integer(l), Decimal(r)) => Ok(Decimal(rust_decimal::Decimal::from(*l) * *r)),
                (Decimal(l), Integer(r)) => Ok(Decimal(*l * rust_decimal::Decimal::from(*r))),
                _ => Err(FunctionError::EvaluationError {
                    name: "test_evaluator".to_string(),
                    message: format!(
                        "Cannot multiply {} and {}",
                        left.type_name(),
                        right.type_name()
                    ),
                }),
            }
        }
    }

    #[tokio::test]
    async fn test_aggregate_with_lambda_expressions() {
        let func = EnhancedAggregateFunction::new();
        let evaluator = TestLambdaEvaluator;

        // Test: (1|2|3).aggregate($this + $total, 0)
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);

        let context = EvaluationContext::new(collection);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, scope, ctx| Box::pin(evaluator.evaluate(expr, scope, ctx)),
        };

        // Create $this + $total expression
        let this_var = ExpressionNode::Variable("this".to_string());
        let total_var = ExpressionNode::Variable("total".to_string());
        let add_expr = ExpressionNode::BinaryOp(BinaryOpData {
            op: BinaryOperator::Add,
            left: Box::new(this_var),
            right: Box::new(total_var),
        });

        // Arguments: aggregator expression + initial value
        let args = vec![
            ExpressionArgument::expression(add_expr),
            ExpressionArgument::value(FhirPathValue::Integer(0)),
        ];

        let result = func
            .evaluate_with_expressions(&args, &lambda_context)
            .await
            .unwrap();

        // Should return [6]: 0 + 1 = 1, 1 + 2 = 3, 3 + 3 = 6
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1, "Expected single result");
                if let Some(FhirPathValue::Integer(n)) = items.get(0) {
                    assert_eq!(*n, 6, "Expected sum to be 6");
                } else {
                    panic!("Expected integer result, got: {:?}", items.get(0));
                }
            }
            _ => panic!("Expected collection result, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_aggregate_with_different_initial_value() {
        let func = EnhancedAggregateFunction::new();
        let evaluator = TestLambdaEvaluator;

        // Test: (1|2|3).aggregate($this + $total, 10)
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);

        let context = EvaluationContext::new(collection);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, scope, ctx| Box::pin(evaluator.evaluate(expr, scope, ctx)),
        };

        // Create $this + $total expression
        let this_var = ExpressionNode::Variable("this".to_string());
        let total_var = ExpressionNode::Variable("total".to_string());
        let add_expr = ExpressionNode::BinaryOp(BinaryOpData {
            op: BinaryOperator::Add,
            left: Box::new(this_var),
            right: Box::new(total_var),
        });

        // Arguments: aggregator expression + initial value of 10
        let args = vec![
            ExpressionArgument::expression(add_expr),
            ExpressionArgument::value(FhirPathValue::Integer(10)),
        ];

        let result = func
            .evaluate_with_expressions(&args, &lambda_context)
            .await
            .unwrap();

        // Should return [16]: 10 + 1 = 11, 11 + 2 = 13, 13 + 3 = 16
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1, "Expected single result");
                if let Some(FhirPathValue::Integer(n)) = items.get(0) {
                    assert_eq!(*n, 16, "Expected sum to be 16");
                } else {
                    panic!("Expected integer result, got: {:?}", items.get(0));
                }
            }
            _ => panic!("Expected collection result, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_aggregate_with_multiplication() {
        let func = EnhancedAggregateFunction::new();
        let evaluator = TestLambdaEvaluator;

        // Test: (2|3|4).aggregate($this * $total, 1)
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]);

        let context = EvaluationContext::new(collection);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, scope, ctx| Box::pin(evaluator.evaluate(expr, scope, ctx)),
        };

        // Create $this * $total expression
        let this_var = ExpressionNode::Variable("this".to_string());
        let total_var = ExpressionNode::Variable("total".to_string());
        let mul_expr = ExpressionNode::BinaryOp(BinaryOpData {
            op: BinaryOperator::Multiply,
            left: Box::new(this_var),
            right: Box::new(total_var),
        });

        // Arguments: aggregator expression + initial value of 1
        let args = vec![
            ExpressionArgument::expression(mul_expr),
            ExpressionArgument::value(FhirPathValue::Integer(1)),
        ];

        let result = func
            .evaluate_with_expressions(&args, &lambda_context)
            .await
            .unwrap();

        // Should return [24]: 1 * 2 = 2, 2 * 3 = 6, 6 * 4 = 24
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1, "Expected single result");
                if let Some(FhirPathValue::Integer(n)) = items.get(0) {
                    assert_eq!(*n, 24, "Expected product to be 24");
                } else {
                    panic!("Expected integer result, got: {:?}", items.get(0));
                }
            }
            _ => panic!("Expected collection result, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_aggregate_empty_collection() {
        let func = EnhancedAggregateFunction::new();
        let evaluator = TestLambdaEvaluator;

        // Test empty collection
        let collection = FhirPathValue::Empty;

        let context = EvaluationContext::new(collection);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, scope, ctx| Box::pin(evaluator.evaluate(expr, scope, ctx)),
        };

        // Create $this + $total expression
        let this_var = ExpressionNode::Variable("this".to_string());
        let total_var = ExpressionNode::Variable("total".to_string());
        let add_expr = ExpressionNode::BinaryOp(BinaryOpData {
            op: BinaryOperator::Add,
            left: Box::new(this_var),
            right: Box::new(total_var),
        });

        let args = vec![
            ExpressionArgument::expression(add_expr),
            ExpressionArgument::value(FhirPathValue::Integer(0)),
        ];

        let result = func
            .evaluate_with_expressions(&args, &lambda_context)
            .await
            .unwrap();

        // Should return Empty for empty input
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_aggregate_function_metadata() {
        let func = EnhancedAggregateFunction::new();

        // Verify function properties
        assert_eq!(func.name(), "aggregate");
        assert!(func.supports_lambda_expressions());
        assert_eq!(func.lambda_argument_indices(), vec![0]); // First argument is lambda
        assert_eq!(func.signature().min_arity, 1);
        assert_eq!(func.signature().max_arity, Some(2));
    }
}
