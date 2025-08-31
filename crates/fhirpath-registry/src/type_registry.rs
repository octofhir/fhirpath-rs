//! FHIRPath Type Registry with Bridge Support
//!
//! This module provides O(1) resource type checking using the bridge API
//! from octofhir-fhirschema for efficient type resolution and validation.

//! Removed unused import
use octofhir_fhirpath_model::BridgeResourceInfo;
use octofhir_fhirschema::package::FhirSchemaPackageManager;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur in the type registry
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Failed to initialize registry: {source}")]
    InitializationError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Resource type '{resource_type}' not found: {source}")]
    ResourceNotFound {
        resource_type: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Schema error: {0}")]
    SchemaError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// FHIRPath Type Registry with O(1) operations via bridge API
pub struct FhirPathTypeRegistry {
    resource_types: HashMap<String, bool>,
    data_types: HashMap<String, bool>,
    primitive_types: HashMap<String, bool>,
    schema_manager: Arc<FhirSchemaPackageManager>,
}

impl FhirPathTypeRegistry {
    /// Create a new type registry from schema manager
    pub async fn new(
        schema_manager: Arc<FhirSchemaPackageManager>,
    ) -> std::result::Result<Self, RegistryError> {
        // Initialize with common FHIR types for now
        // In a full implementation, this would be populated from the schema manager
        let resource_types = [
            "Patient",
            "Observation",
            "Bundle",
            "Organization",
            "Practitioner",
            "Location",
            "Medication",
            "Device",
            "Procedure",
            "Condition",
            "AllergyIntolerance",
            "DiagnosticReport",
            "Encounter",
            "DomainResource",
            "Resource",
        ]
        .iter()
        .map(|&s| (s.to_string(), true))
        .collect();

        let data_types = [
            "HumanName",
            "Coding",
            "Address",
            "ContactPoint",
            "Identifier",
            "CodeableConcept",
            "Quantity",
            "Period",
            "Range",
            "DataType",
        ]
        .iter()
        .map(|&s| (s.to_string(), true))
        .collect();

        let primitive_types = [
            "string",
            "boolean",
            "decimal",
            "integer",
            "date",
            "dateTime",
            "time",
            "code",
            "uri",
            "url",
            "canonical",
            "PrimitiveType",
        ]
        .iter()
        .map(|&s| (s.to_string(), true))
        .collect();

        Ok(Self {
            resource_types,
            data_types,
            primitive_types,
            schema_manager,
        })
    }

    /// O(1) resource type checking
    pub fn is_resource_type(&self, type_name: &str) -> bool {
        self.resource_types.contains_key(type_name)
    }

    /// O(1) data type checking
    pub fn is_data_type(&self, type_name: &str) -> bool {
        self.data_types.contains_key(type_name)
    }

    /// O(1) primitive type checking
    pub fn is_primitive_type(&self, type_name: &str) -> bool {
        self.primitive_types.contains_key(type_name)
    }

    /// Get detailed resource information via bridge API
    pub async fn get_resource_info(
        &self,
        resource_type: &str,
    ) -> std::result::Result<BridgeResourceInfo, RegistryError> {
        // Placeholder implementation - would use schema manager in full implementation
        if self.is_resource_type(resource_type) {
            Ok(BridgeResourceInfo {
                resource_type: resource_type.to_string(),
                base_type: Some("Resource".to_string()),
                is_abstract: false,
                properties: vec![],
                profiles: vec![],
                namespace: "FHIR".to_string(),
            })
        } else {
            Err(RegistryError::ResourceNotFound {
                resource_type: resource_type.to_string(),
                source: "Resource not found in simplified registry".into(),
            })
        }
    }

    /// Get all available resource types
    pub fn get_all_resource_types(&self) -> Vec<String> {
        self.resource_types.keys().cloned().collect()
    }

    /// Get all available data types
    pub fn get_all_data_types(&self) -> Vec<String> {
        self.data_types.keys().cloned().collect()
    }

    /// Check if one type is a subtype of another using schema manager
    pub async fn is_subtype_of(
        &self,
        child_type: &str,
        parent_type: &str,
    ) -> std::result::Result<bool, RegistryError> {
        if child_type == parent_type {
            return Ok(true);
        }

        // Simple hardcoded inheritance for demo - would use schema manager in full implementation
        match (child_type, parent_type) {
            ("Patient", "DomainResource") => Ok(true),
            ("Patient", "Resource") => Ok(true),
            ("Observation", "DomainResource") => Ok(true),
            ("Observation", "Resource") => Ok(true),
            ("DomainResource", "Resource") => Ok(true),
            (_, _) => Ok(false),
        }
    }

    /// Get the schema manager for advanced operations
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirschema::package::FhirSchemaPackageManager;

    async fn create_test_schema_manager()
    -> Result<FhirSchemaPackageManager, Box<dyn std::error::Error>> {
        // Mock implementation for testing
        // In real usage, this would be properly initialized
        todo!("Implement test schema manager creation")
    }

    #[tokio::test]
    async fn test_type_registry_operations() -> Result<(), Box<dyn std::error::Error>> {
        let manager = Arc::new(create_test_schema_manager().await?);
        let registry = FhirPathTypeRegistry::new(manager).await?;

        // Test O(1) operations
        assert!(registry.is_resource_type("Patient"));
        assert!(registry.is_data_type("HumanName"));
        assert!(registry.is_primitive_type("string"));
        assert!(!registry.is_resource_type("InvalidType"));

        Ok(())
    }

    #[tokio::test]
    async fn test_subtype_checking() -> Result<(), Box<dyn std::error::Error>> {
        let manager = Arc::new(create_test_schema_manager().await?);
        let registry = FhirPathTypeRegistry::new(manager).await?;

        // Test inheritance checking
        assert!(registry.is_subtype_of("Patient", "DomainResource").await?);
        assert!(registry.is_subtype_of("Patient", "Patient").await?); // Same type
        assert!(!registry.is_subtype_of("Patient", "Observation").await?);

        Ok(())
    }
}
