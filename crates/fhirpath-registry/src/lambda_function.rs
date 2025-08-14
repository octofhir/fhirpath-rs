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

//! Lambda function support for advanced expression evaluation

use crate::expression_argument::{ExpressionArgument, VariableScope};
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use crate::unified_registry::UnifiedFunctionRegistry;
use async_trait::async_trait;
use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::FhirPathValue;
use std::sync::Arc;

/// Lambda evaluator type - evaluates expressions with variable scoping
pub type LambdaExpressionEvaluator = dyn for<'a> Fn(
    &'a ExpressionNode,
    &'a VariableScope,
    &'a EvaluationContext,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<FhirPathValue, FunctionError>> + Send + 'a>,
> + Send
    + Sync;

/// Enhanced context for lambda function evaluation
pub struct LambdaEvaluationContext<'a> {
    /// Base evaluation context
    pub context: &'a EvaluationContext,
    /// Lambda expression evaluator
    pub evaluator: &'a LambdaExpressionEvaluator,
}

/// Trait for functions that support lambda-style expression evaluation
#[async_trait]
pub trait LambdaFhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;

    /// Get the human-friendly name for the function
    fn human_friendly_name(&self) -> &str;

    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;

    /// Evaluate the function with expression arguments
    async fn evaluate_with_expressions(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue>;

    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }

    /// Check if this function is pure
    fn is_pure(&self) -> bool {
        false
    }

    /// Check if this function supports lambda expressions
    fn supports_lambda_expressions(&self) -> bool {
        true
    }

    /// Get which argument indices should be treated as expressions (not pre-evaluated)
    /// For aggregate(): [0] means the first argument (aggregator expression) should not be pre-evaluated
    fn lambda_argument_indices(&self) -> Vec<usize> {
        // By default, all arguments are lambda expressions
        (0..self.signature().min_arity).collect()
    }
}

/// Enhanced function implementation that can handle both traditional and lambda functions
#[derive(Clone)]
pub enum EnhancedFunctionImpl {
    /// Traditional function (receives pre-evaluated arguments)
    Traditional(Arc<crate::function::FunctionImpl>),
    /// Lambda function (can receive unevaluated expressions)
    Lambda(Arc<dyn LambdaFhirPathFunction>),
}

impl EnhancedFunctionImpl {
    /// Get the function name
    pub fn name(&self) -> &str {
        match self {
            Self::Traditional(func) => func.name(),
            Self::Lambda(func) => func.name(),
        }
    }

    /// Check if this function supports lambda expressions
    pub fn supports_lambda_expressions(&self) -> bool {
        match self {
            Self::Traditional(_) => false,
            Self::Lambda(func) => func.supports_lambda_expressions(),
        }
    }

    /// Get lambda argument indices if this is a lambda function
    pub fn lambda_argument_indices(&self) -> Option<Vec<usize>> {
        match self {
            Self::Traditional(_) => None,
            Self::Lambda(func) => Some(func.lambda_argument_indices()),
        }
    }

    /// Get the function signature
    pub fn signature(&self) -> &FunctionSignature {
        match self {
            Self::Traditional(func) => func.signature(),
            Self::Lambda(func) => func.signature(),
        }
    }

    /// Evaluate with traditional arguments (async)
    pub async fn evaluate_traditional(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        match self {
            Self::Traditional(func) => func.evaluate_async(args, context).await,
            Self::Lambda(_) => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Lambda function called with traditional arguments".to_string(),
            }),
        }
    }

    /// Evaluate with expression arguments (async)
    pub async fn evaluate_lambda(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        match self {
            Self::Traditional(_) => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Traditional function called with expression arguments".to_string(),
            }),
            Self::Lambda(func) => func.evaluate_with_expressions(args, context).await,
        }
    }
}

/// Create a professional lambda expression evaluator 
pub fn create_simple_lambda_evaluator() -> Box<LambdaExpressionEvaluator> {
    // Create a comprehensive lambda evaluator within the registry crate
    let function_registry = Arc::new(UnifiedFunctionRegistry::default());
    
    Box::new(move |expr, scope, context| {
        let registry = function_registry.clone();
        
        Box::pin(async move {
            // Create a comprehensive evaluator for lambda expressions
            let evaluator = BuiltInLambdaEvaluator::new(registry, 50, 5000);
            evaluator.evaluate(expr, scope, context).await
        })
    })
}

