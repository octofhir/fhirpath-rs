//! FHIRPath expression evaluator
//!
//! This module provides the main evaluation functionality for FHIRPath expressions.

use crate::ast::ExpressionNode;
use crate::error::{FhirPathError, Result};
use crate::model::{FhirPathValue, FhirResource};
use crate::parser::parse_expression;
use crate::types::FhirTypeRegistry;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde_json::Value;
use std::str::FromStr;

/// Evaluation context for FHIRPath expressions
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// The current input data being evaluated
    pub input: FhirPathValue,
    /// The root input data (for $this variable)
    pub root_input: FhirPathValue,
    /// Environment variables (for future use)
    pub variables: std::collections::HashMap<String, FhirPathValue>,
    /// FHIR type registry for enhanced type checking
    pub type_registry: FhirTypeRegistry,
}

impl EvaluationContext {
    /// Create a new evaluation context with the given input
    pub fn new(input: FhirPathValue) -> Self {
        Self {
            root_input: input.clone(),
            input,
            variables: std::collections::HashMap::new(),
            type_registry: FhirTypeRegistry::new(),
        }
    }

    /// Create a new evaluation context with a different input but preserving the root
    pub fn new_with_root(input: FhirPathValue, root_input: FhirPathValue) -> Self {
        Self {
            input,
            root_input,
            variables: std::collections::HashMap::new(),
            type_registry: FhirTypeRegistry::new(),
        }
    }

    /// Create a context from JSON input
    pub fn from_json(input: Value) -> Self {
        Self::new(FhirPathValue::from(input))
    }
}

/// Main entry point for evaluating FHIRPath expressions
///
/// This function parses the expression string and evaluates it against the input data.
/// It matches the signature expected by the official tests.
pub fn evaluate_expression(expression: &str, input_data: Value) -> Result<FhirPathValue> {
    // Parse the expression into an AST
    let ast = parse_expression(expression)?;

    // Create evaluation context
    let context = EvaluationContext::from_json(input_data);

    // Evaluate the AST
    evaluate_ast(&ast, &context)
}

/// Evaluate an AST node in the given context
pub fn evaluate_ast(node: &ExpressionNode, context: &EvaluationContext) -> Result<FhirPathValue> {
    match node {
        ExpressionNode::Literal(value) => Ok(value.clone()),

        ExpressionNode::Identifier(name) => {
            // For now, treat identifiers as property access on the current context
            evaluate_identifier(name, context)
        }

        ExpressionNode::FunctionCall { name, args } => {
            evaluate_function_call(name, args, context)
        }

        ExpressionNode::BinaryOp { op, left, right } => {
            let left_val = evaluate_ast(left, context)?;
            let right_val = evaluate_ast(right, context)?;
            evaluate_binary_operation(op, &left_val, &right_val)
        }

        ExpressionNode::UnaryOp { op, operand } => {
            let operand_val = evaluate_ast(operand, context)?;
            evaluate_unary_operation(op, &operand_val)
        }

        ExpressionNode::Path { base, path } => {
            let base_val = evaluate_ast(base, context)?;
            evaluate_path_navigation(&base_val, path)
        }

        ExpressionNode::Index { base, index } => {
            let base_val = evaluate_ast(base, context)?;
            let index_val = evaluate_ast(index, context)?;
            evaluate_index_access(&base_val, &index_val)
        }

        ExpressionNode::Filter { base, condition } => {
            let base_val = evaluate_ast(base, context)?;
            evaluate_filter(&base_val, condition, context)
        }

        ExpressionNode::Union { left, right } => {
            let left_val = evaluate_ast(left, context)?;
            let right_val = evaluate_ast(right, context)?;
            evaluate_union(&left_val, &right_val)
        }

        ExpressionNode::TypeCheck { expression, type_name } => {
            let expr_val = evaluate_ast(expression, context)?;
            evaluate_type_check(&expr_val, type_name, &context.type_registry)
        }

        ExpressionNode::TypeCast { expression, type_name } => {
            let expr_val = evaluate_ast(expression, context)?;
            evaluate_type_cast(&expr_val, type_name)
        }
    }
}

/// Evaluate an identifier (property access or variable)
fn evaluate_identifier(name: &str, context: &EvaluationContext) -> Result<FhirPathValue> {
    // Handle special $this variable
    if name == "$this" {
        return Ok(context.root_input.clone());
    }

    // Check if it's a variable first
    if let Some(value) = context.variables.get(name) {
        return Ok(value.clone());
    }

    // Check if the identifier matches the resource type of the current context
    if let FhirPathValue::Resource(resource) = &context.input {
        if let Some(resource_type) = resource.resource_type() {
            if name == resource_type {
                // Return the resource itself
                return Ok(context.input.clone());
            }
        }
    }

    // Otherwise, treat as property access on current input
    evaluate_path_navigation(&context.input, name)
}

/// Check if a function normally takes zero arguments
fn is_zero_arg_function(name: &str) -> bool {
    matches!(name,
        "empty" | "count" | "first" | "last" | "length" | "single" |
        "tail" | "toChars" | "sqrt" | "allTrue" | "today" | "not" |
        "convertsToString" | "convertsToInteger" | "convertsToDecimal" |
        "convertsToBoolean" | "convertsToDate" | "convertsToQuantity" |
        "convertsToDateTime" | "convertsToTime" | "toInteger" | "toDecimal" |
        "ceiling" | "floor" | "truncate" | "abs" | "exp" | "ln" | "toString" |
        "lower" | "upper" | "distinct"
    )
}

/// Check if a function normally takes one argument
fn is_one_arg_function(name: &str) -> bool {
    matches!(name,
        "is" | "as" | "where" | "select" | "skip" | "take" | "all" | "any" |
        "substring" | "contains" | "startsWith" | "endsWith" | "indexOf" |
        "subsetOf" | "supersetOf" | "conformsTo" | "power" | "log" | "extension" |
        "union" | "combine" | "intersect" | "exclude"
    )
}

/// Check if a function can take zero or one argument (optional argument)
fn is_optional_arg_function(name: &str) -> bool {
    matches!(name, "join" | "split")
}

