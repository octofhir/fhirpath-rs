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

//! Mock model provider for testing

use super::provider::*;
use std::collections::HashMap;

/// Mock model provider for testing
#[derive(Debug, Clone, Default)]
pub struct MockModelProvider {
    /// Registered types
    types: HashMap<String, TypeReflectionInfo>,
    /// Type properties
    properties: HashMap<String, HashMap<String, TypeReflectionInfo>>,
}

impl MockModelProvider {
    /// Helper method to check if a resource is a Bundle
    fn is_bundle_resource(&self, resource: &crate::FhirPathValue) -> bool {
        match resource {
            crate::FhirPathValue::Resource(res) => res.resource_type() == Some("Bundle"),
            crate::FhirPathValue::JsonValue(json) => json
                .as_json()
                .get("resourceType")
                .and_then(|rt| rt.as_str())
                .map(|rt| rt == "Bundle")
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Helper method to check if two resources are the same
    fn is_same_resource(
        &self,
        resource1: &crate::FhirPathValue,
        resource2: &crate::FhirPathValue,
    ) -> bool {
        std::ptr::eq(resource1, resource2)
    }
    /// Create a new mock provider with basic FHIR types
    pub fn new() -> Self {
        let mut provider = Self::default();
        provider.initialize_basic_types();
        provider
    }

    /// Create a minimal mock provider with no predefined types
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add a type to the mock provider
    pub fn add_type(&mut self, type_name: String, type_info: TypeReflectionInfo) {
        self.types.insert(type_name, type_info);
    }

    /// Add a property to a type
    pub fn add_property(
        &mut self,
        type_name: String,
        property_name: String,
        property_type: TypeReflectionInfo,
    ) {
        self.properties
            .entry(type_name)
            .or_default()
            .insert(property_name, property_type);
    }

    /// Initialize basic FHIR types for testing
    fn initialize_basic_types(&mut self) {
        // Add primitive types
        self.add_type(
            "boolean".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
                base_type: None,
            },
        );

        self.add_type(
            "integer".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Integer".to_string(),
                base_type: None,
            },
        );

        self.add_type(
            "string".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            },
        );

        // Add Patient resource
        let patient_elements = vec![
            ElementInfo {
                name: "active".to_string(),
                type_info: TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "Boolean".to_string(),
                    base_type: None,
                },
                min_cardinality: 0,
                max_cardinality: Some(1),
                is_modifier: false,
                is_summary: false,
                documentation: Some("Whether the patient record is active".to_string()),
            },
            ElementInfo {
                name: "name".to_string(),
                type_info: TypeReflectionInfo::ListType {
                    element_type: Box::new(TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: "HumanName".to_string(),
                        base_type: Some("Element".to_string()),
                        elements: vec![],
                    }),
                },
                min_cardinality: 0,
                max_cardinality: None,
                is_modifier: false,
                is_summary: true,
                documentation: Some("A human name for the patient".to_string()),
            },
        ];

        self.add_type(
            "Patient".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "Patient".to_string(),
                base_type: Some("DomainResource".to_string()),
                elements: patient_elements,
            },
        );

        // Add HumanName complex type
        let human_name_elements = vec![
            ElementInfo {
                name: "family".to_string(),
                type_info: TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "String".to_string(),
                    base_type: None,
                },
                min_cardinality: 0,
                max_cardinality: Some(1),
                is_modifier: false,
                is_summary: true,
                documentation: Some("Family name".to_string()),
            },
            ElementInfo {
                name: "given".to_string(),
                type_info: TypeReflectionInfo::ListType {
                    element_type: Box::new(TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: "String".to_string(),
                        base_type: None,
                    }),
                },
                min_cardinality: 0,
                max_cardinality: None,
                is_modifier: false,
                is_summary: true,
                documentation: Some("Given names".to_string()),
            },
        ];

        self.add_type(
            "HumanName".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "HumanName".to_string(),
                base_type: Some("Element".to_string()),
                elements: human_name_elements,
            },
        );

        // Add properties for easy lookup
        self.add_property(
            "Patient".to_string(),
            "active".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
                base_type: None,
            },
        );

        self.add_property(
            "Patient".to_string(),
            "name".to_string(),
            TypeReflectionInfo::ListType {
                element_type: Box::new(TypeReflectionInfo::ClassInfo {
                    namespace: "FHIR".to_string(),
                    name: "HumanName".to_string(),
                    base_type: Some("Element".to_string()),
                    elements: vec![],
                }),
            },
        );

        self.add_property(
            "HumanName".to_string(),
            "family".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            },
        );

        self.add_property(
            "HumanName".to_string(),
            "given".to_string(),
            TypeReflectionInfo::ListType {
                element_type: Box::new(TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "String".to_string(),
                    base_type: None,
                }),
            },
        );
    }
}

