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

//! System types using bridge support API
//!
//! This module provides system type information using the FhirSchemaPackageManager
//! bridge support API, replacing all hardcoded type definitions with dynamic
//! schema-based type resolution.

use crate::type_resolution::{TypeInfo, TypeResolver};
use octofhir_fhirpath_core::Result;
use octofhir_fhirschema::FhirSchemaPackageManager;
use std::sync::Arc;

/// System type categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemTypeCategory {
    /// Primitive types (string, boolean, integer, etc.)
    Primitive,
    /// Complex data types (HumanName, Address, etc.)
    Complex,
    /// Resource types (Patient, Observation, etc.)
    Resource,
    /// Backbone elements
    Backbone,
    /// Unknown or unresolved type
    Unknown,
}

/// System types manager using bridge support API
#[derive(Clone)]
pub struct SystemTypes {
    /// Schema manager for dynamic type resolution
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Type resolver for comprehensive type information
    type_resolver: TypeResolver,
}

impl SystemTypes {
    /// Create a new SystemTypes with the given schema manager
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        let type_resolver = TypeResolver::new(schema_manager.clone());
        Self {
            schema_manager,
            type_resolver,
        }
    }

    /// Get system type category using bridge API (O(1) operation)
    pub async fn get_system_type_category(&self, type_name: &str) -> SystemTypeCategory {
        // Use schema manager for dynamic type categorization
        if self.schema_manager.is_primitive_type(type_name).await {
            SystemTypeCategory::Primitive
        } else if self.schema_manager.has_resource_type(type_name).await {
            SystemTypeCategory::Resource
        } else if self.schema_manager.is_complex_type(type_name).await {
            SystemTypeCategory::Complex
        } else {
            SystemTypeCategory::Unknown
        }
    }

    /// Check if a type is polymorphic (choice type) using bridge API
    pub async fn is_polymorphic(&self, property_path: &str) -> Result<bool> {
        // Check if property path represents a choice type
        Ok(property_path.ends_with("[x]")
            || self
                .schema_manager
                .is_choice_type_expansion(property_path)
                .await)
    }

    /// Get comprehensive type information using TypeResolver
    pub async fn get_type_info(&mut self, type_name: &str) -> Result<TypeInfo> {
        self.type_resolver.get_type_info(type_name).await
    }

    /// Check if a type is a subtype of another type using bridge API
    pub async fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool {
        self.type_resolver
            .is_subtype_of(child_type, parent_type)
            .await
    }

    /// Get all resource types from the schema (O(1) cached operation)
    pub async fn get_all_resource_types(&self) -> Vec<String> {
        self.schema_manager.get_resource_types().await
    }

    /// Check if a type name is valid in the current schema
    pub async fn is_valid_type(&self, type_name: &str) -> bool {
        // A type is valid if it's a resource, primitive, or complex type
        self.schema_manager.has_resource_type(type_name).await
            || self.schema_manager.is_primitive_type(type_name).await
            || self.schema_manager.is_complex_type(type_name).await
    }

    /// Get the namespace for a type ("System" or "FHIR")
    pub async fn get_namespace(&self, type_name: &str) -> String {
        if self.schema_manager.is_primitive_type(type_name).await {
            "System".to_string()
        } else {
            "FHIR".to_string()
        }
    }

    /// Get the base type of a given type using schema hierarchy
    pub async fn get_base_type(&self, type_name: &str) -> Option<String> {
        // Use schema manager to get type hierarchy information
        // This is a simplified implementation - in practice would use schema inheritance data
        match type_name {
            "DomainResource" => Some("Resource".to_string()),
            name if self.schema_manager.has_resource_type(name).await => {
                // Most resources inherit from DomainResource
                if name == "Resource" {
                    None
                } else {
                    Some("DomainResource".to_string())
                }
            }
            _ => None,
        }
    }

    /// Get type resolver for external use
    pub fn type_resolver(&mut self) -> &mut TypeResolver {
        &mut self.type_resolver
    }

    /// Get schema manager for external use
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}

/// System type utilities for common operations
pub mod utils {

    /// Check if a type name follows FHIRPath naming conventions
    pub fn is_valid_type_name(type_name: &str) -> bool {
        !type_name.is_empty()
            && type_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            && !type_name.starts_with('.')
            && !type_name.ends_with('.')
    }

    /// Normalize type name for consistent lookup
    pub fn normalize_type_name(type_name: &str) -> String {
        // Remove backticks and namespace prefixes
        let cleaned = type_name.trim_matches('`');
        if let Some(dot_pos) = cleaned.find('.') {
            cleaned[dot_pos + 1..].to_string()
        } else {
            cleaned.to_string()
        }
    }

    /// Check if a path represents a collection access
    pub fn is_collection_access(path: &str) -> bool {
        path.contains('[') && path.contains(']') && !path.ends_with("[x]")
    }

    /// Extract collection index from path (e.g., "name[0]" -> Some(0))
    pub fn extract_collection_index(path: &str) -> Option<usize> {
        if let Some(start) = path.find('[') {
            if let Some(end) = path.find(']') {
                if start < end {
                    let index_str = &path[start + 1..end];
                    return index_str.parse().ok();
                }
            }
        }
        None
    }