/// Evaluate a function call
fn evaluate_function_call(
    name: &str,
    args: &[ExpressionNode],
    context: &EvaluationContext,
) -> Result<FhirPathValue> {
    // Handle method calls: if this is a function that normally takes 0 arguments
    // but we have 1 argument, treat the first argument as the new context
    if args.len() == 1 && is_zero_arg_function(name) {
        let new_input = evaluate_ast(&args[0], context)?;
        let new_context = EvaluationContext::new_with_root(new_input, context.root_input.clone());
        return evaluate_function_call(name, &[], &new_context);
    }

    // Handle method calls: if this is a function that normally takes 1 argument
    // but we have 2 arguments, treat the first argument as the new context
    if args.len() == 2 && is_one_arg_function(name) {
        let new_input = evaluate_ast(&args[0], context)?;
        let new_context = EvaluationContext::new_with_root(new_input, context.root_input.clone());
        return evaluate_function_call(name, &args[1..], &new_context);
    }

    // Handle method calls for optional argument functions: if this is a function that can take 0 or 1 arguments
    // but we have 1 or 2 arguments, treat the first argument as the new context
    if (args.len() == 1 || args.len() == 2) && is_optional_arg_function(name) {
        let new_input = evaluate_ast(&args[0], context)?;
        let new_context = EvaluationContext::new_with_root(new_input, context.root_input.clone());
        return evaluate_function_call(name, &args[1..], &new_context);
    }

    match name {
        "empty" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("empty", 0, args.len()));
            }
            // empty() returns true if the collection is empty
            Ok(FhirPathValue::Boolean(context.input.is_empty()))
        }

        "exists" => {
            if args.len() > 1 {
                return Err(FhirPathError::invalid_argument_count("exists", 1, args.len()));
            }

            if args.is_empty() {
                // exists() without arguments - check if collection is not empty
                Ok(FhirPathValue::Boolean(!context.input.is_empty()))
            } else {
                // exists(condition) - filter collection and check if any items match
                let filtered = evaluate_filter(&context.input, &args[0], context)?;
                Ok(FhirPathValue::Boolean(!filtered.is_empty()))
            }
        }

        "count" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("count", 0, args.len()));
            }
            Ok(FhirPathValue::Integer(context.input.len() as i64))
        }

        "first" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("first", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Collection(items) => {
                    if items.is_empty() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(items[0].clone())
                    }
                }
                FhirPathValue::Empty => Ok(FhirPathValue::empty()),
                single => Ok(single.clone()),
            }
        }

        "last" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("last", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Collection(items) => {
                    if items.is_empty() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(items[items.len() - 1].clone())
                    }
                }
                FhirPathValue::Empty => Ok(FhirPathValue::empty()),
                single => Ok(single.clone()),
            }
        }

        "length" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("length", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
                _ => Err(FhirPathError::type_error(format!(
                    "length() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "lower" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("lower", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_lowercase())),
                _ => Err(FhirPathError::type_error(format!(
                    "lower() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "upper" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("upper", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase())),
                _ => Err(FhirPathError::type_error(format!(
                    "upper() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "startsWith" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("startsWith", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let prefix_val = evaluate_ast(&args[0], context)?;
                    match prefix_val {
                        FhirPathValue::String(prefix) => Ok(FhirPathValue::Boolean(s.starts_with(&prefix))),
                        _ => Err(FhirPathError::type_error("startsWith() argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "startsWith() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "endsWith" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("endsWith", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let suffix_val = evaluate_ast(&args[0], context)?;
                    match suffix_val {
                        FhirPathValue::String(suffix) => Ok(FhirPathValue::Boolean(s.ends_with(&suffix))),
                        _ => Err(FhirPathError::type_error("endsWith() argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "endsWith() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "contains" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("contains", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let substring_val = evaluate_ast(&args[0], context)?;
                    match substring_val {
                        FhirPathValue::String(substring) => Ok(FhirPathValue::Boolean(s.contains(&substring))),
                        _ => Err(FhirPathError::type_error("contains() argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "contains() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "indexOf" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("indexOf", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let substring_val = evaluate_ast(&args[0], context)?;
                    match substring_val {
                        FhirPathValue::String(substring) => {
                            match s.find(&substring) {
                                Some(index) => Ok(FhirPathValue::Integer(index as i64)),
                                None => Ok(FhirPathValue::Integer(-1)),
                            }
                        }
                        _ => Err(FhirPathError::type_error("indexOf() argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "indexOf() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "substring" => {
            if args.len() < 1 || args.len() > 2 {
                return Err(FhirPathError::invalid_argument_count("substring", 1, args.len()));
            }

            match &context.input {
                FhirPathValue::String(s) => {
                    let start_val = evaluate_ast(&args[0], context)?;
                    let start = match start_val {
                        FhirPathValue::Integer(i) => i as usize,
                        _ => return Err(FhirPathError::type_error("substring start must be an integer".to_string())),
                    };

                    if args.len() == 2 {
                        let length_val = evaluate_ast(&args[1], context)?;
                        let length = match length_val {
                            FhirPathValue::Integer(i) => i as usize,
                            _ => return Err(FhirPathError::type_error("substring length must be an integer".to_string())),
                        };

                        if start >= s.len() {
                            Ok(FhirPathValue::String(String::new()))
                        } else {
                            let end = std::cmp::min(start + length, s.len());
                            Ok(FhirPathValue::String(s[start..end].to_string()))
                        }
                    } else {
                        // Only start index provided
                        if start >= s.len() {
                            Ok(FhirPathValue::String(String::new()))
                        } else {
                            Ok(FhirPathValue::String(s[start..].to_string()))
                        }
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "substring() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "single" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("single", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Collection(items) => {
                    if items.len() == 1 {
                        Ok(items[0].clone())
                    } else {
                        Err(FhirPathError::evaluation_error(format!(
                            "single() called on collection with {} items",
                            items.len()
                        )))
                    }
                }
                FhirPathValue::Empty => Err(FhirPathError::evaluation_error("single() called on empty collection".to_string())),
                single => Ok(single.clone()),
            }
        }

        "convertsToString" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToString", 0, args.len()));
            }
            // Check if the current input can be converted to string
            match &context.input {
                FhirPathValue::String(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Boolean(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Integer(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Decimal(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Date(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::DateTime(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Time(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Quantity { .. } => Ok(FhirPathValue::Boolean(true)),
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToInteger" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToInteger", 0, args.len()));
            }
            // Check if the current input can be converted to integer
            match &context.input {
                FhirPathValue::Integer(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    Ok(FhirPathValue::Boolean(s.parse::<i64>().is_ok()))
                }
                FhirPathValue::Boolean(_) => Ok(FhirPathValue::Boolean(true)),
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToDecimal" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToDecimal", 0, args.len()));
            }
            // Check if the current input can be converted to decimal
            match &context.input {
                FhirPathValue::Decimal(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Integer(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::Boolean(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    Ok(FhirPathValue::Boolean(rust_decimal::Decimal::from_str(s).is_ok()))
                }
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToBoolean" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToBoolean", 0, args.len()));
            }
            // Check if the current input can be converted to boolean
            match &context.input {
                FhirPathValue::Boolean(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    let lower = s.to_lowercase();
                    Ok(FhirPathValue::Boolean(lower == "true" || lower == "false"))
                }
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Boolean(*i == 0 || *i == 1)),
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToDate" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToDate", 0, args.len()));
            }
            // Check if the current input can be converted to date
            match &context.input {
                FhirPathValue::Date(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    // Handle partial dates: YYYY, YYYY-MM, YYYY-MM-DD
                    let can_convert = if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
                        // Year only (YYYY)
                        s.parse::<i32>().map(|year| year >= 1 && year <= 9999).unwrap_or(false)
                    } else if s.len() == 7 && s.chars().nth(4) == Some('-') {
                        // Year-Month (YYYY-MM)
                        let parts: Vec<&str> = s.split('-').collect();
                        if parts.len() == 2 {
                            if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                                year >= 1 && year <= 9999 && month >= 1 && month <= 12
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        // Full date (YYYY-MM-DD) - use chrono parsing
                        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
                    };
                    Ok(FhirPathValue::Boolean(can_convert))
                }
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToQuantity" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToQuantity", 0, args.len()));
            }
            // Check if the current input can be converted to quantity
            match &context.input {
                FhirPathValue::Quantity { .. } => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    // Simple check for quantity format (number + unit)
                    let parts: Vec<&str> = s.trim().split_whitespace().collect();
                    if parts.len() >= 1 {
                        // Check if first part is a number
                        if let Ok(_) = parts[0].parse::<f64>() {
                            Ok(FhirPathValue::Boolean(true))
                        } else {
                            Ok(FhirPathValue::Boolean(false))
                        }
                    } else {
                        Ok(FhirPathValue::Boolean(false))
                    }
                }
                FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(FhirPathValue::Boolean(true)),
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToDateTime" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToDateTime", 0, args.len()));
            }
            // Check if the current input can be converted to datetime
            match &context.input {
                FhirPathValue::DateTime(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    // Handle partial dates and datetimes
                    let can_convert = if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
                        // Year only (YYYY)
                        s.parse::<i32>().map(|year| year >= 1 && year <= 9999).unwrap_or(false)
                    } else if s.len() == 7 && s.chars().nth(4) == Some('-') {
                        // Year-Month (YYYY-MM)
                        let parts: Vec<&str> = s.split('-').collect();
                        if parts.len() == 2 {
                            if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                                year >= 1 && year <= 9999 && month >= 1 && month <= 12
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
                        // Full date (YYYY-MM-DD)
                        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
                    } else if s.contains('T') {
                        // DateTime formats - try parsing with various formats
                        let datetime_formats = [
                            "%Y-%m-%dT%H", "%Y-%m-%dT%H:%M", "%Y-%m-%dT%H:%M:%S",
                            "%Y-%m-%dT%H:%M:%S%.3f", "%Y-%m-%dT%H:%M:%SZ",
                            "%Y-%m-%dT%H:%M:%S%.3fZ", "%Y-%m-%dT%H:%M:%S%z",
                            "%Y-%m-%dT%H:%M:%S%.3f%z"
                        ];
                        datetime_formats.iter().any(|fmt| {
                            chrono::DateTime::parse_from_str(s, fmt).is_ok() ||
                            chrono::NaiveDateTime::parse_from_str(s, fmt).is_ok()
                        })
                    } else {
                        false
                    };
                    Ok(FhirPathValue::Boolean(can_convert))
                }
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "convertsToTime" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("convertsToTime", 0, args.len()));
            }
            // Check if the current input can be converted to time
            match &context.input {
                FhirPathValue::Time(_) => Ok(FhirPathValue::Boolean(true)),
                FhirPathValue::String(s) => {
                    // Try to parse as time with various formats
                    let time_formats = ["%H", "%H:%M", "%H:%M:%S", "%H:%M:%S%.3f"];
                    let can_convert = time_formats.iter().any(|fmt| {
                        chrono::NaiveTime::parse_from_str(s, fmt).is_ok()
                    });
                    Ok(FhirPathValue::Boolean(can_convert))
                }
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        }

        "toInteger" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("toInteger", 0, args.len()));
            }
            // Convert the current input to integer
            match &context.input {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
                FhirPathValue::String(s) => {
                    match s.parse::<i64>() {
                        Ok(i) => Ok(FhirPathValue::Integer(i)),
                        Err(_) => Ok(FhirPathValue::empty()),
                    }
                }
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Integer(if *b { 1 } else { 0 })),
                _ => Ok(FhirPathValue::empty()),
            }
        }

        "toDecimal" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("toDecimal", 0, args.len()));
            }
            // Convert the current input to decimal
            match &context.input {
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i))),
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(if *b { 1 } else { 0 }))),
                FhirPathValue::String(s) => {
                    match rust_decimal::Decimal::from_str(s) {
                        Ok(d) => Ok(FhirPathValue::Decimal(d)),
                        Err(_) => Ok(FhirPathValue::empty()),
                    }
                }
                _ => Ok(FhirPathValue::empty()),
            }
        }

        "where" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("where", 1, args.len()));
            }
            // where() filters a collection based on a condition
            evaluate_filter(&context.input, &args[0], context)
        }

        "select" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("select", 1, args.len()));
            }
            // select() maps each item in a collection using an expression and flattens results
            match &context.input {
                FhirPathValue::Collection(items) => {
                    let mut results = Vec::new();
                    for item in items {
                        // Create new context with current item
                        let item_context = EvaluationContext::new(item.clone());
                        let result = evaluate_ast(&args[0], &item_context)?;

                        // Flatten the result - if it's a collection, add all items
                        match result {
                            FhirPathValue::Collection(sub_items) => {
                                results.extend(sub_items);
                            }
                            FhirPathValue::Empty => {
                                // Empty results are not added (filtered out)
                            }
                            single_result => {
                                results.push(single_result);
                            }
                        }
                    }
                    Ok(FhirPathValue::Collection(results))
                }
                FhirPathValue::Empty => Ok(FhirPathValue::empty()),
                single => {
                    // For single items, create context and evaluate expression
                    let item_context = EvaluationContext::new(single.clone());
                    let result = evaluate_ast(&args[0], &item_context)?;
                    Ok(result)
                }
            }
        }

        "tail" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("tail", 0, args.len()));
            }
            // tail() returns all but the first item in a collection
            match &context.input {
                FhirPathValue::Collection(items) => {
                    if items.len() <= 1 {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(FhirPathValue::Collection(items[1..].to_vec()))
                    }
                }
                FhirPathValue::Empty => Ok(FhirPathValue::empty()),
                _ => Ok(FhirPathValue::empty()), // Single item has no tail
            }
        }

        "skip" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("skip", 1, args.len()));
            }
            // skip() returns all but the first N items in a collection
            let num_val = evaluate_ast(&args[0], context)?;
            let num = match num_val {
                FhirPathValue::Integer(i) => {
                    if i < 0 {
                        return Err(FhirPathError::evaluation_error("skip() requires a non-negative integer".to_string()));
                    }
                    i as usize
                }
                _ => return Err(FhirPathError::type_error("skip() requires an integer argument".to_string())),
            };

            match &context.input {
                FhirPathValue::Collection(items) => {
                    if num >= items.len() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(FhirPathValue::Collection(items[num..].to_vec()))
                    }
                }
                FhirPathValue::Empty => Ok(FhirPathValue::empty()),
                single => {
                    if num == 0 {
                        Ok(single.clone())
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
            }
        }

        "take" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("take", 1, args.len()));
            }
            // take() returns the first N items in a collection
            let num_val = evaluate_ast(&args[0], context)?;
            let num = match num_val {
                FhirPathValue::Integer(i) => {
                    if i < 0 {
                        return Err(FhirPathError::evaluation_error("take() requires a non-negative integer".to_string()));
                    }
                    i as usize
                }
                _ => return Err(FhirPathError::type_error("take() requires an integer argument".to_string())),
            };

            match &context.input {
                FhirPathValue::Collection(items) => {
                    let end = std::cmp::min(num, items.len());
                    Ok(FhirPathValue::Collection(items[0..end].to_vec()))
                }
                FhirPathValue::Empty => Ok(FhirPathValue::empty()),
                single => {
                    if num > 0 {
                        Ok(single.clone())
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
            }
        }

        "all" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("all", 1, args.len()));
            }
            // all() returns true if all items in the collection satisfy the condition
            match &context.input {
                FhirPathValue::Collection(items) => {
                    if items.is_empty() {
                        return Ok(FhirPathValue::Boolean(true)); // Empty collection returns true
                    }

                    for item in items {
                        // Create new context with current item
                        let item_context = EvaluationContext::new(item.clone());
                        let condition_result = evaluate_ast(&args[0], &item_context)?;

                        if let Some(false) = condition_result.to_boolean() {
                            return Ok(FhirPathValue::Boolean(false));
                        }
                    }
                    Ok(FhirPathValue::Boolean(true))
                }
                FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)), // Empty collection returns true
                single => {
                    // For single items, create context and evaluate condition
                    let item_context = EvaluationContext::new(single.clone());
                    let condition_result = evaluate_ast(&args[0], &item_context)?;

                    match condition_result.to_boolean() {
                        Some(b) => Ok(FhirPathValue::Boolean(b)),
                        None => Ok(FhirPathValue::Boolean(false)),
                    }
                }
            }
        }


        "join" => {
            if args.len() > 1 {
                return Err(FhirPathError::evaluation_error(format!(
                    "join() takes 0 or 1 arguments, got {}",
                    args.len()
                )));
            }

            let separator = if args.is_empty() {
                String::new() // Default to empty string when no separator provided
            } else {
                let separator_val = evaluate_ast(&args[0], context)?;
                match separator_val {
                    FhirPathValue::String(s) => s,
                    _ => return Err(FhirPathError::type_error("join() separator must be a string".to_string())),
                }
            };

            match &context.input {
                FhirPathValue::Collection(items) => {
                    let strings: Result<Vec<String>> = items.iter().map(|item| {
                        match item {
                            FhirPathValue::String(s) => Ok(s.clone()),
                            _ => Ok(item.to_string()),
                        }
                    }).collect();

                    match strings {
                        Ok(str_vec) => Ok(FhirPathValue::String(str_vec.join(&separator))),
                        Err(e) => Err(e),
                    }
                }
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),
                _ => Ok(FhirPathValue::String(context.input.to_string())),
            }
        }

        "split" => {
            if args.len() > 1 {
                return Err(FhirPathError::evaluation_error(format!(
                    "split() takes 0 or 1 arguments, got {}",
                    args.len()
                )));
            }

            let separator = if args.is_empty() {
                " ".to_string() // Default to space when no separator provided
            } else {
                let separator_val = evaluate_ast(&args[0], context)?;
                match separator_val {
                    FhirPathValue::String(s) => s,
                    _ => return Err(FhirPathError::type_error("split() separator must be a string".to_string())),
                }
            };

            match &context.input {
                FhirPathValue::String(s) => {
                    let parts: Vec<FhirPathValue> = s.split(&separator)
                        .map(|part| FhirPathValue::String(part.to_string()))
                        .collect();
                    Ok(FhirPathValue::Collection(parts))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "split() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "toChars" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("toChars", 0, args.len()));
            }

            match &context.input {
                FhirPathValue::String(s) => {
                    let chars: Vec<FhirPathValue> = s.chars()
                        .map(|c| FhirPathValue::String(c.to_string()))
                        .collect();
                    Ok(FhirPathValue::Collection(chars))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "toChars() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "sqrt" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("sqrt", 0, args.len()));
            }

            match &context.input {
                FhirPathValue::Integer(i) => {
                    if *i < 0 {
                        return Ok(FhirPathValue::empty());
                    }
                    let result = (*i as f64).sqrt();
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                FhirPathValue::Decimal(d) => {
                    let val = d.to_f64().unwrap_or(0.0);
                    if val < 0.0 {
                        return Ok(FhirPathValue::empty());
                    }
                    let result = val.sqrt();
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "sqrt() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "round" => {
            let precision = if args.is_empty() {
                0
            } else if args.len() == 1 {
                let precision_val = evaluate_ast(&args[0], context)?;
                match precision_val {
                    FhirPathValue::Integer(i) => i as u32,
                    _ => return Err(FhirPathError::type_error("round() precision must be an integer".to_string())),
                }
            } else {
                return Err(FhirPathError::invalid_argument_count("round", 1, args.len()));
            };

            match &context.input {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
                FhirPathValue::Decimal(d) => {
                    Ok(FhirPathValue::Decimal(d.round_dp(precision)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "round() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "subsetOf" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("subsetOf", 1, args.len()));
            }

            let other_val = evaluate_ast(&args[0], context)?;

            // For now, implement basic subset checking
            // This is a simplified implementation
            match (&context.input, &other_val) {
                (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2)) => {
                    let is_subset = items1.iter().all(|item1| {
                        items2.iter().any(|item2| item1 == item2)
                    });
                    Ok(FhirPathValue::Boolean(is_subset))
                }
                (single, FhirPathValue::Collection(items)) => {
                    let is_subset = items.iter().any(|item| single == item);
                    Ok(FhirPathValue::Boolean(is_subset))
                }
                (a, b) => Ok(FhirPathValue::Boolean(a == b)),
            }
        }

        "conformsTo" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("conformsTo", 1, args.len()));
            }

            // For now, always return false as we don't have profile validation implemented
            Ok(FhirPathValue::Boolean(false))
        }

        "today" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("today", 0, args.len()));
            }

            // Return today's date as a proper Date value
            let today = chrono::Utc::now().date_naive();
            Ok(FhirPathValue::Date(today))
        }

        "allTrue" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("allTrue", 0, args.len()));
            }

            match &context.input {
                FhirPathValue::Collection(items) => {
                    // Return true if all items are true, false if any item is false
                    // Empty collection returns true (vacuous truth)
                    for item in items {
                        match item.to_boolean() {
                            Some(false) => return Ok(FhirPathValue::Boolean(false)),
                            Some(true) => continue,
                            None => return Ok(FhirPathValue::Boolean(false)), // Non-boolean values are considered false
                        }
                    }
                    Ok(FhirPathValue::Boolean(true))
                }
                single => {
                    // For single items, return their boolean value
                    match single.to_boolean() {
                        Some(b) => Ok(FhirPathValue::Boolean(b)),
                        None => Ok(FhirPathValue::Boolean(false)),
                    }
                }
            }
        }

        "is" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("is", 1, args.len()));
            }

            // Handle type names specially - if the argument is an identifier that represents
            // a known type name, treat it as a string literal instead of evaluating it
            let type_name = match &args[0] {
                ExpressionNode::Identifier(name) => {
                    // For identifiers, always treat as type name (this matches Java behavior)
                    name.clone()
                }
                ExpressionNode::Path { base, path } => {
                    // Handle dotted type names like System.Integer, System.String, etc.
                    if let ExpressionNode::Identifier(base_name) = base.as_ref() {
                        format!("{}.{}", base_name, path)
                    } else {
                        // For complex path expressions, evaluate normally
                        let type_arg = evaluate_ast(&args[0], context)?;
                        match type_arg {
                            FhirPathValue::String(s) => s,
                            FhirPathValue::Collection(ref items) => {
                                // If it's a collection with a single string, use that
                                if items.len() == 1 {
                                    if let FhirPathValue::String(s) = &items[0] {
                                        s.clone()
                                    } else {
                                        // For non-string items in collection, try to get their type name
                                        items[0].type_name().to_string()
                                    }
                                } else if items.is_empty() {
                                    // Empty collection means no type to check - return false
                                    return Ok(FhirPathValue::Boolean(false));
                                } else {
                                    return Err(FhirPathError::type_error(format!(
                                        "Type checking 'is' requires a single type name, got collection with {} items",
                                        items.len()
                                    )));
                                }
                            }
                            _ => type_arg.type_name().to_string(),
                        }
                    }
                }
                _ => {
                    // For other expressions, evaluate and extract string
                    let type_arg = evaluate_ast(&args[0], context)?;
                    match type_arg {
                        FhirPathValue::String(s) => s,
                        FhirPathValue::Collection(ref items) => {
                            // If it's a collection with a single string, use that
                            if items.len() == 1 {
                                if let FhirPathValue::String(s) = &items[0] {
                                    s.clone()
                                } else {
                                    // For non-string items in collection, try to get their type name
                                    items[0].type_name().to_string()
                                }
                            } else if items.is_empty() {
                                // Empty collection means no type to check - return false
                                return Ok(FhirPathValue::Boolean(false));
                            } else {
                                return Err(FhirPathError::type_error(format!(
                                    "Type checking 'is' requires a single type name, got collection with {} items",
                                    items.len()
                                )));
                            }
                        }
                        _ => type_arg.type_name().to_string(),
                    }
                }
            };

            evaluate_type_check(&context.input, &type_name, &context.type_registry)
        }

        "not" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("not", 0, args.len()));
            }

            match context.input.to_boolean() {
                Some(b) => Ok(FhirPathValue::Boolean(!b)),
                None => Ok(FhirPathValue::Boolean(true)), // Empty/null is considered false, so not() returns true
            }
        }

        "as" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("as", 1, args.len()));
            }

            // Handle type names specially - if the argument is an identifier that represents
            // a known type name, treat it as a string literal instead of evaluating it
            let type_name = match &args[0] {
                ExpressionNode::Identifier(name) => {
                    // Check if this is a known type name
                    let lower_name = name.to_lowercase();
                    if matches!(lower_name.as_str(),
                        "boolean" | "integer" | "decimal" | "string" |
                        "date" | "datetime" | "time" | "quantity" |
                        "codeableconcept" | "coding" | "identifier" | "humanname" |
                        "address" | "contactpoint" | "period" | "range" | "ratio" |
                        "sampleddata" | "attachment" | "annotation" | "signature" |
                        "patient" | "observation" | "practitioner" | "organization" |
                        "encounter" | "condition" | "procedure" | "medication" |
                        "medicationrequest" | "diagnosticreport" | "specimen" |
                        "immunization" | "allergyintolerance" | "careplan" |
                        "goal" | "servicerequest" | "device" | "location" |
                        "healthcareservice" | "endpoint" | "schedule" |
                        "slot" | "appointment" | "appointmentresponse" |
                        "account" | "invoice" | "paymentnotice" | "paymentreconciliation" |
                        "coverage" | "coverageeligibilityrequest" | "coverageeligibilityresponse" |
                        "enrollmentrequest" | "enrollmentresponse" | "claim" | "claimresponse" |
                        "explanationofbenefit" | "contract" | "person" | "relatedperson" |
                        "group" | "bodystructure" | "substance" | "biologicallyderivedproduct" |
                        "nutritionorder" | "visionprescription" | "riskassessment" |
                        "requestgroup" | "communicationrequest" | "devicerequest" |
                        "deviceusestatement" | "guidanceresponse" | "supplyrequest" |
                        "supplydelivery" | "inventoryreport" | "task" | "provenance" |
                        "auditevent" | "consent" | "composition" | "documentmanifest" |
                        "documentreference" | "catalogentry" | "basic" | "binary" |
                        "bundle" | "linkage" | "messagedefinition" | "messageheader" |
                        "operationoutcome" | "parameters" | "subscription" |
                        "subscriptionstatus" | "subscriptiontopic") {
                        name.clone()
                    } else {
                        // Not a known type name, evaluate as normal expression
                        let type_arg = evaluate_ast(&args[0], context)?;
                        match type_arg {
                            FhirPathValue::String(s) => s,
                            _ => return Err(FhirPathError::type_error(format!(
                                "Type casting 'as' requires a type name as string, got {}",
                                type_arg.type_name()
                            ))),
                        }
                    }
                }
                ExpressionNode::Path { base, path } => {
                    // Handle dotted type names like System.Integer, System.String, etc.
                    if let ExpressionNode::Identifier(base_name) = base.as_ref() {
                        format!("{}.{}", base_name, path)
                    } else {
                        // For complex path expressions, evaluate normally
                        let type_arg = evaluate_ast(&args[0], context)?;
                        match type_arg {
                            FhirPathValue::String(s) => s,
                            _ => return Err(FhirPathError::type_error(format!(
                                "Type casting 'as' requires a type name as string, got {}",
                                type_arg.type_name()
                            ))),
                        }
                    }
                }
                _ => {
                    // Evaluate the expression normally
                    let type_arg = evaluate_ast(&args[0], context)?;
                    match type_arg {
                        FhirPathValue::String(s) => s,
                        _ => return Err(FhirPathError::type_error(format!(
                            "Type casting 'as' requires a type name as string, got {}",
                            type_arg.type_name()
                        ))),
                    }
                }
            };

            evaluate_type_cast(&context.input, &type_name)
        }

        "distinct" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("distinct", 0, args.len()));
            }

            match &context.input {
                FhirPathValue::Collection(items) => {
                    let mut unique_items = Vec::new();
                    for item in items {
                        if !unique_items.contains(item) {
                            unique_items.push(item.clone());
                        }
                    }
                    Ok(FhirPathValue::Collection(unique_items))
                }
                single => Ok(single.clone()),
            }
        }

        "now" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("now", 0, args.len()));
            }

            // Return current date and time as a proper DateTime value
            let now = chrono::Utc::now();
            Ok(FhirPathValue::DateTime(now))
        }



        // Mathematical functions
        "ceiling" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("ceiling", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
                FhirPathValue::Decimal(d) => {
                    let ceiling_val = d.ceil();
                    Ok(FhirPathValue::Integer(ceiling_val.to_i64().unwrap_or(0)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "ceiling() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "floor" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("floor", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
                FhirPathValue::Decimal(d) => {
                    let floor_val = d.floor();
                    Ok(FhirPathValue::Integer(floor_val.to_i64().unwrap_or(0)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "floor() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "truncate" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("truncate", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
                FhirPathValue::Decimal(d) => {
                    let truncate_val = d.trunc();
                    Ok(FhirPathValue::Integer(truncate_val.to_i64().unwrap_or(0)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "truncate() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "abs" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("abs", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(i.abs())),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(d.abs())),
                FhirPathValue::Quantity { value, unit, ucum_expr } => {
                    Ok(FhirPathValue::Quantity {
                        value: value.abs(),
                        unit: unit.clone(),
                        ucum_expr: ucum_expr.clone(),
                    })
                }
                _ => Err(FhirPathError::type_error(format!(
                    "abs() can only be called on numbers or quantities, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "power" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("power", 1, args.len()));
            }
            let exponent_val = evaluate_ast(&args[0], context)?;
            match (&context.input, &exponent_val) {
                (FhirPathValue::Integer(base), FhirPathValue::Integer(exp)) => {
                    // Check for negative base with fractional exponent (should return empty)
                    if *base < 0 && *exp != 0 {
                        // For negative base, only integer exponents are valid
                        let result = (*base as f64).powf(*exp as f64);
                        if result.is_finite() && result.fract() == 0.0 {
                            Ok(FhirPathValue::Integer(result as i64))
                        } else {
                            Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                        }
                    } else {
                        let result = (*base as f64).powf(*exp as f64);
                        if result.fract() == 0.0 && result.is_finite() {
                            Ok(FhirPathValue::Integer(result as i64))
                        } else {
                            Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                        }
                    }
                }
                (FhirPathValue::Decimal(base), FhirPathValue::Integer(exp)) => {
                    let base_f64 = base.to_f64().unwrap_or(0.0);
                    let result = base_f64.powf(*exp as f64);
                    if result.is_nan() || result.is_infinite() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                    }
                }
                (FhirPathValue::Integer(base), FhirPathValue::Decimal(exp)) => {
                    let base_f64 = *base as f64;
                    let exp_f64 = exp.to_f64().unwrap_or(0.0);

                    // Check for negative base with fractional exponent
                    if base_f64 < 0.0 && exp_f64.fract() != 0.0 {
                        return Ok(FhirPathValue::empty());
                    }

                    let result = base_f64.powf(exp_f64);
                    if result.is_nan() || result.is_infinite() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                    }
                }
                (FhirPathValue::Decimal(base), FhirPathValue::Decimal(exp)) => {
                    let base_f64 = base.to_f64().unwrap_or(0.0);
                    let exp_f64 = exp.to_f64().unwrap_or(0.0);

                    // Check for negative base with fractional exponent
                    if base_f64 < 0.0 && exp_f64.fract() != 0.0 {
                        return Ok(FhirPathValue::empty());
                    }

                    let result = base_f64.powf(exp_f64);
                    if result.is_nan() || result.is_infinite() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "power() requires numeric arguments, got {} and {}",
                    context.input.type_name(),
                    exponent_val.type_name()
                ))),
            }
        }

        "exp" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("exp", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => {
                    let result = (*i as f64).exp();
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                FhirPathValue::Decimal(d) => {
                    let result = d.to_f64().unwrap_or(0.0).exp();
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "exp() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "ln" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("ln", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => {
                    if *i <= 0 {
                        return Ok(FhirPathValue::empty());
                    }
                    let result = (*i as f64).ln();
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                FhirPathValue::Decimal(d) => {
                    let val = d.to_f64().unwrap_or(0.0);
                    if val <= 0.0 {
                        return Ok(FhirPathValue::empty());
                    }
                    let result = val.ln();
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "ln() can only be called on numbers, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "log" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("log", 1, args.len()));
            }
            let base_val = evaluate_ast(&args[0], context)?;
            match (&context.input, &base_val) {
                (FhirPathValue::Integer(val), FhirPathValue::Integer(base)) => {
                    if *val <= 0 || *base <= 0 || *base == 1 {
                        return Ok(FhirPathValue::empty());
                    }
                    let result = (*val as f64).log(*base as f64);
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                (FhirPathValue::Decimal(val), FhirPathValue::Decimal(base)) => {
                    let val_f64 = val.to_f64().unwrap_or(0.0);
                    let base_f64 = base.to_f64().unwrap_or(0.0);
                    if val_f64 <= 0.0 || base_f64 <= 0.0 || base_f64 == 1.0 {
                        return Ok(FhirPathValue::empty());
                    }
                    let result = val_f64.log(base_f64);
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default()))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "log() requires numeric arguments, got {} and {}",
                    context.input.type_name(),
                    base_val.type_name()
                ))),
            }
        }


        // Utility functions
        "toString" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("toString", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string())),
                FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string())),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::String(d.to_string())),
                FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.to_string())),
                FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.to_string())),
                FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.to_string())),
                _ => Ok(FhirPathValue::empty()),
            }
        }

        "trace" => {
            // trace() is a debugging function that returns its input unchanged
            // It can take 0, 1, or 2 arguments (name and optional selector)
            match &context.input {
                FhirPathValue::Collection(items) => {
                    Ok(FhirPathValue::Collection(items.clone()))
                }
                single => Ok(single.clone()),
            }
        }

        "extension" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("extension", 1, args.len()));
            }

            let url_val = evaluate_ast(&args[0], context)?;
            let url = match url_val {
                FhirPathValue::String(s) => s,
                _ => return Err(FhirPathError::type_error("extension() requires a string URL argument".to_string())),
            };

            // Extract extensions from the current context
            match &context.input {
                FhirPathValue::Resource(resource) => {
                    // Look for extension array in the resource
                    if let Some(extensions_value) = resource.get_property("extension") {
                        if let Value::Array(extensions) = extensions_value {
                            let mut results = Vec::new();
                            for ext in extensions {
                                if let Value::Object(ext_obj) = ext {
                                    if let Some(Value::String(ext_url)) = ext_obj.get("url") {
                                        if ext_url == &url {
                                            // Found matching extension, return it as a resource
                                            results.push(FhirPathValue::Resource(FhirResource::new(ext.clone())));
                                        }
                                    }
                                }
                            }
                            if results.is_empty() {
                                Ok(FhirPathValue::empty())
                            } else if results.len() == 1 {
                                Ok(results.into_iter().next().unwrap())
                            } else {
                                Ok(FhirPathValue::Collection(results))
                            }
                        } else {
                            Ok(FhirPathValue::empty())
                        }
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
                FhirPathValue::Collection(items) => {
                    // Apply extension search to each item in the collection
                    let mut results = Vec::new();
                    for item in items {
                        let item_context = EvaluationContext::new(item.clone());
                        let item_result = evaluate_function_call("extension", &[args[0].clone()], &item_context)?;
                        match item_result {
                            FhirPathValue::Collection(sub_items) => {
                                results.extend(sub_items);
                            }
                            FhirPathValue::Empty => {
                                // Skip empty results
                            }
                            single => {
                                results.push(single);
                            }
                        }
                    }
                    if results.is_empty() {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(FhirPathValue::Collection(results))
                    }
                }
                _ => Ok(FhirPathValue::empty()),
            }
        }

        "supersetOf" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("supersetOf", 1, args.len()));
            }
            let other_val = evaluate_ast(&args[0], context)?;

            // A superset contains all elements of the other collection
            match (&context.input, &other_val) {
                (FhirPathValue::Collection(left_items), FhirPathValue::Collection(right_items)) => {
                    let is_superset = right_items.iter().all(|item2| {
                        left_items.iter().any(|item1| item1 == item2)
                    });
                    Ok(FhirPathValue::Boolean(is_superset))
                }
                (FhirPathValue::Collection(items), single) => {
                    let is_superset = items.iter().any(|item| item == single);
                    Ok(FhirPathValue::Boolean(is_superset))
                }
                (single, FhirPathValue::Collection(items)) => {
                    // A single item can only be a superset of an empty collection or itself
                    let is_superset = items.is_empty() || (items.len() == 1 && items[0] == *single);
                    Ok(FhirPathValue::Boolean(is_superset))
                }
                (a, b) => Ok(FhirPathValue::Boolean(a == b)),
            }
        }

        "any" => {
            if args.len() > 1 {
                return Err(FhirPathError::invalid_argument_count("any", 1, args.len()));
            }

            if args.is_empty() {
                // any() without arguments returns true if collection is not empty
                Ok(FhirPathValue::Boolean(!context.input.is_empty()))
            } else {
                // any(condition) returns true if any item satisfies the condition
                match &context.input {
                    FhirPathValue::Collection(items) => {
                        if items.is_empty() {
                            return Ok(FhirPathValue::Boolean(false)); // Empty collection returns false
                        }

                        for item in items {
                            // Create new context with current item
                            let item_context = EvaluationContext::new(item.clone());
                            let condition_result = evaluate_ast(&args[0], &item_context)?;

                            if let Some(true) = condition_result.to_boolean() {
                                return Ok(FhirPathValue::Boolean(true));
                            }
                        }
                        Ok(FhirPathValue::Boolean(false))
                    }
                    FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)), // Empty collection returns false
                    single => {
                        // For single items, create context and evaluate condition
                        let item_context = EvaluationContext::new(single.clone());
                        let condition_result = evaluate_ast(&args[0], &item_context)?;

                        match condition_result.to_boolean() {
                            Some(b) => Ok(FhirPathValue::Boolean(b)),
                            None => Ok(FhirPathValue::Boolean(false)),
                        }
                    }
                }
            }
        }


        "union" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("union", 1, args.len()));
            }
            let other_val = evaluate_ast(&args[0], context)?;
            evaluate_union(&context.input, &other_val)
        }

        "combine" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("combine", 1, args.len()));
            }
            let other_val = evaluate_ast(&args[0], context)?;
            // combine() is like union but doesn't remove duplicates
            match (&context.input, &other_val) {
                (FhirPathValue::Collection(left_items), FhirPathValue::Collection(right_items)) => {
                    let mut result = left_items.clone();
                    result.extend(right_items.clone());
                    Ok(FhirPathValue::Collection(result))
                }
                (FhirPathValue::Collection(items), single) | (single, FhirPathValue::Collection(items)) => {
                    let mut result = items.clone();
                    if !matches!(single, FhirPathValue::Empty) {
                        result.push(single.clone());
                    }
                    Ok(FhirPathValue::Collection(result))
                }
                (FhirPathValue::Empty, other) | (other, FhirPathValue::Empty) => Ok(other.clone()),
                (left, right) => {
                    Ok(FhirPathValue::Collection(vec![left.clone(), right.clone()]))
                }
            }
        }

        "intersect" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("intersect", 1, args.len()));
            }
            let other_val = evaluate_ast(&args[0], context)?;
            match (&context.input, &other_val) {
                (FhirPathValue::Collection(left_items), FhirPathValue::Collection(right_items)) => {
                    let mut result = Vec::new();
                    for item in left_items {
                        if right_items.contains(item) && !result.contains(item) {
                            result.push(item.clone());
                        }
                    }
                    Ok(FhirPathValue::Collection(result))
                }
                (FhirPathValue::Collection(items), single) => {
                    if items.contains(single) {
                        Ok(single.clone())
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
                (single, FhirPathValue::Collection(items)) => {
                    if items.contains(single) {
                        Ok(single.clone())
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
                (left, right) => {
                    if left == right {
                        Ok(left.clone())
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
            }
        }

        "exclude" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("exclude", 1, args.len()));
            }
            let other_val = evaluate_ast(&args[0], context)?;
            match (&context.input, &other_val) {
                (FhirPathValue::Collection(left_items), FhirPathValue::Collection(right_items)) => {
                    let mut result = Vec::new();
                    for item in left_items {
                        if !right_items.contains(item) {
                            result.push(item.clone());
                        }
                    }
                    Ok(FhirPathValue::Collection(result))
                }
                (FhirPathValue::Collection(items), single) => {
                    let mut result = Vec::new();
                    for item in items {
                        if item != single {
                            result.push(item.clone());
                        }
                    }
                    Ok(FhirPathValue::Collection(result))
                }
                (single, FhirPathValue::Collection(items)) => {
                    if items.contains(single) {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(single.clone())
                    }
                }
                (left, right) => {
                    if left == right {
                        Ok(FhirPathValue::empty())
                    } else {
                        Ok(left.clone())
                    }
                }
            }
        }

        "toQuantity" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("toQuantity", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Integer(i) => {
                    Ok(FhirPathValue::Quantity {
                        value: rust_decimal::Decimal::from(*i),
                        unit: None,
                        ucum_expr: None,
                    })
                }
                FhirPathValue::Decimal(d) => {
                    Ok(FhirPathValue::Quantity {
                        value: *d,
                        unit: None,
                        ucum_expr: None,
                    })
                }
                FhirPathValue::String(s) => {
                    // Try to parse as a quantity string like "5 mg" or just "5"
                    let s = s.trim();
                    if let Some(space_pos) = s.find(' ') {
                        let (value_str, unit_str) = s.split_at(space_pos);
                        let unit_str = unit_str.trim();
                        if let Ok(value) = rust_decimal::Decimal::from_str(value_str.trim()) {
                            Ok(FhirPathValue::Quantity {
                                value,
                                unit: Some(unit_str.to_string()),
                                ucum_expr: FhirPathValue::parse_ucum_unit(unit_str),
                            })
                        } else {
                            Ok(FhirPathValue::empty())
                        }
                    } else if let Ok(value) = rust_decimal::Decimal::from_str(s) {
                        Ok(FhirPathValue::Quantity {
                            value,
                            unit: None,
                            ucum_expr: None,
                        })
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                }
                FhirPathValue::Boolean(b) => {
                    let value = if *b { rust_decimal::Decimal::ONE } else { rust_decimal::Decimal::ZERO };
                    Ok(FhirPathValue::Quantity {
                        value,
                        unit: None,
                        ucum_expr: None,
                    })
                }
                _ => Ok(FhirPathValue::empty()),
            }
        }

        "toBoolean" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("toBoolean", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
                FhirPathValue::Integer(i) => {
                    match *i {
                        0 => Ok(FhirPathValue::Boolean(false)),
                        1 => Ok(FhirPathValue::Boolean(true)),
                        _ => Ok(FhirPathValue::empty()),
                    }
                }
                FhirPathValue::String(s) => {
                    match s.to_lowercase().as_str() {
                        "true" => Ok(FhirPathValue::Boolean(true)),
                        "false" => Ok(FhirPathValue::Boolean(false)),
                        _ => Ok(FhirPathValue::empty()),
                    }
                }
                _ => Ok(FhirPathValue::empty()),
            }
        }

        "iif" => {
            if args.len() != 2 && args.len() != 3 {
                return Err(FhirPathError::invalid_argument_count("iif", 2, args.len()));
            }
            let condition = evaluate_ast(&args[0], context)?;
            let condition_bool = match condition {
                FhirPathValue::Boolean(b) => b,
                FhirPathValue::Empty => false,
                _ => return Err(FhirPathError::type_error("iif condition must be boolean".to_string())),
            };

            if condition_bool {
                evaluate_ast(&args[1], context)
            } else if args.len() == 3 {
                evaluate_ast(&args[2], context)
            } else {
                Ok(FhirPathValue::empty())
            }
        }

        "isDistinct" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("isDistinct", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::Collection(items) => {
                    // Check for duplicates using Vec instead of HashSet
                    for (i, item1) in items.iter().enumerate() {
                        for item2 in items.iter().skip(i + 1) {
                            if item1 == item2 {
                                return Ok(FhirPathValue::Boolean(false));
                            }
                        }
                    }
                    Ok(FhirPathValue::Boolean(true))
                }
                FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
                _ => Ok(FhirPathValue::Boolean(true)), // Single item is always distinct
            }
        }

        "descendants" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("descendants", 0, args.len()));
            }
            // For now, return empty - this is a complex function that requires tree traversal
            Ok(FhirPathValue::empty())
        }

        "trace" => {
            if args.len() > 2 {
                return Err(FhirPathError::invalid_argument_count("trace", 1, args.len()));
            }

            let label = if !args.is_empty() {
                match evaluate_ast(&args[0], context)? {
                    FhirPathValue::String(s) => s,
                    _ => "trace".to_string(),
                }
            } else {
                "trace".to_string()
            };

            // Optional second argument for projection
            let projection = if args.len() == 2 {
                Some(&args[1])
            } else {
                None
            };

            // Log the trace information
            match &context.input {
                FhirPathValue::Collection(items) => {
                    for (i, item) in items.iter().enumerate() {
                        if let Some(proj) = projection {
                            let proj_context = EvaluationContext::new(item.clone());
                            match evaluate_ast(proj, &proj_context) {
                                Ok(proj_result) => println!("TRACE[{}][{}]: {}", label, i, proj_result),
                                Err(_) => println!("TRACE[{}][{}]: <error>", label, i),
                            }
                        } else {
                            println!("TRACE[{}][{}]: {}", label, i, item);
                        }
                    }
                }
                single => {
                    if let Some(proj) = projection {
                        let proj_context = EvaluationContext::new(single.clone());
                        match evaluate_ast(proj, &proj_context) {
                            Ok(proj_result) => println!("TRACE[{}]: {}", label, proj_result),
                            Err(_) => println!("TRACE[{}]: <error>", label),
                        }
                    } else {
                        println!("TRACE[{}]: {}", label, single);
                    }
                }
            }

            // Return the input unchanged
            Ok(context.input.clone())
        }

        "children" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("children", 0, args.len()));
            }
            // For now, return empty - this is a complex function that requires tree traversal
            Ok(FhirPathValue::empty())
        }

        "repeat" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("repeat", 1, args.len()));
            }
            // For now, return empty - this is a complex function that requires iterative evaluation
            Ok(FhirPathValue::empty())
        }

        "aggregate" => {
            if args.len() < 1 || args.len() > 2 {
                return Err(FhirPathError::invalid_argument_count("aggregate", 1, args.len()));
            }
            // For now, return empty - this is a complex function that requires iterative evaluation
            Ok(FhirPathValue::empty())
        }

        "encode" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("encode", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let encoding_val = evaluate_ast(&args[0], context)?;
                    match encoding_val {
                        FhirPathValue::String(encoding) => {
                            match encoding.as_str() {
                                "base64" => {
                                    use base64::{Engine as _, engine::general_purpose};
                                    let encoded = general_purpose::STANDARD.encode(s.as_bytes());
                                    Ok(FhirPathValue::String(encoded))
                                }
                                "hex" => {
                                    let encoded = hex::encode(s.as_bytes());
                                    Ok(FhirPathValue::String(encoded))
                                }
                                _ => Err(FhirPathError::type_error(format!("Unsupported encoding: {}", encoding))),
                            }
                        }
                        _ => Err(FhirPathError::type_error("encode() encoding argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "encode() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "decode" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("decode", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let encoding_val = evaluate_ast(&args[0], context)?;
                    match encoding_val {
                        FhirPathValue::String(encoding) => {
                            match encoding.as_str() {
                                "base64" => {
                                    use base64::{Engine as _, engine::general_purpose};
                                    match general_purpose::STANDARD.decode(s) {
                                        Ok(decoded_bytes) => {
                                            match String::from_utf8(decoded_bytes) {
                                                Ok(decoded_string) => Ok(FhirPathValue::String(decoded_string)),
                                                Err(_) => Ok(FhirPathValue::empty()),
                                            }
                                        }
                                        Err(_) => Ok(FhirPathValue::empty()),
                                    }
                                }
                                "hex" => {
                                    match hex::decode(s) {
                                        Ok(decoded_bytes) => {
                                            match String::from_utf8(decoded_bytes) {
                                                Ok(decoded_string) => Ok(FhirPathValue::String(decoded_string)),
                                                Err(_) => Ok(FhirPathValue::empty()),
                                            }
                                        }
                                        Err(_) => Ok(FhirPathValue::empty()),
                                    }
                                }
                                _ => Err(FhirPathError::type_error(format!("Unsupported encoding: {}", encoding))),
                            }
                        }
                        _ => Err(FhirPathError::type_error("decode() encoding argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "decode() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "escape" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("escape", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let format_val = evaluate_ast(&args[0], context)?;
                    match format_val {
                        FhirPathValue::String(format) => {
                            match format.as_str() {
                                "json" => {
                                    let escaped = s.replace('\\', "\\\\")
                                                  .replace('"', "\\\"")
                                                  .replace('\n', "\\n")
                                                  .replace('\r', "\\r")
                                                  .replace('\t', "\\t");
                                    Ok(FhirPathValue::String(escaped))
                                }
                                "html" => {
                                    let escaped = s.replace('&', "&amp;")
                                                  .replace('<', "&lt;")
                                                  .replace('>', "&gt;")
                                                  .replace('"', "&quot;")
                                                  .replace('\'', "&#39;");
                                    Ok(FhirPathValue::String(escaped))
                                }
                                _ => Err(FhirPathError::type_error(format!("Unsupported escape format: {}", format))),
                            }
                        }
                        _ => Err(FhirPathError::type_error("escape() format argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "escape() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "unescape" => {
            if args.len() != 1 {
                return Err(FhirPathError::invalid_argument_count("unescape", 1, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => {
                    let format_val = evaluate_ast(&args[0], context)?;
                    match format_val {
                        FhirPathValue::String(format) => {
                            match format.as_str() {
                                "json" => {
                                    let unescaped = s.replace("\\\\", "\\")
                                                   .replace("\\\"", "\"")
                                                   .replace("\\n", "\n")
                                                   .replace("\\r", "\r")
                                                   .replace("\\t", "\t");
                                    Ok(FhirPathValue::String(unescaped))
                                }
                                "html" => {
                                    let unescaped = s.replace("&amp;", "&")
                                                   .replace("&lt;", "<")
                                                   .replace("&gt;", ">")
                                                   .replace("&quot;", "\"")
                                                   .replace("&#39;", "'");
                                    Ok(FhirPathValue::String(unescaped))
                                }
                                _ => Err(FhirPathError::type_error(format!("Unsupported unescape format: {}", format))),
                            }
                        }
                        _ => Err(FhirPathError::type_error("unescape() format argument must be a string".to_string())),
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "unescape() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        "trim" => {
            if !args.is_empty() {
                return Err(FhirPathError::invalid_argument_count("trim", 0, args.len()));
            }
            match &context.input {
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.trim().to_string())),
                _ => Err(FhirPathError::type_error(format!(
                    "trim() can only be called on strings, got {}",
                    context.input.type_name()
                ))),
            }
        }

        _ => Err(FhirPathError::unknown_function(name)),
    }
}

/// Evaluate a binary operation
fn evaluate_binary_operation(
    op: &crate::ast::BinaryOperator,
    left: &FhirPathValue,
    right: &FhirPathValue,
) -> Result<FhirPathValue> {
    use crate::ast::BinaryOperator;

    match op {
        BinaryOperator::Equal => {
            match (left, right) {
                (FhirPathValue::Quantity { value: _value1, unit: Some(unit1), .. },
                 FhirPathValue::Quantity { value: _value2, unit: Some(unit2), .. }) => {
                    // Check if units are comparable
                    match octofhir_ucum_core::is_comparable(unit1, unit2) {
                        Ok(true) => {
                            // Units are comparable, convert to common unit and compare values
                            let most_granular = left.most_granular_unit(right).unwrap_or_else(|| unit1.clone());

                            // Convert both values to the most granular unit
                            let left_converted = left.convert_to_unit(&most_granular)?;
                            let right_converted = right.convert_to_unit(&most_granular)?;

                            // Compare the converted values
                            if let (FhirPathValue::Quantity { value: value1, .. },
                                    FhirPathValue::Quantity { value: value2, .. }) = (&left_converted, &right_converted) {
                                Ok(FhirPathValue::Boolean(value1 == value2))
                            } else {
                                // This should never happen
                                Err(FhirPathError::evaluation_error("Conversion error in quantity comparison".to_string()))
                            }
                        },
                        Ok(false) => {
                            // Units are not comparable, return false
                            Ok(FhirPathValue::Boolean(false))
                        },
                        Err(_) => {
                            // Error in unit comparison, return empty
                            Ok(FhirPathValue::Empty)
                        }
                    }
                },
                (FhirPathValue::Quantity { value: value1, unit: None, .. },
                 FhirPathValue::Quantity { value: value2, unit: None, .. }) => {
                    // Both quantities have no units, compare values directly
                    Ok(FhirPathValue::Boolean(value1 == value2))
                },
                (FhirPathValue::Quantity { .. }, FhirPathValue::Quantity { .. }) => {
                    // One has a unit and one doesn't, return empty
                    Ok(FhirPathValue::Empty)
                },
                // Handle collection equality
                (FhirPathValue::Collection(left_items), FhirPathValue::Collection(right_items)) => {
                    // Collections are equal if they have the same elements in the same order
                    Ok(FhirPathValue::Boolean(left_items == right_items))
                },
                // For all other types, use the default equality
                _ => Ok(FhirPathValue::Boolean(left == right)),
            }
        },
        BinaryOperator::NotEqual => {
            // NotEqual is the negation of Equal
            let equal_result = evaluate_binary_operation(&BinaryOperator::Equal, left, right)?;
            match equal_result {
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
                FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                _ => Err(FhirPathError::evaluation_error("Equality operation should return boolean or empty".to_string())),
            }
        },

        BinaryOperator::LessThan => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a < b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(a < b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(&rust_decimal::Decimal::from(*a) < b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a < &rust_decimal::Decimal::from(*b)))
                }
                (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::Boolean(a < b))
                }
                (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                    Ok(FhirPathValue::Boolean(a < b))
                }
                (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                    Ok(FhirPathValue::Boolean(a < b))
                }
                (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                    Ok(FhirPathValue::Boolean(a < b))
                }
                // Type coercion: String to Integer/Decimal
                (FhirPathValue::String(a), FhirPathValue::Integer(b)) => {
                    if let Ok(a_int) = a.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(a_int < *b))
                    } else if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal < rust_decimal::Decimal::from(*b)))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Integer(a), FhirPathValue::String(b)) => {
                    if let Ok(b_int) = b.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(*a < b_int))
                    } else if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(rust_decimal::Decimal::from(*a) < b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::String(a), FhirPathValue::Decimal(b)) => {
                    if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal < *b))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Decimal(a), FhirPathValue::String(b)) => {
                    if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(*a < b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot compare {} and {} with <",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::LessThanOrEqual => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a <= b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(a <= b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(&rust_decimal::Decimal::from(*a) <= b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a <= &rust_decimal::Decimal::from(*b)))
                }
                (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::Boolean(a <= b))
                }
                (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                    Ok(FhirPathValue::Boolean(a <= b))
                }
                (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                    Ok(FhirPathValue::Boolean(a <= b))
                }
                (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                    Ok(FhirPathValue::Boolean(a <= b))
                }
                // Type coercion: String to Integer/Decimal
                (FhirPathValue::String(a), FhirPathValue::Integer(b)) => {
                    if let Ok(a_int) = a.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(a_int <= *b))
                    } else if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal <= rust_decimal::Decimal::from(*b)))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Integer(a), FhirPathValue::String(b)) => {
                    if let Ok(b_int) = b.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(*a <= b_int))
                    } else if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(rust_decimal::Decimal::from(*a) <= b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::String(a), FhirPathValue::Decimal(b)) => {
                    if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal <= *b))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Decimal(a), FhirPathValue::String(b)) => {
                    if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(*a <= b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with <=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot compare {} and {} with <=",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::GreaterThan => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a > b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(a > b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(&rust_decimal::Decimal::from(*a) > b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a > &rust_decimal::Decimal::from(*b)))
                }
                (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::Boolean(a > b))
                }
                (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                    Ok(FhirPathValue::Boolean(a > b))
                }
                (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                    Ok(FhirPathValue::Boolean(a > b))
                }
                (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                    Ok(FhirPathValue::Boolean(a > b))
                }
                // Type coercion: String to Integer/Decimal
                (FhirPathValue::String(a), FhirPathValue::Integer(b)) => {
                    if let Ok(a_int) = a.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(a_int > *b))
                    } else if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal > rust_decimal::Decimal::from(*b)))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Integer(a), FhirPathValue::String(b)) => {
                    if let Ok(b_int) = b.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(*a > b_int))
                    } else if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(rust_decimal::Decimal::from(*a) > b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::String(a), FhirPathValue::Decimal(b)) => {
                    if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal > *b))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Decimal(a), FhirPathValue::String(b)) => {
                    if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(*a > b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot compare {} and {} with >",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::GreaterThanOrEqual => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a >= b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(a >= b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Boolean(&rust_decimal::Decimal::from(*a) >= b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Boolean(a >= &rust_decimal::Decimal::from(*b)))
                }
                (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::Boolean(a >= b))
                }
                (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                    Ok(FhirPathValue::Boolean(a >= b))
                }
                (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                    Ok(FhirPathValue::Boolean(a >= b))
                }
                (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                    Ok(FhirPathValue::Boolean(a >= b))
                }
                // Type coercion: String to Integer/Decimal
                (FhirPathValue::String(a), FhirPathValue::Integer(b)) => {
                    if let Ok(a_int) = a.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(a_int >= *b))
                    } else if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal >= rust_decimal::Decimal::from(*b)))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Integer(a), FhirPathValue::String(b)) => {
                    if let Ok(b_int) = b.parse::<i64>() {
                        Ok(FhirPathValue::Boolean(*a >= b_int))
                    } else if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(rust_decimal::Decimal::from(*a) >= b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::String(a), FhirPathValue::Decimal(b)) => {
                    if let Ok(a_decimal) = a.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(a_decimal >= *b))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                (FhirPathValue::Decimal(a), FhirPathValue::String(b)) => {
                    if let Ok(b_decimal) = b.parse::<rust_decimal::Decimal>() {
                        Ok(FhirPathValue::Boolean(*a >= b_decimal))
                    } else {
                        Err(FhirPathError::type_error(format!(
                            "Cannot compare {} and {} with >=",
                            left.type_name(),
                            right.type_name()
                        )))
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot compare {} and {} with >=",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::Equivalent => {
            // FHIRPath equivalence (~) - handles null/empty values differently than equality
            match (left, right) {
                (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(FhirPathValue::Boolean(true)),
                (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) if a.is_empty() && b.is_empty() => {
                    Ok(FhirPathValue::Boolean(true))
                }
                (FhirPathValue::Empty, FhirPathValue::Collection(b)) if b.is_empty() => {
                    Ok(FhirPathValue::Boolean(true))
                }
                (FhirPathValue::Collection(a), FhirPathValue::Empty) if a.is_empty() => {
                    Ok(FhirPathValue::Boolean(true))
                }
                (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                    Ok(FhirPathValue::Boolean(false))
                }
                (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) if a.is_empty() || b.is_empty() => {
                    Ok(FhirPathValue::Boolean(a.is_empty() && b.is_empty()))
                }
                _ => {
                    // For non-empty values, equivalence is the same as equality
                    Ok(FhirPathValue::Boolean(left == right))
                }
            }
        }

        BinaryOperator::NotEquivalent => {
            // FHIRPath not-equivalence (!~) - negation of equivalence
            let equiv_result = evaluate_binary_operation(&BinaryOperator::Equivalent, left, right)?;
            match equiv_result {
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
                _ => Err(FhirPathError::evaluation_error("Equivalence operation should return boolean".to_string())),
            }
        }

        BinaryOperator::Add => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Integer(a + b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Decimal(a + b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) + b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Decimal(a + rust_decimal::Decimal::from(*b)))
                }
                (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::String(format!("{}{}", a, b)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot add {} and {}",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::Subtract => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Integer(a - b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Decimal(a - b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) - b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Decimal(a - rust_decimal::Decimal::from(*b)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot subtract {} and {}",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::Multiply => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Integer(a * b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Decimal(a * b))
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) * b))
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    Ok(FhirPathValue::Decimal(a * rust_decimal::Decimal::from(*b)))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot multiply {} and {}",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::Divide => {
            match (left, right) {
                (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                    if *b == 0 {
                        Err(FhirPathError::arithmetic_error("Division by zero"))
                    } else {
                        // Integer division in FHIRPath returns decimal
                        let a_decimal = rust_decimal::Decimal::from(*a);
                        let b_decimal = rust_decimal::Decimal::from(*b);
                        Ok(FhirPathValue::Decimal(a_decimal / b_decimal))
                    }
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                    if b.is_zero() {
                        Err(FhirPathError::arithmetic_error("Division by zero"))
                    } else {
                        Ok(FhirPathValue::Decimal(a / b))
                    }
                }
                (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                    if b.is_zero() {
                        Err(FhirPathError::arithmetic_error("Division by zero"))
                    } else {
                        Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) / b))
                    }
                }
                (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                    if *b == 0 {
                        Err(FhirPathError::arithmetic_error("Division by zero"))
                    } else {
                        Ok(FhirPathValue::Decimal(a / rust_decimal::Decimal::from(*b)))
                    }
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot divide {} and {}",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::And => {
            match (left.to_boolean(), right.to_boolean()) {
                (Some(a), Some(b)) => Ok(FhirPathValue::Boolean(a && b)),
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot apply 'and' to {} and {}",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::Or => {
            match (left.to_boolean(), right.to_boolean()) {
                (Some(a), Some(b)) => Ok(FhirPathValue::Boolean(a || b)),
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot apply 'or' to {} and {}",
                    left.type_name(),
                    right.type_name()
                ))),
            }
        }

        BinaryOperator::Union => evaluate_union(left, right),

        BinaryOperator::Concatenate => {
            // String concatenation (&)
            match (left, right) {
                (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::String(format!("{}{}", a, b)))
                }
                (FhirPathValue::String(a), FhirPathValue::Empty) => {
                    Ok(FhirPathValue::String(a.clone()))
                }
                (FhirPathValue::Empty, FhirPathValue::String(b)) => {
                    Ok(FhirPathValue::String(b.clone()))
                }
                (FhirPathValue::Empty, FhirPathValue::Empty) => {
                    Ok(FhirPathValue::Empty)
                }
                _ => {
                    // Convert to strings - handle collections specially
                    let left_str = match left {
                        FhirPathValue::Collection(items) => {
                            items.iter()
                                .map(|item| match item.to_string_value() {
                                    Some(s) => s,
                                    None => item.to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(",")
                        }
                        _ => left.to_string_value().unwrap_or_else(|| left.to_string()),
                    };

                    let right_str = match right {
                        FhirPathValue::Collection(items) => {
                            items.iter()
                                .map(|item| match item.to_string_value() {
                                    Some(s) => s,
                                    None => item.to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(",")
                        }
                        _ => right.to_string_value().unwrap_or_else(|| right.to_string()),
                    };

                    Ok(FhirPathValue::String(format!("{}{}", left_str, right_str)))
                }
            }
        }

        BinaryOperator::Is => {
            // Type checking (is)
            match right {
                FhirPathValue::String(type_name) => {
                    let actual_type = left.type_name().to_lowercase();
                    let expected_type = type_name.to_lowercase();
                    Ok(FhirPathValue::Boolean(actual_type == expected_type))
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Type checking 'is' requires a type name as string, got {}",
                    right.type_name()
                ))),
            }
        }

        _ => Err(FhirPathError::evaluation_error(format!(
            "Binary operator not yet implemented: {:?}",
            op
        ))),
    }
}