// Implement the async ModelProvider trait
use async_trait::async_trait;

#[async_trait]
impl ModelProvider for MockModelProvider {
    async fn get_type_reflection(&self, type_name: &str) -> Option<TypeReflectionInfo> {
        self.types.get(type_name).cloned()
    }

    async fn get_element_reflection(
        &self,
        type_name: &str,
        element_path: &str,
    ) -> Option<TypeReflectionInfo> {
        self.properties
            .get(type_name)
            .and_then(|props| props.get(element_path))
            .cloned()
    }

    async fn get_property_type(
        &self,
        parent_type: &str,
        property: &str,
    ) -> Option<TypeReflectionInfo> {
        self.properties
            .get(parent_type)
            .and_then(|props| props.get(property))
            .cloned()
    }

    async fn get_structure_definition(&self, _type_name: &str) -> Option<StructureDefinition> {
        None // Not implemented for mock
    }

    async fn validate_conformance(
        &self,
        _value: &dyn ValueReflection,
        _profile_url: &str,
    ) -> Result<octofhir_fhir_model::conformance::ConformanceResult, ModelError> {
        Ok(octofhir_fhir_model::conformance::ConformanceResult {
            is_valid: true,
            violations: vec![],
            warnings: vec![],
            metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
            profile_url: _profile_url.to_string(),
            resource_type: "Unknown".to_string(),
        })
    }

    async fn get_constraints(
        &self,
        _type_name: &str,
    ) -> Vec<octofhir_fhir_model::constraints::ConstraintInfo> {
        vec![]
    }

    async fn resolve_reference(
        &self,
        _reference_url: &str,
        _context: &dyn ResolutionContext,
    ) -> Option<Box<dyn ValueReflection>> {
        None
    }

    async fn analyze_expression(
        &self,
        _expression: &str,
    ) -> Result<ExpressionAnalysis, ModelError> {
        Ok(ExpressionAnalysis {
            referenced_types: vec![],
            navigation_paths: vec![],
            requires_runtime_types: false,
            optimization_hints: vec![],
            type_safety_warnings: vec![],
        })
    }

    async fn box_value_with_metadata(
        &self,
        _value: &dyn ValueReflection,
        _type_name: &str,
    ) -> Result<BoxedValueWithMetadata, ModelError> {
        Err(ModelError::validation_error(
            "box_value_with_metadata not implemented in mock",
        ))
    }

    async fn extract_primitive_extensions(
        &self,
        _value: &dyn ValueReflection,
        _element_path: &str,
    ) -> Option<PrimitiveExtensionData> {
        None
    }

