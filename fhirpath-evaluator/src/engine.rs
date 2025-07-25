//\! Main FHIRPath evaluation engine

use crate::{EvaluationContext, EvaluationError, EvaluationResult};
use fhirpath_ast::{ExpressionNode, LiteralValue, BinaryOperator, UnaryOperator};
use fhirpath_model::FhirPathValue;
use fhirpath_registry::{FunctionRegistry, OperatorRegistry};
use rust_decimal::Decimal;
use std::sync::Arc;
use std::str::FromStr;

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
        let functions = FunctionRegistry::new();

        let operators = OperatorRegistry::new();

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
        Self { functions, operators }
    }

    /// Evaluate an FHIRPath expression against input data
    pub fn evaluate(
        &self,
        expression: &ExpressionNode,
        input: FhirPathValue
    ) -> EvaluationResult<FhirPathValue> {
        let context = EvaluationContext::new(
            input,
            self.functions.clone(),
            self.operators.clone(),
        );

        self.evaluate_with_context(expression, &context)
    }

    /// Evaluate with explicit context
    pub fn evaluate_with_context(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        match expression {
            ExpressionNode::Literal(literal) => {
                self.evaluate_literal(literal)
            }

            ExpressionNode::Identifier(name) => {
                self.evaluate_identifier(name, context)
            }

            ExpressionNode::Variable(name) => {
                self.evaluate_variable(name, context)
            }

            ExpressionNode::FunctionCall { name, args } => {
                self.evaluate_function_call(name, args, context)
            }

            ExpressionNode::MethodCall { base, method, args } => {
                self.evaluate_method_call(base, method, args, context)
            }

            ExpressionNode::BinaryOp { op, left, right } => {
                self.evaluate_binary_op(op, left, right, context)
            }

            ExpressionNode::UnaryOp { op, operand } => {
                self.evaluate_unary_op(op, operand, context)
            }

            ExpressionNode::Path { base, path } => {
                self.evaluate_path(base, path, context)
            }

            ExpressionNode::Index { base, index } => {
                self.evaluate_index(base, index, context)
            }

            ExpressionNode::Filter { base, condition } => {
                self.evaluate_filter(base, condition, context)
            }

            ExpressionNode::Union { left, right } => {
                self.evaluate_union(left, right, context)
            }

            ExpressionNode::TypeCheck { expression, type_name } => {
                self.evaluate_type_check(expression, type_name, context)
            }

            ExpressionNode::TypeCast { expression, type_name } => {
                self.evaluate_type_cast(expression, type_name, context)
            }

            ExpressionNode::Lambda { param: _, body } => {
                // Lambda expressions are context-dependent
                // For now, evaluate body directly
                self.evaluate_with_context(body, context)
            }

            ExpressionNode::Conditional { condition, then_expr, else_expr } => {
                self.evaluate_conditional(condition, then_expr, else_expr.as_deref(), context)
            }
        }
    }

    /// Evaluate a literal value
    fn evaluate_literal(&self, literal: &LiteralValue) -> EvaluationResult<FhirPathValue> {
        let value = match literal {
            LiteralValue::Boolean(b) => FhirPathValue::Boolean(*b),
            LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
            LiteralValue::Decimal(s) => {
                match Decimal::from_str(s) {
                    Ok(d) => FhirPathValue::Decimal(d),
                    Err(_) => return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid decimal literal: {}", s),
                    }),
                }
            }
            LiteralValue::String(s) => FhirPathValue::String(s.clone()),
            LiteralValue::Date(s) => {
                // For now, store dates as strings until proper date parsing is implemented
                FhirPathValue::String(s.clone())
            }
            LiteralValue::DateTime(s) => {
                // For now, store datetimes as strings until proper parsing is implemented
                FhirPathValue::String(s.clone())
            }
            LiteralValue::Time(s) => {
                // For now, store times as strings until proper parsing is implemented
                FhirPathValue::String(s.clone())
            }
            LiteralValue::Quantity { value, unit } => {
                match Decimal::from_str(value) {
                    Ok(d) => FhirPathValue::quantity(d, Some(unit.clone())),
                    Err(_) => return Err(EvaluationError::InvalidOperation {
                        message: format!("Invalid quantity value: {}", value),
                    }),
                }
            }
            LiteralValue::Null => return Ok(FhirPathValue::Empty),
        };
        
        // In FHIRPath, all values are conceptually collections
        Ok(FhirPathValue::collection(vec![value]))
    }

    /// Evaluate an identifier (property access)
    fn evaluate_identifier(&self, name: &str, context: &EvaluationContext) -> EvaluationResult<FhirPathValue> {
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
                    Some(value) => Ok(FhirPathValue::from(value.clone())),
                    None => Err(EvaluationError::PropertyNotFound {
                        property: name.to_string(),
                        resource_type: resource.resource_type().unwrap_or("Unknown").to_string(),
                    }),
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
            _ => Err(EvaluationError::PropertyNotFound {
                property: name.to_string(),
                resource_type: context.input.type_name().to_string(),
            }),
        }
    }

    /// Evaluate a variable reference
    fn evaluate_variable(&self, name: &str, context: &EvaluationContext) -> EvaluationResult<FhirPathValue> {
        match name {
            "$this" | "$" => Ok(context.input.clone()),
            "$$" => Ok(context.root.clone()),
            _ => {
                context.get_variable(name)
                    .cloned()
                    .ok_or_else(|| EvaluationError::VariableNotFound {
                        name: name.to_string(),
                    })
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
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            let value = self.evaluate_with_context(arg, context)?;
            arg_values.push(value);
        }

        // Get function from a registry
        let function = context.functions.get(name)
            .ok_or_else(|| EvaluationError::InvalidOperation {
                message: format!("Unknown function: {}", name),
            })?;

        // Create a compatible context for the function registry
        let registry_context = fhirpath_registry::function::EvaluationContext::new(context.input.clone());

        // Evaluate function
        let result = function.evaluate(&arg_values, &registry_context)?;
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
        
        // Create a new context with the base value as input
        let method_context = context.with_input(base_value);
        
        // Evaluate the method as a function call with the new context
        self.evaluate_function_call(method, args, &method_context)
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
        let operator = context.operators.get_binary(op_symbol)
            .ok_or_else(|| EvaluationError::Operator(format!("Unknown binary operator: {}", op_symbol)))?;

        // For binary operations, we need to unwrap single-element collections
        // according to FHIRPath semantics
        let left_operand = match &left_val {
            FhirPathValue::Collection(items) if items.len() == 1 => {
                items.get(0).unwrap().clone()
            }
            _ => left_val.clone(),
        };
        
        let right_operand = match &right_val {
            FhirPathValue::Collection(items) if items.len() == 1 => {
                items.get(0).unwrap().clone()
            }
            _ => right_val.clone(),
        };

        operator.evaluate_binary(&left_operand, &right_operand)
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
            UnaryOperator::Not => {
                match operand_val {
                    FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)])),
                    FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)])),
                    _ => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
                }
            }
            UnaryOperator::Minus => {
                match operand_val {
                    FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(-i)])),
                    FhirPathValue::Decimal(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(-d)])),
                    _ => Err(EvaluationError::TypeError {
                        expected: "Number".to_string(),
                        actual: operand_val.type_name().to_string(),
                    }),
                }
            }
            UnaryOperator::Plus => {
                match operand_val {
                    FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(FhirPathValue::collection(vec![operand_val])),
                    _ => Err(EvaluationError::TypeError {
                        expected: "Number".to_string(),
                        actual: operand_val.type_name().to_string(),
                    }),
                }
            }
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

        let index_num = match index_val {
            FhirPathValue::Integer(i) => i,
            _ => return Err(EvaluationError::TypeError {
                expected: "Integer".to_string(),
                actual: index_val.type_name().to_string(),
            }),
        };

        match base_val {
            FhirPathValue::Collection(items) => {
                if index_num < 0 || index_num as usize >= items.len() {
                    Err(EvaluationError::IndexOutOfBounds {
                        index: index_num,
                        size: items.len(),
                    })
                } else {
                    Ok(items.get(index_num as usize).unwrap().clone())
                }
            }
            _ => Err(EvaluationError::TypeError {
                expected: "Collection".to_string(),
                actual: base_val.type_name().to_string(),
            }),
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
        let left_val = self.evaluate_with_context(left, context)?;
        let right_val = self.evaluate_with_context(right, context)?;

        let mut items = Vec::new();

        // Add items from left
        match left_val {
            FhirPathValue::Collection(left_items) => items.extend(left_items),
            FhirPathValue::Empty => {}
            other => items.push(other),
        }

        // Add items from right
        match right_val {
            FhirPathValue::Collection(right_items) => items.extend(right_items),
            FhirPathValue::Empty => {}
            other => items.push(other),
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

        // Basic type checking - can be enhanced with FHIR Schema integration
        let matches = match (type_name, &value) {
            ("Boolean", FhirPathValue::Boolean(_)) => true,
            ("Integer", FhirPathValue::Integer(_)) => true,
            ("Decimal", FhirPathValue::Decimal(_)) => true,
            ("String", FhirPathValue::String(_)) => true,
            ("Quantity", FhirPathValue::Quantity(_)) => true,
            _ => false,
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(matches)]))
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
            FhirPathValue::Boolean(true) => {
                self.evaluate_with_context(then_expr, context)
            }
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