/// Evaluate a unary operation
fn evaluate_unary_operation(
    op: &crate::ast::UnaryOperator,
    operand: &FhirPathValue,
) -> Result<FhirPathValue> {
    use crate::ast::UnaryOperator;

    match op {
        UnaryOperator::Not => {
            if let Some(bool_val) = operand.to_boolean() {
                Ok(FhirPathValue::Boolean(!bool_val))
            } else {
                Err(FhirPathError::type_error(format!(
                    "Cannot apply 'not' to {}",
                    operand.type_name()
                )))
            }
        }

        UnaryOperator::Minus => {
            match operand {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(-i)),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
                FhirPathValue::Quantity { value, unit, ucum_expr } => {
                    Ok(FhirPathValue::Quantity {
                        value: -value,
                        unit: unit.clone(),
                        ucum_expr: ucum_expr.clone(),
                    })
                }
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot apply unary minus to {}",
                    operand.type_name()
                ))),
            }
        }

        UnaryOperator::Plus => {
            match operand {
                FhirPathValue::Integer(_) => Ok(operand.clone()),
                FhirPathValue::Decimal(_) => Ok(operand.clone()),
                FhirPathValue::Quantity { .. } => Ok(operand.clone()),
                _ => Err(FhirPathError::type_error(format!(
                    "Cannot apply unary plus to {}",
                    operand.type_name()
                ))),
            }
        }
    }
}

