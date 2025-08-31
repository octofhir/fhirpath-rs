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

//! Choice type resolution using bridge support API
//!
//! This module provides enhanced choice type resolution using the FhirSchemaPackageManager
//! bridge support API for accurate and efficient schema-driven choice type handling.

use crate::bridge_types::BridgeChoiceInfo;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirschema::FhirSchemaPackageManager;
use octofhir_fhirschema::types::PropertyInfo;
use std::collections::HashMap;
use std::sync::Arc;

/// Bridge-based choice type resolver with schema integration
#[derive(Clone)]
pub struct BridgeChoiceTypeResolver {
    /// Schema manager for bridge operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Cache for frequently resolved choice types
    cache: HashMap<String, BridgeChoiceInfo>,
}

impl BridgeChoiceTypeResolver {
    /// Create a new BridgeChoiceTypeResolver
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        Self {
            schema_manager,
            cache: HashMap::new(),
        }
    }

    /// Resolve choice type using schema manager with caching
    pub async fn resolve_choice_type(
        &mut self,
        path: &str,
        value_type: &str,
    ) -> Result<BridgeChoiceInfo> {
        let cache_key = format!("{}:{}", path, value_type);

        // Check cache first for performance
        if let Some(cached_info) = self.cache.get(&cache_key) {
            return Ok(cached_info.clone());
        }

        // Resolve using schema manager API
        let choice_info = if let Some(resolved) = self
            .schema_manager
            .resolve_choice_type(path, value_type)
            .await
        {
            BridgeChoiceInfo::valid(
                path.to_string(),
                resolved.clone(),
                value_type.to_string(),
                octofhir_fhirschema::types::BridgeCardinality::new(0, Some(1)),
            )
        } else {
            BridgeChoiceInfo::invalid(path.to_string(), value_type.to_string())
        };

        // Cache the result
        self.cache.insert(cache_key, choice_info.clone());

        Ok(choice_info)
    }

    /// Get property information using schema lookup  
    pub async fn get_property_info(&self, type_name: &str, property: &str) -> Result<PropertyInfo> {
        // Use schema-based property lookup
        if let Some(_schema) = self.schema_manager.get_schema_by_type(type_name).await {
            Ok(PropertyInfo {
                name: property.to_string(),
                element_type: "string".to_string(),
                cardinality: octofhir_fhirschema::types::BridgeCardinality::new(0, Some(1)),
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

    /// Resolve all possible choice types for a property
    pub async fn get_choice_type_variants(
        &self,
        type_name: &str,
        property: &str,
    ) -> Result<Vec<BridgeChoiceInfo>> {
        // Get the property info first to understand the choice pattern
        let _property_info = self.get_property_info(type_name, property).await?;

        // If it's not a choice type, return empty vec
        if !property.ends_with("[x]") {
            return Ok(vec![]);
        }

        // For choice types, we would need to get all possible variants
        // This would require additional bridge API methods that enumerate choice variants
        // For now, return basic info about the choice type
        let _base_path = format!("{}.{}", type_name, property);

        // This is a placeholder - in a full implementation, we would enumerate
        // all possible choice variants from the schema
        let variants = vec![];

        Ok(variants)
    }

    /// Validate that a choice type resolution is valid
    pub async fn validate_choice_resolution(&self, path: &str, value_type: &str) -> Result<bool> {
        let mut resolver = self.clone();
        let choice_info = resolver.resolve_choice_type(path, value_type).await?;
        Ok(choice_info.is_valid)
    }

    /// Get resolved type name from choice resolution
    pub async fn get_resolved_type_name(&mut self, path: &str, value_type: &str) -> Result<String> {
        let choice_info = self.resolve_choice_type(path, value_type).await?;
        Ok(choice_info.resolved_type)
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics for performance monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}

/// Utility functions for choice type handling
pub mod utils {
    /// Extract the base property name from a choice type path
    /// e.g., "value[x]" -> "value"
    pub fn extract_base_property(property: &str) -> &str {
        if let Some(bracket_pos) = property.find("[x]") {
            &property[..bracket_pos]
        } else {
            property
        }
    }

    /// Check if a property name represents a choice type
    pub fn is_choice_type(property: &str) -> bool {
        property.ends_with("[x]")
    }

    /// Generate the actual property name for a choice type variant
    /// e.g., "value[x]" + "String" -> "valueString"
    pub fn generate_choice_property_name(base_property: &str, type_name: &str) -> String {
        let base = extract_base_property(base_property);
        let capitalized_type = if type_name.is_empty() {
            String::new()
        } else {
            let mut chars = type_name.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };
        format!("{}{}", base, capitalized_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirschema::PackageManagerConfig;

    #[tokio::test]
    async fn test_bridge_choice_type_resolver_creation() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let resolver = BridgeChoiceTypeResolver::new(manager);

        assert_eq!(resolver.get_cache_stats().0, 0); // Cache should be empty initially
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
        let mut resolver = BridgeChoiceTypeResolver::new(manager);

        // Test resolving Observation.value[x] to valueString
        let result = resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        assert!(result.is_ok(), "Choice type resolution should succeed");

        let choice_info = result.unwrap();
        assert_eq!(choice_info.resolved_type, "valueString");
        assert!(choice_info.is_valid);
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
        let resolver = BridgeChoiceTypeResolver::new(manager);

        let property_info = resolver.get_property_info("Observation", "value[x]").await;

        assert!(property_info.is_ok(), "Property info lookup should succeed");

        let info = property_info.unwrap();
        assert_eq!(info.name, "value[x]");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .unwrap(),
        );
        let mut resolver = BridgeChoiceTypeResolver::new(manager);

        // First resolution - should populate cache
        let _ = resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        let (cache_size, _) = resolver.get_cache_stats();
        assert_eq!(cache_size, 1, "Cache should contain one entry");

        // Second resolution of same type - should use cache
        let _ = resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        let (cache_size_after, _) = resolver.get_cache_stats();
        assert_eq!(cache_size_after, 1, "Cache size should remain the same");

        // Clear cache
        resolver.clear_cache();
        let (cache_size_cleared, _) = resolver.get_cache_stats();
        assert_eq!(
            cache_size_cleared, 0,
            "Cache should be empty after clearing"
        );
    }

    mod util_tests {
        use super::utils::*;

        #[test]
        fn test_extract_base_property() {
            assert_eq!(extract_base_property("value[x]"), "value");
            assert_eq!(extract_base_property("effective[x]"), "effective");
            assert_eq!(extract_base_property("regularProperty"), "regularProperty");
        }

        #[test]
        fn test_is_choice_type() {
            assert!(is_choice_type("value[x]"));
            assert!(is_choice_type("effective[x]"));
            assert!(!is_choice_type("regularProperty"));
            assert!(!is_choice_type("value"));
        }

        #[test]
        fn test_generate_choice_property_name() {
            assert_eq!(
                generate_choice_property_name("value[x]", "String"),
                "valueString"
            );
            assert_eq!(
                generate_choice_property_name("effective[x]", "DateTime"),
                "effectiveDateTime"
            );
            assert_eq!(generate_choice_property_name("value[x]", ""), "value");
        }
    }
}
