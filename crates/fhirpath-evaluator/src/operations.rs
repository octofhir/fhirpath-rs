//! FHIRPath binary and unary operation evaluation
//!
//! This module handles the evaluation of binary operations (arithmetic, comparison, logical)
//! and unary operations (plus, minus, not) by delegating to specialized evaluators.

use crate::context::EvaluationContext as LocalEvaluationContext;
use crate::evaluators::{
    ArithmeticEvaluator, CollectionEvaluator, ComparisonEvaluator, LogicalEvaluator,
};
use octofhir_fhirpath_ast::{
    BinaryOpData, BinaryOperator, ConditionalData, ExpressionNode, UnaryOperator,
};
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;
use std::sync::Arc;

/// Binary operation evaluator methods
impl crate::FhirPathEngine {
    /// Evaluate binary operations using specialized evaluator modules
    pub async fn evaluate_binary_operation(
        &self,
        op_data: &BinaryOpData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate left operand
        let left = self
            .evaluate_node_async(&op_data.left, input.clone(), context, depth + 1)
            .await?;

        // Evaluate right operand with special handling for type operators
        let right = if matches!(op_data.op, BinaryOperator::Is)
            && Self::is_type_identifier_expression(&op_data.right)
        {
            // Convert type identifier to TypeInfoObject for type operators
            match &op_data.right {
                ExpressionNode::Identifier(type_name) => {
                    if self.is_type_identifier(type_name) {
                        // Create a TypeInfoObject for known type identifiers
                        let (namespace, name) = if type_name.contains('.') {
                            let parts: Vec<&str> = type_name.split('.').collect();
                            (parts[0], parts[1])
                        } else {
                            // Handle common FHIRPath types
                            match type_name.to_lowercase().as_str() {
                                "boolean" | "integer" | "decimal" | "string" | "date"
                                | "datetime" | "time" | "quantity" | "collection" => {
                                    ("System", type_name.as_str())
                                }
                                "code" | "uri" | "url" | "canonical" | "oid" | "uuid" | "id"
                                | "markdown" | "base64binary" | "instant" | "positiveint"
                                | "unsignedint" | "xhtml" => ("FHIR", type_name.as_str()),
                                _ => ("System", type_name.as_str()),
                            }
                        };
                        FhirPathValue::TypeInfoObject {
                            namespace: Arc::from(namespace),
                            name: Arc::from(name),
                        }
                    } else {
                        // Treat as string literal for backward compatibility
                        FhirPathValue::String(type_name.clone().into())
                    }
                }
                ExpressionNode::Path { base, path } => {
                    // Handle qualified type names like FHIR.uuid, System.Boolean
                    if let ExpressionNode::Identifier(namespace) = base.as_ref() {
                        if matches!(namespace.as_str(), "FHIR" | "System") {
                            FhirPathValue::TypeInfoObject {
                                namespace: Arc::from(namespace.as_str()),
                                name: Arc::from(path.as_str()),
                            }
                        } else {
                            // Evaluate as normal path expression
                            self.evaluate_node_async(
                                &op_data.right,
                                input.clone(),
                                context,
                                depth + 1,
                            )
                            .await?
                        }
                    } else {
                        // Evaluate as normal path expression
                        self.evaluate_node_async(&op_data.right, input.clone(), context, depth + 1)
                            .await?
                    }
                }
                _ => {
                    // For other type expressions, evaluate normally
                    self.evaluate_node_async(&op_data.right, input.clone(), context, depth + 1)
                        .await?
                }
            }
        } else {
            // Standard operand evaluation
            self.evaluate_node_async(&op_data.right, input.clone(), context, depth + 1)
                .await?
        };

        // Use evaluators - these return natural types without forced collection wrapping
        let result = match &op_data.op {
            // Arithmetic operations
            BinaryOperator::Add => ArithmeticEvaluator::evaluate_addition(&left, &right).await?,
            BinaryOperator::Subtract => {
                ArithmeticEvaluator::evaluate_subtraction(&left, &right).await?
            }
            BinaryOperator::Multiply => {
                ArithmeticEvaluator::evaluate_multiplication(&left, &right).await?
            }
            BinaryOperator::Divide => ArithmeticEvaluator::evaluate_division(&left, &right).await?,
            BinaryOperator::Modulo => ArithmeticEvaluator::evaluate_modulo(&left, &right).await?,
            BinaryOperator::IntegerDivide => {
                ArithmeticEvaluator::evaluate_integer_division(&left, &right).await?
            }
            BinaryOperator::Concatenate => self.evaluate_concatenate(left, right).await?,

            // Comparison operations
            BinaryOperator::Equal => ComparisonEvaluator::evaluate_equals(&left, &right).await?,
            BinaryOperator::NotEqual => {
                ComparisonEvaluator::evaluate_not_equals(&left, &right).await?
            }
            BinaryOperator::LessThan => {
                ComparisonEvaluator::evaluate_less_than(&left, &right).await?
            }
            BinaryOperator::LessThanOrEqual => {
                ComparisonEvaluator::evaluate_less_than_or_equal(&left, &right).await?
            }
            BinaryOperator::GreaterThan => {
                ComparisonEvaluator::evaluate_greater_than(&left, &right).await?
            }
            BinaryOperator::GreaterThanOrEqual => {
                ComparisonEvaluator::evaluate_greater_than_or_equal(&left, &right).await?
            }
            BinaryOperator::Equivalent => {
                ComparisonEvaluator::evaluate_equivalent(&left, &right).await?
            }
            BinaryOperator::NotEquivalent => {
                ComparisonEvaluator::evaluate_not_equivalent(&left, &right).await?
            }

            // Logical operations
            BinaryOperator::And => LogicalEvaluator::evaluate_and(&left, &right).await?,
            BinaryOperator::Or => LogicalEvaluator::evaluate_or(&left, &right).await?,
            BinaryOperator::Xor => LogicalEvaluator::evaluate_xor(&left, &right).await?,
            BinaryOperator::Implies => LogicalEvaluator::evaluate_implies(&left, &right).await?,

            // Collection operations
            BinaryOperator::Contains => {
                CollectionEvaluator::evaluate_contains(&left, &right).await?
            }
            BinaryOperator::In => CollectionEvaluator::evaluate_in(&left, &right).await?,
            BinaryOperator::Union => CollectionEvaluator::evaluate_union(&left, &right).await?,

            // Type checking operations
            BinaryOperator::Is => self.evaluate_is_operator(&left, &right, context).await?,
        };

        // Return raw result - collection wrapping handled by main evaluate functions
        Ok(result)
    }