/// Check if a property name represents a FHIR polymorphic element
fn is_polymorphic_element(property_name: &str) -> bool {
    // Common FHIR polymorphic elements
    matches!(property_name,
        "value" | "effective" | "onset" | "abatement" | "occurrence" |
        "performed" | "deceased" | "multipleBirth" | "contact" | "address" |
        "telecom" | "photo" | "qualification" | "communication" | "link" |
        "contained" | "extension" | "modifierExtension"
    )
}

/// Find a polymorphic property by searching for properties that start with the base name
fn find_polymorphic_property<'a>(resource: &'a crate::model::FhirResource, base_name: &str) -> Option<&'a serde_json::Value> {
    let obj = resource.to_json();
    if let serde_json::Value::Object(map) = obj {
        // Look for any property that starts with the base name
        for (key, value) in map {
            if key.starts_with(base_name) && key.len() > base_name.len() {
                // Make sure the next character is uppercase (following FHIR naming convention)
                if let Some(next_char) = key.chars().nth(base_name.len()) {
                    if next_char.is_uppercase() {
                        return Some(value);
                    }
                }
            }
        }
    }
    None
}

/// Evaluate path navigation (property access)
fn evaluate_path_navigation(base: &FhirPathValue, path: &str) -> Result<FhirPathValue> {
    match base {
        FhirPathValue::Resource(resource) => {
            // First try direct property access
            if let Some(property_value) = resource.get_property(path) {
                Ok(FhirPathValue::from(property_value.clone()))
            } else {
                // Handle FHIR polymorphic elements (e.g., value[x])
                // If the requested property is a known polymorphic element,
                // look for any property that starts with that name
                if is_polymorphic_element(path) {
                    if let Some(typed_property_value) = find_polymorphic_property(resource, path) {
                        Ok(FhirPathValue::from(typed_property_value.clone()))
                    } else {
                        Ok(FhirPathValue::empty())
                    }
                } else {
                    Ok(FhirPathValue::empty())
                }
            }
        }

        FhirPathValue::Collection(items) => {
            let mut results = Vec::new();
            for item in items {
                let result = evaluate_path_navigation(item, path)?;
                match result {
                    FhirPathValue::Collection(mut sub_items) => {
                        results.append(&mut sub_items);
                    }
                    FhirPathValue::Empty => {
                        // Skip empty results
                    }
                    single => {
                        results.push(single);
                    }
                }
            }
            Ok(FhirPathValue::Collection(results))
        }

        _ => Ok(FhirPathValue::empty()),
    }
}

