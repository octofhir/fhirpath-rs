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

//! Enhanced iif() function with proper short-circuiting support

use crate::{
    expression_argument::{ExpressionArgument, VariableScope},
    lambda_function::{LambdaEvaluationContext, LambdaFhirPathFunction},
    function::{FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Enhanced iif() function with short-circuiting support
///
/// The iif() function evaluates a condition and returns one of two expressions
/// based on the result. Key features:
/// 1. **Short-circuiting**: Only evaluates the relevant branch (then or else)
/// 2. **Expression arguments**: Then and else branches are expressions, not pre-evaluated
/// 3. **Proper condition handling**: Follows FHIRPath truthiness rules
pub struct EnhancedIifFunction {
    signature: FunctionSignature,
}

impl EnhancedIifFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature {
            name: "iif".to_string(),
            min_arity: 3,
            max_arity: Some(3),
            parameters: vec![
                ParameterInfo::required("condition", TypeInfo::Boolean), // Evaluated normally
                ParameterInfo::required("then", TypeInfo::Any),          // Expression, not pre-evaluated
                ParameterInfo::required("else", TypeInfo::Any),          // Expression, not pre-evaluated
            ],
            return_type: TypeInfo::Any,
        };

        Self { signature }
    }

    /// Check if a value is "truthy" according to FHIRPath rules
    /// Returns None if the value is not a valid boolean condition
    fn evaluate_condition(&self, value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Empty => Some(false), // Empty is treated as false
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Some(false) // Empty collection is false
                } else if items.len() == 1 {
                    // Single-item collection - recurse on the item
                    self.evaluate_condition(items.get(0)?)
                } else {
                    None // Multi-item collections are invalid conditions
                }
            }
            _ => None, // Non-boolean values are invalid conditions
        }
    }
}

#[async_trait]
impl LambdaFhirPathFunction for EnhancedIifFunction {
    fn name(&self) -> &str {
        "iif"
    }

    fn human_friendly_name(&self) -> &str {
        "Conditional Expression (If-Then-Else)"
    }

    fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    fn lambda_argument_indices(&self) -> Vec<usize> {
        vec![1, 2] // Then and else expressions (indices 1 and 2) should not be pre-evaluated
    }