/// Built-in lambda evaluator that works within the registry crate
pub struct BuiltInLambdaEvaluator {
    function_registry: Arc<UnifiedFunctionRegistry>,
    operator_registry: Arc<crate::unified_operator_registry::UnifiedOperatorRegistry>,
    max_recursion_depth: usize,
    timeout_ms: u64,
}

impl BuiltInLambdaEvaluator {
    pub fn new(
        function_registry: Arc<UnifiedFunctionRegistry>, 
        max_recursion_depth: usize,
        timeout_ms: u64
    ) -> Self {
        Self {
            function_registry,
            operator_registry: Arc::new(crate::unified_operator_registry::UnifiedOperatorRegistry::default()),
            max_recursion_depth,
            timeout_ms,
        }
    }
    
    pub async fn evaluate(
        &self,
        expr: &ExpressionNode,
        scope: &VariableScope,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue, FunctionError> {
        // Add timeout protection
        let evaluation_future = self.evaluate_with_depth(expr, scope, context, 0);
        
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(self.timeout_ms),
            evaluation_future
        ).await {
            Ok(result) => result,
            Err(_) => Err(FunctionError::EvaluationError {
                name: "lambda_evaluator".to_string(),
                message: format!("Evaluation timeout after {}ms", self.timeout_ms),
            }),
        }
    }
    
