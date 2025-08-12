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

//! Main FHIRPath evaluation engine

use super::{
    context::EvaluationContext,
    error::{EvaluationError, EvaluationResult},
};
use crate::ast::{BinaryOperator, ExpressionNode, LiteralValue, UnaryOperator};
use crate::model::{FhirPathValue, ModelProvider};
use crate::registry::{FunctionRegistry, OperatorRegistry};
// Lambda functions are not yet fully implemented
// use crate::registry::function::{AllFunction, AnyFunction, ExistsFunction};
use rust_decimal::Decimal;
use std::hash::BuildHasherDefault;
use std::str::FromStr;
use std::sync::Arc;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;
// Variable context is now managed through EvaluationContext to avoid thread-local storage
// This ensures WASM compatibility and proper variable scoping

/// Main FHIRPath evaluation engine
#[derive(Clone)]
pub struct FhirPathEngine {
    /// Function registry
    functions: Arc<FunctionRegistry>,
    /// Operator registry
    operators: Arc<OperatorRegistry>,
    /// Model provider for type checking and validation
    model_provider: Arc<dyn ModelProvider>,
    /// Reusable virtual machine for bytecode execution
    vm: crate::compiler::VirtualMachine,
}

impl FhirPathEngine {
    /// Create a new engine with custom registries and model provider
    pub fn with_registries(
        functions: Arc<FunctionRegistry>,
        operators: Arc<OperatorRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            vm: crate::compiler::VirtualMachine::new(functions.clone(), operators.clone()),
            functions,
            operators,
            model_provider,
        }
    }

    /// Get the model provider used by this engine
    pub fn model_provider(&self) -> &Arc<dyn ModelProvider> {
        &self.model_provider
    }

    /// Extract a type name from an expression node (for handling 'is' function arguments)
    /// Returns the full dotted path as a string for identifiers and path expressions
    fn extract_type_name(&self, expr: &ExpressionNode) -> Option<String> {
        match expr {
            ExpressionNode::Identifier(name) => Some(name.clone()),
            ExpressionNode::Path { base, path } => {
                // Recursively build the dotted path (e.g., System.Boolean)
                self.extract_type_name(base)
                    .map(|base_name| format!("{base_name}.{path}"))
            }
            _ => None,
        }
    }

    /// Evaluate an FHIRPath expression against input data
    ///
    /// This method automatically selects the optimal evaluation strategy:
    /// - Simple expressions: Fast AST interpretation
    /// - Complex expressions: High-performance bytecode VM with fallback
    ///
    /// The selection is transparent and provides optimal performance without configuration.
    pub async fn evaluate(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Analyze expression complexity for optimal strategy selection
        let complexity = self.estimate_expression_complexity(expression);

        // For complex expressions, try VM compilation first
        if complexity >= 15 {
            match self.try_vm_evaluation(expression, input.clone()) {
                Ok(result) => return Ok(result),
                Err(_) => {
                    // VM failed, fall back to traditional interpretation
                    // This ensures reliability - we never fail due to VM issues
                }
            }
        }

        // Use traditional AST interpretation (simple expressions or VM fallback)
        self.evaluate_traditional_async(expression, input).await
    }

    /// Async version of evaluate - supports async function calls
    pub async fn evaluate_async(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Analyze expression complexity for optimal strategy selection
        let complexity = self.estimate_expression_complexity(expression);

        // For complex expressions, try VM evaluation first (currently sync only)
        if complexity >= 15 {
            match self.try_vm_evaluation(expression, input.clone()) {
                Ok(result) => return Ok(result),
                Err(_) => {
                    // VM failed, fall back to traditional interpretation
                    // This ensures reliability - we never fail due to VM issues
                }
            }
        }

        // Use traditional AST interpretation with async support
        self.evaluate_traditional_async(expression, input).await
    }

    /// Traditional AST interpretation (internal method)
    fn evaluate_traditional(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let context = EvaluationContext::new(
            input,
            self.functions.clone(),
            self.operators.clone(),
            self.model_provider.clone(),
        );

        // Check if expression needs variable scoping - if so, use threaded evaluation
        if self.needs_variable_scoping(expression) {
            let (result, _) = self.evaluate_with_context_threaded(expression, context)?;
            Ok(result)
        } else {
            self.evaluate_with_context_old(expression, &context)
        }
    }

    /// Async traditional AST interpretation (internal method)
    async fn evaluate_traditional_async(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let context = EvaluationContext::new(
            input,
            self.functions.clone(),
            self.operators.clone(),
            self.model_provider.clone(),
        );

        // Check if expression needs variable scoping - if so, use threaded evaluation
        if self.needs_variable_scoping(expression) {
            let (result, _) = self
                .evaluate_with_context_threaded_async(expression, context)
                .await?;
            Ok(result)
        } else {
            self.evaluate_with_context_old_async(expression, &context)
                .await
        }
    }

    /// VM-based evaluation (internal method)
    fn try_vm_evaluation(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        use crate::compiler::ExpressionCompiler;

        // Compile expression to bytecode
        let mut compiler = ExpressionCompiler::new(self.functions.clone());
        let bytecode =
            compiler
                .compile(expression)
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Compilation failed: {e}"),
                })?;

        // Execute on VM (reuse the cached VM instance)
        self.vm
            .execute(&bytecode, &input)
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("VM execution failed: {e}"),
            })
    }

    /// Evaluate with explicit context and return updated context for defineVariable chains
    pub async fn evaluate_with_context_ext(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<(FhirPathValue, EvaluationContext)> {
        match expression {
            ExpressionNode::MethodCall(data) if data.method == "defineVariable" => {
                // Special handling for defineVariable to thread context through method chains
                let base_result = self.evaluate_with_context(&data.base, context).await?;
                let define_context = context.with_input(base_result.clone());

                if data.args.is_empty() || data.args.len() > 2 {
                    return Err(EvaluationError::InvalidOperation {
                        message: "defineVariable requires 1-2 arguments: name and optional value"
                            .to_string(),
                    });
                }

                // Evaluate variable name and value
                let name_value = self
                    .evaluate_with_context(&data.args[0], &define_context)
                    .await?;
                let var_name = match name_value {
                    FhirPathValue::String(name) => name,
                    FhirPathValue::Collection(items) if items.len() == 1 => match items.get(0) {
                        Some(FhirPathValue::String(name)) => name.clone(),
                        _ => {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable first argument must be a string"
                                    .to_string(),
                            });
                        }
                    },
                    _ => {
                        return Err(EvaluationError::InvalidOperation {
                            message: "defineVariable first argument must be a string".to_string(),
                        });
                    }
                };

                let var_value = if data.args.len() == 2 {
                    self.evaluate_with_context(&data.args[1], &define_context)
                        .await?
                } else {
                    // If no value provided, use current base result
                    base_result.clone()
                };

                // Create new context with variable set
                let mut new_context = define_context.clone();
                new_context.set_variable(var_name.to_string(), var_value);

                Ok((base_result, new_context))
            }
            _ => {
                // For other expressions, use regular evaluation
                let result = self.evaluate_with_context(expression, context).await?;
                Ok((result, context.clone()))
            }
        }
    }

    /// Evaluate with explicit context and return both result and updated context (async version)
    pub fn evaluate_with_context_threaded_async<'a>(
        &'a self,
        expression: &'a ExpressionNode,
        context: EvaluationContext,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = EvaluationResult<(FhirPathValue, EvaluationContext)>>
                + 'a,
        >,
    > {
        Box::pin(async move {
            match expression {
                ExpressionNode::MethodCall(data) if data.method == "defineVariable" => {
                    // Special handling for defineVariable to thread context properly
                    let (base_value, mut updated_context) = self
                        .evaluate_with_context_threaded_async(&data.base, context)
                        .await?;

                    if data.args.is_empty() || data.args.len() > 2 {
                        return Err(EvaluationError::InvalidOperation {
                            message:
                                "defineVariable requires 1-2 arguments: name and optional value"
                                    .to_string(),
                        });
                    }

                    // Create context with base value as input
                    let define_context = updated_context.with_input(base_value.clone());

                    // Evaluate variable name and value
                    let (name_value, _) = self
                        .evaluate_with_context_threaded_async(&data.args[0], define_context.clone())
                        .await?;
                    let var_name = match name_value {
                        FhirPathValue::String(name) => name,
                        FhirPathValue::Collection(items) if items.len() == 1 => {
                            match items.get(0) {
                                Some(FhirPathValue::String(name)) => name.clone(),
                                _ => {
                                    return Err(EvaluationError::InvalidOperation {
                                        message: "defineVariable first argument must be a string"
                                            .to_string(),
                                    });
                                }
                            }
                        }
                        _ => {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable first argument must be a string"
                                    .to_string(),
                            });
                        }
                    };

                    // Check for protected system variables
                    if self.is_protected_variable(&var_name) {
                        return Err(EvaluationError::InvalidOperation {
                            message: format!("Cannot redefine system variable '{var_name}'"),
                        });
                    }

                    let (var_value, _) = if data.args.len() == 2 {
                        self.evaluate_with_context_threaded_async(&data.args[1], define_context)
                            .await?
                    } else {
                        // If no value provided, use current base value
                        (base_value.clone(), updated_context.clone())
                    };

                    // Store the variable in the context
                    updated_context.set_variable(var_name.to_string(), var_value.clone());

                    // Return the input value with updated context (defineVariable returns its input, not the variable value)
                    Ok((base_value, updated_context))
                }

                ExpressionNode::FunctionCall(data) if data.name == "defineVariable" => {
                    // Special handling for defineVariable function call
                    if data.args.is_empty() || data.args.len() > 2 {
                        return Err(EvaluationError::InvalidOperation {
                            message:
                                "defineVariable requires 1-2 arguments: name and optional value"
                                    .to_string(),
                        });
                    }

                    // Evaluate variable name and value
                    let (name_value, _) = self
                        .evaluate_with_context_threaded_async(&data.args[0], context.clone())
                        .await?;
                    let var_name = match name_value {
                        FhirPathValue::String(name) => name,
                        FhirPathValue::Collection(items) if items.len() == 1 => {
                            match items.get(0) {
                                Some(FhirPathValue::String(name)) => name.clone(),
                                _ => {
                                    return Err(EvaluationError::InvalidOperation {
                                        message: "defineVariable first argument must be a string"
                                            .to_string(),
                                    });
                                }
                            }
                        }
                        _ => {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable first argument must be a string"
                                    .to_string(),
                            });
                        }
                    };

                    // Check for protected system variables
                    if self.is_protected_variable(&var_name) {
                        return Err(EvaluationError::InvalidOperation {
                            message: format!("Cannot redefine system variable '{var_name}'"),
                        });
                    }

                    let (var_value, mut updated_context) = if data.args.len() == 2 {
                        self.evaluate_with_context_threaded_async(&data.args[1], context.clone())
                            .await?
                    } else {
                        // If no value provided, use current input
                        (context.input.clone(), context.clone())
                    };

                    // Store the variable in the context
                    updated_context.set_variable(var_name.to_string(), var_value.clone());

                    // Return the input value with updated context (defineVariable returns its input)
                    Ok((context.input.clone(), updated_context))
                }

                ExpressionNode::Union { left, right } => {
                    // For union operations, evaluate each side with isolated variable scopes
                    // This prevents variables defined in one side from affecting the other
                    let left_context = context.with_fresh_variable_scope();
                    let right_context = context.with_fresh_variable_scope();

                    let (left_val, _) = self
                        .evaluate_with_context_threaded_async(left, left_context)
                        .await?;
                    let (right_val, _) = self
                        .evaluate_with_context_threaded_async(right, right_context)
                        .await?;

                    let mut items = Vec::new();

                    // Add items from left
                    match left_val {
                        FhirPathValue::Collection(left_items) => items.extend(left_items),
                        FhirPathValue::Empty => {}
                        other => items.push(other),
                    }

                    // Add items from right, removing duplicates
                    match right_val {
                        FhirPathValue::Collection(right_items) => {
                            for item in right_items {
                                if !items.contains(&item) {
                                    items.push(item);
                                }
                            }
                        }
                        FhirPathValue::Empty => {}
                        other => {
                            if !items.contains(&other) {
                                items.push(other);
                            }
                        }
                    }

                    Ok((FhirPathValue::collection(items), context))
                }

                ExpressionNode::MethodCall(data) => {
                    // For other method calls, thread context through base evaluation
                    let (base_value, updated_context) = self
                        .evaluate_with_context_threaded_async(&data.base, context)
                        .await?;
                    let method_context = updated_context.with_input(base_value);
                    let result = self
                        .evaluate_method_call_direct_async(
                            &data.method,
                            &data.args,
                            &method_context,
                        )
                        .await?;
                    Ok((result, updated_context))
                }

                ExpressionNode::Path { base, path } => {
                    // Thread context through path navigation
                    let (base_value, updated_context) = self
                        .evaluate_with_context_threaded_async(base, context)
                        .await?;
                    let path_context = updated_context.with_input(base_value);
                    let result = self.evaluate_identifier(path, &path_context)?;
                    Ok((result, updated_context))
                }

                ExpressionNode::Variable(name) => {
                    // Variable evaluation uses current context
                    let result = self.evaluate_variable(name, &context)?;
                    Ok((result, context))
                }

                _ => {
                    // For other expressions, use the old evaluation method and wrap the result
                    let result = self
                        .evaluate_with_context_old_async(expression, &context)
                        .await?;
                    Ok((result, context))
                }
            }
        })
    }

    /// Evaluate with explicit context and return both result and updated context
    pub fn evaluate_with_context_threaded(
        &self,
        expression: &ExpressionNode,
        context: EvaluationContext,
    ) -> EvaluationResult<(FhirPathValue, EvaluationContext)> {
        match expression {
            ExpressionNode::MethodCall(data) if data.method == "defineVariable" => {
                // Special handling for defineVariable to thread context properly
                let (base_value, mut updated_context) =
                    self.evaluate_with_context_threaded(&data.base, context)?;

                if data.args.is_empty() || data.args.len() > 2 {
                    return Err(EvaluationError::InvalidOperation {
                        message: "defineVariable requires 1-2 arguments: name and optional value"
                            .to_string(),
                    });
                }

                // Create context with base value as input
                let define_context = updated_context.with_input(base_value.clone());

                // Evaluate variable name and value
                let (name_value, _) =
                    self.evaluate_with_context_threaded(&data.args[0], define_context.clone())?;
                let var_name = match name_value {
                    FhirPathValue::String(name) => name,
                    FhirPathValue::Collection(items) if items.len() == 1 => match items.get(0) {
                        Some(FhirPathValue::String(name)) => name.clone(),
                        _ => {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable first argument must be a string"
                                    .to_string(),
                            });
                        }
                    },
                    _ => {
                        return Err(EvaluationError::InvalidOperation {
                            message: "defineVariable first argument must be a string".to_string(),
                        });
                    }
                };

                // Check for protected system variables
                if self.is_protected_variable(&var_name) {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Cannot redefine system variable '{var_name}'"),
                    });
                }

                let (var_value, _) = if data.args.len() == 2 {
                    self.evaluate_with_context_threaded(&data.args[1], define_context)?
                } else {
                    // If no value provided, use current base value
                    (base_value.clone(), updated_context.clone())
                };

                // Store the variable in the context
                updated_context.set_variable(var_name.to_string(), var_value.clone());

                // Return the input value with updated context (defineVariable returns its input, not the variable value)
                Ok((base_value, updated_context))
            }

            ExpressionNode::FunctionCall(data) if data.name == "defineVariable" => {
                // Special handling for defineVariable function call
                if data.args.is_empty() || data.args.len() > 2 {
                    return Err(EvaluationError::InvalidOperation {
                        message: "defineVariable requires 1-2 arguments: name and optional value"
                            .to_string(),
                    });
                }

                // Evaluate variable name and value
                let (name_value, _) =
                    self.evaluate_with_context_threaded(&data.args[0], context.clone())?;
                let var_name = match name_value {
                    FhirPathValue::String(name) => name,
                    FhirPathValue::Collection(items) if items.len() == 1 => match items.get(0) {
                        Some(FhirPathValue::String(name)) => name.clone(),
                        _ => {
                            return Err(EvaluationError::InvalidOperation {
                                message: "defineVariable first argument must be a string"
                                    .to_string(),
                            });
                        }
                    },
                    _ => {
                        return Err(EvaluationError::InvalidOperation {
                            message: "defineVariable first argument must be a string".to_string(),
                        });
                    }
                };

                // Check for protected system variables
                if self.is_protected_variable(&var_name) {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Cannot redefine system variable '{var_name}'"),
                    });
                }

                let (var_value, mut updated_context) = if data.args.len() == 2 {
                    self.evaluate_with_context_threaded(&data.args[1], context.clone())?
                } else {
                    // If no value provided, use current input
                    (context.input.clone(), context.clone())
                };

                // Store the variable in the context
                updated_context.set_variable(var_name.to_string(), var_value.clone());

                // Return the input value with updated context (defineVariable returns its input)
                Ok((context.input.clone(), updated_context))
            }

            ExpressionNode::Union { left, right } => {
                // For union operations, evaluate each side with isolated variable scopes
                // This prevents variables defined in one side from affecting the other
                let left_context = context.with_fresh_variable_scope();
                let right_context = context.with_fresh_variable_scope();

                let (left_val, _) = self.evaluate_with_context_threaded(left, left_context)?;
                let (right_val, _) = self.evaluate_with_context_threaded(right, right_context)?;

                let mut items = Vec::new();

                // Add items from left
                match left_val {
                    FhirPathValue::Collection(left_items) => items.extend(left_items),
                    FhirPathValue::Empty => {}
                    other => items.push(other),
                }

                // Add items from right, removing duplicates
                match right_val {
                    FhirPathValue::Collection(right_items) => {
                        for item in right_items {
                            if !items.contains(&item) {
                                items.push(item);
                            }
                        }
                    }
                    FhirPathValue::Empty => {}
                    other => {
                        if !items.contains(&other) {
                            items.push(other);
                        }
                    }
                }

                Ok((FhirPathValue::collection(items), context))
            }

            ExpressionNode::MethodCall(data) => {
                // For other method calls, thread context through base evaluation
                let (base_value, updated_context) =
                    self.evaluate_with_context_threaded(&data.base, context)?;
                let method_context = updated_context.with_input(base_value);
                let result =
                    self.evaluate_method_call_direct(&data.method, &data.args, &method_context)?;
                Ok((result, updated_context))
            }

            ExpressionNode::Path { base, path } => {
                // Thread context through path navigation
                let (base_value, updated_context) =
                    self.evaluate_with_context_threaded(base, context)?;
                let path_context = updated_context.with_input(base_value);
                let result = self.evaluate_identifier(path, &path_context)?;
                Ok((result, updated_context))
            }

            ExpressionNode::Variable(name) => {
                // Variable evaluation uses current context
                let result = self.evaluate_variable(name, &context)?;
                Ok((result, context))
            }

            _ => {
                // For other expressions, use the old evaluation method and wrap the result
                let result = self.evaluate_with_context_old(expression, &context)?;
                Ok((result, context))
            }
        }
    }

    /// Legacy evaluation method (async version)
    pub fn evaluate_with_context_old_async<'a>(
        &'a self,
        expression: &'a ExpressionNode,
        context: &'a EvaluationContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + 'a>>
    {
        Box::pin(async move {
            match expression {
                ExpressionNode::Literal(literal) => self.evaluate_literal(literal),

                ExpressionNode::Identifier(name) => self.evaluate_identifier(name, context),

                ExpressionNode::Variable(name) => self.evaluate_variable(name, context),

                ExpressionNode::FunctionCall(data) => {
                    self.evaluate_function_call_async(&data.name, &data.args, context)
                        .await
                }

                ExpressionNode::MethodCall(data) => {
                    self.evaluate_method_call_async(&data.base, &data.method, &data.args, context)
                        .await
                }

                ExpressionNode::BinaryOp(data) => {
                    self.evaluate_binary_op_async(&data.op, &data.left, &data.right, context)
                        .await
                }

                ExpressionNode::UnaryOp { op, operand } => {
                    self.evaluate_unary_op_async(op, operand, context).await
                }

                ExpressionNode::Path { base, path } => {
                    let base_val = self.evaluate_with_context(base, context).await?;
                    let new_context = context.with_input(base_val);
                    self.evaluate_identifier(path, &new_context)
                }

                ExpressionNode::Index { base, index } => {
                    let base_val = self.evaluate_with_context(base, context).await?;
                    let index_val = self.evaluate_with_context(index, context).await?;

                    let index_num = match &index_val {
                        FhirPathValue::Integer(i) => *i,
                        FhirPathValue::Collection(items) if items.len() == 1 => {
                            match items.get(0) {
                                Some(FhirPathValue::Integer(i)) => *i,
                                _ => {
                                    return Err(EvaluationError::TypeError {
                                        expected: "Integer".to_string(),
                                        actual: index_val.type_name().to_string(),
                                    });
                                }
                            }
                        }
                        _ => {
                            return Err(EvaluationError::TypeError {
                                expected: "Integer".to_string(),
                                actual: index_val.type_name().to_string(),
                            });
                        }
                    };

                    match base_val {
                        FhirPathValue::Collection(items) => {
                            // Handle negative indexing (from end of collection)
                            let effective_index = if index_num < 0 {
                                let len = items.len() as i64;
                                len + index_num
                            } else {
                                index_num
                            };

                            // Return empty collection for out-of-bounds access (FHIRPath spec)
                            if effective_index < 0 || effective_index as usize >= items.len() {
                                Ok(FhirPathValue::Collection(vec![].into()))
                            } else {
                                Ok(items.get(effective_index as usize).unwrap().clone())
                            }
                        }
                        _ => {
                            // Single item is treated as single-item collection for indexing
                            let single_item_collection = [base_val];

                            // Handle negative indexing
                            let effective_index = if index_num < 0 {
                                1 + index_num // Length is 1 for single item
                            } else {
                                index_num
                            };

                            // Return empty collection for out-of-bounds access
                            if effective_index < 0 || effective_index as usize >= 1 {
                                Ok(FhirPathValue::Collection(vec![].into()))
                            } else {
                                Ok(single_item_collection
                                    .get(effective_index as usize)
                                    .unwrap()
                                    .clone())
                            }
                        }
                    }
                }

                ExpressionNode::Filter { base, condition } => {
                    let base_val = self.evaluate_with_context(base, context).await?;

                    match base_val {
                        FhirPathValue::Collection(items) => {
                            let mut results = Vec::new();

                            for item in items {
                                let item_context = context.with_input(item.clone());
                                let condition_result =
                                    self.evaluate_with_context(condition, &item_context).await?;

                                if let FhirPathValue::Boolean(true) = condition_result {
                                    results.push(item)
                                }
                            }

                            Ok(FhirPathValue::collection(results))
                        }
                        other => {
                            // For non-collections, treat as single-item collection
                            let item_context = context.with_input(other.clone());
                            let condition_result =
                                self.evaluate_with_context(condition, &item_context).await?;

                            match condition_result {
                                FhirPathValue::Boolean(true) => Ok(other),
                                _ => Ok(FhirPathValue::Empty),
                            }
                        }
                    }
                }

                ExpressionNode::Union { left, right } => {
                    // For union operations, each side should be evaluated with a fresh variable context
                    // to ensure proper variable scoping as per FHIRPath specification
                    let left_context = context.with_fresh_variable_scope();
                    let right_context = context.with_fresh_variable_scope();

                    let left_val = self.evaluate_with_context(left, &left_context).await?;
                    let right_val = self.evaluate_with_context(right, &right_context).await?;

                    let mut items = Vec::new();

                    // Add items from left
                    match left_val {
                        FhirPathValue::Collection(left_items) => items.extend(left_items),
                        FhirPathValue::Empty => {}
                        other => items.push(other),
                    }

                    // Add items from right, removing duplicates
                    match right_val {
                        FhirPathValue::Collection(right_items) => {
                            for item in right_items {
                                if !items.contains(&item) {
                                    items.push(item);
                                }
                            }
                        }
                        FhirPathValue::Empty => {}
                        other => {
                            if !items.contains(&other) {
                                items.push(other);
                            }
                        }
                    }

                    Ok(FhirPathValue::collection(items))
                }

                ExpressionNode::TypeCheck {
                    expression,
                    type_name,
                } => {
                    let value = self.evaluate_with_context(expression, context).await?;

                    let matches = match &value {
                        FhirPathValue::Collection(items) => {
                            // For collections, check if it has exactly one item of the specified type
                            if items.len() == 1 {
                                check_value_type(items.get(0).unwrap(), type_name)
                            } else {
                                false
                            }
                        }
                        single_value => check_value_type(single_value, type_name),
                    };

                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        matches,
                    )]))
                }

                ExpressionNode::TypeCast {
                    expression,
                    type_name,
                } => {
                    let value = self.evaluate_with_context(expression, context).await?;

                    // Basic type casting - can be enhanced later
                    match (type_name.as_str(), &value) {
                        ("String", _) => {
                            if let Some(s) = value.to_string_value() {
                                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                                    s.into(),
                                )]))
                            } else {
                                Ok(FhirPathValue::collection(vec![]))
                            }
                        }
                        _ => Ok(FhirPathValue::collection(vec![value])), // For now, just return the value as-is
                    }
                }

                ExpressionNode::Lambda(data) => {
                    // Lambda expressions are context-dependent
                    // For now, evaluate body directly
                    self.evaluate_with_context_old_async(&data.body, context)
                        .await
                }

                ExpressionNode::Conditional(data) => {
                    let condition_val =
                        self.evaluate_with_context(&data.condition, context).await?;

                    match condition_val {
                        FhirPathValue::Boolean(true) => {
                            self.evaluate_with_context(&data.then_expr, context).await
                        }
                        _ => {
                            if let Some(else_branch) = data.else_expr.as_deref() {
                                self.evaluate_with_context(else_branch, context).await
                            } else {
                                Ok(FhirPathValue::collection(vec![]))
                            }
                        }
                    }
                }
            }
        })
    }

    /// Legacy evaluation method (renamed)
    pub fn evaluate_with_context_old(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        match expression {
            ExpressionNode::Literal(literal) => self.evaluate_literal(literal),

            ExpressionNode::Identifier(name) => self.evaluate_identifier(name, context),

            ExpressionNode::Variable(name) => self.evaluate_variable(name, context),

            ExpressionNode::FunctionCall(data) => {
                self.evaluate_function_call(&data.name, &data.args, context)
            }

            ExpressionNode::MethodCall(data) => {
                self.evaluate_method_call(&data.base, &data.method, &data.args, context)
            }

            ExpressionNode::BinaryOp(data) => {
                self.evaluate_binary_op(&data.op, &data.left, &data.right, context)
            }

            ExpressionNode::UnaryOp { op, operand } => self.evaluate_unary_op(op, operand, context),

            ExpressionNode::Path { base, path } => self.evaluate_path(base, path, context),

            ExpressionNode::Index { base, index } => self.evaluate_index(base, index, context),

            ExpressionNode::Filter { base, condition } => {
                self.evaluate_filter(base, condition, context)
            }

            ExpressionNode::Union { left, right } => self.evaluate_union(left, right, context),

            ExpressionNode::TypeCheck {
                expression,
                type_name,
            } => self.evaluate_type_check(expression, type_name, context),

            ExpressionNode::TypeCast {
                expression,
                type_name,
            } => self.evaluate_type_cast(expression, type_name, context),

            ExpressionNode::Lambda(data) => {
                // Lambda expressions are context-dependent
                // For now, evaluate body directly
                self.evaluate_with_context_old(&data.body, context)
            }

            ExpressionNode::Conditional(data) => self.evaluate_conditional(
                &data.condition,
                &data.then_expr,
                data.else_expr.as_deref(),
                context,
            ),
        }
    }

    /// Evaluate with explicit context (wrapper for backward compatibility)
    pub async fn evaluate_with_context(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Use the new threaded evaluation for expressions that need variable scoping
        if self.needs_variable_scoping(expression) {
            let (result, _) = self
                .evaluate_with_context_threaded_async(expression, context.clone())
                .await?;
            Ok(result)
        } else {
            // Use the old method for simple expressions with async support
            self.evaluate_with_context_old_async(expression, context)
                .await
        }
    }

    /// Check if an expression needs variable scoping (contains defineVariable or union)
    fn needs_variable_scoping(&self, expression: &ExpressionNode) -> bool {
        match expression {
            ExpressionNode::MethodCall(data) => {
                data.method == "defineVariable"
                    || self.needs_variable_scoping(&data.base)
                    || data.args.iter().any(|arg| self.needs_variable_scoping(arg))
            }
            ExpressionNode::Union { left, right } => {
                self.needs_variable_scoping(left) || self.needs_variable_scoping(right)
            }
            ExpressionNode::BinaryOp(data) => {
                self.needs_variable_scoping(&data.left) || self.needs_variable_scoping(&data.right)
            }
            ExpressionNode::UnaryOp { operand, op: _ } => self.needs_variable_scoping(operand),
            ExpressionNode::Path { base, path: _ } => self.needs_variable_scoping(base),
            ExpressionNode::Index { base, index } => {
                self.needs_variable_scoping(base) || self.needs_variable_scoping(index)
            }
            ExpressionNode::Filter { base, condition } => {
                self.needs_variable_scoping(base) || self.needs_variable_scoping(condition)
            }
            ExpressionNode::FunctionCall(data) => {
                data.name == "defineVariable"
                    || data.args.iter().any(|arg| self.needs_variable_scoping(arg))
            }
            ExpressionNode::Lambda(data) => self.needs_variable_scoping(&data.body),
            ExpressionNode::Conditional(data) => {
                self.needs_variable_scoping(&data.condition)
                    || self.needs_variable_scoping(&data.then_expr)
                    || data
                        .else_expr
                        .as_ref()
                        .is_some_and(|e| self.needs_variable_scoping(e))
            }
            ExpressionNode::TypeCheck {
                expression,
                type_name: _,
            } => self.needs_variable_scoping(expression),
            ExpressionNode::TypeCast {
                expression,
                type_name: _,
            } => self.needs_variable_scoping(expression),
            ExpressionNode::Variable(_) => true, // Variable references need proper scoping
            _ => false,
        }
    }

    /// Evaluate a literal value
    fn evaluate_literal(&self, literal: &LiteralValue) -> EvaluationResult<FhirPathValue> {
        let value = match literal {
            LiteralValue::Boolean(b) => FhirPathValue::Boolean(*b),
            LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
            LiteralValue::Decimal(s) => match Decimal::from_str(s) {
                Ok(d) => FhirPathValue::Decimal(d),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid decimal literal: {s}"),
                    });
                }
            },
            LiteralValue::String(s) => FhirPathValue::String(s.clone().into()),
            LiteralValue::Date(s) => match parse_fhirpath_date(s) {
                Ok(date) => FhirPathValue::Date(date),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid date literal: {s}"),
                    });
                }
            },
            LiteralValue::DateTime(s) => match parse_fhirpath_datetime(s) {
                Ok(datetime) => FhirPathValue::DateTime(datetime),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid datetime literal: {s}"),
                    });
                }
            },
            LiteralValue::Time(s) => match parse_fhirpath_time(s) {
                Ok(time) => FhirPathValue::Time(time),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid time literal: {s}"),
                    });
                }
            },
            LiteralValue::Quantity { value, unit } => match Decimal::from_str(value) {
                Ok(d) => FhirPathValue::quantity(d, Some(unit.clone())),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid quantity value: {value}"),
                    });
                }
            },
            LiteralValue::Null => return Ok(FhirPathValue::Empty),
        };

        // Return the literal value directly - literals are not automatically collections
        Ok(value)
    }

    /// Evaluate an identifier (property access)
    fn evaluate_identifier(
        &self,
        name: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle special namespace identifiers first
        match name {
            "FHIR" => {
                // Return a TypeInfoObject that represents the FHIR namespace
                return Ok(FhirPathValue::TypeInfoObject {
                    namespace: "FHIR".into(),
                    name: "namespace".into(),
                });
            }
            "System" => {
                // Return a TypeInfoObject that represents the System namespace
                return Ok(FhirPathValue::TypeInfoObject {
                    namespace: "System".into(),
                    name: "namespace".into(),
                });
            }
            _ => {}
        }

        match &context.input {
            FhirPathValue::Resource(resource) => {
                // First check if the identifier matches the resource type
                if let Some(resource_type) = resource.resource_type() {
                    if resource_type == name {
                        // Return the resource itself when accessing by resource type
                        return Ok(context.input.clone());
                    }
                }

                // Otherwise try to get the property with polymorphic support
                match resource.get_property_with_name(name) {
                    Some((value, actual_property_name)) => {
                        // Convert values properly handling arrays and objects
                        match value {
                            serde_json::Value::Object(_) => {
                                // Check if this is a FHIR quantity object that should be converted
                                if actual_property_name.starts_with("value")
                                    && actual_property_name.len() > 5
                                    && &actual_property_name[5..] == "Quantity"
                                {
                                    // Convert FHIR valueQuantity to FHIRPath Quantity
                                    if let Some(quantity_obj) = value.as_object() {
                                        let value_num = quantity_obj
                                            .get("value")
                                            .and_then(|v| v.as_f64())
                                            .or_else(|| {
                                                quantity_obj
                                                    .get("value")
                                                    .and_then(|v| v.as_i64())
                                                    .map(|i| i as f64)
                                            });

                                        let unit = quantity_obj
                                            .get("code")
                                            .and_then(|v| v.as_str())
                                            .or_else(|| {
                                                quantity_obj.get("unit").and_then(|v| v.as_str())
                                            });

                                        if let Some(num) = value_num {
                                            let decimal_val = rust_decimal::Decimal::try_from(num)
                                                .unwrap_or_default();
                                            return Ok(FhirPathValue::quantity(
                                                decimal_val,
                                                unit.map(|u| u.to_string()),
                                            ));
                                        }
                                    }
                                }

                                // Wrap JSON objects as FhirResource so functions like resolve() can inspect fields
                                Ok(FhirPathValue::Resource(Arc::new(
                                    crate::model::FhirResource::from_json(value.clone()),
                                )))
                            }
                            serde_json::Value::Array(arr) => {
                                // Convert array elements, wrapping objects as FhirResources
                                let mut results = Vec::new();
                                for item in arr {
                                    match item {
                                        serde_json::Value::Object(_) => {
                                            results.push(FhirPathValue::Resource(Arc::new(
                                                crate::model::FhirResource::from_json(item.clone()),
                                            )));
                                        }
                                        _ => {
                                            results.push(FhirPathValue::from(item.clone()));
                                        }
                                    }
                                }
                                Ok(FhirPathValue::collection(results))
                            }
                            _ => Ok(FhirPathValue::from(value.clone())),
                        }
                    }
                    None => Ok(FhirPathValue::Empty), // Return empty collection per FHIRPath spec
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    let item_context = context.with_input(item.clone());
                    match self.evaluate_identifier(name, &item_context) {
                        Ok(value) => {
                            if !value.is_empty() {
                                // Flatten collections according to FHIRPath semantics
                                match value {
                                    FhirPathValue::Collection(sub_items) => {
                                        for sub_item in sub_items.iter() {
                                            results.push(sub_item.clone());
                                        }
                                    }
                                    single_value => {
                                        results.push(single_value);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Ignore errors for collection items that don't have the property
                        }
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::TypeInfoObject {
                namespace,
                name: type_name,
            } => {
                // Handle property access on TypeInfo objects
                if type_name.as_ref() == "namespace" {
                    // This is a namespace object - accessing any property returns the type name
                    // For FHIR namespace, return the qualified type name for is() function
                    if namespace.as_ref() == "FHIR" {
                        return Ok(FhirPathValue::String(format!("FHIR.{name}").into()));
                    } else if namespace.as_ref() == "System" {
                        return Ok(FhirPathValue::String(format!("System.{name}").into()));
                    }
                }

                // Regular TypeInfo object property access
                match name {
                    "namespace" => Ok(FhirPathValue::String(namespace.clone())),
                    "name" => Ok(FhirPathValue::String(type_name.clone())),
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Quantity(quantity) => {
                // Handle property access on Quantity objects
                match name {
                    "value" => {
                        // Return the numeric value of the quantity
                        Ok(FhirPathValue::Decimal(quantity.value))
                    }
                    "unit" => {
                        // Return the unit string if it exists
                        match &quantity.unit {
                            Some(unit) => Ok(FhirPathValue::String(unit.clone().into())),
                            None => Ok(FhirPathValue::Empty),
                        }
                    }
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            _ => Ok(FhirPathValue::Empty), // Return empty collection for non-resource types per FHIRPath spec
        }
    }

    /// Evaluate a variable reference
    fn evaluate_variable(
        &self,
        name: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        match name {
            "$this" | "$" | "this" => {
                // Check if $this is set as a variable first (for lambda functions)
                if let Some(value) = context.get_variable("this") {
                    Ok(value.clone())
                } else {
                    // Fall back to root for regular function arguments
                    Ok(context.root.clone())
                }
            }
            "$$" | "$resource" | "resource" => Ok(context.root.clone()),
            "$total" | "total" => {
                // $total is used in aggregate functions - check for it in variables
                if let Some(value) = context.get_variable("total") {
                    Ok(value.clone())
                } else {
                    // If $total is not defined, return empty
                    Ok(FhirPathValue::Empty)
                }
            }
            "$index" | "index" => {
                // $index is used in lambda functions - check for it in variables
                if let Some(value) = context.get_variable("index") {
                    Ok(value.clone())
                } else {
                    // If $index is not defined, return empty
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => {
                // Environment variables parsed as Variable("name") where % is stripped by parser
                if let Some(value) = context.get_variable(name) {
                    Ok(value.clone())
                } else {
                    // Variable not found - return empty per FHIRPath spec
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    /// Evaluate a function call (async version)
    async fn evaluate_function_call_async(
        &self,
        name: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Get function from registry
        let function =
            context
                .functions
                .get(name)
                .ok_or_else(|| EvaluationError::InvalidOperation {
                    message: format!("Unknown function: {name}"),
                })?;

        // Check if this is a lambda function that needs special evaluation
        if is_lambda_function(name) {
            // For lambda functions, we don't evaluate arguments first - we pass the expressions
            return self
                .evaluate_lambda_function_async(function, args, context)
                .await;
        }

        // For regular functions, evaluate arguments normally
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context(arg, context).await?;

            // Special handling for type-related functions: if an argument evaluates to Empty,
            // check if it represents a type name (identifier or dotted path) and treat as string literal
            if (name == "is" || name == "as" || name == "ofType")
                && matches!(value, FhirPathValue::Empty)
            {
                if let Some(type_name) = self.extract_type_name(arg) {
                    arg_values.push(FhirPathValue::String(type_name.into()));
                } else {
                    arg_values.push(value);
                }
            } else {
                arg_values.push(value);
            }
        }

        // Unwrap single-item collections for function arguments
        // This is required by FHIRPath semantics - functions should receive unwrapped values
        let unwrapped_args = unwrap_function_arguments(arg_values);

        // Create a compatible context for the function registry
        let mut registry_context =
            crate::registry::function::EvaluationContext::with_model_provider(
                context.input.clone(),
                context.model_provider.clone(),
            );
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        // Evaluate function with async support
        let result = function
            .evaluate_async(&unwrapped_args, &registry_context)
            .await?;
        Ok(result)
    }

    /// Evaluate a function call
    fn evaluate_function_call(
        &self,
        name: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Get function from registry
        let function =
            context
                .functions
                .get(name)
                .ok_or_else(|| EvaluationError::InvalidOperation {
                    message: format!("Unknown function: {name}"),
                })?;

        // Check if this is a lambda function that needs special evaluation
        if is_lambda_function(name) {
            // For lambda functions, we don't evaluate arguments first - we pass the expressions
            // Note: This sync function should not be used - prefer async version
            panic!("Sync lambda evaluation not supported - use async version");
        }

        // For regular functions, evaluate arguments normally
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context_old(arg, context)?;

            // Special handling for type-related functions: if an argument evaluates to Empty,
            // check if it represents a type name (identifier or dotted path) and treat as string literal
            if (name == "is" || name == "as" || name == "ofType")
                && matches!(value, FhirPathValue::Empty)
            {
                if let Some(type_name) = self.extract_type_name(arg) {
                    arg_values.push(FhirPathValue::String(type_name.into()));
                } else {
                    arg_values.push(value);
                }
            } else {
                arg_values.push(value);
            }
        }

        // Unwrap single-item collections for function arguments
        // This is required by FHIRPath semantics - functions should receive unwrapped values
        let unwrapped_args = unwrap_function_arguments(arg_values);

        // Create a compatible context for the function registry
        let mut registry_context =
            crate::registry::function::EvaluationContext::with_model_provider(
                context.input.clone(),
                context.model_provider.clone(),
            );
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        // Evaluate function
        let result = function.evaluate(&unwrapped_args, &registry_context)?;
        Ok(result)
    }

    /// Evaluate a lambda function with unevaluated expression arguments (async version)
    async fn evaluate_lambda_function_async(
        &self,
        function: &crate::registry::function::FunctionImpl,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Create an async lambda evaluator closure
        let evaluator = |expr: &ExpressionNode, item_context: &FhirPathValue| {
            let expr_clone = expr.clone();
            let item_context_clone = item_context.clone();
            let self_clone = self.clone();
            let context_clone = context.clone();

            Box::pin(async move {
                // Create a new evaluation context with the item as input
                let mut item_eval_context = context_clone.with_input(item_context_clone.clone());

                // Explicitly set $this to the current item for lambda functions
                item_eval_context.set_variable("this".to_string(), item_context_clone.clone());

                // Always use async evaluation
                self_clone
                    .evaluate_with_context_threaded_async(&expr_clone, item_eval_context)
                    .await
                    .map(|(result, _)| result)
                    .map_err(
                        |e| crate::registry::function::FunctionError::EvaluationError {
                            name: "lambda".to_string(),
                            message: format!("Lambda evaluation error: {e}"),
                        },
                    )
            })
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = Result<
                                    crate::model::FhirPathValue,
                                    crate::registry::function::FunctionError,
                                >,
                            > + '_,
                    >,
                >
        };

        // Create an enhanced async lambda evaluator that supports additional variables
        let enhanced_evaluator = |expr: &ExpressionNode,
                                  item_context: &FhirPathValue,
                                  additional_vars: &VarMap| {
            let expr_clone = expr.clone();
            let item_context_clone = item_context.clone();
            let additional_vars_clone = additional_vars.clone();
            let self_clone = self.clone();
            let context_clone = context.clone();

            Box::pin(async move {
                // Create a new evaluation context with the item as input
                let mut item_eval_context = context_clone.with_input(item_context_clone.clone());

                // Explicitly set $this to the current item for lambda functions
                item_eval_context.set_variable("this".to_string(), item_context_clone.clone());

                // Inject additional variables into the context
                for (name, value) in &additional_vars_clone {
                    item_eval_context.set_variable(name.clone(), value.clone());
                }

                // Always use async evaluation
                self_clone
                    .evaluate_with_context_threaded_async(&expr_clone, item_eval_context)
                    .await
                    .map(|(result, _)| result)
                    .map_err(
                        |e| crate::registry::function::FunctionError::EvaluationError {
                            name: "enhanced_lambda".to_string(),
                            message: format!("Enhanced lambda evaluation error: {e}"),
                        },
                    )
            })
                as std::pin::Pin<
                    Box<
                        dyn std::future::Future<
                                Output = Result<
                                    crate::model::FhirPathValue,
                                    crate::registry::function::FunctionError,
                                >,
                            > + '_,
                    >,
                >
        };

        // Try to cast to LambdaFunction and use lambda evaluation
        use crate::registry::function::LambdaFunction;

        // Create lambda evaluation context
        let mut registry_context =
            crate::registry::function::EvaluationContext::with_model_provider(
                context.input.clone(),
                context.model_provider.clone(),
            );
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        let lambda_context = crate::registry::function::LambdaEvaluationContext {
            context: &registry_context,
            evaluator: &evaluator,
            enhanced_evaluator: Some(&enhanced_evaluator),
        };

        // Check if function implements LambdaFunction trait
        // For now, we'll handle known lambda functions explicitly
        match function.name() {
            "all" => {
                use crate::registry::functions::boolean::AllFunction;
                let all_fn = AllFunction;
                all_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .await
                    .map_err(EvaluationError::Function)
            }
            "select" => {
                use crate::registry::functions::filtering::SelectFunction;
                let select_fn = SelectFunction;
                select_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .await
                    .map_err(EvaluationError::Function)
            }
            "where" => {
                use crate::registry::functions::filtering::WhereFunction;
                let where_fn = WhereFunction;
                where_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .await
                    .map_err(EvaluationError::Function)
            }
            "aggregate" => {
                use crate::registry::functions::collection::AggregateFunction;
                let aggregate_fn = AggregateFunction;
                aggregate_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .await
                    .map_err(EvaluationError::Function)
            }
            "sort" => {
                use crate::registry::functions::collection::SortFunction;
                let sort_fn = SortFunction;
                sort_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .await
                    .map_err(EvaluationError::Function)
            }
            "exists" => {
                use crate::registry::functions::collection::ExistsFunction;
                let exists_fn = ExistsFunction;
                exists_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .await
                    .map_err(EvaluationError::Function)
            }
            _ => {
                // Fall back to regular function evaluation for other functions
                self.evaluate_function_call_regular_async(function, args, context)
                    .await
            }
        }
    }

    /// Regular function evaluation for functions that don't support lambdas (async version)
    async fn evaluate_function_call_regular_async(
        &self,
        function: &crate::registry::function::FunctionImpl,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context(arg, context).await?;
            arg_values.push(value);
        }

        // Unwrap single-item collections for function arguments
        let unwrapped_args = unwrap_function_arguments(arg_values);

        // Create a compatible context for the function registry
        let mut registry_context =
            crate::registry::function::EvaluationContext::with_model_provider(
                context.input.clone(),
                context.model_provider.clone(),
            );
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        // Evaluate function with async support
        let result = function
            .evaluate_async(&unwrapped_args, &registry_context)
            .await?;
        Ok(result)
    }

    /// Regular function evaluation for functions that don't support lambdas
    fn evaluate_function_call_regular(
        &self,
        function: &crate::registry::function::FunctionImpl,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context_old(arg, context)?;
            arg_values.push(value);
        }

        // Unwrap single-item collections for function arguments
        let unwrapped_args = unwrap_function_arguments(arg_values);

        // Create a compatible context for the function registry
        let mut registry_context =
            crate::registry::function::EvaluationContext::with_model_provider(
                context.input.clone(),
                context.model_provider.clone(),
            );
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        // Evaluate function
        let result = function.evaluate(&unwrapped_args, &registry_context)?;
        Ok(result)
    }

    /// Evaluate a method call (async version)
    async fn evaluate_method_call_async(
        &self,
        base: &ExpressionNode,
        method: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Check if we need to thread context through the method call chain
        if self.needs_variable_scoping(base) {
            // Use threaded context evaluation to preserve variables from defineVariable calls
            let (base_value, updated_context) = self
                .evaluate_with_context_threaded_async(base, context.clone())
                .await?;
            self.evaluate_method_call_direct_async(
                method,
                args,
                &updated_context.with_input(base_value),
            )
            .await
        } else {
            // First evaluate the base expression to get the context for the method call
            let base_value = self.evaluate_with_context(base, context).await?;
            self.evaluate_method_call_direct_async(method, args, &context.with_input(base_value))
                .await
        }
    }

    /// Evaluate a method call
    fn evaluate_method_call(
        &self,
        base: &ExpressionNode,
        method: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Check if we need to thread context through the method call chain
        if self.needs_variable_scoping(base) {
            // Use threaded context evaluation to preserve variables from defineVariable calls
            let (base_value, updated_context) =
                self.evaluate_with_context_threaded(base, context.clone())?;
            self.evaluate_method_call_direct(method, args, &updated_context.with_input(base_value))
        } else {
            // First evaluate the base expression to get the context for the method call
            let base_value = self.evaluate_with_context_old(base, context)?;
            self.evaluate_method_call_direct(method, args, &context.with_input(base_value))
        }
    }

    /// Evaluate a method call with already-evaluated base value (async version)
    async fn evaluate_method_call_direct_async(
        &self,
        method: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Check if this is a collection-level function that should operate on the entire collection
        let is_collection_level_function = matches!(
            method,
            "count" | "exists" | "isDistinct" | "single" | "distinct" | "empty" |
            "allTrue" | "anyTrue" | "allFalse" | "anyFalse" | "aggregate" |
            "select" | "where" | "all" | "any" |  // Lambda functions should operate on collections
            "first" | "last" | "tail" | "skip" | "take" |  // Collection navigation functions
            "join" | // String functions that operate on collections
            "subsetOf" | "supersetOf" | "intersect" | "exclude" | "combine" | // Set operations
            "sort" | // Sort function should operate on the entire collection
            "repeat" // Repeat function should operate on the entire collection
        );

        // For collection-level functions, always operate on the entire collection
        if is_collection_level_function {
            return self
                .evaluate_function_call_async(method, args, context)
                .await;
        }

        // For method calls on collections, we need to handle them properly
        match &context.input {
            FhirPathValue::Collection(items) => {
                let items_vec: Vec<FhirPathValue> = items.iter().cloned().collect();
                // For single-element collections, unwrap and call method on the element
                if items_vec.len() == 1 {
                    let method_context = context.with_input(items_vec[0].clone());
                    self.evaluate_function_call_async(method, args, &method_context)
                        .await
                } else {
                    // For multi-element collections, call method on each element and collect results
                    let mut results = Vec::new();
                    for item in items_vec {
                        let method_context = context.with_input(item.clone());
                        match self
                            .evaluate_function_call_async(method, args, &method_context)
                            .await
                        {
                            Ok(result) => match result {
                                FhirPathValue::Collection(sub_items) => {
                                    for sub_item in sub_items.iter() {
                                        results.push(sub_item.clone());
                                    }
                                }
                                FhirPathValue::Empty => {}
                                single_item => results.push(single_item),
                            },
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => {
                // For non-collections, call function directly on the current input
                self.evaluate_function_call_async(method, args, context)
                    .await
            }
        }
    }

    /// Evaluate a method call with already-evaluated base value
    fn evaluate_method_call_direct(
        &self,
        method: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Check if this is a collection-level function that should operate on the entire collection
        let is_collection_level_function = matches!(
            method,
            "count" | "exists" | "isDistinct" | "single" | "distinct" | "empty" |
            "allTrue" | "anyTrue" | "allFalse" | "anyFalse" | "aggregate" |
            "select" | "where" | "all" | "any" |  // Lambda functions should operate on collections
            "first" | "last" | "tail" | "skip" | "take" |  // Collection navigation functions
            "join" | // String functions that operate on collections
            "subsetOf" | "supersetOf" | "intersect" | "exclude" | "combine" | // Set operations
            "sort" | // Sort function should operate on the entire collection
            "repeat" // Repeat function should operate on the entire collection
        );

        // For collection-level functions, always operate on the entire collection
        if is_collection_level_function {
            return self.evaluate_function_call(method, args, context);
        }

        // For method calls on collections, we need to handle them properly
        match &context.input {
            FhirPathValue::Collection(items) => {
                let items_vec: Vec<FhirPathValue> = items.iter().cloned().collect();
                // For single-element collections, unwrap and call method on the element
                if items_vec.len() == 1 {
                    let method_context = context.with_input(items_vec[0].clone());
                    self.evaluate_function_call(method, args, &method_context)
                } else {
                    // For multi-element collections, call method on each element and collect results
                    let mut results = Vec::new();
                    for item in items_vec {
                        let method_context = context.with_input(item.clone());
                        match self.evaluate_function_call(method, args, &method_context) {
                            Ok(result) => match result {
                                FhirPathValue::Collection(sub_items) => {
                                    for sub_item in sub_items.iter() {
                                        results.push(sub_item.clone());
                                    }
                                }
                                FhirPathValue::Empty => {}
                                single_item => results.push(single_item),
                            },
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => {
                // For non-collections, call function directly on the current input
                self.evaluate_function_call(method, args, context)
            }
        }
    }

    /// Evaluate a binary operation (async version)
    async fn evaluate_binary_op_async(
        &self,
        op: &BinaryOperator,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let left_val = self.evaluate_with_context(left, context).await?;
        let right_val = self.evaluate_with_context(right, context).await?;

        // Use operator registry
        let op_symbol = op.as_str(); // Convert enum to string
        let operator = context.operators.get_binary(op_symbol).ok_or_else(|| {
            EvaluationError::Operator(format!("Unknown binary operator: {op_symbol}"))
        })?;

        // For binary operations, we need to unwrap single-element collections
        // according to FHIRPath semantics
        let left_operand = match &left_val {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap().clone(),
            _ => left_val.clone(),
        };

        let right_operand = match &right_val {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap().clone(),
            _ => right_val.clone(),
        };

        operator
            .evaluate_binary(&left_operand, &right_operand)
            .map_err(|e| EvaluationError::Operator(e.to_string()))
    }

    /// Evaluate a binary operation
    fn evaluate_binary_op(
        &self,
        op: &BinaryOperator,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let left_val = self.evaluate_with_context_old(left, context)?;
        let right_val = self.evaluate_with_context_old(right, context)?;

        // Use operator registry
        let op_symbol = op.as_str(); // Convert enum to string
        let operator = context.operators.get_binary(op_symbol).ok_or_else(|| {
            EvaluationError::Operator(format!("Unknown binary operator: {op_symbol}"))
        })?;

        // For binary operations, we need to unwrap single-element collections
        // according to FHIRPath semantics
        let left_operand = match &left_val {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap().clone(),
            _ => left_val.clone(),
        };

        let right_operand = match &right_val {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap().clone(),
            _ => right_val.clone(),
        };

        operator
            .evaluate_binary(&left_operand, &right_operand)
            .map_err(|e| EvaluationError::Operator(e.to_string()))
    }

    /// Evaluate a unary operation (async version)
    async fn evaluate_unary_op_async(
        &self,
        op: &UnaryOperator,
        operand: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let operand_val = self.evaluate_with_context(operand, context).await?;

        // Handle basic unary operations
        match op {
            UnaryOperator::Not => match operand_val {
                FhirPathValue::Boolean(b) => {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)]))
                }
                FhirPathValue::Empty => {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        true,
                    )]))
                }
                _ => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    false,
                )])),
            },
            UnaryOperator::Minus => {
                // Handle collections by unwrapping single-element collections
                let value_to_process = match &operand_val {
                    FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
                    _ => &operand_val,
                };

                match value_to_process {
                    FhirPathValue::Integer(i) => {
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(-i)]))
                    }
                    FhirPathValue::Decimal(d) => {
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(-d)]))
                    }
                    FhirPathValue::Quantity(q) => {
                        let negated = q.multiply_scalar(rust_decimal::Decimal::from(-1));
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(
                            negated.into(),
                        )]))
                    }
                    _ => Err(EvaluationError::TypeError {
                        expected: "Number or Quantity".to_string(),
                        actual: value_to_process.type_name().to_string(),
                    }),
                }
            }
            UnaryOperator::Plus => match operand_val {
                FhirPathValue::Integer(_)
                | FhirPathValue::Decimal(_)
                | FhirPathValue::Quantity(_) => Ok(FhirPathValue::collection(vec![operand_val])),
                _ => Err(EvaluationError::TypeError {
                    expected: "Number or Quantity".to_string(),
                    actual: operand_val.type_name().to_string(),
                }),
            },
        }
    }

    /// Evaluate a unary operation
    fn evaluate_unary_op(
        &self,
        op: &UnaryOperator,
        operand: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let operand_val = self.evaluate_with_context_old(operand, context)?;

        // Handle basic unary operations
        match op {
            UnaryOperator::Not => match operand_val {
                FhirPathValue::Boolean(b) => {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)]))
                }
                FhirPathValue::Empty => {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        true,
                    )]))
                }
                _ => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    false,
                )])),
            },
            UnaryOperator::Minus => {
                // Handle collections by unwrapping single-element collections
                let value_to_process = match &operand_val {
                    FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
                    _ => &operand_val,
                };

                match value_to_process {
                    FhirPathValue::Integer(i) => {
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(-i)]))
                    }
                    FhirPathValue::Decimal(d) => {
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(-d)]))
                    }
                    FhirPathValue::Quantity(q) => {
                        let negated = q.multiply_scalar(rust_decimal::Decimal::from(-1));
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(
                            negated.into(),
                        )]))
                    }
                    _ => Err(EvaluationError::TypeError {
                        expected: "Number or Quantity".to_string(),
                        actual: value_to_process.type_name().to_string(),
                    }),
                }
            }
            UnaryOperator::Plus => match operand_val {
                FhirPathValue::Integer(_)
                | FhirPathValue::Decimal(_)
                | FhirPathValue::Quantity(_) => Ok(FhirPathValue::collection(vec![operand_val])),
                _ => Err(EvaluationError::TypeError {
                    expected: "Number or Quantity".to_string(),
                    actual: operand_val.type_name().to_string(),
                }),
            },
        }
    }

    /// Evaluate path navigation
    fn evaluate_path(
        &self,
        base: &ExpressionNode,
        path: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let base_val = self.evaluate_with_context_old(base, context)?;
        let new_context = context.with_input(base_val);
        self.evaluate_identifier(path, &new_context)
    }

    /// Evaluate index access
    fn evaluate_index(
        &self,
        base: &ExpressionNode,
        index: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let base_val = self.evaluate_with_context_old(base, context)?;
        let index_val = self.evaluate_with_context_old(index, context)?;

        let index_num = match &index_val {
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Collection(items) if items.len() == 1 => match items.get(0) {
                Some(FhirPathValue::Integer(i)) => *i,
                _ => {
                    return Err(EvaluationError::TypeError {
                        expected: "Integer".to_string(),
                        actual: index_val.type_name().to_string(),
                    });
                }
            },
            _ => {
                return Err(EvaluationError::TypeError {
                    expected: "Integer".to_string(),
                    actual: index_val.type_name().to_string(),
                });
            }
        };

        match base_val {
            FhirPathValue::Collection(items) => {
                // Handle negative indexing (from end of collection)
                let effective_index = if index_num < 0 {
                    let len = items.len() as i64;
                    len + index_num
                } else {
                    index_num
                };

                // Return empty collection for out-of-bounds access (FHIRPath spec)
                if effective_index < 0 || effective_index as usize >= items.len() {
                    Ok(FhirPathValue::Collection(vec![].into()))
                } else {
                    Ok(items.get(effective_index as usize).unwrap().clone())
                }
            }
            _ => {
                // Single item is treated as single-item collection for indexing
                let single_item_collection = [base_val];

                // Handle negative indexing
                let effective_index = if index_num < 0 {
                    1 + index_num // Length is 1 for single item
                } else {
                    index_num
                };

                // Return empty collection for out-of-bounds access
                if effective_index < 0 || effective_index as usize >= 1 {
                    Ok(FhirPathValue::Collection(vec![].into()))
                } else {
                    Ok(single_item_collection
                        .get(effective_index as usize)
                        .unwrap()
                        .clone())
                }
            }
        }
    }

    /// Evaluate filter expression
    fn evaluate_filter(
        &self,
        base: &ExpressionNode,
        condition: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let base_val = self.evaluate_with_context_old(base, context)?;

        match base_val {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();

                for item in items {
                    let item_context = context.with_input(item.clone());
                    let condition_result =
                        self.evaluate_with_context_old(condition, &item_context)?;

                    if let FhirPathValue::Boolean(true) = condition_result {
                        results.push(item)
                    }
                }

                Ok(FhirPathValue::collection(results))
            }
            other => {
                // For non-collections, treat as single-item collection
                let item_context = context.with_input(other.clone());
                let condition_result = self.evaluate_with_context_old(condition, &item_context)?;

                match condition_result {
                    FhirPathValue::Boolean(true) => Ok(other),
                    _ => Ok(FhirPathValue::Empty),
                }
            }
        }
    }

    /// Evaluate union operation
    fn evaluate_union(
        &self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // For union operations, each side should be evaluated with a fresh variable context
        // to ensure proper variable scoping as per FHIRPath specification
        let left_context = context.with_fresh_variable_scope();

        let right_context = context.with_fresh_variable_scope();

        let left_val = self.evaluate_with_context_old(left, &left_context)?;
        let right_val = self.evaluate_with_context_old(right, &right_context)?;

        let mut items = Vec::new();

        // Add items from left
        match left_val {
            FhirPathValue::Collection(left_items) => items.extend(left_items),
            FhirPathValue::Empty => {}
            other => items.push(other),
        }

        // Add items from right, removing duplicates
        match right_val {
            FhirPathValue::Collection(right_items) => {
                for item in right_items {
                    if !items.contains(&item) {
                        items.push(item);
                    }
                }
            }
            FhirPathValue::Empty => {}
            other => {
                if !items.contains(&other) {
                    items.push(other);
                }
            }
        }

        Ok(FhirPathValue::collection(items))
    }

    /// Evaluate type check
    fn evaluate_type_check(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let value = self.evaluate_with_context_old(expression, context)?;

        let matches = match &value {
            FhirPathValue::Collection(items) => {
                // For collections, check if it has exactly one item of the specified type
                if items.len() == 1 {
                    check_value_type(items.get(0).unwrap(), type_name)
                } else {
                    false
                }
            }
            single_value => check_value_type(single_value, type_name),
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            matches,
        )]))
    }

    /// Evaluate type cast
    fn evaluate_type_cast(
        &self,
        expression: &ExpressionNode,
        type_name: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let value = self.evaluate_with_context_old(expression, context)?;

        // Basic type casting - can be enhanced later
        match (type_name, &value) {
            ("String", _) => {
                if let Some(s) = value.to_string_value() {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                        s.into(),
                    )]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            _ => Ok(FhirPathValue::collection(vec![value])), // For now, just return the value as-is
        }
    }

    /// Evaluate conditional expression
    fn evaluate_conditional(
        &self,
        condition: &ExpressionNode,
        then_expr: &ExpressionNode,
        else_expr: Option<&ExpressionNode>,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let condition_val = self.evaluate_with_context_old(condition, context)?;

        match condition_val {
            FhirPathValue::Boolean(true) => self.evaluate_with_context_old(then_expr, context),
            _ => {
                if let Some(else_branch) = else_expr {
                    self.evaluate_with_context_old(else_branch, context)
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }
}

/// Helper function to check if a value matches a type name
fn check_value_type(value: &FhirPathValue, type_name: &str) -> bool {
    match value {
        FhirPathValue::Boolean(_) => {
            matches!(
                type_name,
                "Boolean" | "System.Boolean" | "boolean" | "FHIR.boolean"
            )
        }
        FhirPathValue::Integer(_) => {
            matches!(
                type_name,
                "Integer" | "System.Integer" | "integer" | "FHIR.integer"
            )
        }
        FhirPathValue::Decimal(_) => {
            matches!(
                type_name,
                "Decimal" | "System.Decimal" | "decimal" | "FHIR.decimal"
            )
        }
        FhirPathValue::String(_) => {
            matches!(
                type_name,
                "String"
                    | "System.String"
                    | "string"
                    | "FHIR.string"
                    | "uri"
                    | "FHIR.uri"
                    | "uuid"
                    | "FHIR.uuid"
            )
        }
        FhirPathValue::Date(_) => {
            matches!(type_name, "Date" | "System.Date" | "date" | "FHIR.date")
        }
        FhirPathValue::DateTime(_) => {
            matches!(
                type_name,
                "DateTime" | "System.DateTime" | "dateTime" | "FHIR.dateTime"
            )
        }
        FhirPathValue::Time(_) => {
            matches!(type_name, "Time" | "System.Time" | "time" | "FHIR.time")
        }
        FhirPathValue::Quantity(_) => {
            matches!(type_name, "Quantity" | "System.Quantity" | "FHIR.Quantity")
        }
        FhirPathValue::Empty => false,
        FhirPathValue::Resource(resource) => {
            // Check FHIR resource type - support both with and without FHIR prefix
            if let Some(resource_type) = resource.resource_type() {
                resource_type == type_name
                    || type_name == format!("FHIR.{resource_type}")
                    || type_name == format!("FHIR.`{resource_type}`")
            } else {
                false
            }
        }
        FhirPathValue::Collection(_) => {
            matches!(type_name, "Collection")
        }
        FhirPathValue::TypeInfoObject { .. } => {
            matches!(type_name, "TypeInfo" | "System.TypeInfo")
        }
        FhirPathValue::JsonValue(_) => {
            // JsonValue can match various types depending on content
            matches!(type_name, "JsonValue" | "Object" | "Any")
        }
    }
}

/// Helper function to unwrap function arguments that should be single values
/// According to FHIRPath semantics, single-item collections should be unwrapped for function arguments
fn unwrap_function_arguments(args: Vec<FhirPathValue>) -> Vec<FhirPathValue> {
    args.into_iter()
        .map(|arg| match arg {
            FhirPathValue::Collection(items) if items.len() == 1 => {
                items.into_iter().next().unwrap()
            }
            other => other,
        })
        .collect()
}

/// Parse a FHIRPath date literal (supports partial dates: @YYYY, @YYYY-MM, @YYYY-MM-DD)
fn parse_fhirpath_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    // Remove the @ prefix
    let date_str = s.strip_prefix('@').unwrap_or(s);

    // Try full date format first
    if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(date);
    }

    // Try year-month format (default to first day of month)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{date_str}-01"), "%Y-%m-%d") {
        return Ok(date);
    }

    // Try year-only format (default to January 1st)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{date_str}-01-01"), "%Y-%m-%d") {
        return Ok(date);
    }

    // If none work, return error with the original string for better error reporting
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

