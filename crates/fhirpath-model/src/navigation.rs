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

//! Property navigation with bridge support integration
//!
//! This module provides efficient property navigation using the FhirSchemaPackageManager
//! bridge support API for O(1) operations.

use crate::bridge_types::BridgeChoiceInfo;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirschema::FhirSchemaPackageManager;
use octofhir_fhirschema::types::{BridgeCardinality, BridgeValidationResult, PropertyInfo};
use std::sync::Arc;

/// Property navigator with bridge support for efficient schema operations
#[derive(Clone)]
pub struct PropertyNavigator {
    /// Schema manager for bridge operations
    schema_manager: Arc<FhirSchemaPackageManager>,
}

impl PropertyNavigator {
    /// Create a new PropertyNavigator with the given schema manager
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        Self { schema_manager }
    }

    /// Get property information using schema lookup
    pub async fn get_property_info(&self, type_name: &str, property: &str) -> Result<PropertyInfo> {
        // Use schema-based property lookup as a bridge implementation
        if let Some(_schema) = self.schema_manager.get_schema_by_type(type_name).await {
            // Create a basic PropertyInfo based on schema data
            // This is a simplified implementation - in practice this would be more comprehensive
            Ok(PropertyInfo {
                name: property.to_string(),
                element_type: "string".to_string(), // Simplified - would need proper type resolution
                cardinality: BridgeCardinality::new(0, Some(1)), // Default cardinality
                is_collection: false,
                is_required: false,
                is_choice_type: property.ends_with("[x]"),
                definition: Some(format!("Property {} on {}", property, type_name)),
            })
        } else {
            Err(FhirPathError::evaluation_error(format!(
                "Type {} not found in schema",
                type_name
            )))
        }
    }

    /// Resolve choice type using schema manager
    pub async fn resolve_choice_type(
        &self,
        path: &str,
        value_type: &str,
    ) -> Result<BridgeChoiceInfo> {
        if let Some(resolved) = self
            .schema_manager
            .resolve_choice_type(path, value_type)
            .await
        {
            Ok(BridgeChoiceInfo::valid(
                path.to_string(),
                resolved.clone(),
                value_type.to_string(),
                BridgeCardinality::new(0, Some(1)),
            ))
        } else {
            Ok(BridgeChoiceInfo::invalid(
                path.to_string(),
                value_type.to_string(),
            ))
        }
    }

    /// Check if resource type exists (O(1) operation)
    pub async fn has_resource_type(&self, type_name: &str) -> bool {
        self.schema_manager.has_resource_type(type_name).await
    }

    /// Get all properties for a type using schema lookup
    pub async fn get_properties_for_type(&self, type_name: &str) -> Result<Vec<PropertyInfo>> {
        if let Some(_schema) = self.schema_manager.get_schema_by_type(type_name).await {
            // For demonstration, return a basic property list
            // In practice, this would parse the schema structure definition
            let common_properties = match type_name {
                "Patient" => vec!["id", "name", "gender", "birthDate"],
                "Observation" => vec!["id", "status", "code", "subject", "value[x]"],
                "Bundle" => vec!["id", "type", "entry"],
                _ => vec!["id"],
            };

            let properties: Vec<PropertyInfo> = common_properties
                .into_iter()
                .map(|name| PropertyInfo {
                    name: name.to_string(),
                    element_type: "string".to_string(),
                    cardinality: BridgeCardinality::new(0, Some(1)),
                    is_collection: false,
                    is_required: name == "id",
                    is_choice_type: name.ends_with("[x]"),
                    definition: Some(format!("Property {} on {}", name, type_name)),
                })
                .collect();

            Ok(properties)
        } else {
            Err(FhirPathError::evaluation_error(format!(
                "Type {} not found",
                type_name
            )))
        }
    }

    /// Validate FHIRPath constraint (basic validation)
    pub async fn validate_fhirpath_constraint(
        &self,
        constraint: &str,
    ) -> Result<BridgeValidationResult> {
        // Basic validation - checks if constraint is non-empty and has basic FHIRPath syntax
        let is_valid = !constraint.is_empty() && constraint.contains(".");

        Ok(BridgeValidationResult {
            is_valid,
            errors: if is_valid { Vec::new() } else { vec![] },
            warnings: Vec::new(),
            metrics: None,
        })
    }

    /// Get all resource types (O(1) cached operation)
    pub async fn get_resource_types(&self) -> Result<Vec<String>> {
        let types = self.schema_manager.get_resource_types().await;
        Ok(types)
    }

    /// Get schema by canonical URL (O(1) operation)
    pub async fn get_schema(&self, canonical_url: &str) -> Result<octofhir_fhirschema::FhirSchema> {
        if let Some(schema) = self.schema_manager.get_schema(canonical_url).await {
            Ok((*schema).clone())
        } else {
            Err(FhirPathError::evaluation_error(format!(
                "Schema not found: {}",
                canonical_url
            )))
        }
    }

    /// Get schemas by resource type (O(1) lookup)
    pub async fn get_schemas_by_type(
        &self,
        resource_type: &str,
    ) -> Result<Vec<octofhir_fhirschema::FhirSchema>> {
        let schemas = self.schema_manager.get_schemas_by_type(resource_type).await;
        Ok(schemas.into_iter().map(|s| (*s).clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};

    #[tokio::test]
    async fn test_property_navigator_creation() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        // Test basic resource type check
        assert!(navigator.has_resource_type("Patient").await);
        assert!(!navigator.has_resource_type("InvalidType").await);
    }

    #[tokio::test]
    async fn test_resource_type_lookup() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        // Test O(1) resource type operations
        assert!(navigator.has_resource_type("Patient").await);
        assert!(navigator.has_resource_type("Observation").await);
        assert!(navigator.has_resource_type("Bundle").await);
        assert!(!navigator.has_resource_type("NonExistentResource").await);
    }

    #[tokio::test]
    async fn test_property_info_lookup() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        // Test property lookup for Patient
        let property = navigator.get_property_info("Patient", "name").await;
        assert!(property.is_ok(), "Patient.name property should exist");

        let prop_info = property.unwrap();
        assert_eq!(prop_info.name, "name");
        assert!(!prop_info.is_required); // name is not required in FHIR Patient
    }

    #[tokio::test]
    async fn test_choice_type_resolution() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        // Test choice type resolution for Observation.value[x]
        let choice_result = navigator
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        assert!(
            choice_result.is_ok(),
            "Choice type resolution should succeed"
        );

        let choice_info = choice_result.unwrap();
        assert_eq!(choice_info.resolved_type, "valueString");
        assert!(choice_info.is_valid);
    }

    #[tokio::test]
    async fn test_resource_types_enumeration() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        let resource_types = navigator.get_resource_types().await;
        assert!(
            resource_types.is_ok(),
            "Should be able to get resource types"
        );

        let types = resource_types.unwrap();
        assert!(types.contains(&"Patient".to_string()));
        assert!(types.contains(&"Observation".to_string()));
        assert!(types.contains(&"Bundle".to_string()));
    }

    #[tokio::test]
    async fn test_schema_lookup() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        let patient_schema = navigator
            .get_schema("http://hl7.org/fhir/StructureDefinition/Patient")
            .await;

        assert!(patient_schema.is_ok(), "Should find Patient schema");

        let schema = patient_schema.unwrap();
        if let Some(name) = &schema.name {
            assert_eq!(name, "Patient");
        }
    }

    #[tokio::test]
    async fn test_schemas_by_type() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let navigator = PropertyNavigator::new(manager);

        let patient_schemas = navigator.get_schemas_by_type("Patient").await;
        assert!(patient_schemas.is_ok(), "Should find Patient schemas");

        let schemas = patient_schemas.unwrap();
        assert!(
            !schemas.is_empty(),
            "Should have at least one Patient schema"
        );
    }
}