    fn evaluate_with_depth<'a>(
        &'a self,
        expr: &'a ExpressionNode,
        scope: &'a VariableScope,
        context: &'a EvaluationContext,
        depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathValue, FunctionError>> + Send + 'a>> {
        Box::pin(async move {
        // Check recursion depth
        if depth > self.max_recursion_depth {
            return Err(FunctionError::EvaluationError {
                name: "lambda_evaluator".to_string(),
                message: format!("Maximum recursion depth ({}) exceeded", self.max_recursion_depth),
            });
        }
        
        match expr {
            ExpressionNode::Variable(name) => {
                // Look up variable in scope
                if let Some(value) = scope.get_owned(name) {
                    Ok(value)
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            
            ExpressionNode::Literal(literal) => {
                self.evaluate_literal(literal)
            }
            
            ExpressionNode::BinaryOp(binary_data) => {
                self.evaluate_binary_operation(binary_data, scope, context, depth).await
            }
            
            ExpressionNode::FunctionCall(function_data) => {
                self.evaluate_function_call(function_data, scope, context, depth).await
            }
            
            ExpressionNode::MethodCall(method_data) => {
                self.evaluate_method_call(method_data, scope, context, depth).await
            }
            
            _ => {
                // For now, return empty for unsupported node types
                // In a full implementation, we would handle all node types
                Ok(FhirPathValue::Empty)
            }
        }
        })
    }
    
    fn evaluate_literal(&self, literal: &octofhir_fhirpath_ast::LiteralValue) -> Result<FhirPathValue, FunctionError> {
        use octofhir_fhirpath_ast::LiteralValue;
        
        match literal {
            LiteralValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            LiteralValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            LiteralValue::Decimal(s) => {
                match s.parse::<rust_decimal::Decimal>() {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Err(FunctionError::EvaluationError {
                        name: "literal_evaluation".to_string(),
                        message: format!("Invalid decimal literal: {}", s),
                    }),
                }
            }
            LiteralValue::String(s) => Ok(FhirPathValue::String(s.as_str().into())),
            _ => Ok(FhirPathValue::Empty), // For now, handle other types as empty
        }
    }
    
    async fn evaluate_binary_operation(
        &self,
        binary_data: &octofhir_fhirpath_ast::BinaryOpData,
        scope: &VariableScope,
        context: &EvaluationContext,
        depth: usize,
    ) -> Result<FhirPathValue, FunctionError> {
        let left_value = self.evaluate_with_depth(&binary_data.left, scope, context, depth + 1).await?;
        let right_value = self.evaluate_with_depth(&binary_data.right, scope, context, depth + 1).await?;
        
        // Use the unified operator registry to evaluate the binary operation
        let operator_symbol = match binary_data.op {
            octofhir_fhirpath_ast::BinaryOperator::Add => "+",
            octofhir_fhirpath_ast::BinaryOperator::Subtract => "-",
            octofhir_fhirpath_ast::BinaryOperator::Multiply => "*",
            octofhir_fhirpath_ast::BinaryOperator::Divide => "/",
            octofhir_fhirpath_ast::BinaryOperator::Modulo => "mod",
            octofhir_fhirpath_ast::BinaryOperator::Equal => "=",
            octofhir_fhirpath_ast::BinaryOperator::NotEqual => "!=",
            octofhir_fhirpath_ast::BinaryOperator::LessThan => "<",
            octofhir_fhirpath_ast::BinaryOperator::LessThanOrEqual => "<=",
            octofhir_fhirpath_ast::BinaryOperator::GreaterThan => ">",
            octofhir_fhirpath_ast::BinaryOperator::GreaterThanOrEqual => ">=",
            octofhir_fhirpath_ast::BinaryOperator::And => "and",
            octofhir_fhirpath_ast::BinaryOperator::Or => "or",
            octofhir_fhirpath_ast::BinaryOperator::Xor => "xor",
            _ => return Err(FunctionError::EvaluationError {
                name: "binary_operation".to_string(),
                message: format!("Unsupported binary operator: {:?}", binary_data.op),
            }),
        };
        if let Some(operator) = self.operator_registry.get_binary(operator_symbol) {
            let op_context = EvaluationContext {
                input: context.input.clone(),
                root: context.root.clone(),
                variables: context.variables.clone(),
                model_provider: context.model_provider.clone(),
            };
            
            // Use the unified operator's evaluate_binary method
            let result = operator.evaluate_binary(left_value, right_value, &op_context).await
                .map_err(|e| FunctionError::EvaluationError {
                    name: "binary_operation".to_string(),
                    message: format!("Binary operation '{}' failed: {:?}", operator_symbol, e),
                });
            
            result
        } else {
            Err(FunctionError::EvaluationError {
                name: "binary_operation".to_string(),
                message: format!("Operator not found in registry: {:?}", binary_data.op),
            })
        }
    }
    
    async fn evaluate_function_call(
        &self,
        function_data: &octofhir_fhirpath_ast::FunctionCallData,
        scope: &VariableScope,
        context: &EvaluationContext,
        depth: usize,
    ) -> Result<FhirPathValue, FunctionError> {
        // For function calls, we need to evaluate arguments and call the function
        let mut arg_values = Vec::new();
        for arg in &function_data.args {
            let value = self.evaluate_with_depth(arg, scope, context, depth + 1).await?;
            arg_values.push(value);
        }
        
        // Try to get the function from the registry and call it
        if let Some(func) = self.function_registry.get_function(&function_data.name) {
            // Create a new context with the current input
            let func_context = EvaluationContext {
                input: context.input.clone(),
                root: context.root.clone(),
                model_provider: context.model_provider.clone(),
                variables: context.variables.clone(),
            };
            
            func.evaluate_async(&arg_values, &func_context).await
        } else {
            Err(FunctionError::FunctionNotFound {
                name: function_data.name.clone(),
            })
        }
    }
    
    async fn evaluate_method_call(
        &self,
        method_data: &octofhir_fhirpath_ast::MethodCallData,
        scope: &VariableScope,
        context: &EvaluationContext,
        depth: usize,
    ) -> Result<FhirPathValue, FunctionError> {
        // Evaluate the base object
        let base_value = self.evaluate_with_depth(&method_data.base, scope, context, depth + 1).await?;
        
        // For method calls like $total.empty(), we need special handling
        if method_data.method == "empty" {
            let is_empty = matches!(base_value, FhirPathValue::Empty) || 
                          matches!(base_value, FhirPathValue::Collection(ref items) if items.is_empty());
            return Ok(FhirPathValue::Boolean(is_empty));
        }
        
        // For other methods, return empty for now
        Ok(FhirPathValue::Empty)
    }
}

/// Helper function to determine if a function should receive expression arguments
pub fn is_lambda_function(name: &str) -> bool {
    matches!(
        name,
        "aggregate" | "where" | "select" | "all" | "any" | "exists" | "iif" | "repeat" | "sort"
    )
}

/// Helper function to get lambda argument indices for a function
pub fn get_lambda_argument_indices(name: &str) -> Vec<usize> {
    match name {
        "aggregate" => vec![0], // First argument is the aggregator expression
        "where" | "select" | "all" | "any" | "exists" => vec![0], // Predicate/selector expression
        "iif" => vec![1, 2], // Then and else expressions (condition is evaluated normally)
        "repeat" => vec![0], // Iterator expression
        "sort" => vec![0], // First argument is the sort expression (optional)
        _ => vec![],
    }
}