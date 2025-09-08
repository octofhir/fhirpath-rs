//! Core evaluator implementation for basic FHIRPath expressions
//!
//! This module implements the CoreEvaluator which handles fundamental expression types:
//! - Literals (strings, numbers, booleans, dates, etc.)
//! - Identifiers (simple property references)
//! - Variables (context variables like $this, %resource, etc.)
//! - Parenthesized expressions

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::sync::Arc;

use crate::{
    ast::{ExpressionNode, IdentifierNode, LiteralNode, VariableNode},
    core::{FhirPathError, FhirPathValue, Result, error_code::*},
    evaluator::metadata_core::MetadataCoreEvaluator,
    evaluator::{
        EvaluationContext,
        traits::{ExpressionEvaluator, MetadataAwareEvaluator},
    },
    path::CanonicalPath,
    typing::{TypeResolver, type_utils},
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue, collection_utils},
};

/// Core evaluator for basic expression types
///
/// This evaluator handles the most fundamental FHIRPath expression types
/// that don't require complex navigation or function calls.
pub struct CoreEvaluator;

impl CoreEvaluator {
    /// Create a new core evaluator instance
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a literal expression
    fn evaluate_literal(&self, literal: &LiteralNode) -> Result<FhirPathValue> {
        use crate::ast::literal::LiteralValue;

        let value = match &literal.value {
            LiteralValue::String(s) => FhirPathValue::String(s.clone()),
            LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
            LiteralValue::Decimal(d) => FhirPathValue::Decimal(*d),
            LiteralValue::Boolean(b) => FhirPathValue::Boolean(*b),
            LiteralValue::Date(d) => FhirPathValue::Date(d.clone()),
            LiteralValue::DateTime(dt) => FhirPathValue::DateTime(dt.clone()),
            LiteralValue::Time(t) => FhirPathValue::Time(t.clone()),
            LiteralValue::Quantity { value, unit } => FhirPathValue::quantity(*value, unit.clone()),
        };

        Ok(value)
    }

    /// Evaluate an identifier expression
    ///
    /// For identifiers, we check the current context to see if we can resolve
    /// the identifier to a property access on the current context.
    fn evaluate_identifier(
        &self,
        identifier: &IdentifierNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let name = &identifier.name;

        // Special case: empty context
        if context.start_context.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // For identifiers, we perform property access on the current start context
        // This handles cases like "Patient" when the context is a Patient resource
        let mut results = Vec::new();

        for value in context.start_context.iter() {
            match self.resolve_identifier_in_value(value, name)? {
                FhirPathValue::Empty => {} // Skip empty results
                FhirPathValue::Collection(vec) => {
                    results.extend(vec);
                }
                single_value => {
                    results.push(single_value);
                }
            }
        }

        Ok(match results.len() {
            0 => FhirPathValue::Empty,
            1 => results.into_iter().next().unwrap(),
            _ => FhirPathValue::Collection(results),
        })
    }

