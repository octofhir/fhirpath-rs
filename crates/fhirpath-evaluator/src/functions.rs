//! FHIRPath function call evaluation
//!
//! This module handles the evaluation of both standard functions (delegated to the registry)
//! and lambda functions (evaluated directly with custom context handling).

use crate::context::EvaluationContext as LocalEvaluationContext;
use octofhir_fhirpath_ast::FunctionCallData;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;

/// Function evaluation dispatch methods
impl crate::FhirPathEngine {
    /// Check if a function name refers to a lambda function
    pub async fn is_lambda_function(&self, name: &str) -> bool {
        // Lambda functions are those that take expression arguments and need special evaluation
        // These are the actual lambda functions that exist in FHIRPath
        matches!(name, "where" | "select" | "all" | "sort" | "repeat" | "aggregate" | "iif" | "exists")
    }

    /// Evaluate standard functions by delegating to the registry
    pub async fn evaluate_standard_function(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Special handling for defineVariable function - it needs to modify the evaluation context
        if func_data.name == "defineVariable" {
            return self
                .evaluate_define_variable_function(func_data, input, context, depth)
                .await;
        }

        // For standard functions, evaluate all arguments first
        let mut evaluated_args = Vec::new();
        for arg in &func_data.args {
            let arg_result = self
                .evaluate_node_async(arg, input.clone(), context, depth + 1)
                .await?;
            evaluated_args.push(arg_result);
        }

        // Create registry context for function evaluation
        let registry_context = octofhir_fhirpath_registry::traits::EvaluationContext {
            input: input.clone(),
            root: context.root.clone(),
            variables: context.variable_scope.variables.as_ref().clone(),
            model_provider: self.model_provider().clone(),
        };

        // Delegate to registry
        if self.registry().has_function(&func_data.name) {
            self.registry()
                .evaluate(&func_data.name, &evaluated_args, &registry_context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Function '{}' evaluation failed: {}", func_data.name, e),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unknown function: {}", func_data.name),
            })
        }
    }

    /// Evaluate defineVariable function with special context handling
    pub async fn evaluate_define_variable_function(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Validate arguments: defineVariable(name) or defineVariable(name, value)
        if func_data.args.is_empty() || func_data.args.len() > 2 {
            return Err(EvaluationError::InvalidOperation {
                message: "defineVariable() requires 1 or 2 arguments (name, [value])".to_string(),
            });
        }

        // First argument should be the variable name
        let var_name_result = self
            .evaluate_node_async(&func_data.args[0], input.clone(), context, depth + 1)
            .await?;

        let var_name = match var_name_result {
            FhirPathValue::String(name) => name.to_string(),
            _ => {
                return Err(EvaluationError::InvalidOperation {
                    message: "defineVariable() first argument must be a string".to_string(),
                });
            }
        };

        // Check if the variable name is a system variable (protected)
        if crate::FhirPathEngine::is_system_variable(&var_name) {
            return Err(EvaluationError::InvalidOperation {
                message: format!("Cannot override system variable '{var_name}'"),
            });
        }

        // Check if variable already exists in current scope (redefinition error)
        if context.variable_scope.get_variable(&var_name).is_some() {
            return Err(EvaluationError::InvalidOperation {
                message: format!("Variable '{var_name}' is already defined in current scope"),
            });
        }

        // Get variable value (second argument or current context)
        let _var_value = if func_data.args.len() == 2 {
            self.evaluate_node_async(&func_data.args[1], input.clone(), context, depth + 1)
                .await?
        } else {
            input.clone() // Use current context as value
        };

        // The defineVariable function should return the input unchanged
        // The variable definition itself is handled by the caller that propagates context
        // TODO: Implement proper variable storage in evaluation context (complex architectural change needed)
        Ok(input)
    }

    /// Evaluate iif (if-then-else) function with lazy evaluation
    pub async fn evaluate_iif_function(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Validate arguments: iif(condition, then_expr) or iif(condition, then_expr, else_expr)
        if func_data.args.len() < 2 || func_data.args.len() > 3 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "iif() requires 2 or 3 arguments, got {}",
                    func_data.args.len()
                ),
            });
        }

        // iif() can work with any input - the key is evaluating the condition properly

        // Create lambda context preserving existing lambda variables from outer context
        // but set $this to current input
        let lambda_context = context.with_lambda_context_preserving_index(input.clone());

        // Evaluate condition
        let condition = self
            .evaluate_node_async(
                &func_data.args[0],
                input.clone(),
                &lambda_context,
                depth + 1,
            )
            .await?;

        // Convert condition to boolean using FHIRPath boolean conversion rules
        // Non-empty collections, non-zero numbers, non-empty strings are truthy
        let boolean_result = self.to_boolean_fhirpath(&condition);

        match boolean_result {
            Some(true) => {
                // Evaluate then expression
                self.evaluate_node_async(&func_data.args[1], input, &lambda_context, depth + 1)
                    .await
            }
            Some(false) => {
                if func_data.args.len() == 3 {
                    // Evaluate else expression
                    self.evaluate_node_async(&func_data.args[2], input, &lambda_context, depth + 1)
                        .await
                } else {
                    // No else expression provided
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            None => {
                // Invalid condition (shouldn't happen with FHIRPath conversion) - return empty
                Ok(FhirPathValue::collection(vec![]))
            }
        }
    }

    /// Evaluate lambda functions with expression arguments
    ///
    /// Lambda functions receive raw expressions instead of pre-evaluated values,
    /// allowing them to control evaluation context and implement proper variable
    /// scoping for $this, $index, $total, etc.
    pub async fn evaluate_lambda_function(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Create registry context for lambda function evaluation
        let _registry_context = octofhir_fhirpath_registry::traits::EvaluationContext {
            input: input.clone(),
            root: context.root.clone(),
            variables: context.variable_scope.variables.as_ref().clone(),
            model_provider: self.model_provider().clone(),
        };

        // For now, delegate lambda functions to the engine's lambda module
        // This is a temporary solution until the registry API is aligned
        match func_data.name.as_str() {
            "where" => {
                self.evaluate_where_lambda(func_data, input, context, depth)
                    .await
            }
            "select" => {
                self.evaluate_select_lambda(func_data, input, context, depth)
                    .await
            }
            "sort" => {
                self.evaluate_sort_lambda(func_data, input, context, depth)
                    .await
            }
            "repeat" => {
                self.evaluate_repeat_lambda(func_data, input, context, depth)
                    .await
            }
            "aggregate" => {
                self.evaluate_aggregate_lambda(func_data, input, context, depth)
                    .await
            }
            "all" => {
                self.evaluate_all_lambda(func_data, input, context, depth)
                    .await
            }
            "iif" => {
                self.evaluate_iif_lambda(func_data, input, context, depth)
                    .await
            }
            "exists" => {
                self.evaluate_exists_lambda(func_data, input, context, depth)
                    .await
            }
            _ => Err(EvaluationError::InvalidOperation {
                message: format!("Unknown lambda function: {}", func_data.name),
            }),
        }
    }
}
