//! Metadata-aware core evaluator for basic FHIRPath expressions
//!
//! This module provides evaluation for identifiers, literals, variables, and other
//! core expressions while maintaining rich metadata throughout the process.

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    ast::{ExpressionNode, IdentifierNode, LiteralNode, LiteralValue, VariableNode},
    core::{Collection, FhirPathError, FhirPathValue, Result},
    evaluator::{EvaluationContext, traits::MetadataAwareEvaluator},
    path::CanonicalPath,
    typing::{TypeResolver, type_utils},
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue, collection_utils},
};

/// Metadata-aware core evaluator for basic expressions
#[derive(Debug, Clone)]
pub struct MetadataCoreEvaluator;

impl MetadataCoreEvaluator {
    /// Create a new metadata-aware core evaluator
    pub fn new() -> Self {
        Self
    }

    /// Evaluate an identifier (resource type or property reference)
    async fn evaluate_identifier(
        &self,
        identifier: &IdentifierNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let name = &identifier.name;

        // Check if this identifier is a resource type filter
        if let Some(first_char) = name.chars().next() {
            if first_char.is_uppercase() {
                return self
                    .evaluate_resource_type_filter(name, context, resolver)
                    .await;
            }
        }

        // For lowercase identifiers, they are property accesses on the current context
        // We need to navigate the property on each item in the start context
        let start_context = self.get_start_context_as_wrapped(context, resolver).await?;
        let mut result = Vec::new();

        for wrapped_value in start_context {
            let property_results = self
                .navigate_property_on_value(&wrapped_value, name, resolver)
                .await?;
            result.extend(property_results);
        }

        Ok(result)
    }