/// Evaluate index access
fn evaluate_index_access(base: &FhirPathValue, index: &FhirPathValue) -> Result<FhirPathValue> {
    let index_num = match index {
        FhirPathValue::Integer(i) => *i,
        _ => return Err(FhirPathError::type_error("Index must be an integer".to_string())),
    };

    match base {
        FhirPathValue::Collection(items) => {
            if index_num < 0 || index_num as usize >= items.len() {
                Ok(FhirPathValue::empty())
            } else {
                Ok(items[index_num as usize].clone())
            }
        }
        _ => {
            if index_num == 0 {
                Ok(base.clone())
            } else {
                Ok(FhirPathValue::empty())
            }
        }
    }
}

/// Evaluate filter expression
fn evaluate_filter(
    base: &FhirPathValue,
    condition: &ExpressionNode,
    context: &EvaluationContext,
) -> Result<FhirPathValue> {
    match base {
        FhirPathValue::Collection(items) => {
            let mut results = Vec::new();
            for item in items {
                // Create new context with current item
                let item_context = EvaluationContext::new(item.clone());
                let condition_result = evaluate_ast(condition, &item_context)?;

                if let Some(true) = condition_result.to_boolean() {
                    results.push(item.clone());
                }
            }
            Ok(FhirPathValue::Collection(results))
        }
        _ => {
            // For single items, create context and evaluate condition
            let item_context = EvaluationContext::new(base.clone());
            let condition_result = evaluate_ast(condition, &item_context)?;

            if let Some(true) = condition_result.to_boolean() {
                Ok(base.clone())
            } else {
                Ok(FhirPathValue::empty())
            }
        }
    }
}