    /// Evaluate concatenation (&) operator
    /// Per FHIRPath spec: For strings, concatenates the strings, where an empty operand
    /// is taken to be the empty string. This differs from + which returns empty collection
    /// when one operand is empty.
    pub async fn evaluate_concatenate(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Extract string from left operand
        let left_str = match &left {
            FhirPathValue::Empty => String::new(),
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Collection(col) => {
                if col.is_empty() {
                    String::new()
                } else if col.len() == 1 {
                    // Single element collection - check if it's a string
                    match col.first().unwrap() {
                        FhirPathValue::String(s) => s.to_string(),
                        _ => String::new(),
                    }
                } else {
                    // Multiple elements - per spec, this should return empty
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => return Ok(FhirPathValue::Empty), // Non-string operand
        };

        // Extract string from right operand
        let right_str = match &right {
            FhirPathValue::Empty => String::new(), // Empty operand returns Empty
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Collection(col) => {
                if col.is_empty() {
                    String::new()
                } else if col.len() == 1 {
                    // Single element collection - check if it's a string
                    match col.first().unwrap() {
                        FhirPathValue::String(s) => s.to_string(),
                        _ => String::new(),
                    }
                } else {
                    // Multiple elements - per spec, this should return empty
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => return Ok(FhirPathValue::Empty), // Non-string operand
        };

        Ok(FhirPathValue::String(
            format!("{left_str}{right_str}").into(),
        ))
    }

    /// Evaluate unary operations
    pub async fn evaluate_unary_operation(
        &self,
        op: &UnaryOperator,
        operand: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate operand
        let operand_value = self
            .evaluate_node_async(operand, input.clone(), context, depth + 1)
            .await?;

        // Use specialized evaluators based on operation type
        match op {
            UnaryOperator::Plus => ArithmeticEvaluator::evaluate_unary_plus(&operand_value).await,
            UnaryOperator::Minus => ArithmeticEvaluator::evaluate_unary_minus(&operand_value).await,
            UnaryOperator::Not => LogicalEvaluator::evaluate_not(&operand_value).await,
        }
    }

    /// Evaluate conditional expressions (iif)
    pub async fn evaluate_conditional(
        &self,
        cond_data: &ConditionalData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate condition
        let condition = self
            .evaluate_node_async(&cond_data.condition, input.clone(), context, depth + 1)
            .await?;

        // Check if condition is a valid boolean (strict checking for iif function)
        let is_true = match &condition {
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Empty => Some(false),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(false)
                } else if c.len() == 1 {
                    // Single item collection: check if it's a boolean
                    match c.first().unwrap() {
                        FhirPathValue::Boolean(b) => Some(*b),
                        FhirPathValue::Empty => Some(false),
                        _ => None, // Non-boolean single item
                    }
                } else {
                    // Multiple items - not valid for iif condition
                    None
                }
            }
            _ => None, // Non-boolean values are not valid for iif
        };

        match is_true {
            Some(true) => {
                // Valid boolean condition that's true
                self.evaluate_node_async(&cond_data.then_expr, input, context, depth + 1)
                    .await
            }
            Some(false) => {
                // Valid boolean condition that's false
                if let Some(else_expr) = &cond_data.else_expr {
                    self.evaluate_node_async(else_expr, input, context, depth + 1)
                        .await
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            None => {
                // Invalid condition (non-boolean) - return empty
                Ok(FhirPathValue::Empty)
            }
        }
    }
}
