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

//! Enhanced aggregate() function with proper lambda expression support

use crate::expression_argument::{ExpressionArgument, VariableScope};
use crate::lambda_function::{LambdaEvaluationContext, LambdaFhirPathFunction};
use crate::function::{FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Enhanced aggregate function with lambda expression support
/// 
/// Implements the aggregate() function correctly by:
/// 1. Receiving the aggregator expression as an AST node (not pre-evaluated)
/// 2. Evaluating the expression for each collection item with proper variable scoping
/// 3. Supporting $this, $total, and $index variables
pub struct EnhancedAggregateFunction {
    signature: FunctionSignature,
}

impl EnhancedAggregateFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature {
            name: "aggregate".to_string(),
            min_arity: 1,
            max_arity: Some(2),
            parameters: vec![
                ParameterInfo::required("aggregator", TypeInfo::Any), // Expression, not pre-evaluated
                ParameterInfo::optional("init", TypeInfo::Any),       // Optional initial value
            ],
            return_type: TypeInfo::Any,
        };

        Self { signature }
    }
}

#[async_trait]
impl LambdaFhirPathFunction for EnhancedAggregateFunction {
    fn name(&self) -> &str {
        "aggregate"
    }

    fn human_friendly_name(&self) -> &str {
        "Aggregate Function"
    }

    fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    fn lambda_argument_indices(&self) -> Vec<usize> {
        vec![0] // First argument (aggregator expression) should not be pre-evaluated
    }

    async fn evaluate_with_expressions(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments
        if args.is_empty() || args.len() > 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(2),
                actual: args.len(),
            });
        }

        // First argument must be an expression (the aggregator)
        let aggregator_expr = args[0].as_expression().ok_or_else(|| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "First argument must be an expression".to_string(),
            }
        })?;

        // Get the input collection
        let collection = match &context.context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single_item => vec![single_item.clone()].into(),
        };

        // Get initial value (second argument, if provided)
        let mut total = if args.len() > 1 {
            match &args[1] {
                ExpressionArgument::Value(value) => value.clone(),
                ExpressionArgument::Expression(expr) => {
                    // If second argument is also an expression, evaluate it in current context
                    let scope = VariableScope::from_variables(context.context.variables.clone());
                    (context.evaluator)(expr, &scope, context.context).await?
                }
            }
        } else {
            FhirPathValue::Empty
        };

        // Iterate through collection items and aggregate
        for (index, item) in collection.iter().enumerate() {
            // Create variable scope with $this, $total, and $index
            let scope = VariableScope::from_variables(context.context.variables.clone())
                .with_this(item.clone())
                .with_total(total.clone())
                .with_index(index as i32);

            // Evaluate the aggregator expression with the current scope
            let aggregated_value = (context.evaluator)(aggregator_expr, &scope, context.context).await?;
            
            // Update total with the aggregated value
            total = aggregated_value;
        }

        // Return result as a single-item collection (following FHIRPath spec)
        Ok(FhirPathValue::collection(vec![total]))
    }

    fn documentation(&self) -> &str {
        r#"
Performs general-purpose aggregation using an aggregator expression for each element.

Within the aggregator expression, the following variables are available:
- $this: The current element being processed
- $total: The accumulated total so far  
- $index: The 0-based index of the current element

Examples:
- (1|2|3).aggregate($this + $total, 0) returns [6]
- items.aggregate(iif($total.empty(), $this, iif($this < $total, $this, $total))) finds minimum
- collection.aggregate($total.combine($this)) flattens collections

The function returns a single-element collection containing the final aggregated value.
"#
    }

    fn is_pure(&self) -> bool {
        // The aggregate function itself is pure, but the aggregator expression may not be
        false
    }

    fn supports_lambda_expressions(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_argument::ExpressionArgument;
    use crate::lambda_function::{LambdaEvaluationContext, create_simple_lambda_evaluator};
    use octofhir_fhirpath_ast::{ExpressionNode, BinaryOperator, BinaryOpData};
    
    /// Create a test lambda evaluator that handles basic expressions
    fn create_test_lambda_evaluator() -> Box<crate::lambda_function::LambdaExpressionEvaluator> {
        Box::new(|expr, scope, _context| {
            Box::pin(async move {
                // Simple test implementation for $this + $total expressions
                match expr {
                    ExpressionNode::BinaryOp(BinaryOpData { op: BinaryOperator::Add, left, right }) => {
                        // Handle $this + $total pattern
                        let left_val = match left.as_ref() {
                            ExpressionNode::Variable(name) if name == "this" => {
                                scope.get("this").cloned().unwrap_or(FhirPathValue::Empty)
                            }
                            ExpressionNode::Variable(name) if name == "total" => {
                                scope.get("total").cloned().unwrap_or(FhirPathValue::Empty)
                            }
                            _ => FhirPathValue::Empty,
                        };
                        
                        let right_val = match right.as_ref() {
                            ExpressionNode::Variable(name) if name == "this" => {
                                scope.get("this").cloned().unwrap_or(FhirPathValue::Empty)
                            }
                            ExpressionNode::Variable(name) if name == "total" => {
                                scope.get("total").cloned().unwrap_or(FhirPathValue::Empty)
                            }
                            _ => FhirPathValue::Empty,
                        };
                        
                        // Add the values
                        match (&left_val, &right_val) {
                            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                                Ok(FhirPathValue::Integer(a + b))
                            }
                            (FhirPathValue::Empty, val) | (val, FhirPathValue::Empty) => {
                                Ok(val.clone())
                            }
                            _ => Err(FunctionError::EvaluationError {
                                name: "test_evaluator".to_string(),
                                message: "Cannot add non-numeric values in test".to_string(),
                            }),
                        }
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: "test_evaluator".to_string(),
                        message: "Unsupported expression in test evaluator".to_string(),
                    }),
                }
            })
        })
    }

    #[tokio::test] 
    async fn test_enhanced_aggregate_basic() {
        let func = EnhancedAggregateFunction::new();
        
        // Test collection: [1, 2, 3]
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2), 
            FhirPathValue::Integer(3),
        ]);
        
        let eval_context = EvaluationContext::new(collection);
        let evaluator = create_test_lambda_evaluator();
        let lambda_context = LambdaEvaluationContext {
            context: &eval_context,
            evaluator: evaluator.as_ref(),
        };
        
        // Create expression: $this + $total
        let this_var = ExpressionNode::Variable("this".to_string());
        let total_var = ExpressionNode::Variable("total".to_string());
        let add_expr = ExpressionNode::BinaryOp(BinaryOpData {
            op: BinaryOperator::Add,
            left: Box::new(this_var),
            right: Box::new(total_var),
        });
        
        let args = vec![
            ExpressionArgument::expression(add_expr),
            ExpressionArgument::value(FhirPathValue::Integer(0)), // Initial value
        ];
        
        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();
        
        // Should return [6] (0 + 1 = 1, 1 + 2 = 3, 3 + 3 = 6)
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Integer(n)) = items.get(0) {
                    assert_eq!(*n, 6);
                } else {
                    panic!("Expected integer result");
                }
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[test]
    fn test_metadata() {
        let func = EnhancedAggregateFunction::new();
        assert_eq!(func.name(), "aggregate");
        assert_eq!(func.signature().min_arity, 1);
        assert_eq!(func.signature().max_arity, Some(2));
        assert!(func.supports_lambda_expressions());
        assert_eq!(func.lambda_argument_indices(), vec![0]);
    }
}