    /// Remove collection notation from property name (e.g., "name[0]" -> "name")
    pub fn remove_collection_notation(property: &str) -> &str {
        if let Some(bracket_pos) = property.find('[') {
            &property[..bracket_pos]
        } else {
            property
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_canonical_manager::FcmConfig;
    use octofhir_fhirschema::PackageManagerConfig;

    async fn create_test_schema_manager() -> Arc<FhirSchemaPackageManager> {
        let fcm_config = FcmConfig::default();
        let config = PackageManagerConfig::default();
        Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .expect("Failed to create schema manager"),
        )
    }

    #[tokio::test]
    async fn test_system_types_creation() {
        let schema_manager = create_test_schema_manager().await;
        let system_types = SystemTypes::new(schema_manager);

        // Test basic functionality
        assert!(system_types.is_valid_type("Patient").await);
        assert!(!system_types.is_valid_type("InvalidType").await);
    }

    #[tokio::test]
    async fn test_system_type_categories() {
        let schema_manager = create_test_schema_manager().await;
        let system_types = SystemTypes::new(schema_manager);

        // Test primitive types
        let category = system_types.get_system_type_category("string").await;
        assert_eq!(category, SystemTypeCategory::Primitive);

        // Test resource types
        let category = system_types.get_system_type_category("Patient").await;
        assert_eq!(category, SystemTypeCategory::Resource);

        // Test unknown types
        let category = system_types.get_system_type_category("InvalidType").await;
        assert_eq!(category, SystemTypeCategory::Unknown);
    }

    #[tokio::test]
    async fn test_type_namespaces() {
        let schema_manager = create_test_schema_manager().await;
        let system_types = SystemTypes::new(schema_manager);

        // Primitive types should be in System namespace
        let namespace = system_types.get_namespace("string").await;
        assert_eq!(namespace, "System");

        // Resource types should be in FHIR namespace
        let namespace = system_types.get_namespace("Patient").await;
        assert_eq!(namespace, "FHIR");
    }

    #[tokio::test]
    async fn test_subtype_checking() {
        let schema_manager = create_test_schema_manager().await;
        let system_types = SystemTypes::new(schema_manager);

        // Test inheritance relationships
        assert!(system_types.is_subtype_of("Patient", "Patient").await); // Same type
        assert!(
            system_types
                .is_subtype_of("Patient", "DomainResource")
                .await
        ); // Inheritance
        assert!(system_types.is_subtype_of("Patient", "Resource").await); // Deep inheritance
        assert!(!system_types.is_subtype_of("string", "Patient").await); // Different categories
    }

    #[tokio::test]
    async fn test_polymorphic_detection() {
        let schema_manager = create_test_schema_manager().await;
        let system_types = SystemTypes::new(schema_manager);

        // Test choice type detection
        assert!(
            system_types
                .is_polymorphic("Observation.value[x]")
                .await
                .unwrap()
        );
        assert!(!system_types.is_polymorphic("Patient.name").await.unwrap());
    }

    #[tokio::test]
    async fn test_resource_types_enumeration() {
        let schema_manager = create_test_schema_manager().await;
        let system_types = SystemTypes::new(schema_manager);

        let resource_types = system_types.get_all_resource_types().await;
        assert!(resource_types.contains(&"Patient".to_string()));
        assert!(resource_types.contains(&"Observation".to_string()));
    }

    mod util_tests {
        use super::super::utils::*;

        #[test]
        fn test_type_name_validation() {
            assert!(is_valid_type_name("Patient"));
            assert!(is_valid_type_name("HumanName"));
            assert!(is_valid_type_name("FHIR.Patient")); // Namespace qualified
            assert!(!is_valid_type_name("")); // Empty
            assert!(!is_valid_type_name(".Patient")); // Leading dot
            assert!(!is_valid_type_name("Patient.")); // Trailing dot
        }

        #[test]
        fn test_type_name_normalization() {
            assert_eq!(normalize_type_name("Patient"), "Patient");
            assert_eq!(normalize_type_name("`Patient`"), "Patient");
            assert_eq!(normalize_type_name("FHIR.Patient"), "Patient");
            assert_eq!(normalize_type_name("`FHIR.Patient`"), "Patient");
        }

        #[test]
        fn test_collection_access_detection() {
            assert!(is_collection_access("name[0]"));
            assert!(is_collection_access("telecom[1].value"));
            assert!(!is_collection_access("value[x]")); // Choice type, not collection
            assert!(!is_collection_access("name")); // No brackets
        }

        #[test]
        fn test_collection_index_extraction() {
            assert_eq!(extract_collection_index("name[0]"), Some(0));
            assert_eq!(extract_collection_index("telecom[5]"), Some(5));
            assert_eq!(extract_collection_index("name"), None);
            assert_eq!(extract_collection_index("value[x]"), None); // Invalid index
        }

        #[test]
        fn test_collection_notation_removal() {
            assert_eq!(remove_collection_notation("name[0]"), "name");
            assert_eq!(remove_collection_notation("telecom[1]"), "telecom");
            assert_eq!(remove_collection_notation("value[x]"), "value");
            assert_eq!(remove_collection_notation("name"), "name");
        }
    }
}
