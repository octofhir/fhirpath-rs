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

//! Schema caching layer for performance optimization
//!
//! This module provides caching for frequently accessed schema information
//! to reduce the overhead of bridge API calls during evaluation.

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{BridgeChoiceInfo, TypeInfo};
use octofhir_fhirschema::FhirSchemaPackageManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// High-performance caching layer for schema operations
#[derive(Clone)]
pub struct SchemaCache {
    /// Cache for property information (type.property -> String)
    property_cache: Arc<RwLock<HashMap<String, String>>>,
    /// Cache for choice type resolution (choice_key -> BridgeChoiceInfo)
    choice_cache: Arc<RwLock<HashMap<String, BridgeChoiceInfo>>>,
    /// Cache for type checking (type_name -> bool)
    resource_type_cache: Arc<RwLock<HashMap<String, bool>>>,
    primitive_type_cache: Arc<RwLock<HashMap<String, bool>>>,
    /// Cache for complete type information
    type_info_cache: Arc<RwLock<HashMap<String, TypeInfo>>>,
}

impl SchemaCache {
    /// Create a new empty schema cache
    pub fn new() -> Self {
        Self {
            property_cache: Arc::new(RwLock::new(HashMap::new())),
            choice_cache: Arc::new(RwLock::new(HashMap::new())),
            resource_type_cache: Arc::new(RwLock::new(HashMap::new())),
            primitive_type_cache: Arc::new(RwLock::new(HashMap::new())),
            type_info_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if property exists with caching
    pub async fn has_property(
        &self,
        schema_manager: &FhirSchemaPackageManager,
        type_name: &str,
        property: &str,
    ) -> bool {
        let cache_key = format!("{}.{}", type_name, property);

        // Check cache first
        {
            let cache = self.property_cache.read().await;
            if cache.contains_key(&cache_key) {
                return true; // Property exists (cached)
            }
        }

        // Check schema manager - simplified implementation
        // In practice this would use proper schema API
        let has_property = schema_manager.has_resource_type(type_name).await;

        // Cache the result if property exists
        if has_property {
            let mut cache = self.property_cache.write().await;
            cache.insert(cache_key, property.to_string());
        }

        has_property
    }

    /// Check if type is a resource type with caching
    pub async fn is_resource_type(
        &self,
        schema_manager: &FhirSchemaPackageManager,
        type_name: &str,
    ) -> bool {
        // Check cache first
        {
            let cache = self.resource_type_cache.read().await;
            if let Some(&result) = cache.get(type_name) {
                return result;
            }
        }

        // Fetch from schema manager
        let result = schema_manager.has_resource_type(type_name).await;

        // Cache the result
        {
            let mut cache = self.resource_type_cache.write().await;
            cache.insert(type_name.to_string(), result);
        }

        result
    }

    /// Check if type is a primitive type with caching
    pub async fn is_primitive_type(
        &self,
        schema_manager: &FhirSchemaPackageManager,
        type_name: &str,
    ) -> bool {
        // Check cache first
        {
            let cache = self.primitive_type_cache.read().await;
            if let Some(&result) = cache.get(type_name) {
                return result;
            }
        }

        // Fetch from schema manager
        let result = schema_manager.is_primitive_type(type_name).await;

        // Cache the result
        {
            let mut cache = self.primitive_type_cache.write().await;
            cache.insert(type_name.to_string(), result);
        }

        result
    }

    /// Get choice type information with caching
    pub async fn get_choice_info(
        &self,
        _schema_manager: &FhirSchemaPackageManager,
        base_path: &str,
        concrete_type: &str,
    ) -> EvaluationResult<BridgeChoiceInfo> {
        let cache_key = format!("{}:{}", base_path, concrete_type);

        // Check cache first
        {
            let cache = self.choice_cache.read().await;
            if let Some(info) = cache.get(&cache_key) {
                return Ok(info.clone());
            }
        }

        // Create a simplified choice info - in practice would use real bridge API
        let choice_info = BridgeChoiceInfo {
            original_path: base_path.to_string(),
            resolved_property: concrete_type.to_string(),
            resolved_type: concrete_type.to_string(),
            is_valid: true,
            possible_variants: vec![concrete_type.to_string()],
            cardinality: octofhir_fhirschema::types::BridgeCardinality {
                min: 0,
                max: Some(1),
            },
            metadata: None,
        };

        // Cache the result
        {
            let mut cache = self.choice_cache.write().await;
            cache.insert(cache_key, choice_info.clone());
        }

        Ok(choice_info)
    }

    /// Get comprehensive type information with caching
    pub async fn get_type_info(
        &self,
        schema_manager: &FhirSchemaPackageManager,
        type_name: &str,
    ) -> EvaluationResult<TypeInfo> {
        // Check cache first
        {
            let cache = self.type_info_cache.read().await;
            if let Some(info) = cache.get(type_name) {
                return Ok(info.clone());
            }
        }

        // Create type info based on schema manager data
        let type_info = if schema_manager.has_resource_type(type_name).await {
            TypeInfo {
                name: type_name.to_string(),
                is_resource: true,
                is_primitive: false,
                is_complex: false,
                namespace: "FHIR".to_string(),
                base_type: Some("DomainResource".to_string()), // Simplified
            }
        } else if schema_manager.is_primitive_type(type_name).await {
            TypeInfo {
                name: type_name.to_string(),
                is_resource: false,
                is_primitive: true,
                is_complex: false,
                namespace: "System".to_string(),
                base_type: None,
            }
        } else {
            return Err(EvaluationError::InvalidOperation {
                message: format!("Unknown type: {}", type_name),
            });
        };

        // Cache the result
        {
            let mut cache = self.type_info_cache.write().await;
            cache.insert(type_name.to_string(), type_info.clone());
        }

        Ok(type_info)
    }

    /// Check if a type is a subtype of another using schema manager
    pub async fn is_subtype_of(
        &self,
        schema_manager: &FhirSchemaPackageManager,
        child_type: &str,
        parent_type: &str,
    ) -> bool {
        // For now, use a simplified implementation
        // In practice, this would use the schema manager's inheritance checking
        if child_type == parent_type {
            return true;
        }

        // Check common FHIR inheritance patterns
        if parent_type == "DomainResource" && schema_manager.has_resource_type(child_type).await {
            return child_type != "Resource" && child_type != "Bundle";
        }

        if parent_type == "Resource" && schema_manager.has_resource_type(child_type).await {
            return true;
        }

        false
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut property_cache = self.property_cache.write().await;
        property_cache.clear();

        let mut choice_cache = self.choice_cache.write().await;
        choice_cache.clear();

        let mut resource_type_cache = self.resource_type_cache.write().await;
        resource_type_cache.clear();

        let mut primitive_type_cache = self.primitive_type_cache.write().await;
        primitive_type_cache.clear();

        let mut type_info_cache = self.type_info_cache.write().await;
        type_info_cache.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let property_count = self.property_cache.read().await.len();
        let choice_count = self.choice_cache.read().await.len();
        let resource_type_count = self.resource_type_cache.read().await.len();
        let primitive_type_count = self.primitive_type_cache.read().await.len();
        let type_info_count = self.type_info_cache.read().await.len();

        CacheStats {
            property_cache_entries: property_count,
            choice_cache_entries: choice_count,
            resource_type_cache_entries: resource_type_count,
            primitive_type_cache_entries: primitive_type_count,
            type_info_cache_entries: type_info_count,
        }
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about cache usage
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub property_cache_entries: usize,
    pub choice_cache_entries: usize,
    pub resource_type_cache_entries: usize,
    pub primitive_type_cache_entries: usize,
    pub type_info_cache_entries: usize,
}