/// Evaluate union of two values
fn evaluate_union(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
    let mut left_items = left.clone().to_collection();
    let right_items = right.clone().to_collection();

    // Add items from right that are not already in left (deduplication)
    for item in right_items {
        if !left_items.contains(&item) {
            left_items.push(item);
        }
    }

    Ok(FhirPathValue::Collection(left_items))
}

/// Evaluate type check with enhanced inheritance support
fn evaluate_type_check(value: &FhirPathValue, type_name: &str, type_registry: &FhirTypeRegistry) -> Result<FhirPathValue> {
    let matches = match value {
        FhirPathValue::Collection(items) => {
            // For collections, check if all items match the type
            if items.is_empty() {
                false
            } else {
                items.iter().all(|item| {
                    evaluate_single_type_check(item, type_name, type_registry)
                })
            }
        }
        _ => evaluate_single_type_check(value, type_name, type_registry),
    };

    Ok(FhirPathValue::Boolean(matches))
}

/// Check if a single value matches the given type with enhanced inheritance support
fn evaluate_single_type_check(value: &FhirPathValue, type_name: &str, type_registry: &FhirTypeRegistry) -> bool {
    match type_name.to_lowercase().as_str() {
        // Primitive types
        "boolean" => matches!(value, FhirPathValue::Boolean(_)),
        "integer" => matches!(value, FhirPathValue::Integer(_)),
        "decimal" => matches!(value, FhirPathValue::Decimal(_)),
        "string" => matches!(value, FhirPathValue::String(_)),
        "date" => matches!(value, FhirPathValue::Date(_)),
        "datetime" => matches!(value, FhirPathValue::DateTime(_)),
        "time" => matches!(value, FhirPathValue::Time(_)),
        "quantity" => matches!(value, FhirPathValue::Quantity { .. }),

        // FHIR System types
        "system.boolean" => matches!(value, FhirPathValue::Boolean(_)),
        "system.integer" => matches!(value, FhirPathValue::Integer(_)),
        "system.decimal" => matches!(value, FhirPathValue::Decimal(_)),
        "system.string" => matches!(value, FhirPathValue::String(_)),
        "system.date" => matches!(value, FhirPathValue::Date(_)),
        "system.datetime" => matches!(value, FhirPathValue::DateTime(_)),
        "system.time" => matches!(value, FhirPathValue::Time(_)),
        "system.quantity" => matches!(value, FhirPathValue::Quantity { .. }),

        // For complex types and resources, use enhanced type checking with inheritance
        _ => {
            match value {
                FhirPathValue::Resource(resource) => {
                    let obj = resource.to_json();

                    // Check resourceType field for FHIR resources
                    if let Some(resource_type) = obj.get("resourceType") {
                        if let Some(rt_str) = resource_type.as_str() {
                            // Use type registry for inheritance-based checking
                            return type_registry.is_type_compatible(rt_str, type_name);
                        }
                    }

                    // Check type field for complex types
                    if let Some(type_field) = obj.get("type") {
                        if let Some(type_str) = type_field.as_str() {
                            // Use type registry for inheritance-based checking
                            return type_registry.is_type_compatible(type_str, type_name);
                        }
                    }

                    // For Quantity type, check if it has value and unit fields
                    if type_name.to_lowercase() == "quantity" {
                        return obj.get("value").is_some() && (obj.get("unit").is_some() || obj.get("code").is_some());
                    }

                    // Check if the requested type is known in the registry
                    if type_registry.is_known_type(type_name) {
                        // If it's a known type but we couldn't match it, return false
                        return false;
                    }

                    // For unknown types, fall back to exact string matching
                    false
                }
                _ => {
                    // For non-resource values, check if the value's type matches
                    let value_type = value.type_name();
                    type_registry.is_type_compatible(value_type, type_name)
                }
            }
        }
    }
}

