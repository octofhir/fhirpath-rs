//! ModelProvider re-exports and utilities
//!
//! This module re-exports the ModelProvider trait from octofhir-fhir-model
//! and provides utility functions for working with ModelProviders.
//!
//! All concrete ModelProvider implementations are now in fhir-model-rs to maintain
//! clean dependency separation and avoid circular dependencies.

use serde_json::Value as JsonValue;

use super::error::{FhirPathError, Result};
use super::error_code::*;

// Re-export ModelProvider trait and types from octofhir-fhir-model
pub use octofhir_fhir_model::{ModelProvider, TypeInfo, NavigationResult, FhirVersion};

/// Utility functions for working with ModelProviders
pub mod utils {
    use super::*;

    /// Extract resource type from a JsonValue safely
    pub fn extract_resource_type(resource: &JsonValue) -> Option<String> {
        resource
            .get("resourceType")
            .and_then(|rt| rt.as_str())
            .map(|s| s.to_string())
    }

    /// Check if a JsonValue represents a FHIR resource
    pub fn is_fhir_resource(value: &JsonValue) -> bool {
        value.is_object() && value.get("resourceType").is_some()
    }

    /// Extract reference target from a Reference object
    pub fn extract_reference_target(reference: &JsonValue) -> Option<String> {
        reference
            .get("reference")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
    }

    /// Parse a reference string into its components (resource_type/id)
    pub fn parse_reference(reference: &str) -> Result<(String, String)> {
        if let Some(slash_pos) = reference.find('/') {
            let resource_type = reference[..slash_pos].to_string();
            let id = reference[slash_pos + 1..].to_string();
            Ok((resource_type, id))
        } else {
            Err(FhirPathError::model_error(
                FP0151,
                format!("Invalid reference format: {}", reference),
            ))
        }
    }
}

/// Simple MockModelProvider for internal FHIRPath tests
#[derive(Debug, Clone, Default)]
pub struct MockModelProvider;