/// Parse a FHIRPath datetime literal (supports partial: @YYYY, @YYYY-MM, @YYYY-MM-DD, @YYYY-MM-DDTHH, etc.)
fn parse_fhirpath_datetime(
    s: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, chrono::ParseError> {
    use chrono::TimeZone;

    // Remove the @ prefix
    let datetime_str = s.strip_prefix('@').unwrap_or(s);

    // Try different datetime formats
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(datetime_str) {
        return Ok(dt.fixed_offset());
    }

    // Try format with timezone offset
    if let Ok(dt) = chrono::DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3f%z") {
        return Ok(dt.fixed_offset());
    }

    // Try format with timezone offset (hour:minute only, no seconds)
    if let Ok(dt) = chrono::DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M%z") {
        return Ok(dt.fixed_offset());
    }

    // Try format without timezone (assume UTC)
    if let Ok(naive_dt) =
        chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3f")
    {
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt).fixed_offset());
    }

    // Try basic format without milliseconds
    if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S") {
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt).fixed_offset());
    }

    // Try format with just hour and minute
    if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M") {
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt).fixed_offset());
    }

    // Try format with just hour
    if let Ok(naive_dt) =
        chrono::NaiveDateTime::parse_from_str(&format!("{datetime_str}:00:00"), "%Y-%m-%dT%H:%M:%S")
    {
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt).fixed_offset());
    }

    // Handle partial datetime formats ending with T
    if datetime_str.ends_with('T') {
        let date_part = datetime_str.trim_end_matches('T');

        // Try to parse as a date and convert to datetime
        if let Ok(date) = parse_fhirpath_date(&format!("@{date_part}")) {
            let naive_dt = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(chrono::Utc.from_utc_datetime(&naive_dt).fixed_offset());
        }
    }

    // Handle date-only parts as datetime (midnight)
    if let Ok(date) = parse_fhirpath_date(&format!("@{datetime_str}")) {
        let naive_dt = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt).fixed_offset());
    }

    // Return a simple parse error by trying an invalid format to get a real ParseError
    chrono::NaiveDateTime::parse_from_str("invalid", "%Y-%m-%d").map(|_| {
        chrono::Utc
            .from_utc_datetime(&chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ))
            .fixed_offset()
    })
}

