//\! Main FHIRPath evaluation engine

use crate::{EvaluationContext, EvaluationError, EvaluationResult};
use fhirpath_ast::{BinaryOperator, ExpressionNode, LiteralValue, UnaryOperator};
use fhirpath_model::{FhirPathValue, FhirResource};
use fhirpath_registry::{FunctionRegistry, OperatorRegistry};
use serde_json::Value;
// Lambda functions are not yet fully implemented
// use fhirpath_registry::function::{AllFunction, AnyFunction, ExistsFunction};
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
}

impl FhirPathEngine {
    /// Create a new engine with default built-in functions and operators
    pub fn new() -> Self {
        let (functions, operators) = fhirpath_registry::create_standard_registries();

        Self {
            functions: Arc::new(functions),
            operators: Arc::new(operators),
        }
    }

    /// Create a new engine with custom registries
    pub fn with_registries(
        functions: Arc<FunctionRegistry>,
        operators: Arc<OperatorRegistry>,
    ) -> Self {
        Self {
            functions,
            operators,
        }
    }

    /// Extract a type name from an expression node (for handling 'is' function arguments)
    /// Returns the full dotted path as a string for identifiers and path expressions
    fn extract_type_name(&self, expr: &ExpressionNode) -> Option<String> {
        match expr {
            ExpressionNode::Identifier(name) => Some(name.clone()),
            ExpressionNode::Path { base, path } => {
                // Recursively build the dotted path (e.g., System.Boolean)
                if let Some(base_name) = self.extract_type_name(base) {
                    Some(format!("{}.{}", base_name, path))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Evaluate an FHIRPath expression against input data
    pub fn evaluate(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let context = EvaluationContext::new(input, self.functions.clone(), self.operators.clone());

        self.evaluate_with_context(expression, &context)
    }

    /// Evaluate with explicit context and return updated context for defineVariable chains
    pub fn evaluate_with_context_ext(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<(FhirPathValue, EvaluationContext)> {
        match expression {
            ExpressionNode::MethodCall { base, method, args } if method == "defineVariable" => {
                // Special handling for defineVariable to thread context through method chains
                let base_result = self.evaluate_with_context(base, context)?;
                let define_context = context.with_input(base_result.clone());

                if args.len() != 2 {
                    return Err(EvaluationError::InvalidOperation {
                        message: "defineVariable requires exactly 2 arguments: name and value"
                            .to_string(),
                    });
                }

                // Evaluate variable name and value
                let name_value = self.evaluate_with_context(&args[0], &define_context)?;
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

                let var_value = self.evaluate_with_context(&args[1], &define_context)?;

                // Create new context with variable set
                let mut new_context = define_context.clone();
                new_context.set_variable(var_name, var_value);

                Ok((base_result, new_context))
            }
            _ => {
                // For other expressions, use regular evaluation
                let result = self.evaluate_with_context(expression, context)?;
                Ok((result, context.clone()))
            }
        }
    }

    /// Evaluate with explicit context and return both result and updated context
    pub fn evaluate_with_context_threaded(
        &self,
        expression: &ExpressionNode,
        context: EvaluationContext,
    ) -> EvaluationResult<(FhirPathValue, EvaluationContext)> {
        match expression {
            ExpressionNode::MethodCall { base, method, args } if method == "defineVariable" => {
                // Special handling for defineVariable to thread context properly
                let (base_value, mut updated_context) =
                    self.evaluate_with_context_threaded(base, context)?;

                if args.len() != 2 {
                    return Err(EvaluationError::InvalidOperation {
                        message: "defineVariable requires exactly 2 arguments: name and value"
                            .to_string(),
                    });
                }

                // Create context with base value as input
                let define_context = updated_context.with_input(base_value.clone());

                // Evaluate variable name and value
                let (name_value, _) =
                    self.evaluate_with_context_threaded(&args[0], define_context.clone())?;
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

                let (var_value, _) =
                    self.evaluate_with_context_threaded(&args[1], define_context)?;

                // Store the variable in the context
                updated_context.set_variable(var_name, var_value);

                // Return the base value with updated context
                let result = match &base_value {
                    FhirPathValue::Empty => FhirPathValue::collection(vec![FhirPathValue::Empty]),
                    _ => base_value,
                };
                Ok((result, updated_context))
            }

            ExpressionNode::Union { left, right } => {
                // For union operations, evaluate each side with the same initial context
                // but isolated variable scoping will happen during defineVariable evaluation
                let (left_val, _) = self.evaluate_with_context_threaded(left, context.clone())?;
                let (right_val, _) = self.evaluate_with_context_threaded(right, context.clone())?;

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

            ExpressionNode::MethodCall { base, method, args } => {
                // For other method calls, thread context through base evaluation
                let (base_value, updated_context) =
                    self.evaluate_with_context_threaded(base, context)?;
                let method_context = updated_context.with_input(base_value);
                let result = self.evaluate_method_call_direct(method, args, &method_context)?;
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

            ExpressionNode::FunctionCall { name, args } => {
                self.evaluate_function_call(name, args, context)
            }

            ExpressionNode::MethodCall { base, method, args } => {
                self.evaluate_method_call(base, method, args, context)
            }

            ExpressionNode::BinaryOp { op, left, right } => {
                self.evaluate_binary_op(op, left, right, context)
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

            ExpressionNode::Lambda { param: _, body } => {
                // Lambda expressions are context-dependent
                // For now, evaluate body directly
                self.evaluate_with_context_old(body, context)
            }

            ExpressionNode::Conditional {
                condition,
                then_expr,
                else_expr,
            } => self.evaluate_conditional(condition, then_expr, else_expr.as_deref(), context),
        }
    }

    /// Evaluate with explicit context (wrapper for backward compatibility)
    pub fn evaluate_with_context(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Use the new threaded evaluation for expressions that need variable scoping
        if self.needs_variable_scoping(expression) {
            let (result, _) = self.evaluate_with_context_threaded(expression, context.clone())?;
            Ok(result)
        } else {
            // Use the old method for simple expressions
            self.evaluate_with_context_old(expression, context)
        }
    }

    /// Check if an expression needs variable scoping (contains defineVariable or union)
    fn needs_variable_scoping(&self, expression: &ExpressionNode) -> bool {
        match expression {
            ExpressionNode::MethodCall {
                method,
                base,
                args: _,
            } => method == "defineVariable" || self.needs_variable_scoping(base),
            ExpressionNode::Union { left, right } => {
                self.needs_variable_scoping(left) || self.needs_variable_scoping(right)
            }
            ExpressionNode::BinaryOp { left, right, op: _ } => {
                self.needs_variable_scoping(left) || self.needs_variable_scoping(right)
            }
            ExpressionNode::UnaryOp { operand, op: _ } => self.needs_variable_scoping(operand),
            ExpressionNode::Path { base, path: _ } => self.needs_variable_scoping(base),
            ExpressionNode::Index { base, index } => {
                self.needs_variable_scoping(base) || self.needs_variable_scoping(index)
            }
            ExpressionNode::Filter { base, condition } => {
                self.needs_variable_scoping(base) || self.needs_variable_scoping(condition)
            }
            ExpressionNode::FunctionCall { args, name: _ } => {
                args.iter().any(|arg| self.needs_variable_scoping(arg))
            }
            ExpressionNode::Lambda { body, param: _ } => self.needs_variable_scoping(body),
            ExpressionNode::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.needs_variable_scoping(condition)
                    || self.needs_variable_scoping(then_expr)
                    || else_expr
                        .as_ref()
                        .map_or(false, |e| self.needs_variable_scoping(e))
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
                        message: format!("Invalid decimal literal: {}", s),
                    });
                }
            },
            LiteralValue::String(s) => FhirPathValue::String(s.clone()),
            LiteralValue::Date(s) => match parse_fhirpath_date(s) {
                Ok(date) => FhirPathValue::Date(date),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid date literal: {}", s),
                    });
                }
            },
            LiteralValue::DateTime(s) => match parse_fhirpath_datetime(s) {
                Ok(datetime) => FhirPathValue::DateTime(datetime.into()),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid datetime literal: {}", s),
                    });
                }
            },
            LiteralValue::Time(s) => match parse_fhirpath_time(s) {
                Ok(time) => FhirPathValue::Time(time),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid time literal: {}", s),
                    });
                }
            },
            LiteralValue::Quantity { value, unit } => match Decimal::from_str(value) {
                Ok(d) => FhirPathValue::quantity(d, Some(unit.clone())),
                Err(_) => {
                    return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid quantity value: {}", value),
                    });
                }
            },
            LiteralValue::Null => return Ok(FhirPathValue::Empty),
        };

        // In FHIRPath, all values are conceptually collections
        Ok(FhirPathValue::collection(vec![value]))
    }

    /// Evaluate an identifier (property access)
    fn evaluate_identifier(
        &self,
        name: &str,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        match &context.input {
            FhirPathValue::Resource(resource) => {
                // First check if the identifier matches the resource type
                if let Some(resource_type) = resource.resource_type() {
                    if resource_type == name {
                        // Return the resource itself when accessing by resource type
                        return Ok(context.input.clone());
                    }
                }

                // Otherwise try to get the property
                match resource.get_property(name) {
                    Some(value) => {
                        // Convert all values using the standard conversion logic
                        Ok(FhirPathValue::from(value.clone()))
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
                match name {
                    "namespace" => Ok(FhirPathValue::String(namespace.clone())),
                    "name" => Ok(FhirPathValue::String(type_name.clone())),
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
            "$this" | "$" => Ok(context.input.clone()),
            "$$" => Ok(context.root.clone()),
            _ => {
                // Check for variables in the context
                if let Some(value) = context.get_variable(name) {
                    Ok(value.clone())
                } else {
                    // Variable not found - return empty per FHIRPath spec
                    // This ensures proper scoping where undefined variables evaluate to empty
                    Ok(FhirPathValue::Empty)
                }
            }
        }
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
                    message: format!("Unknown function: {}", name),
                })?;

        // Check if this is a lambda function that needs special evaluation
        if is_lambda_function(name) {
            // For lambda functions, we don't evaluate arguments first - we pass the expressions
            return self.evaluate_lambda_function(function, args, context);
        }

        // For regular functions, evaluate arguments normally
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context(arg, context)?;

            // Special handling for 'is' function: if an argument evaluates to Empty,
            // check if it represents a type name (identifier or dotted path) and treat as string literal
            if name == "is" && matches!(value, FhirPathValue::Empty) {
                if let Some(type_name) = self.extract_type_name(arg) {
                    arg_values.push(FhirPathValue::String(type_name));
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
            fhirpath_registry::function::EvaluationContext::new(context.input.clone());
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        // Evaluate function
        let result = function.evaluate(&unwrapped_args, &registry_context)?;
        Ok(result)
    }

    /// Evaluate a lambda function with unevaluated expression arguments
    fn evaluate_lambda_function(
        &self,
        function: std::sync::Arc<dyn fhirpath_registry::function::FhirPathFunction>,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Create a lambda evaluator closure
        let evaluator =
            |expr: &ExpressionNode,
             item_context: &FhirPathValue|
             -> Result<FhirPathValue, fhirpath_registry::function::FunctionError> {
                // Create a new evaluation context with the item as input
                let item_eval_context = context.with_input(item_context.clone());

                // Evaluate the expression in the item context
                self.evaluate_with_context_threaded(expr, item_eval_context.clone())
                    .map(|(result, _)| result)
                    .map_err(
                        |e| fhirpath_registry::function::FunctionError::EvaluationError {
                            name: "lambda".to_string(),
                            message: format!("Lambda evaluation error: {}", e),
                        },
                    )
            };

        // Create an enhanced lambda evaluator that supports additional variables
        let enhanced_evaluator =
            |expr: &ExpressionNode,
             item_context: &FhirPathValue,
             additional_vars: &VarMap|
             -> Result<FhirPathValue, fhirpath_registry::function::FunctionError> {
                // Create a new evaluation context with the item as input
                let mut item_eval_context = context.with_input(item_context.clone());

                // Inject additional variables into the context
                for (name, value) in additional_vars {
                    item_eval_context.set_variable(name.clone(), value.clone());
                }

                // Evaluate the expression in the enhanced context
                self.evaluate_with_context_threaded(expr, item_eval_context.clone())
                    .map(|(result, _)| result)
                    .map_err(
                        |e| fhirpath_registry::function::FunctionError::EvaluationError {
                            name: "enhanced_lambda".to_string(),
                            message: format!("Enhanced lambda evaluation error: {}", e),
                        },
                    )
            };

        // Try to cast to LambdaFunction and use lambda evaluation
        use fhirpath_registry::function::LambdaFunction;

        // Create lambda evaluation context
        let mut registry_context =
            fhirpath_registry::function::EvaluationContext::new(context.input.clone());
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();
        let lambda_context = fhirpath_registry::function::LambdaEvaluationContext {
            context: &registry_context,
            evaluator: &evaluator,
            enhanced_evaluator: None, // Temporarily disabled due to lifetime issues
        };

        // Check if function implements LambdaFunction trait
        // For now, we'll handle known lambda functions explicitly
        match function.name() {
            "all" => {
                use fhirpath_registry::functions::boolean::AllFunction;
                let all_fn = AllFunction;
                all_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .map_err(|e| EvaluationError::Function(e))
            }
            "select" => {
                use fhirpath_registry::functions::filtering::SelectFunction;
                let select_fn = SelectFunction;
                select_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .map_err(|e| EvaluationError::Function(e))
            }
            "where" => {
                use fhirpath_registry::functions::filtering::WhereFunction;
                let where_fn = WhereFunction;
                where_fn
                    .evaluate_with_lambda(args, &lambda_context)
                    .map_err(|e| EvaluationError::Function(e))
            }
            _ => {
                // Fall back to regular function evaluation for other functions
                self.evaluate_function_call_regular(function, args, context)
            }
        }
    }

    /// Regular function evaluation for functions that don't support lambdas
    fn evaluate_function_call_regular(
        &self,
        function: std::sync::Arc<dyn fhirpath_registry::function::FhirPathFunction>,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context(arg, context)?;
            arg_values.push(value);
        }

        // Unwrap single-item collections for function arguments
        let unwrapped_args = unwrap_function_arguments(arg_values);

        // Create a compatible context for the function registry
        let mut registry_context =
            fhirpath_registry::function::EvaluationContext::new(context.input.clone());
        registry_context
            .variables
            .extend(context.variable_scope.collect_all_variables());
        registry_context.root = context.root.clone();

        // Evaluate function
        let result = function.evaluate(&unwrapped_args, &registry_context)?;
        Ok(result)
    }

    /// Evaluate a method call
    fn evaluate_method_call(
        &self,
        base: &ExpressionNode,
        method: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // First evaluate the base expression to get the context for the method call
        let base_value = self.evaluate_with_context(base, context)?;
        self.evaluate_method_call_direct(method, args, &context.with_input(base_value))
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
            "join" // String functions that operate on collections
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

    /// Evaluate a binary operation
    fn evaluate_binary_op(
        &self,
        op: &BinaryOperator,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let left_val = self.evaluate_with_context(left, context)?;
        let right_val = self.evaluate_with_context(right, context)?;

        // Use operator registry
        let op_symbol = op.as_str(); // Convert enum to string
        let operator = context.operators.get_binary(op_symbol).ok_or_else(|| {
            EvaluationError::Operator(format!("Unknown binary operator: {}", op_symbol))
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

    /// Evaluate a unary operation
    fn evaluate_unary_op(
        &self,
        op: &UnaryOperator,
        operand: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let operand_val = self.evaluate_with_context(operand, context)?;

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
                            negated,
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
        let base_val = self.evaluate_with_context(base, context)?;
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
        let base_val = self.evaluate_with_context(base, context)?;
        let index_val = self.evaluate_with_context(index, context)?;

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
                let single_item_collection = vec![base_val];

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
        let base_val = self.evaluate_with_context(base, context)?;

        match base_val {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();

                for item in items {
                    let item_context = context.with_input(item.clone());
                    let condition_result = self.evaluate_with_context(condition, &item_context)?;

                    match condition_result {
                        FhirPathValue::Boolean(true) => results.push(item),
                        _ => {}
                    }
                }

                Ok(FhirPathValue::collection(results))
            }
            other => {
                // For non-collections, treat as single-item collection
                let item_context = context.with_input(other.clone());
                let condition_result = self.evaluate_with_context(condition, &item_context)?;

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

        let left_val = self.evaluate_with_context(left, &left_context)?;
        let right_val = self.evaluate_with_context(right, &right_context)?;

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
        let value = self.evaluate_with_context(expression, context)?;

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
        let value = self.evaluate_with_context(expression, context)?;

        // Basic type casting - can be enhanced later
        match (type_name, &value) {
            ("String", _) => {
                if let Some(s) = value.to_string_value() {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::String(s)]))
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
        let condition_val = self.evaluate_with_context(condition, context)?;

        match condition_val {
            FhirPathValue::Boolean(true) => self.evaluate_with_context(then_expr, context),
            _ => {
                if let Some(else_branch) = else_expr {
                    self.evaluate_with_context(else_branch, context)
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }
}

impl Default for FhirPathEngine {
    fn default() -> Self {
        Self::new()
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
                    || type_name == format!("FHIR.{}", resource_type)
                    || type_name == format!("FHIR.`{}`", resource_type)
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

/// Parse a FHIRPath date literal (format: @YYYY-MM-DD)
fn parse_fhirpath_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    // Remove the @ prefix
    let date_str = s.strip_prefix('@').unwrap_or(s);
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

/// Parse a FHIRPath datetime literal (format: @YYYY-MM-DDTHH:MM:SS.sss+ZZ:ZZ)
fn parse_fhirpath_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>, chrono::ParseError> {
    use chrono::TimeZone;

    // Remove the @ prefix
    let datetime_str = s.strip_prefix('@').unwrap_or(s);

    // Try different datetime formats
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(datetime_str) {
        return Ok(dt.with_timezone(&chrono::Utc));
    }

    // Try format with timezone offset
    if let Ok(dt) = chrono::DateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3f%z") {
        return Ok(dt.with_timezone(&chrono::Utc));
    }

    // Try format without timezone (assume UTC)
    if let Ok(naive_dt) =
        chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S%.3f")
    {
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt));
    }

    // Try basic format without milliseconds
    if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S") {
        return Ok(chrono::Utc.from_utc_datetime(&naive_dt));
    }

    // Return a simple parse error by trying an invalid format to get a real ParseError
    chrono::NaiveDateTime::parse_from_str("invalid", "%Y-%m-%d")
        .map(|_| {
            chrono::Utc.from_utc_datetime(&chrono::NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ))
        })
        .map_err(|e| e)
}

/// Parse a FHIRPath time literal (format: @THH:MM:SS.sss)
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
    chrono::NaiveTime::parse_from_str(time_str, "%H:%M:%S")
}

/// Check if a function name corresponds to a lambda function
fn is_lambda_function(name: &str) -> bool {
    matches!(name, "all" | "any" | "exists" | "select" | "where")
}