    /// Resolve an identifier within a specific value
    fn resolve_identifier_in_value(
        &self,
        value: &FhirPathValue,
        identifier: &str,
    ) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Resource(json) => self.resolve_identifier_in_json(json, identifier),
            FhirPathValue::JsonValue(json) => self.resolve_identifier_in_json(json, identifier),
            // For other value types, check if the identifier matches the type name
            _ => {
                if identifier == value.type_name() {
                    Ok(value.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    /// Resolve an identifier within JSON data
    fn resolve_identifier_in_json(
        &self,
        json: &JsonValue,
        identifier: &str,
    ) -> Result<FhirPathValue> {
        match json {
            JsonValue::Object(obj) => {
                // Check for direct property match
                if let Some(property_value) = obj.get(identifier) {
                    return Ok(self.json_to_fhir_path_value(property_value.clone()));
                }

                // Check for resourceType match (special FHIR case)
                if let Some(resource_type) = obj.get("resourceType") {
                    if let Some(type_str) = resource_type.as_str() {
                        if identifier == type_str {
                            // Return the entire resource when resourceType matches
                            return Ok(FhirPathValue::Resource(Arc::new(json.clone())));
                        }
                    }
                }

                // No match found
                Ok(FhirPathValue::Empty)
            }
            JsonValue::Array(arr) => {
                // For arrays, try to resolve the identifier in each element
                let mut results = Vec::new();
                for item in arr {
                    match self.resolve_identifier_in_json(item, identifier)? {
                        FhirPathValue::Empty => {} // Skip empty results
                        FhirPathValue::Collection(vec) => {
                            results.extend(vec);
                        }
                        single_value => {
                            results.push(single_value);
                        }
                    }
                }

                Ok(match results.len() {
                    0 => FhirPathValue::Empty,
                    1 => results.into_iter().next().unwrap(),
                    _ => FhirPathValue::Collection(results),
                })
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Convert JSON value to FhirPathValue
    fn json_to_fhir_path_value(&self, json: JsonValue) -> FhirPathValue {
        match json {
            JsonValue::Null => FhirPathValue::Empty,
            JsonValue::Bool(b) => FhirPathValue::Boolean(b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    match rust_decimal::Decimal::from_f64_retain(f) {
                        Some(d) => FhirPathValue::Decimal(d),
                        None => FhirPathValue::JsonValue(Arc::new(JsonValue::Number(n))),
                    }
                } else {
                    FhirPathValue::JsonValue(Arc::new(JsonValue::Number(n)))
                }
            }
            JsonValue::String(s) => {
                // For now, just return as string - temporal parsing will be added later
                FhirPathValue::String(s.clone())
            }
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    FhirPathValue::Empty
                } else {
                    let values: Vec<FhirPathValue> = arr
                        .into_iter()
                        .map(|v| self.json_to_fhir_path_value(v))
                        .collect();

                    if values.len() == 1 {
                        values.into_iter().next().unwrap()
                    } else {
                        FhirPathValue::Collection(values)
                    }
                }
            }
            JsonValue::Object(_) => FhirPathValue::Resource(Arc::new(json)),
        }
    }

    /// Evaluate a variable expression
    fn evaluate_variable(
        &self,
        variable: &VariableNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        let name = &variable.name;

        // Check for user-defined variables first
        if let Some(value) = context.get_variable(name) {
            return Ok(value.clone());
        }

        // Check for built-in variables (with % prefix)
        let prefixed_name = format!("%{}", name);
        if let Some(value) = context.get_variable(&prefixed_name) {
            return Ok(value.clone());
        }

        // Handle special built-in variables explicitly
        match name.as_str() {
            "this" => {
                // $this refers to the current context item
                if context.start_context.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if context.start_context.len() == 1 {
                    Ok(context.start_context.first().unwrap().clone())
                } else {
                    // Multiple items - return as collection
                    let values: Vec<FhirPathValue> =
                        context.start_context.iter().cloned().collect();
                    Ok(FhirPathValue::Collection(values))
                }
            }
            "index" => {
                // $index is used in lambda contexts - for now return 0
                // This will be properly handled by the lambda evaluator
                Ok(FhirPathValue::Integer(0))
            }
            "total" => {
                // $total is used in aggregate contexts - return empty for now
                // This will be properly handled by aggregate functions
                Ok(FhirPathValue::Empty)
            }
            _ => {
                // Unknown variable
                Err(FhirPathError::evaluation_error(
                    FP0055,
                    format!("Unknown variable: ${}", name),
                ))
            }
        }
    }

    /// Bridge method to evaluate with metadata awareness
    pub async fn evaluate_with_metadata_bridge(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Use the new metadata-aware evaluator
        let mut metadata_evaluator = MetadataCoreEvaluator::new();
        metadata_evaluator
            .evaluate_with_metadata(expr, context, resolver)
            .await
    }

    /// Convert evaluation result to wrapped collection
    pub async fn wrap_evaluation_result(
        &self,
        result: FhirPathValue,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match result {
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            FhirPathValue::Collection(values) => {
                let mut wrapped_values = Vec::new();
                for (i, value) in values.into_iter().enumerate() {
                    let wrapped = self.infer_wrapped_value(value, i, resolver).await?;
                    wrapped_values.push(wrapped);
                }
                Ok(wrapped_values)
            }
            single_value => {
                let wrapped = self.infer_wrapped_value(single_value, 0, resolver).await?;
                Ok(collection_utils::single(wrapped))
            }
        }
    }

    /// Infer wrapped value from plain FhirPathValue
    async fn infer_wrapped_value(
        &self,
        value: FhirPathValue,
        index: usize,
        _resolver: &TypeResolver,
    ) -> Result<WrappedValue> {
        let fhir_type = type_utils::fhirpath_value_to_fhir_type(&value);
        let path = if index > 0 {
            CanonicalPath::parse(&format!("[{}]", index)).unwrap()
        } else {
            CanonicalPath::empty()
        };

        let metadata = ValueMetadata {
            fhir_type,
            resource_type: None,
            path,
            index: if index > 0 { Some(index) } else { None },
        };

        Ok(WrappedValue::new(value, metadata))
    }
}

impl Default for CoreEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExpressionEvaluator for CoreEvaluator {
    async fn evaluate(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match expr {
            ExpressionNode::Literal(literal) => self.evaluate_literal(literal),
            ExpressionNode::Identifier(identifier) => self.evaluate_identifier(identifier, context),
            ExpressionNode::Variable(variable) => self.evaluate_variable(variable, context),
            ExpressionNode::Parenthesized(inner) => {
                // For parenthesized expressions, just evaluate the inner expression
                Box::pin(self.evaluate(inner, context)).await
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0051,
                format!(
                    "CoreEvaluator cannot handle expression type: {}",
                    expr.node_type()
                ),
            )),
        }
    }

    fn can_evaluate(&self, expr: &ExpressionNode) -> bool {
        matches!(
            expr,
            ExpressionNode::Literal(_)
                | ExpressionNode::Identifier(_)
                | ExpressionNode::Variable(_)
                | ExpressionNode::Parenthesized(_)
        )
    }

    fn evaluator_name(&self) -> &'static str {
        "CoreEvaluator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::literal::LiteralValue,
        core::{Collection, temporal::*},
        evaluator::EvaluationContext,
    };
    use rust_decimal_macros::dec;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_context() -> EvaluationContext {
        let patient = Collection::single(FhirPathValue::resource(json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [{
                "family": "Smith",
                "given": ["John", "Q"]
            }],
            "age": 25
        })));