    async fn evaluate_with_expressions(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments
        if args.len() != 3 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 3,
                max: Some(3),
                actual: args.len(),
            });
        }

        // First argument (condition) should be pre-evaluated
        let condition_value = args[0].as_value().ok_or_else(|| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Condition argument should be pre-evaluated".to_string(),
            }
        })?;

        // Then and else arguments should be expressions
        let then_expr = args[1].as_expression().ok_or_else(|| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Then argument must be an expression".to_string(),
            }
        })?;

        let else_expr = args[2].as_expression().ok_or_else(|| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Else argument must be an expression".to_string(),
            }
        })?;

        // Evaluate condition - return empty for invalid conditions
        let is_condition_true = match self.evaluate_condition(condition_value) {
            Some(true) => true,
            Some(false) => false,
            None => {
                // Invalid condition (non-boolean) - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };

        // Create variable scope (no special variables for iif)
        let scope = VariableScope::from_variables(context.context.variables.clone());

        // Short-circuit: evaluate only the relevant branch
        if is_condition_true {
            // Evaluate then expression
            (context.evaluator)(then_expr, &scope, context.context).await
        } else {
            // Evaluate else expression
            (context.evaluator)(else_expr, &scope, context.context).await
        }
    }

    fn documentation(&self) -> &str {
        r#"
Conditional expression that evaluates one of two branches based on a condition.

Syntax: iif(condition, then-expression, else-expression)

The iif() function provides short-circuiting behavior:
- If the condition is truthy, only the then-expression is evaluated
- If the condition is falsy, only the else-expression is evaluated
- This prevents evaluation of potentially expensive or error-prone expressions

FHIRPath truthiness rules:
- true: Boolean true
- false: Boolean false, empty collection, null/empty values
- Collections: Truthy if non-empty and contains at least one truthy value
- Other values: Generally truthy (integers, strings, etc.)

Examples:
- iif(Patient.active, Patient.name, 'Inactive Patient') 
- iif(count() > 0, first(), {})
- iif(exists(), value.toString(), 'No value')

The function ensures type safety and proper error handling while maintaining
optimal performance through expression-level short-circuiting.
"#
    }

    fn is_pure(&self) -> bool {
        // iif is conceptually pure, but the expressions it evaluates may not be
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
    use crate::lambda_function::{LambdaEvaluationContext};
    use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};
    
    #[tokio::test]
    async fn test_iif_with_true_condition() {
        let func = EnhancedIifFunction::new();
        
        let context = crate::function::EvaluationContext::new(FhirPathValue::Empty);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, _scope, _ctx| {
                Box::pin(async move {
                    // Simple evaluator that returns the literal value
                    match expr {
                        ExpressionNode::Literal(LiteralValue::String(s)) => {
                            Ok(FhirPathValue::String(s.clone().into()))
                        }
                        ExpressionNode::Literal(LiteralValue::Integer(n)) => {
                            Ok(FhirPathValue::Integer(*n))
                        }
                        _ => Err(FunctionError::EvaluationError {
                            name: "test_evaluator".to_string(),
                            message: "Unsupported expression".to_string(),
                        }),
                    }
                })
            },
        };
        
        // Test: iif(true, "then", "else") should return "then"
        let args = vec![
            ExpressionArgument::value(FhirPathValue::Boolean(true)),
            ExpressionArgument::expression(ExpressionNode::Literal(LiteralValue::String("then".to_string()))),
            ExpressionArgument::expression(ExpressionNode::Literal(LiteralValue::String("else".to_string()))),
        ];
        
        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("then".into()));
    }

    #[tokio::test]
    async fn test_iif_with_false_condition() {
        let func = EnhancedIifFunction::new();
        
        let context = crate::function::EvaluationContext::new(FhirPathValue::Empty);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, _scope, _ctx| {
                Box::pin(async move {
                    match expr {
                        ExpressionNode::Literal(LiteralValue::String(s)) => {
                            Ok(FhirPathValue::String(s.clone().into()))
                        }
                        ExpressionNode::Literal(LiteralValue::Integer(n)) => {
                            Ok(FhirPathValue::Integer(*n))
                        }
                        _ => Err(FunctionError::EvaluationError {
                            name: "test_evaluator".to_string(),
                            message: "Unsupported expression".to_string(),
                        }),
                    }
                })
            },
        };
        
        // Test: iif(false, "then", "else") should return "else"
        let args = vec![
            ExpressionArgument::value(FhirPathValue::Boolean(false)),
            ExpressionArgument::expression(ExpressionNode::Literal(LiteralValue::String("then".to_string()))),
            ExpressionArgument::expression(ExpressionNode::Literal(LiteralValue::String("else".to_string()))),
        ];
        
        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("else".into()));
    }

    #[tokio::test]
    async fn test_iif_with_empty_condition() {
        let func = EnhancedIifFunction::new();
        
        let context = crate::function::EvaluationContext::new(FhirPathValue::Empty);
        let lambda_context = LambdaEvaluationContext {
            context: &context,
            evaluator: &|expr, _scope, _ctx| {
                Box::pin(async move {
                    match expr {
                        ExpressionNode::Literal(LiteralValue::Integer(n)) => {
                            Ok(FhirPathValue::Integer(*n))
                        }
                        _ => Err(FunctionError::EvaluationError {
                            name: "test_evaluator".to_string(),
                            message: "Unsupported expression".to_string(),
                        }),
                    }
                })
            },
        };
        
        // Test: iif(empty, 42, 99) should return 99 (empty is falsy)
        let args = vec![
            ExpressionArgument::value(FhirPathValue::Empty),
            ExpressionArgument::expression(ExpressionNode::Literal(LiteralValue::Integer(42))),
            ExpressionArgument::expression(ExpressionNode::Literal(LiteralValue::Integer(99))),
        ];
        
        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(99));
    }

    #[test]
    fn test_iif_condition_evaluation() {
        let func = EnhancedIifFunction::new();
        
        // Test Boolean values
        assert_eq!(func.evaluate_condition(&FhirPathValue::Boolean(true)), Some(true));
        assert_eq!(func.evaluate_condition(&FhirPathValue::Boolean(false)), Some(false));
        
        // Test empty values
        assert_eq!(func.evaluate_condition(&FhirPathValue::Empty), Some(false));
        
        // Test collections
        let empty_collection = FhirPathValue::collection(vec![]);
        assert_eq!(func.evaluate_condition(&empty_collection), Some(false));
        
        let boolean_collection = FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]);
        assert_eq!(func.evaluate_condition(&boolean_collection), Some(true));
        
        // Test non-boolean values (should return None - invalid condition)
        assert_eq!(func.evaluate_condition(&FhirPathValue::Integer(42)), None);
        assert_eq!(func.evaluate_condition(&FhirPathValue::String("test".into())), None);
        
        // Test multi-item collections (should return None - invalid condition)
        let multi_collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true), 
            FhirPathValue::Boolean(false)
        ]);
        assert_eq!(func.evaluate_condition(&multi_collection), None);
    }

    #[test]
    fn test_iif_metadata() {
        let func = EnhancedIifFunction::new();
        
        assert_eq!(func.name(), "iif");
        assert_eq!(func.human_friendly_name(), "Conditional Expression (If-Then-Else)");
        assert!(func.supports_lambda_expressions());
        assert_eq!(func.lambda_argument_indices(), vec![1, 2]);
        
        let signature = func.signature();
        assert_eq!(signature.min_arity, 3);
        assert_eq!(signature.max_arity, Some(3));
        assert_eq!(signature.parameters.len(), 3);
    }
}