/// Parse a FHIRPath time literal (supports partial: @T14, @T14:30, @THH:MM:SS.sss)
fn parse_fhirpath_time(s: &str) -> Result<chrono::NaiveTime, chrono::ParseError> {
    // Remove the @T prefix
    let time_str = s
        .strip_prefix('@')
        .and_then(|s| s.strip_prefix('T'))
        .unwrap_or(s);

    // Try format with milliseconds
    if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str, "%H:%M:%S%.3f") {
        return Ok(time);
    }

    // Try basic format without milliseconds
    if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str, "%H:%M:%S") {
        return Ok(time);
    }

    // Try hour and minute format (default seconds to 0)
    if let Ok(time) = chrono::NaiveTime::parse_from_str(time_str, "%H:%M") {
        return Ok(time);
    }

    // Try hour-only format (default minutes and seconds to 0)
    if let Ok(hour) = time_str.parse::<u32>() {
        if let Some(time) = chrono::NaiveTime::from_hms_opt(hour, 0, 0) {
            return Ok(time);
        }
    }

    // If none work, return error with the original format for better error reporting
    chrono::NaiveTime::parse_from_str(time_str, "%H:%M:%S")
}

/// Check if a function name corresponds to a lambda function
fn is_lambda_function(name: &str) -> bool {
    matches!(
        name,
        "all" | "any" | "exists" | "select" | "where" | "aggregate" | "sort"
    )
}