/// Evaluate type cast
fn evaluate_type_cast(value: &FhirPathValue, type_name: &str) -> Result<FhirPathValue> {
    // For now, just return the value as-is
    // Type casting will be implemented in later phases
    Ok(value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_literal_evaluation() {
        let input = json!({});
        let result = evaluate_expression("42", input).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[test]
    fn test_property_access() {
        let input = json!({"name": "John"});
        let result = evaluate_expression("name", input).unwrap();
        assert_eq!(result, FhirPathValue::String("John".to_string()));
    }

    #[test]
    fn test_function_call() {
        let input = json!([1, 2, 3]);
        let result = evaluate_expression("count()", input).unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));
    }

    #[test]
    fn debug_patient_property_access() {
        let patient_data = json!({
            "resourceType": "Patient",
            "name": [
                {
                    "given": ["John", "Q"],
                    "family": "Doe"
                }
            ]
        });

        println!("Input data: {}", serde_json::to_string_pretty(&patient_data).unwrap());

        // Test simple property access
        println!("\n=== Testing Patient ===");
        match evaluate_expression("Patient", patient_data.clone()) {
            Ok(result) => println!("Patient result: {:?}", result),
            Err(e) => println!("Patient error: {:?}", e),
        }

        println!("\n=== Testing name ===");
        match evaluate_expression("name", patient_data.clone()) {
            Ok(result) => println!("name result: {:?}", result),
            Err(e) => println!("name error: {:?}", e),
        }

        println!("\n=== Testing Patient.name ===");
        match evaluate_expression("Patient.name", patient_data.clone()) {
            Ok(result) => println!("Patient.name result: {:?}", result),
            Err(e) => println!("Patient.name error: {:?}", e),
        }

        println!("\n=== Testing name.given ===");
        match evaluate_expression("name.given", patient_data.clone()) {
            Ok(result) => println!("name.given result: {:?}", result),
            Err(e) => println!("name.given error: {:?}", e),
        }

        println!("\n=== Testing Patient.name.given ===");
        match evaluate_expression("Patient.name.given", patient_data.clone()) {
            Ok(result) => println!("Patient.name.given result: {:?}", result),
            Err(e) => println!("Patient.name.given error: {:?}", e),
        }
    }

    #[test]
    fn debug_type_checking() {
        use crate::parser::parse_expression;
        use chrono::NaiveDate;

        println!("=== Debugging type checking expressions ===");

        // Test date parsing directly
        println!("\n=== Testing date parsing directly ===");
        match NaiveDate::parse_from_str("2015", "%Y") {
            Ok(date) => println!("Parsed date: {:?}", date),
            Err(e) => println!("Date parse error: {:?}", e),
        }

        // Test parsing of @2015 alone
        println!("\n=== Parsing @2015 ===");
        match parse_expression("@2015") {
            Ok(ast) => println!("AST: {:#?}", ast),
            Err(e) => println!("Parse error: {:?}", e),
        }

        // Test parsing of @2015.is(Date)
        println!("\n=== Parsing @2015.is(Date) ===");
        match parse_expression("@2015.is(Date)") {
            Ok(ast) => println!("AST: {:#?}", ast),
            Err(e) => println!("Parse error: {:?}", e),
        }

        // Test parsing of just Date
        println!("\n=== Parsing Date ===");
        match parse_expression("Date") {
            Ok(ast) => println!("AST: {:#?}", ast),
            Err(e) => println!("Parse error: {:?}", e),
        }

        // Test parsing of is(Date)
        println!("\n=== Parsing is(Date) ===");
        match parse_expression("is(Date)") {
            Ok(ast) => println!("AST: {:#?}", ast),
            Err(e) => println!("Parse error: {:?}", e),
        }

        // Test evaluation
        println!("\n=== Evaluating @2015.is(Date) ===");
        let input = json!({});
        match evaluate_expression("@2015.is(Date)", input) {
            Ok(result) => println!("Result: {:?}", result),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    #[test]
    fn debug_predicate_expressions() {
        println!("Testing predicate expressions:");

        // Test simple predicate with boolean context
        let test_data = json!(true);
        println!("\n=== Testing '= true' with boolean context ===");
        match evaluate_expression("= true", test_data.clone()) {
            Ok(result) => println!("Result: {:?}", result),
            Err(e) => println!("Error: {:?}", e),
        }

        // Test with patient data
        let patient_data = json!({
            "resourceType": "Patient",
            "name": [{"given": ["Peter"]}]
        });

        println!("\n=== Testing 'Patient.name.exists() = true' ===");
        match evaluate_expression("Patient.name.exists() = true", patient_data.clone()) {
            Ok(result) => println!("Result: {:?}", result),
            Err(e) => println!("Error: {:?}", e),
        }

        println!("\n=== Testing just 'Patient.name.exists()' ===");
        match evaluate_expression("Patient.name.exists()", patient_data.clone()) {
            Ok(result) => println!("Result: {:?}", result),
            Err(e) => println!("Error: {:?}", e),
        }

        println!("\n=== Testing '= false' with boolean context ===");
        let false_data = json!(false);
        match evaluate_expression("= false", false_data.clone()) {
            Ok(result) => println!("Result: {:?}", result),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    #[test]
    fn debug_date_conversion() {
        use chrono::NaiveDate;

        println!("=== Debugging date conversion functions ===");

        // Test chrono parsing directly
        let test_strings = ["2015", "2015-02", "2015-02-04"];
        let date_formats = ["%Y", "%Y-%m", "%Y-%m-%d"];

        for test_str in &test_strings {
            println!("\nTesting string: '{}'", test_str);
            let can_convert = date_formats.iter().any(|fmt| {
                let result = NaiveDate::parse_from_str(test_str, fmt);
                println!("  Format '{}': {:?}", fmt, result);
                result.is_ok()
            });
            println!("  Can convert: {}", can_convert);
        }

        // Test the actual convertsToDate function
        println!("\n=== Testing convertsToDate function ===");
        for test_str in &test_strings {
            let input = json!(test_str);
            match evaluate_expression(&format!("'{}'.convertsToDate()", test_str), input) {
                Ok(result) => println!("'{}' convertsToDate(): {:?}", test_str, result),
                Err(e) => println!("'{}' convertsToDate() error: {:?}", test_str, e),
            }
        }
    }
}