        EvaluationContext::new(patient)
    }

    #[tokio::test]
    async fn test_literal_evaluation() {
        let mut evaluator = CoreEvaluator::new();
        let context = create_test_context();

        // String literal
        let string_literal = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String("test".to_string()),
            location: None,
        });
        let result = evaluator.evaluate(&string_literal, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("test".to_string()));

        // Integer literal
        let int_literal = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Integer(42),
            location: None,
        });
        let result = evaluator.evaluate(&int_literal, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Boolean literal
        let bool_literal = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Boolean(true),
            location: None,
        });
        let result = evaluator.evaluate(&bool_literal, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Decimal literal
        let decimal_literal = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::Decimal(dec!(3.14)),
            location: None,
        });
        let result = evaluator
            .evaluate(&decimal_literal, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(dec!(3.14)));
    }

    #[tokio::test]
    async fn test_identifier_evaluation() {
        let mut evaluator = CoreEvaluator::new();
        let context = create_test_context();

        // Resource type identifier
        let patient_identifier = ExpressionNode::Identifier(IdentifierNode {
            name: "Patient".to_string(),
            location: None,
        });
        let result = evaluator
            .evaluate(&patient_identifier, &context)
            .await
            .unwrap();

        // Should return the Patient resource since resourceType matches
        match result {
            FhirPathValue::Resource(json) => {
                assert_eq!(
                    json.get("resourceType").unwrap().as_str().unwrap(),
                    "Patient"
                );
            }
            _ => panic!("Expected Resource value"),
        }

        // Property identifier
        let id_identifier = ExpressionNode::Identifier(IdentifierNode {
            name: "id".to_string(),
            location: None,
        });
        let result = evaluator.evaluate(&id_identifier, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("example".to_string()));

        // Non-existent property
        let missing_identifier = ExpressionNode::Identifier(IdentifierNode {
            name: "nonexistent".to_string(),
            location: None,
        });
        let result = evaluator
            .evaluate(&missing_identifier, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_variable_evaluation() {
        let mut evaluator = CoreEvaluator::new();
        let mut context = create_test_context();

        // Add a custom variable
        context.set_variable(
            "myVar".to_string(),
            FhirPathValue::String("custom".to_string()),
        );

        // Test custom variable
        let custom_var = ExpressionNode::Variable(VariableNode {
            name: "myVar".to_string(),
            location: None,
        });
        let result = evaluator.evaluate(&custom_var, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("custom".to_string()));

        // Test $this variable
        let this_var = ExpressionNode::Variable(VariableNode {
            name: "this".to_string(),
            location: None,
        });
        let result = evaluator.evaluate(&this_var, &context).await.unwrap();

        // Should return the current context (Patient resource)
        match result {
            FhirPathValue::Resource(json) => {
                assert_eq!(
                    json.get("resourceType").unwrap().as_str().unwrap(),
                    "Patient"
                );
            }
            _ => panic!("Expected Resource value for $this"),
        }

        // Test unknown variable
        let unknown_var = ExpressionNode::Variable(VariableNode {
            name: "unknown".to_string(),
            location: None,
        });
        let result = evaluator.evaluate(&unknown_var, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parenthesized_evaluation() {
        let mut evaluator = CoreEvaluator::new();
        let context = create_test_context();

        // Parenthesized literal
        let inner = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String("test".to_string()),
            location: None,
        });
        let parenthesized = ExpressionNode::Parenthesized(Box::new(inner));

        let result = evaluator.evaluate(&parenthesized, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("test".to_string()));
    }

    #[tokio::test]
    async fn test_empty_context() {
        let mut evaluator = CoreEvaluator::new();
        let context = EvaluationContext::new(Collection::empty());

        // Identifier in empty context should return empty
        let identifier = ExpressionNode::Identifier(IdentifierNode {
            name: "anything".to_string(),
            location: None,
        });
        let result = evaluator.evaluate(&identifier, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // $this in empty context should return empty
        let this_var = ExpressionNode::Variable(VariableNode {
            name: "this".to_string(),
            location: None,
        });
        let result = evaluator.evaluate(&this_var, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_array_property_resolution() {
        let mut evaluator = CoreEvaluator::new();

        // Create context with array property
        let patient = Collection::single(FhirPathValue::resource(json!({
            "resourceType": "Patient",
            "name": [
                {"family": "Smith", "given": ["John"]},
                {"family": "Jones", "given": ["Jane"]}
            ]
        })));
        let context = EvaluationContext::new(patient);

        // Access array property
        let name_identifier = ExpressionNode::Identifier(IdentifierNode {
            name: "name".to_string(),
            location: None,
        });
        let result = evaluator
            .evaluate(&name_identifier, &context)
            .await
            .unwrap();

        // Should return collection of name objects
        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 2);
                // Both should be Resource values (JSON objects)
                for value in values {
                    match value {
                        FhirPathValue::Resource(json) => {
                            assert!(json.get("family").is_some());
                            assert!(json.get("given").is_some());
                        }
                        _ => panic!("Expected Resource value in collection"),
                    }
                }
            }
            _ => panic!("Expected Collection for array property"),
        }
    }
}