impl FhirPathEngine {
    /// Check if a variable name is protected (system variable that cannot be redefined)
    fn is_protected_variable(&self, name: &str) -> bool {
        matches!(
            name,
            "context"
                | "resource"
                | "rootResource"
                | "this"
                | "index"
                | "total"
                | "$context"
                | "$resource"
                | "$rootResource"
                | "$this"
                | "$index"
                | "$total"
                | "ucum"
                | "$ucum"
        )
    }

    /// Estimate the computational complexity of an expression for optimization decisions
    ///
    /// Returns a complexity score that helps determine whether bytecode compilation
    /// would be beneficial. Higher scores indicate more complex expressions.
    pub fn estimate_expression_complexity(&self, expression: &ExpressionNode) -> usize {
        match expression {
            // Simple operations - low complexity
            ExpressionNode::Literal(_) => 1,
            ExpressionNode::Identifier(_) => 1,
            ExpressionNode::Variable(_) => 1,

            // Function and method calls - moderate complexity
            ExpressionNode::FunctionCall(data) => {
                let args_complexity: usize = data
                    .args
                    .iter()
                    .map(|arg| self.estimate_expression_complexity(arg))
                    .sum();
                5 + args_complexity // Function calls have significant overhead
            }
            ExpressionNode::MethodCall(data) => {
                let base_complexity = self.estimate_expression_complexity(&data.base);
                let args_complexity: usize = data
                    .args
                    .iter()
                    .map(|arg| self.estimate_expression_complexity(arg))
                    .sum();
                base_complexity + 5 + args_complexity
            }

            // Binary and unary operations - low to moderate complexity
            ExpressionNode::BinaryOp(data) => {
                let left_complexity = self.estimate_expression_complexity(&data.left);
                let right_complexity = self.estimate_expression_complexity(&data.right);
                3 + left_complexity + right_complexity
            }
            ExpressionNode::UnaryOp { operand, .. } => {
                2 + self.estimate_expression_complexity(operand)
            }

            // Path operations - moderate complexity
            ExpressionNode::Path { base, .. } => 3 + self.estimate_expression_complexity(base),
            ExpressionNode::Index { base, index } => {
                let base_complexity = self.estimate_expression_complexity(base);
                let index_complexity = self.estimate_expression_complexity(index);
                4 + base_complexity + index_complexity
            }

            // High-complexity operations that benefit most from compilation
            ExpressionNode::Filter { base, condition } => {
                let base_complexity = self.estimate_expression_complexity(base);
                let condition_complexity = self.estimate_expression_complexity(condition);
                // Filters are expensive - they potentially iterate over large collections
                15 + base_complexity + condition_complexity
            }
            ExpressionNode::Union { left, right } => {
                let left_complexity = self.estimate_expression_complexity(left);
                let right_complexity = self.estimate_expression_complexity(right);
                8 + left_complexity + right_complexity
            }
            ExpressionNode::Lambda(data) => {
                // Lambda expressions are very expensive and benefit greatly from compilation
                25 + self.estimate_expression_complexity(&data.body)
            }

            // Control flow - moderate complexity
            ExpressionNode::Conditional(data) => {
                let condition_complexity = self.estimate_expression_complexity(&data.condition);
                let then_complexity = self.estimate_expression_complexity(&data.then_expr);
                let else_complexity = data
                    .else_expr
                    .as_ref()
                    .map(|e| self.estimate_expression_complexity(e))
                    .unwrap_or(0);
                5 + condition_complexity + then_complexity + else_complexity
            }

            // Type operations - low to moderate complexity
            ExpressionNode::TypeCheck { expression, .. } => {
                3 + self.estimate_expression_complexity(expression)
            }
            ExpressionNode::TypeCast { expression, .. } => {
                4 + self.estimate_expression_complexity(expression)
            }
        }
    }
}