    /// Navigate a property on a single wrapped value (similar to MetadataNavigator but internal)
    async fn navigate_property_on_value(
        &self,
        source: &WrappedValue,
        property: &str,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match source.as_plain() {
            FhirPathValue::JsonValue(json) | FhirPathValue::Resource(json) => {
                self.extract_property_from_json(json, property, &source.metadata, resolver)
                    .await
            }
            FhirPathValue::Collection(values) => {
                // Navigate property on each element in collection
                let mut result = Vec::new();

                for (i, value) in values.iter().enumerate() {
                    // Create temporary wrapped value for each collection element
                    let element_metadata = source.metadata.derive_index(i, None);
                    let wrapped_element = WrappedValue::new(value.clone(), element_metadata);

                    // Navigate property on this element (box the recursive call to avoid infinite future size)
                    let property_results = Box::pin(self.navigate_property_on_value(
                        &wrapped_element,
                        property,
                        resolver,
                    ))
                    .await?;

                    result.extend(property_results);
                }

                Ok(result)
            }
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            _ => {
                // Cannot navigate property on primitive values
                // For invalid property access, we need to check if this should error or return empty
                // Use navigate_typed_path to check if property exists on this type
                let property_check = resolver
                    .model_provider()
                    .navigate_typed_path(&source.metadata.fhir_type, property)
                    .await;

                if property_check.is_ok() {
                    // Property exists but can't be accessed on primitive - empty result
                    Ok(collection_utils::empty())
                } else {
                    // Property doesn't exist - this should be an error
                    Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0052,
                        format!(
                            "Invalid property access: property '{}' does not exist on type '{}' at path '{}'",
                            property, source.metadata.fhir_type, source.metadata.path
                        ),
                    ))
                }
            }
        }
    }

    /// Extract property value from JSON with metadata awareness (duplicated from MetadataNavigator for internal use)
    async fn extract_property_from_json(
        &self,
        json: &serde_json::Value,
        property: &str,
        source_metadata: &ValueMetadata,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match json.get(property) {
            Some(property_value) => {
                // Resolve the property type
                let property_type = resolver
                    .resolve_property_type(&source_metadata.fhir_type, property)
                    .await
                    .unwrap_or_else(|_| "unknown".to_string());

                // Create new path for the property
                let property_path = source_metadata.path.append_property(property);

                // Convert JSON value to FhirPathValue and wrap with metadata
                Ok(self.json_to_wrapped_collection(property_value, property_path, property_type))
            }
            None => {
                // Property not found - check if property should exist on this type
                let property_check = resolver
                    .model_provider()
                    .navigate_typed_path(&source_metadata.fhir_type, property)
                    .await;

                if property_check.is_ok() {
                    // Property should exist but doesn't in this instance - return empty
                    Ok(collection_utils::empty())
                } else {
                    // Property doesn't exist on this type - this should be an error
                    Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0052,
                        format!(
                            "Invalid property access: property '{}' does not exist on type '{}' at path '{}'",
                            property, source_metadata.fhir_type, source_metadata.path
                        ),
                    ))
                }
            }
        }
    }

    /// Convert JSON value to wrapped collection with metadata (duplicated from MetadataNavigator for internal use)
    fn json_to_wrapped_collection(
        &self,
        json: &serde_json::Value,
        path: CanonicalPath,
        fhir_type: String,
    ) -> WrappedCollection {
        match json {
            serde_json::Value::Array(array) => {
                // Array property - create indexed wrapped values
                array
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let indexed_path = path.append_index(i);
                        let fhir_path_value = self.json_to_fhir_path_value(item);
                        let metadata = ValueMetadata {
                            fhir_type: fhir_type.clone(),
                            resource_type: None,
                            path: indexed_path,
                            index: Some(i),
                        };
                        WrappedValue::new(fhir_path_value, metadata)
                    })
                    .collect()
            }
            _ => {
                // Single value
                let fhir_path_value = self.json_to_fhir_path_value(json);
                let metadata = ValueMetadata {
                    fhir_type,
                    resource_type: None,
                    path,
                    index: None,
                };
                collection_utils::single(WrappedValue::new(fhir_path_value, metadata))
            }
        }
    }

    /// Convert JSON value to FhirPathValue (duplicated from MetadataNavigator for internal use)
    fn json_to_fhir_path_value(&self, json: &serde_json::Value) -> FhirPathValue {
        match json {
            serde_json::Value::Null => FhirPathValue::Empty,
            serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64_retain(f)
                            .unwrap_or_else(|| rust_decimal::Decimal::new(0, 0)),
                    )
                } else {
                    FhirPathValue::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => FhirPathValue::String(s.clone()),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                // Complex values remain as JSON for now
                FhirPathValue::JsonValue(Arc::new(json.clone()))
            }
        }
    }

    /// Evaluate a resource type filter (e.g., "Patient", "Observation")
    async fn evaluate_resource_type_filter(
        &self,
        resource_type: &str,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // First, validate that the resource type exists in the model provider
        if !resolver
            .model_provider()
            .resource_type_exists(resource_type)
            .unwrap_or(false)
        {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0052,
                format!("Unknown resource type: '{}'", resource_type),
            ));
        }

        let start_context = self.get_start_context_as_wrapped(context, resolver).await?;

        // Filter resources that match the specified type
        let mut filtered_results = Vec::new();
        let mut found_any_resource = false;
        let mut actual_resource_type: Option<String> = None;

        for wrapped in start_context {
            if let Some(current_resource_type) = &wrapped.metadata.resource_type {
                found_any_resource = true;
                actual_resource_type = Some(current_resource_type.clone());

                if current_resource_type == resource_type {
                    filtered_results.push(wrapped);
                }
                // If resource types don't match, continue without error (empty result)
            } else {
                // Not a resource - check if the path suggests this could match
                if wrapped.metadata.fhir_type == resource_type {
                    filtered_results.push(wrapped);
                }
            }
        }

        // If we found resources but none matched the requested type,
        // and this is a semantic error according to the test expectation
        if found_any_resource && filtered_results.is_empty() {
            if let Some(actual_type) = actual_resource_type {
                if actual_type != resource_type {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0052,
                        format!(
                            "Resource type mismatch: expression expects '{}' but input data has '{}'",
                            resource_type, actual_type
                        ),
                    ));
                }
            }
        }

        Ok(filtered_results)
    }

    /// Evaluate a literal value
    fn evaluate_literal(
        &self,
        literal: &LiteralNode,
        _context: &EvaluationContext,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let fhir_path_value = self.literal_node_to_fhir_path_value(literal);
        let fhir_type = type_utils::fhirpath_value_to_fhir_type(&fhir_path_value);

        // Literals don't have meaningful paths - use empty path
        let path = CanonicalPath::empty();
        let metadata = ValueMetadata {
            fhir_type,
            resource_type: None,
            path,
            index: None,
        };

        let wrapped = WrappedValue::new(fhir_path_value, metadata);
        Ok(collection_utils::single(wrapped))
    }

    /// Evaluate a variable reference
    fn evaluate_variable(
        &self,
        variable: &VariableNode,
        context: &EvaluationContext,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let var_name = &variable.name;

        // Special handling for $this - it should refer to the current context with metadata
        if var_name == "this" {
            // Check if we have stored metadata for $this (in lambda contexts)
            if let Some(metadata_var) = context.get_variable("__$this_metadata__") {
                if let FhirPathValue::JsonValue(metadata_json) = metadata_var {
                    if context.start_context.len() == 1 {
                        let value = context.start_context.first().unwrap();

                        // Extract metadata from the stored JSON
                        let fhir_type = metadata_json
                            .get("fhir_type")
                            .and_then(|ft| ft.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| type_utils::fhirpath_value_to_fhir_type(value));

                        let resource_type = metadata_json
                            .get("resource_type")
                            .and_then(|rt| rt.as_str())
                            .map(|s| s.to_string());

                        let path = metadata_json
                            .get("path")
                            .and_then(|p| p.as_str())
                            .and_then(|s| CanonicalPath::parse(s).ok())
                            .unwrap_or_else(|| {
                                CanonicalPath::parse("$this")
                                    .unwrap_or_else(|_| CanonicalPath::empty())
                            });

                        let metadata = ValueMetadata {
                            fhir_type,
                            resource_type,
                            path,
                            index: None,
                        };

                        let wrapped = WrappedValue::new(value.clone(), metadata);
                        return Ok(collection_utils::single(wrapped));
                    }
                }
            }

            // Fallback: Create wrapped collection with basic metadata inference for $this
            if context.start_context.is_empty() {
                return Ok(collection_utils::empty());
            } else if context.start_context.len() == 1 {
                let value = context.start_context.first().unwrap();
                let fhir_type = type_utils::fhirpath_value_to_fhir_type(value);
                let path = CanonicalPath::parse("$this").unwrap_or_else(|_| CanonicalPath::empty());

                let resource_type = if let FhirPathValue::JsonValue(json) = value {
                    json.get("resourceType")
                        .and_then(|rt| rt.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                };

                let metadata = ValueMetadata {
                    fhir_type,
                    resource_type,
                    path,
                    index: None,
                };

                let wrapped = WrappedValue::new(value.clone(), metadata);
                return Ok(collection_utils::single(wrapped));
            } else {
                // Multiple items in context - wrap each with metadata
                let mut wrapped_values = Vec::new();
                for (i, value) in context.start_context.iter().enumerate() {
                    let fhir_type = type_utils::fhirpath_value_to_fhir_type(value);
                    let path = CanonicalPath::parse(&format!("$this[{}]", i))
                        .unwrap_or_else(|_| CanonicalPath::empty());

                    let resource_type = if let FhirPathValue::JsonValue(json) = value {
                        json.get("resourceType")
                            .and_then(|rt| rt.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    };

                    let metadata = ValueMetadata {
                        fhir_type,
                        resource_type,
                        path,
                        index: Some(i),
                    };

                    let wrapped = WrappedValue::new(value.clone(), metadata);
                    wrapped_values.push(wrapped);
                }
                return Ok(wrapped_values);
            }
        }

        if let Some(var_value) = context.get_variable(var_name) {
            // Infer metadata from variable value
            let fhir_type = type_utils::fhirpath_value_to_fhir_type(var_value);
            let path = CanonicalPath::parse(&format!("${}", var_name))
                .unwrap_or_else(|_| CanonicalPath::empty());

            let metadata = ValueMetadata {
                fhir_type,
                resource_type: None,
                path,
                index: None,
            };

            let wrapped = WrappedValue::new(var_value.clone(), metadata);
            Ok(collection_utils::single(wrapped))
        } else {
            // Variable not found - return empty
            Ok(collection_utils::empty())
        }
    }

    /// Get the start context as wrapped collection with proper metadata
    async fn get_start_context_as_wrapped(
        &self,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let start_collection = &context.start_context;
        self.collection_to_wrapped(start_collection, resolver).await
    }

    /// Convert a Collection to WrappedCollection with metadata inference
    async fn collection_to_wrapped(
        &self,
        collection: &Collection,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let mut wrapped_values = Vec::new();

        for (i, value) in collection.iter().enumerate() {
            let wrapped = self
                .value_to_wrapped_with_inference(value, i, resolver)
                .await?;
            wrapped_values.push(wrapped);
        }

        Ok(wrapped_values)
    }

    /// Convert a FhirPathValue to WrappedValue with metadata inference
    async fn value_to_wrapped_with_inference(
        &self,
        value: &FhirPathValue,
        index: usize,
        _resolver: &TypeResolver,
    ) -> Result<WrappedValue> {
        match value {
            FhirPathValue::Resource(json) | FhirPathValue::JsonValue(json) => {
                // Try to detect resource type from JSON
                if let Some(resource_type) = json.get("resourceType").and_then(|rt| rt.as_str()) {
                    // This is a FHIR resource
                    let path = CanonicalPath::root(resource_type);
                    let metadata = ValueMetadata {
                        fhir_type: resource_type.to_string(),
                        resource_type: Some(resource_type.to_string()),
                        path,
                        index: if index > 0 { Some(index) } else { None },
                    };

                    Ok(WrappedValue::new(value.clone(), metadata))
                } else {
                    // Generic JSON object
                    let path = if index > 0 {
                        CanonicalPath::parse(&format!("[{}]", index)).unwrap()
                    } else {
                        CanonicalPath::empty()
                    };

                    let metadata = ValueMetadata {
                        fhir_type: "unknown".to_string(),
                        resource_type: None,
                        path,
                        index: if index > 0 { Some(index) } else { None },
                    };

                    Ok(WrappedValue::new(value.clone(), metadata))
                }
            }
            _ => {
                // Primitive or other value type
                let fhir_type = type_utils::fhirpath_value_to_fhir_type(value);
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

                Ok(WrappedValue::new(value.clone(), metadata))
            }
        }
    }

    /// Convert literal AST node to FhirPathValue
    fn literal_node_to_fhir_path_value(&self, literal: &LiteralNode) -> FhirPathValue {
        match &literal.value {
            LiteralValue::Boolean(b) => FhirPathValue::Boolean(*b),
            LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
            LiteralValue::Decimal(d) => FhirPathValue::Decimal(*d),
            LiteralValue::String(s) => FhirPathValue::String(s.clone()),
            LiteralValue::Date(d) => FhirPathValue::Date(d.clone()),
            LiteralValue::DateTime(dt) => FhirPathValue::DateTime(dt.clone()),
            LiteralValue::Time(t) => FhirPathValue::Time(t.clone()),
            LiteralValue::Quantity { value, unit } => FhirPathValue::quantity(*value, unit.clone()),
        }
    }
}

#[async_trait]
impl MetadataAwareEvaluator for MetadataCoreEvaluator {
    async fn evaluate_with_metadata(
        &mut self,
        expr: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match expr {
            ExpressionNode::Identifier(identifier) => {
                self.evaluate_identifier(identifier, context, resolver)
                    .await
            }
            ExpressionNode::Literal(literal) => self.evaluate_literal(literal, context, resolver),
            ExpressionNode::Variable(variable) => {
                self.evaluate_variable(variable, context, resolver)
            }
            ExpressionNode::Parenthesized(inner) => {
                // Evaluate the inner expression
                self.evaluate_with_metadata(inner, context, resolver).await
            }
            _ => {
                // Other expression types should be handled by specialized evaluators
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    format!("Unsupported expression type in core evaluator: {:?}", expr),
                ))
            }
        }
    }

    async fn initialize_root_context(
        &self,
        root_data: &Collection,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        self.collection_to_wrapped(root_data, resolver).await
    }
}

