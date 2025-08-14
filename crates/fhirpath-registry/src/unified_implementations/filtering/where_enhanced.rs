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

//! Enhanced where() function with proper lambda expression support

use crate::expression_argument::{ExpressionArgument, VariableScope};
use crate::lambda_function::{LambdaEvaluationContext, LambdaFhirPathFunction};
use crate::function::{FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Enhanced where() function with lambda expression support
/// 
/// Implements the where() function correctly by:
/// 1. Receiving the criteria expression as an AST node (not pre-evaluated)
/// 2. Evaluating the expression for each collection item with proper variable scoping
/// 3. Supporting $this variable for accessing the current item
pub struct EnhancedWhereFunction {
    signature: FunctionSignature,
}

impl EnhancedWhereFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature {
            name: "where".to_string(),
            min_arity: 1,
            max_arity: Some(1),
            parameters: vec![
                ParameterInfo::required("criteria", TypeInfo::Any), // Expression, not pre-evaluated
            ],
            return_type: TypeInfo::Any,
        };

        Self { signature }
    }
}

#[async_trait]
impl LambdaFhirPathFunction for EnhancedWhereFunction {
    fn name(&self) -> &str {
        "where"
    }

    fn human_friendly_name(&self) -> &str {
        "Where Function"
    }

    fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    fn lambda_argument_indices(&self) -> Vec<usize> {
        vec![0] // First argument (criteria expression) should not be pre-evaluated
    }

    async fn evaluate_with_expressions(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        // First argument must be an expression (the criteria)
        let criteria_expr = args[0].as_expression().ok_or_else(|| {
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

        let mut results = Vec::new();

        // Iterate through collection items and filter based on criteria
        for (index, item) in collection.iter().enumerate() {
            // Create variable scope with $this for current item
            let scope = VariableScope::from_variables(context.context.variables.clone())
                .with_this(item.clone())
                .with_index(index as i32);

            // Evaluate the criteria expression with the current scope
            let criteria_result = (context.evaluator)(criteria_expr, &scope, context.context).await?;
            
            // Check if criteria evaluates to true
            if self.is_truthy(&criteria_result) {
                results.push(item.clone());
            }
        }

        // Return filtered collection
        Ok(FhirPathValue::collection(results))
    }

    fn documentation(&self) -> &str {
        r#"
Filters a collection based on a boolean criteria expression evaluated for each item.

Within the criteria expression, the following variables are available:
- $this: The current item being evaluated
- $index: The 0-based index of the current item

Examples:
- Patient.telecom.where($this.use = 'official') filters telecom entries with official use
- collection.where($this.length() > 3) filters items longer than 3 characters
- items.where($this.value > 10) filters items with value greater than 10

The function returns a collection containing only items for which the criteria evaluates to true.
Items for which the criteria evaluates to false, empty, or error are excluded.
"#
    }

    fn is_pure(&self) -> bool {
        // The where function itself is pure, but the criteria expression may not be
        false
    }

    fn supports_lambda_expressions(&self) -> bool {
        true
    }
}

impl EnhancedWhereFunction {
    /// Check if a FhirPathValue is considered truthy
    fn is_truthy(&self, value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    false
                } else if items.len() == 1 {
                    // Single-item collection - check the item
                    if let Some(item) = items.get(0) {
                        self.is_truthy(item)
                    } else {
                        false
                    }
                } else {
                    // Multi-item collection is truthy
                    true
                }
            }
            FhirPathValue::String(s) => !s.is_empty(),
            FhirPathValue::Integer(i) => *i != 0,
            FhirPathValue::Decimal(d) => !d.is_zero(),
            _ => true, // Other non-empty values are truthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_argument::ExpressionArgument;
    use crate::lambda_function::{LambdaEvaluationContext, create_simple_lambda_evaluator};
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_ast::{ExpressionNode, BinaryOperator, BinaryOpData};
    
    /// Create a test lambda evaluator that handles basic expressions
    fn create_test_lambda_evaluator() -> Box<crate::lambda_function::LambdaExpressionEvaluator> {
        Box::new(|expr, scope, _context| {
            Box::pin(async move {
                match expr {
                    ExpressionNode::Variable(name) if name == "this" => {
                        Ok(scope.get("this").cloned().unwrap_or(FhirPathValue::Empty))
                    }
                    ExpressionNode::BinaryOp(BinaryOpData { op: BinaryOperator::GreaterThan, left, right }) => {
                        // Handle $this.length() > 3 patterns
                        let left_val = match left.as_ref() {
                            ExpressionNode::FunctionCall(call) if call.name == "length" => {
                                // Simulate length() function on $this
                                let this_val = scope.get("this").cloned().unwrap_or(FhirPathValue::Empty);
                                match this_val {
                                    FhirPathValue::String(s) => FhirPathValue::Integer(s.len() as i64),
                                    FhirPathValue::Collection(items) => FhirPathValue::Integer(items.len() as i64),
                                    _ => FhirPathValue::Empty,
                                }
                            }
                            _ => FhirPathValue::Empty,
                        };
                        
                        let right_val = match right.as_ref() {
                            ExpressionNode::Literal(lit) => {
                                match lit {
                                    octofhir_fhirpath_ast::LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
                                    _ => FhirPathValue::Empty,
                                }
                            }
                            _ => FhirPathValue::Empty,
                        };
                        
                        // Compare values
                        match (&left_val, &right_val) {
                            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                                Ok(FhirPathValue::Boolean(l > r))
                            }
                            _ => Err(FunctionError::EvaluationError {
                                name: "test_evaluator".to_string(),
                                message: "Cannot compare non-numeric values in test".to_string(),
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
    async fn test_enhanced_where_basic() {
        let func = EnhancedWhereFunction::new();
        
        // Test collection of strings
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("hello".into()),     // length = 5 > 3 ✓
            FhirPathValue::String("hi".into()),        // length = 2 ≤ 3 ✗
            FhirPathValue::String("world".into()),     // length = 5 > 3 ✓
            FhirPathValue::String("a".into()),         // length = 1 ≤ 3 ✗
        ]);
        
        let eval_context = EvaluationContext::new(collection);
        let evaluator = create_test_lambda_evaluator();
        let lambda_context = LambdaEvaluationContext {
            context: &eval_context,
            evaluator: evaluator.as_ref(),
        };
        
        // Create expression: $this.length() > 3
        let this_var = ExpressionNode::Variable("this".to_string());
        let length_call = ExpressionNode::FunctionCall(octofhir_fhirpath_ast::FunctionCallData {
            name: "length".to_string(),
            args: vec![],
            context: Some(Box::new(this_var)),
        });
        let three_literal = ExpressionNode::Literal(octofhir_fhirpath_ast::LiteralValue::Integer(3));
        let gt_expr = ExpressionNode::BinaryOp(BinaryOpData {
            op: BinaryOperator::GreaterThan,
            left: Box::new(length_call),
            right: Box::new(three_literal),
        });
        
        let args = vec![ExpressionArgument::expression(gt_expr)];
        
        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();
        
        // Should return ["hello", "world"] (lengths 5 and 5, both > 3)
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items.get(0), Some(&FhirPathValue::String("hello".into())));
                assert_eq!(items.get(1), Some(&FhirPathValue::String("world".into())));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[test]
    fn test_metadata() {
        let func = EnhancedWhereFunction::new();
        assert_eq!(func.name(), "where");
        assert_eq!(func.signature().min_arity, 1);
        assert_eq!(func.signature().max_arity, Some(1));
        assert!(func.supports_lambda_expressions());
        assert_eq!(func.lambda_argument_indices(), vec![0]);
    }
}