#[async_trait::async_trait]
impl ModelProvider for MockModelProvider {
    async fn get_type(&self, type_name: &str) -> octofhir_fhir_model::Result<Option<TypeInfo>> {
        match type_name {
            "Patient" | "Observation" | "Bundle" => {
                Ok(Some(TypeInfo {
                    type_name: type_name.to_string(),
                    singleton: true,
                    namespace: Some("FHIR".to_string()),
                    name: Some(type_name.to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }))
            }
            "String" | "Integer" | "Boolean" | "Decimal" => {
                Ok(Some(TypeInfo {
                    type_name: type_name.to_string(),
                    singleton: true,
                    namespace: Some("System".to_string()),
                    name: Some(type_name.to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }))
            }
            _ => Ok(None),
        }
    }

    async fn get_element_type(&self, parent_type: &TypeInfo, property_name: &str) -> octofhir_fhir_model::Result<Option<TypeInfo>> {
        match (parent_type.type_name.as_str(), property_name) {
            ("Patient", "name") => Ok(Some(TypeInfo {
                type_name: "HumanName".to_string(),
                singleton: false,
                namespace: Some("FHIR".to_string()),
                name: Some("HumanName".to_string()),
                is_empty: Some(false),
                is_union_type: Some(false),
                union_choices: None,
            })),
            ("Patient", "gender") => Ok(Some(TypeInfo {
                type_name: "String".to_string(),
                singleton: true,
                namespace: Some("System".to_string()),
                name: Some("String".to_string()),
                is_empty: Some(false),
                is_union_type: Some(false),
                union_choices: None,
            })),
            _ => Ok(None),
        }
    }

    async fn get_element_names(&self, parent_type: &TypeInfo) -> octofhir_fhir_model::Result<Vec<String>> {
        match parent_type.type_name.as_str() {
            "Patient" => Ok(vec!["id".to_string(), "name".to_string()]),
            _ => Ok(Vec::new()),
        }
    }

    async fn get_children_type(&self, parent_type: &TypeInfo) -> octofhir_fhir_model::Result<Option<TypeInfo>> {
        if !parent_type.singleton {
            Ok(Some(TypeInfo {
                type_name: parent_type.type_name.clone(),
                singleton: true,
                namespace: parent_type.namespace.clone(),
                name: parent_type.name.clone(),
                is_empty: Some(false),
                is_union_type: parent_type.is_union_type,
                union_choices: parent_type.union_choices.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_resource_types(&self) -> octofhir_fhir_model::Result<Vec<String>> {
        Ok(vec!["Patient".to_string(), "Observation".to_string()])
    }

    async fn get_complex_types(&self) -> octofhir_fhir_model::Result<Vec<String>> {
        Ok(vec!["HumanName".to_string()])
    }

    async fn get_primitive_types(&self) -> octofhir_fhir_model::Result<Vec<String>> {
        Ok(vec!["String".to_string(), "Integer".to_string()])
    }

    async fn navigate_with_data(&self, _base_type: &str, path: &str, data: &JsonValue) -> octofhir_fhir_model::Result<NavigationResult> {
        if let Some(obj) = data.as_object() {
            if let Some(_value) = obj.get(path) {
                return Ok(NavigationResult {
                    success: true,
                    result_type: Some(TypeInfo {
                        type_name: "String".to_string(),
                        singleton: true,
                        namespace: Some("System".to_string()),
                        name: Some("String".to_string()),
                        is_empty: Some(false),
                        is_union_type: Some(false),
                        union_choices: None,
                    }),
                    resolved_property_name: Some(path.to_string()),
                    primitive_element: None,
                    error_message: None,
                });
            }
        }
        Ok(NavigationResult {
            success: false,
            result_type: None,
            resolved_property_name: None,
            primitive_element: None,
            error_message: Some(format!("Property '{}' not found", path)),
        })
    }

    async fn get_fhir_version(&self) -> octofhir_fhir_model::Result<FhirVersion> {
        Ok(FhirVersion::R4)
    }

    async fn resource_type_exists(&self, resource_type: &str) -> octofhir_fhir_model::Result<bool> {
        Ok(matches!(resource_type, "Patient" | "Observation"))
    }

    async fn refresh_resource_types(&self) -> octofhir_fhir_model::Result<()> {
        Ok(())
    }

    async fn is_mixed_collection(&self, parent_type: &str, property_name: &str) -> octofhir_fhir_model::Result<bool> {
        // Simple mock implementation - identify known mixed collection patterns
        match (parent_type, property_name) {
            ("BundleEntry", "resource") => Ok(true), // Mixed: can be any resource type
            ("ParametersParameter", "resource") => Ok(true), // Mixed: can be any resource type
            _ => Ok(false), // Default to homogeneous
        }
    }

    async fn get_collection_element_types(&self, parent_type: &str, property_name: &str) -> octofhir_fhir_model::Result<Vec<TypeInfo>> {
        match (parent_type, property_name) {
            ("BundleEntry", "resource") => {
                // Return multiple possible resource types
                Ok(vec![
                    TypeInfo {
                        type_name: "Patient".to_string(),
                        singleton: true,
                        namespace: Some("FHIR".to_string()),
                        name: Some("Patient".to_string()),
                        is_empty: Some(false),
                        is_union_type: Some(false),
                        union_choices: None,
                    },
                    TypeInfo {
                        type_name: "Observation".to_string(),
                        singleton: true,
                        namespace: Some("FHIR".to_string()),
                        name: Some("Observation".to_string()),
                        is_empty: Some(false),
                        is_union_type: Some(false),
                        union_choices: None,
                    },
                ])
            }
            ("Patient", "name") => {
                // Homogeneous collection of HumanName
                Ok(vec![TypeInfo {
                    type_name: "HumanName".to_string(),
                    singleton: true,
                    namespace: Some("FHIR".to_string()),
                    name: Some("HumanName".to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }])
            }
            _ => Ok(vec![])
        }
    }

    async fn is_polymorphic_property(&self, parent_type: &str, property_name: &str) -> octofhir_fhir_model::Result<bool> {
        match (parent_type, property_name) {
            ("BundleEntry", "resource") => Ok(true), // Polymorphic: can be any resource
            ("Observation", "value") => Ok(true), // value[x] is polymorphic
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_parsing() {
        let (resource_type, id) = utils::parse_reference("Patient/123").unwrap();
        assert_eq!(resource_type, "Patient");
        assert_eq!(id, "123");

        assert!(utils::parse_reference("invalid-ref").is_err());
    }

    #[test]
    fn test_resource_type_extraction() {
        let patient = serde_json::json!({
            "resourceType": "Patient",
            "id": "123"
        });

        assert_eq!(
            utils::extract_resource_type(&patient),
            Some("Patient".to_string())
        );
        assert!(utils::is_fhir_resource(&patient));
    }
}
