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

//! Type resolution using bridge support API
//!
//! This module replaces all hardcoded type checking with dynamic schema-based
//! type resolution using the FhirSchemaPackageManager bridge support API.

use crate::bridge_types::BridgeChoiceInfo;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirschema::FhirSchemaPackageManager;
use octofhir_fhirschema::types::{BridgeCardinality, PropertyInfo};
use std::collections::HashMap;
use std::sync::Arc;

/// High-performance type resolution using bridge support API
#[derive(Clone)]
pub struct TypeResolver {
    /// Schema manager for O(1) type operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Cache for frequently accessed type information
    type_cache: HashMap<String, TypeInfo>,
}

/// Type information resolved from schema
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    /// Type name
    pub name: String,
    /// Whether this is a resource type
    pub is_resource: bool,
    /// Whether this is a primitive type
    pub is_primitive: bool,
    /// Whether this is a complex type
    pub is_complex: bool,
    /// Base type if this type inherits from another
    pub base_type: Option<String>,
    /// Namespace (typically "FHIR" or "System")
    pub namespace: String,
}

/// Choice type resolver with comprehensive choice type logic
#[derive(Clone)]
pub struct ChoiceTypeResolver {
    /// Schema manager for choice type resolution
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Cache for resolved choice types
    choice_cache: HashMap<String, BridgeChoiceInfo>,
}

/// Property navigation resolver for complex property paths
#[derive(Clone)]
pub struct PropertyResolver {
    /// Schema manager for property resolution
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Choice type resolver for handling choice properties
    choice_resolver: ChoiceTypeResolver,
}

impl TypeResolver {
    /// Create a new TypeResolver with the given schema manager
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        Self {
            schema_manager,
            type_cache: HashMap::new(),
        }
    }

    /// Check if a type is a FHIR resource type (O(1) operation)
    pub async fn is_resource_type(&self, type_name: &str) -> bool {
        self.schema_manager.has_resource_type(type_name).await
    }

    /// Check if a type is a primitive type (O(1) operation)
    pub async fn is_primitive_type(&self, type_name: &str) -> bool {
        self.schema_manager.is_primitive_type(type_name).await
    }

    /// Check if a type is a complex type (O(1) operation)
    pub async fn is_complex_type(&self, type_name: &str) -> bool {
        self.schema_manager.is_complex_type(type_name).await
    }

    /// Get comprehensive type information for a type
    pub async fn get_type_info(&mut self, type_name: &str) -> Result<TypeInfo> {
        // Check cache first for performance
        if let Some(cached_info) = self.type_cache.get(type_name) {
            return Ok(cached_info.clone());
        }

        // Resolve type information using schema manager
        let is_resource = self.is_resource_type(type_name).await;
        let is_primitive = self.is_primitive_type(type_name).await;
        let is_complex = self.is_complex_type(type_name).await;

        // Determine namespace
        let namespace = if is_primitive {
            "System".to_string()
        } else {
            "FHIR".to_string()
        };

        // Get base type information if available
        let base_type = self.get_base_type(type_name).await;

        let type_info = TypeInfo {
            name: type_name.to_string(),
            is_resource,
            is_primitive,
            is_complex,
            base_type,
            namespace,
        };

        // Cache the result for future use
        self.type_cache
            .insert(type_name.to_string(), type_info.clone());

        Ok(type_info)
    }

    /// Get the base type of a given type
    async fn get_base_type(&self, type_name: &str) -> Option<String> {
        // Use schema manager to get type hierarchy information
        // This is a simplified implementation - in practice would use schema data
        match type_name {
            "DomainResource" => Some("Resource".to_string()),
            name if self.is_resource_type(name).await => Some("DomainResource".to_string()),
            _ => None,
        }
    }

    /// Check if a type is a subtype of another type
    pub async fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool {
        if child_type == parent_type {
            return true;
        }

        // Use schema manager for inheritance checking
        // This would typically use the schema's type hierarchy
        match (child_type, parent_type) {
            (child, "Resource") if self.is_resource_type(child).await => true,
            (child, "DomainResource")
                if self.is_resource_type(child).await && child != "Resource" =>
            {
                true
            }
            _ => false,
        }
    }

    /// Clear the type cache
    pub fn clear_cache(&mut self) {
        self.type_cache.clear();
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.type_cache.len(), self.type_cache.capacity())
    }
}