impl Default for MetadataCoreEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{IdentifierNode, LiteralNode, LiteralValue, VariableNode};
    use crate::core::{Collection, FhirPathValue};
    use crate::typing::TypeResolver;
    use octofhir_fhir_model::EmptyModelProvider;
    use serde_json::json;
    use std::sync::Arc;

    fn create_test_resolver() -> TypeResolver {
        let provider = Arc::new(EmptyModelProvider);
        TypeResolver::new(provider)
    }

    fn create_test_context_with_patient() -> EvaluationContext {
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [{"given": ["John"], "family": "Doe"}]
        });

        let patient_value = FhirPathValue::Resource(patient_json);
        let collection = Collection::single(patient_value);
        EvaluationContext::new(collection)
    }

    #[tokio::test]
    async fn test_resource_type_identifier() {
        let mut evaluator = MetadataCoreEvaluator::new();
        let resolver = create_test_resolver();
        let context = create_test_context_with_patient();

        let patient_identifier = ExpressionNode::Identifier(IdentifierNode {
            name: "Patient".to_string(),
        });

        let result = evaluator
            .evaluate_with_metadata(&patient_identifier, &context, &resolver)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        let wrapped = &result[0];
        assert_eq!(wrapped.metadata.fhir_type, "Patient");
        assert_eq!(wrapped.metadata.resource_type, Some("Patient".to_string()));
        assert_eq!(wrapped.metadata.path.to_string(), "Patient");
    }

    #[tokio::test]
    async fn test_literal_evaluation() {
        let mut evaluator = MetadataCoreEvaluator::new();
        let resolver = create_test_resolver();
        let context = create_test_context_with_patient();

        let string_literal = ExpressionNode::Literal(LiteralNode {
            value: LiteralValue::String("test string".to_string()),
        });

        let result = evaluator
            .evaluate_with_metadata(&string_literal, &context, &resolver)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        let wrapped = &result[0];
        assert_eq!(wrapped.metadata.fhir_type, "string");
        assert!(wrapped.metadata.resource_type.is_none());

        match wrapped.as_plain() {
            FhirPathValue::String(s) => assert_eq!(s, "test string"),
            _ => panic!("Expected string value"),
        }
    }

    #[tokio::test]
    async fn test_variable_evaluation() {
        let mut evaluator = MetadataCoreEvaluator::new();
        let resolver = create_test_resolver();
        let mut context = create_test_context_with_patient();

        // Add a variable to the context
        context.set_variable("myVar".to_string(), FhirPathValue::Integer(42));

        let variable = ExpressionNode::Variable(VariableNode {
            name: "myVar".to_string(),
        });

        let result = evaluator
            .evaluate_with_metadata(&variable, &context, &resolver)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        let wrapped = &result[0];
        assert_eq!(wrapped.metadata.fhir_type, "integer");
        assert_eq!(wrapped.metadata.path.to_string(), "$myVar");

        match wrapped.as_plain() {
            FhirPathValue::Integer(i) => assert_eq!(*i, 42),
            _ => panic!("Expected integer value"),
        }
    }

    #[tokio::test]
    async fn test_root_context_initialization() {
        let evaluator = MetadataCoreEvaluator::new();
        let resolver = create_test_resolver();

        let patient_json = json!({
            "resourceType": "Patient",
            "id": "example"
        });

        let collection = Collection::single(FhirPathValue::Resource(patient_json));

        let wrapped_context = evaluator
            .initialize_root_context(&collection, &resolver)
            .await
            .unwrap();

        assert_eq!(wrapped_context.len(), 1);
        let wrapped = &wrapped_context[0];
        assert_eq!(wrapped.metadata.fhir_type, "Patient");
        assert_eq!(wrapped.metadata.resource_type, Some("Patient".to_string()));
        assert_eq!(wrapped.metadata.path.to_string(), "Patient");
    }

    #[tokio::test]
    async fn test_non_resource_data() {
        let evaluator = MetadataCoreEvaluator::new();
        let resolver = create_test_resolver();

        let primitive_value = FhirPathValue::String("test".to_string());
        let collection = Collection::single(primitive_value);

        let wrapped_context = evaluator
            .initialize_root_context(&collection, &resolver)
            .await
            .unwrap();

        assert_eq!(wrapped_context.len(), 1);
        let wrapped = &wrapped_context[0];
        assert_eq!(wrapped.metadata.fhir_type, "string");
        assert!(wrapped.metadata.resource_type.is_none());
    }
}