    async fn find_extensions_by_url(
        &self,
        value: &crate::FhirPathValue,
        parent_resource: &crate::FhirPathValue,
        element_path: Option<&str>,
        url: &str,
    ) -> Vec<crate::FhirPathValue> {
        let _ = element_path; // Unused for now
        use crate::FhirPathValue;

        // First check for direct extensions on the value
        if let FhirPathValue::JsonValue(json) = value {
            if let Some(extensions) = json.as_json().get("extension") {
                if let Some(ext_array) = extensions.as_array() {
                    let mut matching_extensions = Vec::new();
                    for ext in ext_array {
                        if let Some(ext_obj) = ext.as_object() {
                            if let Some(ext_url) = ext_obj.get("url") {
                                if let Some(ext_url_str) = ext_url.as_str() {
                                    if ext_url_str == url {
                                        matching_extensions
                                            .push(FhirPathValue::resource_from_json(ext.clone()));
                                    }
                                }
                            }
                        }
                    }
                    if !matching_extensions.is_empty() {
                        return matching_extensions;
                    }
                }
            }
        }

        // For primitive values, check the underscore element in the parent resource
        if matches!(
            value,
            FhirPathValue::String(_)
                | FhirPathValue::Integer(_)
                | FhirPathValue::Decimal(_)
                | FhirPathValue::Boolean(_)
        ) {
            if let FhirPathValue::JsonValue(parent_json) = parent_resource {
                let parent_obj = parent_json.as_json();

                // Check common underscore properties
                let underscore_properties =
                    ["_birthDate", "_deceasedBoolean", "_active", "_gender"];

                for underscore_prop in &underscore_properties {
                    if let Some(underscore_element) = parent_obj.get(underscore_prop) {
                        if let Some(extensions) = underscore_element.get("extension") {
                            if let Some(ext_array) = extensions.as_array() {
                                let mut matching_extensions = Vec::new();

                                for ext in ext_array {
                                    if let Some(ext_obj) = ext.as_object() {
                                        if let Some(ext_url) = ext_obj.get("url") {
                                            if let Some(ext_url_str) = ext_url.as_str() {
                                                if ext_url_str == url {
                                                    matching_extensions.push(
                                                        FhirPathValue::resource_from_json(
                                                            ext.clone(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                if !matching_extensions.is_empty() {
                                    return matching_extensions;
                                }
                            }
                        }
                    }
                }
            }
        }

        Vec::new()
    }

    async fn get_search_params(&self, _resource_type: &str) -> Vec<SearchParameter> {
        vec![]
    }

    async fn is_resource_type(&self, type_name: &str) -> bool {
        // Normalize type name by removing namespace prefixes and backticks
        let normalize_type = |type_name: &str| -> String {
            // First trim backticks from both ends
            let cleaned = type_name.trim_matches('`');
            // Then handle namespace prefixes
            if let Some(dot_pos) = cleaned.find('.') {
                // Extract the part after the dot and trim backticks again
                cleaned[dot_pos + 1..].trim_matches('`').to_string()
            } else {
                cleaned.to_string()
            }
        };

        let normalized = normalize_type(type_name);
        matches!(
            normalized.as_str(),
            "Patient" | "Observation" | "Condition" | "Procedure"
        )
    }

    fn fhir_version(&self) -> FhirVersion {
        FhirVersion::R4
    }

    async fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool {
        // Normalize type names by removing namespace prefixes and backticks
        let normalize_type = |type_name: &str| -> String {
            // First trim backticks from both ends
            let cleaned = type_name.trim_matches('`');
            // Then handle namespace prefixes
            if let Some(dot_pos) = cleaned.find('.') {
                // Extract the part after the dot and trim backticks again
                cleaned[dot_pos + 1..].trim_matches('`').to_string()
            } else {
                cleaned.to_string()
            }
        };

        let normalized_child = normalize_type(child_type);
        let normalized_parent = normalize_type(parent_type);

        if normalized_child == normalized_parent {
            return true;
        }

        // Define the inheritance hierarchy
        let direct_inheritance = match normalized_child.as_str() {
            "Patient" => Some("DomainResource"),
            "DomainResource" => Some("Resource"),
            "HumanName" => Some("Element"),
            // Add other common FHIR resource types
            "Observation" => Some("DomainResource"),
            "Condition" => Some("DomainResource"),
            "Procedure" => Some("DomainResource"),
            _ => None,
        };

        // Check direct inheritance
        if let Some(direct_parent) = direct_inheritance {
            if direct_parent == normalized_parent {
                return true;
            }
            // Check transitive inheritance
            return self.is_subtype_of(direct_parent, &normalized_parent).await;
        }

        false
    }

    async fn get_properties(&self, type_name: &str) -> Vec<(String, TypeReflectionInfo)> {
        self.properties
            .get(type_name)
            .map(|props| props.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    async fn get_base_type(&self, type_name: &str) -> Option<String> {
        match type_name {
            "Patient" => Some("DomainResource".to_string()),
            "HumanName" => Some("Element".to_string()),
            _ => None,
        }
    }

    async fn validate_navigation_path(
        &self,
        type_name: &str,
        path: &str,
    ) -> Result<NavigationValidation, octofhir_fhir_model::error::ModelError> {
        let is_valid = self
            .properties
            .get(type_name)
            .map(|props| props.contains_key(path))
            .unwrap_or(false);

        Ok(NavigationValidation {
            is_valid,
            result_type: self.get_property_type(type_name, path).await,
            intermediate_types: vec![],
            messages: if is_valid {
                vec![]
            } else {
                vec![format!(
                    "Property '{}' not found on type '{}'",
                    path, type_name
                )]
            },
        })
    }

    // Enhanced reference resolution methods for MockModelProvider
    async fn resolve_reference_in_context(
        &self,
        reference_url: &str,
        root_resource: &crate::FhirPathValue,
        current_resource: Option<&crate::FhirPathValue>,
    ) -> Option<crate::FhirPathValue> {
        // Handle empty references
        if reference_url.is_empty() {
            return None;
        }

        // ALWAYS try to resolve within Bundle context first if root is a Bundle
        // This is critical for Bundle.entry.resource.medicationReference.resolve() scenarios
        if self.is_bundle_resource(root_resource) {
            if let Some(resolved) = self.resolve_in_bundle(reference_url, root_resource).await {
                return Some(resolved);
            }
        }

        // Then try to resolve within contained resources in current resource
        if let Some(current) = current_resource {
            if let Some(resolved) = self.resolve_in_contained(reference_url, current).await {
                return Some(resolved);
            }
        }

        // Also check root resource for contained resources (if different from current)
        if current_resource.is_some()
            && !self.is_same_resource(current_resource.unwrap(), root_resource)
        {
            if let Some(resolved) = self
                .resolve_in_contained(reference_url, root_resource)
                .await
            {
                return Some(resolved);
            }
        }

        // If root is not a Bundle but current is a Bundle, try that too
        if let Some(current) = current_resource {
            if self.is_bundle_resource(current) && !self.is_bundle_resource(root_resource) {
                if let Some(resolved) = self.resolve_in_bundle(reference_url, current).await {
                    return Some(resolved);
                }
            }
        }

        // Finally try external resolution (returns None for MockModelProvider)
        self.resolve_external_reference(reference_url).await
    }

    async fn resolve_in_bundle(
        &self,
        reference_url: &str,
        bundle: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue> {
        let bundle_json = match bundle {
            crate::FhirPathValue::Resource(bundle_resource) => bundle_resource.as_json(),
            crate::FhirPathValue::JsonValue(json_value) => json_value.as_json(),
            _ => return None,
        };

        if let Some(entries) = bundle_json.get("entry").and_then(|e| e.as_array()) {
            for entry in entries {
                if let Some(resource) = entry.get("resource") {
                    // Check fullUrl first (preferred for Bundle resolution)
                    if let Some(full_url) = entry.get("fullUrl").and_then(|u| u.as_str()) {
                        if full_url.ends_with(reference_url) || full_url == reference_url {
                            return Some(crate::FhirPathValue::resource_from_json(
                                resource.clone(),
                            ));
                        }
                    }

                    // Check resource type and ID
                    if let (Some(resource_type), Some(id)) = (
                        resource.get("resourceType").and_then(|rt| rt.as_str()),
                        resource.get("id").and_then(|id| id.as_str()),
                    ) {
                        let resource_ref = format!("{resource_type}/{id}");
                        if resource_ref == reference_url {
                            return Some(crate::FhirPathValue::resource_from_json(
                                resource.clone(),
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    async fn resolve_in_contained(
        &self,
        reference_url: &str,
        containing_resource: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue> {
        let resource_json = match containing_resource {
            crate::FhirPathValue::Resource(resource) => resource.as_json(),
            crate::FhirPathValue::JsonValue(json_value) => json_value.as_json(),
            _ => return None,
        };

        if let Some(contained) = resource_json.get("contained").and_then(|c| c.as_array()) {
            for contained_resource in contained {
                if let (Some(resource_type), Some(id)) = (
                    contained_resource
                        .get("resourceType")
                        .and_then(|rt| rt.as_str()),
                    contained_resource.get("id").and_then(|id| id.as_str()),
                ) {
                    // Check for fragment reference (starts with #)
                    if reference_url.starts_with('#') && &reference_url[1..] == id {
                        return Some(crate::FhirPathValue::resource_from_json(
                            contained_resource.clone(),
                        ));
                    }

                    // Check for full reference
                    let resource_ref = format!("{resource_type}/{id}");
                    if resource_ref == reference_url {
                        return Some(crate::FhirPathValue::resource_from_json(
                            contained_resource.clone(),
                        ));
                    }
                }
            }
        }
        None
    }

    async fn resolve_external_reference(
        &self,
        _reference_url: &str,
    ) -> Option<crate::FhirPathValue> {
        // MockModelProvider should not create placeholders for external references
        // Bundle resolution should take precedence over external resolution
        None
    }

    fn parse_reference_url(
        &self,
        reference_url: &str,
    ) -> Option<super::provider::ReferenceComponents> {
        // Handle fragment references
        if reference_url.starts_with('#') {
            return Some(super::provider::ReferenceComponents {
                resource_type: "".to_string(),
                resource_id: reference_url[1..].to_string(),
                version_id: None,
                fragment: Some(reference_url.to_string()),
                full_url: None,
                base_url: None,
            });
        }

        // Handle full URLs
        if reference_url.starts_with("http://") || reference_url.starts_with("https://") {
            if let Ok(url) = url::Url::parse(reference_url) {
                let path = url.path();
                // Extract resource type and ID from path like /Patient/123
                let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

                if path_segments.len() >= 2 {
                    let resource_type = path_segments[path_segments.len() - 2].to_string();
                    let resource_id = path_segments[path_segments.len() - 1].to_string();

                    // Check for version history
                    let (resource_id, version_id) = if path_segments.len() >= 4
                        && path_segments[path_segments.len() - 3] == "_history"
                    {
                        (
                            path_segments[path_segments.len() - 4].to_string(),
                            Some(resource_id),
                        )
                    } else {
                        (resource_id, None)
                    };

                    if let Some(host_str) = url.host_str() {
                        let base_url = format!("{}://{}", url.scheme(), host_str);

                        return Some(super::provider::ReferenceComponents {
                            resource_type,
                            resource_id,
                            version_id,
                            fragment: url.fragment().map(|f| format!("#{}", f)),
                            full_url: Some(reference_url.to_string()),
                            base_url: Some(base_url),
                        });
                    }
                }
            }
        }

        // Handle ResourceType/id format
        if let Some(slash_pos) = reference_url.find('/') {
            let resource_type = reference_url[..slash_pos].to_string();
            let remaining = &reference_url[slash_pos + 1..];

            // Check for version history
            if let Some(history_pos) = remaining.find("/_history/") {
                let resource_id = remaining[..history_pos].to_string();
                let version_id = remaining[history_pos + 10..].to_string();

                return Some(super::provider::ReferenceComponents {
                    resource_type,
                    resource_id,
                    version_id: Some(version_id),
                    fragment: None,
                    full_url: None,
                    base_url: None,
                });
            } else {
                let resource_id = remaining.to_string();

                // Basic validation - resource type should be capitalized
                if !resource_type.is_empty()
                    && !resource_id.is_empty()
                    && resource_type.chars().next().unwrap().is_uppercase()
                {
                    return Some(super::provider::ReferenceComponents {
                        resource_type,
                        resource_id,
                        version_id: None,
                        fragment: None,
                        full_url: None,
                        base_url: None,
                    });
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_creation() {
        let provider = MockModelProvider::new();

        // Should have Patient type
        assert!(provider.get_type_reflection("Patient").await.is_some());
        assert!(provider.get_type_reflection("HumanName").await.is_some());
    }

    #[tokio::test]
    async fn test_patient_properties() {
        let provider = MockModelProvider::new();

        // Test Patient properties
        assert!(
            provider
                .get_property_type("Patient", "active")
                .await
                .is_some()
        );
        assert!(
            provider
                .get_property_type("Patient", "name")
                .await
                .is_some()
        );

        // Non-existent property should return None
        assert!(
            provider
                .get_property_type("Patient", "nonexistent")
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_human_name_properties() {
        let provider = MockModelProvider::new();

        // Test HumanName properties
        assert!(
            provider
                .get_property_type("HumanName", "family")
                .await
                .is_some()
        );
        assert!(
            provider
                .get_property_type("HumanName", "given")
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_empty_provider() {
        let provider = MockModelProvider::empty();

        // Should have no types
        assert!(provider.get_type_reflection("Patient").await.is_none());
        assert!(
            provider
                .get_property_type("Patient", "name")
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_type_compatibility_with_namespaces() {
        let provider = MockModelProvider::new();

        // Test basic type compatibility
        assert!(provider.is_type_compatible("Patient", "Patient").await);
        assert!(
            provider
                .is_type_compatible("Patient", "DomainResource")
                .await
        );
        assert!(provider.is_type_compatible("Patient", "Resource").await);

        // Test namespace variations
        assert!(provider.is_type_compatible("Patient", "FHIR.Patient").await);
        assert!(provider.is_type_compatible("FHIR.Patient", "Patient").await);
        assert!(
            provider
                .is_type_compatible("FHIR.Patient", "FHIR.Patient")
                .await
        );

        // Test inheritance with namespaces
        assert!(
            provider
                .is_type_compatible("FHIR.Patient", "FHIR.DomainResource")
                .await
        );

        // Test negative cases
        assert!(!provider.is_type_compatible("Patient", "Observation").await);
        assert!(
            !provider
                .is_type_compatible("FHIR.Patient", "FHIR.Observation")
                .await
        );
    }

    #[tokio::test]
    async fn test_type_compatibility_with_backticks() {
        let provider = MockModelProvider::new();

        // Test backticks - these should all be equivalent to "Patient"
        assert!(
            provider
                .is_type_compatible("Patient", "FHIR.`Patient`")
                .await
        );
        assert!(
            provider
                .is_type_compatible("FHIR.`Patient`", "Patient")
                .await
        );
        assert!(
            provider
                .is_type_compatible("FHIR.`Patient`", "FHIR.`Patient`")
                .await
        );
        assert!(
            provider
                .is_type_compatible("FHIR.`Patient`", "FHIR.`DomainResource`")
                .await
        );
    }
}