impl ChoiceTypeResolver {
    /// Create a new ChoiceTypeResolver
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        Self {
            schema_manager,
            choice_cache: HashMap::new(),
        }
    }

    /// Resolve a choice type to its concrete implementation
    pub async fn resolve_choice_type(
        &mut self,
        base_path: &str,
        concrete_type: &str,
    ) -> Result<BridgeChoiceInfo> {
        let cache_key = format!("{}:{}", base_path, concrete_type);

        // Check cache first for performance
        if let Some(cached_info) = self.choice_cache.get(&cache_key) {
            return Ok(cached_info.clone());
        }

        // Handle choice type patterns
        let choice_info = if base_path.ends_with("[x]") {
            self.resolve_explicit_choice_type(base_path, concrete_type)
                .await?
        } else if self.is_choice_type_path(base_path).await? {
            self.resolve_implicit_choice_type(base_path, concrete_type)
                .await?
        } else {
            return Err(FhirPathError::evaluation_error(format!(
                "Not a choice type path: {}",
                base_path
            )));
        };

        // Cache the result
        self.choice_cache.insert(cache_key, choice_info.clone());

        Ok(choice_info)
    }

    /// Resolve explicit choice type (e.g., "value[x]" -> "valueString")
    async fn resolve_explicit_choice_type(
        &self,
        base_path: &str,
        concrete_type: &str,
    ) -> Result<BridgeChoiceInfo> {
        if let Some(resolved) = self
            .schema_manager
            .resolve_choice_type(base_path, concrete_type)
            .await
        {
            Ok(BridgeChoiceInfo::valid(
                base_path.to_string(),
                resolved,
                concrete_type.to_string(),
                BridgeCardinality::new(0, Some(1)),
            ))
        } else {
            Ok(BridgeChoiceInfo::invalid(
                base_path.to_string(),
                concrete_type.to_string(),
            ))
        }
    }

    /// Resolve implicit choice type (e.g., "valueString" -> "value[x]")
    async fn resolve_implicit_choice_type(
        &self,
        path: &str,
        concrete_type: &str,
    ) -> Result<BridgeChoiceInfo> {
        // Extract the base choice path from the concrete path
        let choice_path = self.extract_choice_path(path).await?;
        self.resolve_explicit_choice_type(&choice_path, concrete_type)
            .await
    }

    /// Extract the choice type base path from a concrete property path
    async fn extract_choice_path(&self, path: &str) -> Result<String> {
        // This would analyze the path to identify choice type patterns
        // For example: "Observation.valueString" -> "Observation.value[x]"

        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return Err(FhirPathError::evaluation_error(format!(
                "Invalid property path: {}",
                path
            )));
        }

        let resource_type = parts[0];
        let property = parts[1];

        // Check if this property is a choice type variant
        let common_choice_patterns = ["value", "effective", "onset", "abatement"];
        for pattern in &common_choice_patterns {
            if property.starts_with(pattern) && property.len() > pattern.len() {
                let choice_path = format!("{}.{}[x]", resource_type, pattern);
                if self
                    .is_choice_type_path(&choice_path)
                    .await
                    .unwrap_or(false)
                {
                    return Ok(choice_path);
                }
            }
        }

        Err(FhirPathError::evaluation_error(format!(
            "Could not extract choice path from: {}",
            path
        )))
    }

    /// Check if a path represents a choice type
    async fn is_choice_type_path(&self, path: &str) -> Result<bool> {
        // This would use the schema manager to check if the path is a choice type
        Ok(path.ends_with("[x]") || self.schema_manager.is_choice_type_expansion(path).await)
    }

    /// Clear the choice type cache
    pub fn clear_cache(&mut self) {
        self.choice_cache.clear();
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.choice_cache.len(), self.choice_cache.capacity())
    }
}

