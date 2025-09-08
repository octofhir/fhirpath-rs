//! Metadata-aware navigator implementation for FHIRPath evaluation
//!
//! This module provides navigation capabilities that maintain rich metadata
//! throughout property access and indexing operations.

use async_trait::async_trait;
use serde_json::Value as JsonValue;

use crate::{
    core::{FhirPathError, FhirPathValue, Result},
    evaluator::traits::MetadataAwareNavigator,
    path::CanonicalPath,
    typing::TypeResolver,
    wrapped::{WrappedValue, WrappedCollection, ValueMetadata, collection_utils},
};

/// Metadata-aware navigator that maintains metadata during navigation
#[derive(Debug, Clone)]
pub struct MetadataNavigator;

impl MetadataNavigator {
    /// Create a new metadata-aware navigator
    pub fn new() -> Self {
        Self
    }
    
    /// Extract property value from JSON with metadata awareness
    async fn extract_property_from_json(
        &self,
        json: &JsonValue,
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
                let property_check = resolver.model_provider()
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
                            property,
                            source_metadata.fhir_type,
                            source_metadata.path
                        ),
                    ))
                }
            }
        }
    }
    
    /// Convert JSON value to wrapped collection with metadata
    fn json_to_wrapped_collection(
        &self,
        json: &JsonValue,
        path: CanonicalPath,
        fhir_type: String,
    ) -> WrappedCollection {
        match json {
            JsonValue::Array(array) => {
                // Array property - create indexed wrapped values
                array.iter()
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
    
    /// Convert JSON value to FhirPathValue
    fn json_to_fhir_path_value(&self, json: &JsonValue) -> FhirPathValue {
        match json {
            JsonValue::Null => FhirPathValue::Empty,
            JsonValue::Bool(b) => FhirPathValue::Boolean(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    FhirPathValue::Decimal(rust_decimal::Decimal::from_f64_retain(f)
                        .unwrap_or_else(|| rust_decimal::Decimal::new(0, 0)))
                } else {
                    FhirPathValue::String(n.to_string())
                }
            }
            JsonValue::String(s) => FhirPathValue::String(s.clone()),
            JsonValue::Array(_) | JsonValue::Object(_) => {
                // Complex values remain as JSON for now
                FhirPathValue::JsonValue(json.clone())
            }
        }
    }
    
    /// Extract indexed element from collection with metadata
    async fn extract_indexed_element(
        &self,
        source: &WrappedValue,
        index: usize,
        resolver: &TypeResolver,
    ) -> Result<Option<WrappedValue>> {
        match source.as_plain() {
            FhirPathValue::Collection(values) => {
                if let Some(value) = values.get(index) {
                    // Resolve element type
                    let element_type = resolver
                        .resolve_element_type(&source.metadata.fhir_type)
                        .await
                        .unwrap_or_else(|_| "unknown".to_string());
                    
                    // Create indexed path
                    let indexed_path = source.metadata.path.append_index(index);
                    let metadata = ValueMetadata {
                        fhir_type: element_type,
                        resource_type: None,
                        path: indexed_path,
                        index: Some(index),
                    };
                    
                    Ok(Some(WrappedValue::new(value.clone(), metadata)))
                } else {
                    Ok(None) // Index out of bounds
                }
            }
            FhirPathValue::JsonValue(JsonValue::Array(array)) => {
                if let Some(item) = array.get(index) {
                    let element_type = resolver
                        .resolve_element_type(&source.metadata.fhir_type)
                        .await
                        .unwrap_or_else(|_| "unknown".to_string());
                    
                    let indexed_path = source.metadata.path.append_index(index);
                    let fhir_path_value = self.json_to_fhir_path_value(item);
                    let metadata = ValueMetadata {
                        fhir_type: element_type,
                        resource_type: None,
                        path: indexed_path,
                        index: Some(index),
                    };
                    
                    Ok(Some(WrappedValue::new(fhir_path_value, metadata)))
                } else {
                    Ok(None)
                }
            }
            _ => {
                // Single value - index 0 returns the value, others return None
                if index == 0 {
                    Ok(Some(source.clone()))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[async_trait]
impl MetadataAwareNavigator for MetadataNavigator {
    async fn navigate_property_with_metadata(
        &self,
        source: &WrappedValue,
        property: &str,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        match source.as_plain() {
            FhirPathValue::JsonValue(json) | FhirPathValue::Resource(json) => {
                self.extract_property_from_json(json, property, &source.metadata, resolver).await
            }
            FhirPathValue::Collection(values) => {
                // Navigate property on each element in collection
                let mut result = Vec::new();
                
                for (i, value) in values.iter().enumerate() {
                    // Create temporary wrapped value for each collection element
                    let element_metadata = source.metadata.derive_index(i, None);
                    let wrapped_element = WrappedValue::new(value.clone(), element_metadata);
                    
                    // Navigate property on this element
                    let property_results = self
                        .navigate_property_with_metadata(&wrapped_element, property, resolver)
                        .await?;
                    
                    result.extend(property_results);
                }
                
                Ok(result)
            }
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            _ => {
                // Cannot navigate property on primitive values
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0052,
                    format!(
                        "Cannot access property '{}' on primitive type '{}' at path '{}'",
                        property,
                        source.metadata.fhir_type,
                        source.metadata.path
                    ),
                ))
            }
        }
    }

    async fn navigate_index_with_metadata(
        &self,
        source: &WrappedValue,
        index: usize,
        resolver: &TypeResolver,
    ) -> Result<Option<WrappedValue>> {
        self.extract_indexed_element(source, index, resolver).await
    }

    async fn navigate_path_with_metadata(
        &self,
        source: &WrappedValue,
        path_segments: &[&str],
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        if path_segments.is_empty() {
            return Ok(collection_utils::single(source.clone()));
        }
        
        // Navigate first segment
        let first_segment = path_segments[0];
        let intermediate_results = self
            .navigate_property_with_metadata(source, first_segment, resolver)
            .await?;
        
        // If there are more segments, continue navigation
        if path_segments.len() == 1 {
            Ok(intermediate_results)
        } else {
            let remaining_segments = &path_segments[1..];
            let mut final_results = Vec::new();
            
            for intermediate in intermediate_results {
                let segment_results = self
                    .navigate_path_with_metadata(&intermediate, remaining_segments, resolver)
                    .await?;
                final_results.extend(segment_results);
            }
            
            Ok(final_results)
        }
    }
}

impl Default for MetadataNavigator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        path::CanonicalPath,
        typing::TypeResolver,
        wrapped::{ValueMetadata, WrappedValue},
    };
    use octofhir_fhir_model::EmptyModelProvider;
    use serde_json::json;
    use std::sync::Arc;

    fn create_test_resolver() -> TypeResolver {
        let provider = Arc::new(EmptyModelProvider);
        TypeResolver::new(provider)
    }

    #[tokio::test]
    async fn test_property_navigation_with_metadata() {
        let navigator = MetadataNavigator::new();
        let resolver = create_test_resolver();
        
        let patient_json = json!({
            "resourceType": "Patient",
            "name": [
                {"given": ["John"], "family": "Doe"}
            ]
        });
        
        let source_metadata = ValueMetadata::resource("Patient".to_string());
        let source = WrappedValue::new(
            FhirPathValue::JsonValue(patient_json),
            source_metadata,
        );
        
        let name_results = navigator
            .navigate_property_with_metadata(&source, "name", &resolver)
            .await
            .unwrap();
        
        assert_eq!(name_results.len(), 1);
        let name_value = &name_results[0];
        assert_eq!(name_value.metadata.path.to_string(), "Patient.name[0]");
    }

    #[tokio::test]
    async fn test_index_navigation_with_metadata() {
        let navigator = MetadataNavigator::new();
        let resolver = create_test_resolver();
        
        let collection_json = json!(["John", "Jane", "Bob"]);
        let source_metadata = ValueMetadata::complex(
            "Array<string>".to_string(),
            CanonicalPath::parse("Patient.name.given").unwrap(),
        );
        let source = WrappedValue::new(
            FhirPathValue::JsonValue(collection_json),
            source_metadata,
        );
        
        let indexed_result = navigator
            .navigate_index_with_metadata(&source, 1, &resolver)
            .await
            .unwrap();
        
        assert!(indexed_result.is_some());
        let indexed_value = indexed_result.unwrap();
        assert_eq!(indexed_value.metadata.path.to_string(), "Patient.name.given[1]");
        assert_eq!(indexed_value.metadata.index, Some(1));
    }

    #[tokio::test]
    async fn test_multi_step_path_navigation() {
        let navigator = MetadataNavigator::new();
        let resolver = create_test_resolver();
        
        let patient_json = json!({
            "resourceType": "Patient",
            "name": [
                {"given": ["John", "William"], "family": "Doe"}
            ]
        });
        
        let source_metadata = ValueMetadata::resource("Patient".to_string());
        let source = WrappedValue::new(
            FhirPathValue::JsonValue(patient_json),
            source_metadata,
        );
        
        let path_segments = vec!["name", "given"];
        let results = navigator
            .navigate_path_with_metadata(&source, &path_segments, &resolver)
            .await
            .unwrap();
        
        assert_eq!(results.len(), 2); // Two given names
        assert_eq!(results[0].metadata.path.to_string(), "Patient.name[0].given[0]");
        assert_eq!(results[1].metadata.path.to_string(), "Patient.name[0].given[1]");
    }

    #[tokio::test]
    async fn test_empty_navigation() {
        let navigator = MetadataNavigator::new();
        let resolver = create_test_resolver();
        
        let source_metadata = ValueMetadata::unknown(CanonicalPath::empty());
        let source = WrappedValue::new(FhirPathValue::Empty, source_metadata);
        
        let results = navigator
            .navigate_property_with_metadata(&source, "nonexistent", &resolver)
            .await
            .unwrap();
        
        assert!(results.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::FhirPathValue;
    use serde_json::json;

    #[tokio::test]
    async fn test_patient_name_navigation() {
        let navigator = MetadataNavigator::new();
        let resolver = create_test_resolver();
        
        // Create a realistic Patient resource
        let patient = json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [
                {
                    "use": "official",
                    "given": ["Peter", "James"],
                    "family": "Chalmers"
                },
                {
                    "use": "usual", 
                    "given": ["Jim"]
                }
            ]
        });
        
        let source_metadata = ValueMetadata::resource("Patient".to_string());
        let source = WrappedValue::new(
            FhirPathValue::Resource(patient),
            source_metadata,
        );
        
        // Test: Patient.name
        let names = navigator
            .navigate_property_with_metadata(&source, "name", &resolver)
            .await
            .unwrap();
        
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].path_string(), "Patient.name[0]");
        assert_eq!(names[1].path_string(), "Patient.name[1]");
        
        // Test: Patient.name[0].given
        let first_name = &names[0];
        let given_names = navigator
            .navigate_property_with_metadata(first_name, "given", &resolver)
            .await
            .unwrap();
        
        assert_eq!(given_names.len(), 2);
        assert_eq!(given_names[0].path_string(), "Patient.name[0].given[0]");
        assert_eq!(given_names[1].path_string(), "Patient.name[0].given[1]");
        
        // Verify the actual values
        match given_names[0].as_plain() {
            FhirPathValue::String(s) => assert_eq!(s, "Peter"),
            _ => panic!("Expected string value"),
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        let navigator = MetadataNavigator::new();
        let resolver = create_test_resolver();
        
        // Test navigation on primitive value (should fail)
        let source_metadata = ValueMetadata::primitive(
            "string".to_string(),
            CanonicalPath::parse("Patient.name.family").unwrap(),
        );
        let source = WrappedValue::new(
            FhirPathValue::String("Doe".to_string()),
            source_metadata,
        );
        
        let result = navigator
            .navigate_property_with_metadata(&source, "nonexistent", &resolver)
            .await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("primitive type"));
    }
}