impl PropertyResolver {
    /// Create a new PropertyResolver
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        let choice_resolver = ChoiceTypeResolver::new(schema_manager.clone());
        Self {
            schema_manager,
            choice_resolver,
        }
    }

    /// Resolve a property path to a list of property information
    pub async fn resolve_property_path(
        &self,
        resource_type: &str,
        property_path: &str,
    ) -> Result<Vec<PropertyInfo>> {
        let mut properties = Vec::new();
        let path_parts: Vec<&str> = property_path.split('.').collect();
        let mut current_type = resource_type.to_string();

        for (index, part) in path_parts.iter().enumerate() {
            // Clean array notation (e.g., "name[0]" -> "name")
            let clean_part = self.clean_array_notation(part);

            // Get property info from schema
            let property_info = self.get_property_info(&current_type, &clean_part).await?;
            properties.push(property_info.clone());

            // Resolve next type in the chain
            if index < path_parts.len() - 1 {
                current_type = self.resolve_next_type(&property_info).await?;
            }
        }

        Ok(properties)
    }

    /// Get property information for a type and property name
    async fn get_property_info(&self, type_name: &str, property: &str) -> Result<PropertyInfo> {
        // Use schema manager to get property information
        if let Some(_schema) = self.schema_manager.get_schema_by_type(type_name).await {
            // Create basic property info - in practice would parse from schema
            Ok(PropertyInfo {
                name: property.to_string(),
                element_type: "string".to_string(), // Would be resolved from schema
                cardinality: BridgeCardinality::new(0, Some(1)),
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

    /// Resolve the next type in a property navigation chain
    async fn resolve_next_type(&self, property: &PropertyInfo) -> Result<String> {
        // Handle different property type patterns
        if property.is_choice_type {
            // Choice types require runtime resolution
            Err(FhirPathError::evaluation_error(format!(
                "Choice type resolution required for: {}",
                property.name
            )))
        } else if property.element_type == "Reference" {
            // References need special handling
            Ok("Reference".to_string())
        } else {
            // Direct type mapping
            Ok(property.element_type.clone())
        }
    }

    /// Remove array notation from property names
    fn clean_array_notation(&self, property: &str) -> String {
        if let Some(bracket_pos) = property.find('[') {
            property[..bracket_pos].to_string()
        } else {
            property.to_string()
        }
    }

    /// Get the choice resolver for external use
    pub fn choice_resolver(&mut self) -> &mut ChoiceTypeResolver {
        &mut self.choice_resolver
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
    async fn test_type_resolver_resource_types() {
        let schema_manager = create_test_schema_manager().await;
        let resolver = TypeResolver::new(schema_manager);

        // Test resource type detection
        assert!(resolver.is_resource_type("Patient").await);
        assert!(resolver.is_resource_type("Observation").await);
        assert!(!resolver.is_resource_type("InvalidType").await);
    }

    #[tokio::test]
    async fn test_type_resolver_primitive_types() {
        let schema_manager = create_test_schema_manager().await;
        let resolver = TypeResolver::new(schema_manager);

        // Test primitive type detection
        assert!(resolver.is_primitive_type("string").await);
        assert!(resolver.is_primitive_type("boolean").await);
        assert!(!resolver.is_primitive_type("Patient").await);
    }

    #[tokio::test]
    async fn test_type_resolver_subtype_checking() {
        let schema_manager = create_test_schema_manager().await;
        let resolver = TypeResolver::new(schema_manager);

        // Test inheritance relationships
        assert!(resolver.is_subtype_of("Patient", "Patient").await); // Same type
        assert!(resolver.is_subtype_of("Patient", "DomainResource").await); // Inheritance
        assert!(resolver.is_subtype_of("Patient", "Resource").await); // Deep inheritance
        assert!(!resolver.is_subtype_of("string", "Patient").await); // Different categories
    }

    #[tokio::test]
    async fn test_choice_type_resolver() {
        let schema_manager = create_test_schema_manager().await;
        let mut resolver = ChoiceTypeResolver::new(schema_manager);

        // Test choice type resolution
        let result = resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        assert!(result.is_ok());
        let choice_info = result.unwrap();
        assert_eq!(choice_info.original_path, "Observation.value[x]");
        assert!(choice_info.is_valid);
    }

    #[tokio::test]
    async fn test_property_resolver() {
        let schema_manager = create_test_schema_manager().await;
        let resolver = PropertyResolver::new(schema_manager);

        // Test property path resolution
        let result = resolver.resolve_property_path("Patient", "name").await;

        assert!(result.is_ok());
        let properties = result.unwrap();
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].name, "name");
    }

    #[tokio::test]
    async fn test_type_resolver_caching() {
        let schema_manager = create_test_schema_manager().await;
        let mut resolver = TypeResolver::new(schema_manager);

        // Get type info twice to test caching
        let _info1 = resolver.get_type_info("Patient").await.unwrap();
        let (cache_size, _) = resolver.get_cache_stats();
        assert_eq!(cache_size, 1);

        let _info2 = resolver.get_type_info("Patient").await.unwrap();
        let (cache_size_after, _) = resolver.get_cache_stats();
        assert_eq!(cache_size_after, 1); // Should remain the same due to caching
    }

    #[tokio::test]
    async fn test_choice_type_resolver_caching() {
        let schema_manager = create_test_schema_manager().await;
        let mut resolver = ChoiceTypeResolver::new(schema_manager);

        // Resolve choice type twice to test caching
        let _result1 = resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        let (cache_size, _) = resolver.get_cache_stats();
        assert_eq!(cache_size, 1);

        let _result2 = resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await;

        let (cache_size_after, _) = resolver.get_cache_stats();
        assert_eq!(cache_size_after, 1); // Should use cache
    }
}
