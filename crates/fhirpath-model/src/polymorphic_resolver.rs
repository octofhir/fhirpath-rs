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

//! Polymorphic Path Resolution Engine for FHIR Choice Types
//!
//! This module provides the core engine for resolving polymorphic paths that handle
//! choice types (value[x] patterns) and enables proper navigation of FHIR resources
//! with polymorphic properties.

use dashmap::DashMap;
use regex::Regex;
use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::provider::{ModelError, ModelProvider, TypeReflectionInfo};
use crate::value::FhirPathValue;
use crate::{ChoiceTypeMapper, FhirPathError};

/// Main engine for resolving polymorphic paths
#[derive(Debug)]
pub struct PolymorphicPathResolver {
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,

    /// Choice type mapper for registered patterns
    choice_mapper: Option<Arc<ChoiceTypeMapper>>,

    /// Schema-discovered choice patterns
    schema_patterns: HashMap<String, Vec<SchemaChoicePattern>>,

    /// Cache for resolved paths
    path_cache: Arc<DashMap<String, ResolvedPath>>,

    /// Regex patterns for choice type detection
    choice_pattern_regex: Option<Regex>,

    /// Statistics for cache performance
    cache_stats: CacheStats,
}

/// Pattern discovered from FHIRSchema analysis
#[derive(Debug, Clone)]
pub struct SchemaChoicePattern {
    /// Resource type (e.g., "Observation")
    pub resource_type: String,

    /// Base property name (e.g., "value")
    pub base_property: String,

    /// Specific property variants (e.g., ["valueQuantity", "valueString"])
    pub variants: Vec<SchemaChoiceVariant>,

    /// Schema source information
    pub schema_path: String,
}

/// Individual choice variant from schema
#[derive(Debug, Clone)]
pub struct SchemaChoiceVariant {
    /// Full property name (e.g., "valueQuantity")
    pub property_name: String,

    /// FHIR type reference (e.g., "#/types/Quantity")
    pub type_ref: String,

    /// Resolved type name (e.g., "Quantity")
    pub resolved_type: String,

    /// Whether this variant is required/optional
    pub required: bool,

    /// Cardinality information (e.g., "0..1", "1..1")
    pub cardinality: String,
}

/// Concrete resolved path with value
#[derive(Debug, Clone)]
pub struct ConcretePath {
    /// Full property path (e.g., "valueQuantity.unit")
    pub path: String,

    /// Extracted value if available
    pub value: Option<Value>,

    /// Type information
    pub type_info: String,

    /// Whether this path exists in the data
    pub exists: bool,
}

/// Result of path resolution including metadata
#[derive(Debug, Clone)]
pub struct ResolvedPath {
    /// Original path that was requested
    pub original_path: String,

    /// Resolved path (may be different for choice types)
    pub resolved_path: String,

    /// Expected result type
    pub result_type: Option<TypeReflectionInfo>,

    /// Whether this was a choice type resolution
    pub is_choice_resolution: bool,

    /// Available alternatives (for choice types)
    pub alternatives: Vec<String>,

    /// Concrete resolved paths with values
    pub concrete_paths: Vec<ConcretePath>,
}

impl PolymorphicPathResolver {
    /// Create new resolver
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            model_provider,
            choice_mapper: None,
            schema_patterns: HashMap::new(),
            path_cache: Arc::new(DashMap::new()),
            choice_pattern_regex: None,
            cache_stats: CacheStats::default(),
        }
    }

    /// Create resolver with choice type mapper
    pub fn new_with_mapper(
        model_provider: Arc<dyn ModelProvider>,
        choice_mapper: Arc<ChoiceTypeMapper>,
    ) -> Self {
        Self {
            model_provider,
            choice_mapper: Some(choice_mapper),
            schema_patterns: HashMap::new(),
            path_cache: Arc::new(DashMap::new()),
            choice_pattern_regex: Some(Regex::new(r"^([a-z][a-zA-Z]*?)([A-Z][a-zA-Z]*)$").unwrap()),
            cache_stats: CacheStats::default(),
        }
    }

    /// Create resolver with FHIRSchema discovery
    pub async fn new_with_schema_discovery(
        model_provider: Arc<dyn ModelProvider>,
        schema: &Value,
        choice_mapper: Option<Arc<ChoiceTypeMapper>>,
    ) -> Result<Self, FhirPathError> {
        let mut resolver = Self {
            model_provider,
            choice_mapper,
            schema_patterns: HashMap::new(),
            path_cache: Arc::new(DashMap::new()),
            choice_pattern_regex: Some(
                Regex::new(r"^([a-z][a-zA-Z]*?)([A-Z][a-zA-Z]*)$")
                    .map_err(|e| FhirPathError::invalid_expression(e.to_string()))?,
            ),
            cache_stats: CacheStats::default(),
        };

        // Discover choice types from schema
        resolver.discover_choice_types_from_schema(schema).await?;

        Ok(resolver)
    }

    /// Discover choice types from FHIRSchema
    pub async fn discover_choice_types_from_schema(
        &mut self,
        schema: &Value,
    ) -> Result<(), FhirPathError> {
        self.cache_stats.discovery_runs += 1;

        let types = schema
            .get("types")
            .and_then(|t| t.as_object())
            .ok_or_else(|| FhirPathError::invalid_expression("Schema missing types"))?;

        for (type_name, type_def) in types {
            if self.is_resource_or_element_type(type_def) {
                self.discover_choice_properties_for_type(type_name, type_def)
                    .await?;
            }
        }

        Ok(())
    }

    /// Check if type definition represents a resource or complex element
    fn is_resource_or_element_type(&self, type_def: &Value) -> bool {
        // Has properties (complex type) and either no base or FHIR base types
        if let Some(properties) = type_def.get("properties") {
            if properties.as_object().is_some() {
                // Check if it's a resource or domain resource
                if let Some(resource_type) = type_def.get("resourceType") {
                    return resource_type.as_str().is_some();
                }

                // Check for inheritance from known FHIR types
                if let Some(base_type) = type_def.get("type") {
                    if let Some(base_str) = base_type.as_str() {
                        return matches!(
                            base_str,
                            "Resource" | "DomainResource" | "Element" | "BackboneElement"
                        );
                    }
                }

                // Default to true for complex types with properties
                return true;
            }
        }
        false
    }

    /// Discover choice properties for a specific type
    async fn discover_choice_properties_for_type(
        &mut self,
        type_name: &str,
        type_def: &Value,
    ) -> Result<(), FhirPathError> {
        let properties = type_def
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or_else(|| FhirPathError::invalid_expression("Type missing properties"))?;

        // Group properties by potential base names
        let mut property_groups: HashMap<String, Vec<(String, &Value)>> = HashMap::new();

        for (prop_name, prop_def) in properties {
            if let Some(base_name) = self.extract_choice_base_name(prop_name) {
                property_groups
                    .entry(base_name)
                    .or_default()
                    .push((prop_name.to_string(), prop_def));
            }
        }

        // Process each group to identify true choice types
        for (base_name, prop_list) in property_groups {
            if prop_list.len() > 1 {
                // Multiple variants = choice type
                self.process_choice_group(type_name, &base_name, &prop_list)
                    .await?;
            }
        }

        Ok(())
    }

    /// Extract potential base name from property (e.g., "valueQuantity" -> "value")
    fn extract_choice_base_name(&self, property_name: &str) -> Option<String> {
        if let Some(ref regex) = self.choice_pattern_regex {
            if let Some(captures) = regex.captures(property_name) {
                let potential_base = captures.get(1)?.as_str();

                // Convert camelCase to lowercase base
                // e.g., "valueQuantity" -> "value", "effectiveDateTime" -> "effective"
                let base = if potential_base.len() > 1 {
                    let first_char = potential_base.chars().next()?.to_lowercase().to_string();
                    let rest = &potential_base[1..];

                    // Find where the base likely ends (before the type suffix)
                    let lowercase_end = rest
                        .chars()
                        .position(|c| c.is_uppercase())
                        .unwrap_or(rest.len());

                    format!("{}{}", first_char, &rest[..lowercase_end])
                } else {
                    potential_base.to_lowercase()
                };

                Some(base)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Process a group of properties that appear to be choice variants
    async fn process_choice_group(
        &mut self,
        resource_type: &str,
        base_property: &str,
        properties: &[(String, &Value)],
    ) -> Result<(), FhirPathError> {
        let mut variants = Vec::new();

        for (prop_name, prop_def) in properties {
            let variant = self.create_schema_variant(prop_name, prop_def).await?;
            variants.push(variant);
        }

        if !variants.is_empty() {
            let pattern = SchemaChoicePattern {
                resource_type: resource_type.to_string(),
                base_property: base_property.to_string(),
                variants,
                schema_path: format!("types.{resource_type}.properties"),
            };

            let key = format!("{resource_type}.{base_property}");
            self.schema_patterns.insert(key, vec![pattern]);
            self.cache_stats.patterns_discovered += 1;
        }

        Ok(())
    }

    /// Create schema variant from property definition
    async fn create_schema_variant(
        &self,
        prop_name: &str,
        prop_def: &Value,
    ) -> Result<SchemaChoiceVariant, FhirPathError> {
        let type_ref = self.extract_type_reference(prop_def)?;
        let resolved_type = self.resolve_type_from_reference(&type_ref)?;
        let required = self.is_property_required(prop_def);
        let cardinality = self.extract_cardinality(prop_def);

        Ok(SchemaChoiceVariant {
            property_name: prop_name.to_string(),
            type_ref,
            resolved_type,
            required,
            cardinality,
        })
    }

    /// Extract type reference from property definition
    fn extract_type_reference(&self, prop_def: &Value) -> Result<String, FhirPathError> {
        // Try different schema patterns for type references
        if let Some(type_ref) = prop_def.get("$ref") {
            if let Some(ref_str) = type_ref.as_str() {
                return Ok(ref_str.to_string());
            }
        }

        if let Some(items) = prop_def.get("items") {
            if let Some(type_ref) = items.get("$ref") {
                if let Some(ref_str) = type_ref.as_str() {
                    return Ok(ref_str.to_string());
                }
            }
        }

        if let Some(type_name) = prop_def.get("type") {
            if let Some(type_str) = type_name.as_str() {
                return Ok(format!("#/types/{type_str}"));
            }
        }

        // Default to Element if no specific type found
        Ok("#/types/Element".to_string())
    }

    /// Resolve type name from schema reference
    fn resolve_type_from_reference(&self, type_ref: &str) -> Result<String, FhirPathError> {
        if let Some(type_name) = type_ref.strip_prefix("#/types/") {
            Ok(type_name.to_string())
        } else if type_ref.starts_with('#') {
            // Other internal reference formats
            Ok(type_ref.trim_start_matches('#').replace('/', "."))
        } else {
            Ok(type_ref.to_string())
        }
    }

    /// Check if property is required
    fn is_property_required(&self, prop_def: &Value) -> bool {
        prop_def
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false)
    }

    /// Extract cardinality information
    fn extract_cardinality(&self, prop_def: &Value) -> String {
        if let Some(min_items) = prop_def.get("minItems") {
            if let Some(max_items) = prop_def.get("maxItems") {
                return format!("{min_items}..{max_items}");
            }
        }

        // Default cardinality for choice types is typically 0..1
        "0..1".to_string()
    }

    /// Resolve a polymorphic path against actual data
    pub async fn resolve_path(
        &self,
        base_type: &str,
        path: &str,
        data: Option<&FhirPathValue>,
    ) -> Result<ResolvedPath, ModelError> {
        // Check cache first
        let cache_key = format!("{base_type}#{path}");
        if let Some(cached) = self.path_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Parse path into segments
        let segments: Vec<&str> = path.split('.').collect();
        let resolved = self
            .resolve_path_segments(base_type, &segments, data)
            .await?;

        // Cache the result
        self.path_cache.insert(cache_key, resolved.clone());

        Ok(resolved)
    }

    /// Resolve path segments recursively
    fn resolve_path_segments<'a>(
        &'a self,
        current_type: &'a str,
        segments: &'a [&str],
        data: Option<&'a FhirPathValue>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ResolvedPath, ModelError>> + Send + 'a>,
    > {
        Box::pin(async move {
            if segments.is_empty() {
                return Ok(ResolvedPath {
                    original_path: String::new(),
                    resolved_path: String::new(),
                    result_type: self.model_provider.get_type_reflection(current_type).await,
                    is_choice_resolution: false,
                    alternatives: vec![],
                    concrete_paths: vec![],
                });
            }

            let current_segment = segments[0];
            let remaining_segments = &segments[1..];

            // Check if this segment represents a choice type
            if let Some(resolved_property) = self
                .resolve_choice_segment(current_type, current_segment, data)
                .await?
            {
                // This is a choice type - resolve with actual property
                let resolved_path_part = if remaining_segments.is_empty() {
                    resolved_property.clone()
                } else {
                    format!("{}.{}", resolved_property, remaining_segments.join("."))
                };

                // Get the target type for the resolved property
                let target_type = self
                    .get_property_target_type(current_type, &resolved_property)
                    .await?;

                // If there are more segments, recursively resolve
                let alternatives = self
                    .get_choice_alternatives(current_type, current_segment)
                    .await;
                let final_result = if !remaining_segments.is_empty() && target_type.is_some() {
                    let nested_result = self
                        .resolve_path_segments(&target_type.unwrap(), remaining_segments, data)
                        .await?;
                    ResolvedPath {
                        original_path: segments.join("."),
                        resolved_path: format!(
                            "{}.{}",
                            resolved_property, nested_result.resolved_path
                        ),
                        result_type: nested_result.result_type,
                        is_choice_resolution: true,
                        alternatives,
                        concrete_paths: vec![],
                    }
                } else {
                    ResolvedPath {
                        original_path: segments.join("."),
                        resolved_path: resolved_path_part.clone(),
                        result_type: self
                            .get_property_type_info(current_type, &resolved_property)
                            .await,
                        is_choice_resolution: true,
                        alternatives,
                        concrete_paths: vec![],
                    }
                };

                Ok(final_result)
            } else {
                // Regular property resolution
                self.resolve_regular_property(
                    current_type,
                    current_segment,
                    remaining_segments,
                    data,
                )
                .await
            }
        })
    }

    /// Resolve a potential choice type segment
    async fn resolve_choice_segment(
        &self,
        resource_type: &str,
        segment: &str,
        data: Option<&FhirPathValue>,
    ) -> Result<Option<String>, ModelError> {
        // Check if this segment is a choice type base property
        if self
            .model_provider
            .is_choice_property(resource_type, segment)
            .await
        {
            // If we have data, use it to determine the specific property
            if let Some(fhir_value) = data {
                if let Some(resolved) = self
                    .model_provider
                    .resolve_choice_property(resource_type, segment, fhir_value)
                    .await
                {
                    return Ok(Some(resolved));
                }
            }

            // No data available or no matching property found - return highest priority variant
            let variants = self
                .model_provider
                .get_choice_variants(resource_type, segment)
                .await;
            if variants.is_empty() {
                return Err(ModelError::validation_error(format!(
                    "No variants found for choice property {resource_type}.{segment}"
                )));
            }

            let best_variant = variants
                .iter()
                .min_by_key(|v| v.priority)
                .map(|v| v.property_name.clone());

            return Ok(best_variant);
        }

        Ok(None)
    }

    /// Resolve regular (non-choice) property
    async fn resolve_regular_property(
        &self,
        current_type: &str,
        segment: &str,
        remaining_segments: &[&str],
        data: Option<&FhirPathValue>,
    ) -> Result<ResolvedPath, ModelError> {
        // Get property type information
        let property_type = self.get_property_target_type(current_type, segment).await?;

        if remaining_segments.is_empty() {
            // Final segment
            Ok(ResolvedPath {
                original_path: segment.to_string(),
                resolved_path: segment.to_string(),
                result_type: self.get_property_type_info(current_type, segment).await,
                is_choice_resolution: false,
                alternatives: vec![],
                concrete_paths: vec![],
            })
        } else if let Some(target_type) = property_type {
            // Continue with remaining segments
            let remaining_result = self
                .resolve_path_segments(&target_type, remaining_segments, data)
                .await?;

            Ok(ResolvedPath {
                original_path: format!("{}.{}", segment, remaining_result.original_path),
                resolved_path: format!("{}.{}", segment, remaining_result.resolved_path),
                result_type: remaining_result.result_type,
                is_choice_resolution: remaining_result.is_choice_resolution,
                alternatives: remaining_result.alternatives,
                concrete_paths: remaining_result.concrete_paths,
            })
        } else {
            Err(ModelError::validation_error(format!(
                "Property '{segment}' not found on type '{current_type}'"
            )))
        }
    }

    /// Get target type for a property
    async fn get_property_target_type(
        &self,
        type_name: &str,
        property: &str,
    ) -> Result<Option<String>, ModelError> {
        // First check if it's a choice type property
        if self
            .model_provider
            .is_choice_property(type_name, property)
            .await
        {
            let variants = self
                .model_provider
                .get_choice_variants(type_name, property)
                .await;
            if let Some(best_variant) = variants.iter().min_by_key(|v| v.priority) {
                return Ok(Some(best_variant.target_type.clone()));
            }
        }

        // Use model provider for regular properties
        if let Some(type_info) = self
            .model_provider
            .get_element_reflection(type_name, property)
            .await
        {
            match type_info {
                TypeReflectionInfo::SimpleType { name, .. } => Ok(Some(name)),
                TypeReflectionInfo::ClassInfo { name, .. } => Ok(Some(name)),
                // Note: Adjust these variant names based on actual TypeReflectionInfo definition
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Get type reflection info for a property
    async fn get_property_type_info(
        &self,
        type_name: &str,
        property: &str,
    ) -> Option<TypeReflectionInfo> {
        self.model_provider
            .get_element_reflection(type_name, property)
            .await
    }

    /// Get alternative choice properties  
    async fn get_choice_alternatives(
        &self,
        resource_type: &str,
        base_property: &str,
    ) -> Vec<String> {
        let variants = self
            .model_provider
            .get_choice_variants(resource_type, base_property)
            .await;
        variants.iter().map(|v| v.property_name.clone()).collect()
    }

    /// Clear path cache
    pub fn clear_cache(&self) {
        self.path_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.path_cache.len(),
            memory_usage_estimate: self.path_cache.len() * std::mem::size_of::<ResolvedPath>(),
            cache_hits: self.cache_stats.cache_hits,
            cache_misses: self.cache_stats.cache_misses,
            discovery_runs: self.cache_stats.discovery_runs,
            patterns_discovered: self.cache_stats.patterns_discovered,
        }
    }

    /// Resolve path using both mapper and schema discovery
    pub async fn resolve_path_enhanced(
        &self,
        resource_type: &str,
        path: &str,
        json_data: Option<&Value>,
    ) -> Result<ResolvedPath, FhirPathError> {
        let cache_key = format!("enhanced:{resource_type}:{path}");

        // Check cache first
        if let Some(cached) = self.path_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Split path into segments
        let path_parts: Vec<&str> = path.split('.').collect();
        if path_parts.is_empty() {
            return Ok(ResolvedPath {
                original_path: path.to_string(),
                resolved_path: path.to_string(),
                result_type: None,
                is_choice_resolution: false,
                alternatives: vec![],
                concrete_paths: vec![],
            });
        }

        let base_property = path_parts[0];
        let mut concrete_paths = Vec::new();

        // Check choice mapper first
        if let Some(ref choice_mapper) = self.choice_mapper {
            if choice_mapper.is_choice_property(resource_type, base_property) {
                let variants = choice_mapper.get_variant_properties(resource_type, base_property);

                for variant in variants {
                    let concrete_path = if path_parts.len() > 1 {
                        format!("{}.{}", variant, path_parts[1..].join("."))
                    } else {
                        variant.clone()
                    };

                    let type_info = choice_mapper
                        .get_target_type(resource_type, &variant)
                        .unwrap_or_else(|| "Element".to_string());

                    let exists = if let Some(data) = json_data {
                        self.path_exists_in_data(&concrete_path, data)
                    } else {
                        false
                    };

                    let value = if let Some(data) = json_data.filter(|_| exists) {
                        self.extract_value_at_path(&concrete_path, data)
                    } else {
                        None
                    };

                    concrete_paths.push(ConcretePath {
                        path: concrete_path,
                        value,
                        type_info,
                        exists,
                    });
                }
            }
        }

        // Check schema-discovered patterns
        let schema_key = format!("{resource_type}.{base_property}");
        if let Some(schema_patterns) = self.schema_patterns.get(&schema_key) {
            for pattern in schema_patterns {
                for variant in &pattern.variants {
                    let concrete_path = if path_parts.len() > 1 {
                        format!("{}.{}", variant.property_name, path_parts[1..].join("."))
                    } else {
                        variant.property_name.clone()
                    };

                    let exists = if let Some(data) = json_data {
                        self.path_exists_in_data(&concrete_path, data)
                    } else {
                        false
                    };

                    let value = if let Some(data) = json_data.filter(|_| exists) {
                        self.extract_value_at_path(&concrete_path, data)
                    } else {
                        None
                    };

                    // Avoid duplicates from both sources
                    if !concrete_paths.iter().any(|cp| cp.path == concrete_path) {
                        concrete_paths.push(ConcretePath {
                            path: concrete_path,
                            value,
                            type_info: variant.resolved_type.clone(),
                            exists,
                        });
                    }
                }
            }
        }

        // If no choice resolution found, return original path
        if concrete_paths.is_empty() {
            let exists = if let Some(data) = json_data {
                self.path_exists_in_data(path, data)
            } else {
                false
            };

            let value = if let Some(data) = json_data.filter(|_| exists) {
                self.extract_value_at_path(path, data)
            } else {
                None
            };

            concrete_paths.push(ConcretePath {
                path: path.to_string(),
                value,
                type_info: "Element".to_string(),
                exists,
            });
        }

        let alternatives: Vec<String> = concrete_paths.iter().map(|cp| cp.path.clone()).collect();

        let resolved = concrete_paths
            .iter()
            .find(|cp| cp.exists)
            .map(|cp| cp.path.clone())
            .unwrap_or_else(|| concrete_paths[0].path.clone());

        let resolved_path = ResolvedPath {
            original_path: path.to_string(),
            resolved_path: resolved,
            result_type: None, // Would need model provider integration
            is_choice_resolution: concrete_paths.len() > 1,
            alternatives,
            concrete_paths: concrete_paths.clone(),
        };

        // Cache the result
        self.path_cache.insert(cache_key, resolved_path.clone());

        Ok(resolved_path)
    }

    /// Check if path exists in JSON data
    fn path_exists_in_data(&self, path: &str, data: &Value) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                return false;
            }
        }
        true
    }

    /// Extract value at path from JSON data
    fn extract_value_at_path(&self, path: &str, data: &Value) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                return None;
            }
        }
        Some(current.clone())
    }

    /// Get discovered schema patterns
    pub fn get_schema_patterns(&self) -> &HashMap<String, Vec<SchemaChoicePattern>> {
        &self.schema_patterns
    }

    /// Check if a path requires polymorphic resolution
    pub async fn requires_polymorphic_resolution(&self, base_type: &str, path: &str) -> bool {
        let segments: Vec<&str> = path.split('.').collect();
        self.check_segments_for_choice_types(base_type.to_string(), &segments)
            .await
    }

    /// Check if any segment in a path involves choice types
    async fn check_segments_for_choice_types(
        &self,
        mut current_type: String,
        segments: &[&str],
    ) -> bool {
        for segment in segments {
            if self
                .model_provider
                .is_choice_property(&current_type, segment)
                .await
            {
                return true;
            }

            // Update current type for next iteration (simplified - would need proper type resolution)
            if self
                .model_provider
                .is_choice_property(&current_type, segment)
                .await
            {
                let variants = self
                    .model_provider
                    .get_choice_variants(&current_type, segment)
                    .await;
                if let Some(best_variant) = variants.iter().min_by_key(|v| v.priority) {
                    current_type = best_variant.target_type.clone();
                }
            } else if let Ok(Some(target_type)) =
                self.get_property_target_type(&current_type, segment).await
            {
                current_type = target_type;
            }
        }
        false
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Estimated memory usage in bytes
    pub memory_usage_estimate: usize,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Number of discovery runs
    pub discovery_runs: u64,
    /// Patterns discovered from schema
    pub patterns_discovered: u64,
}

// Re-export factory utilities from the dedicated module
pub use crate::polymorphic_factory::PolymorphicResolverFactory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_provider::MockModelProvider;
    use sonic_rs::json;

    async fn create_test_resolver() -> PolymorphicPathResolver {
        let model_provider = Arc::new(MockModelProvider::new());
        PolymorphicResolverFactory::create_default(model_provider)
    }

    #[tokio::test]
    async fn test_basic_choice_resolution() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        let observation_data = FhirPathValue::resource_from_json(json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        }));

        let resolved = resolver
            .resolve_path("Observation", "value", Some(&observation_data))
            .await?;

        assert!(resolved.is_choice_resolution);
        assert_eq!(resolved.resolved_path, "valueQuantity");
        assert!(resolved.alternatives.contains(&"valueQuantity".to_string()));
        assert!(resolved.alternatives.contains(&"valueString".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_nested_choice_resolution() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        let observation_data = FhirPathValue::resource_from_json(json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        }));

        let resolved = resolver
            .resolve_path("Observation", "value.unit", Some(&observation_data))
            .await?;

        assert!(resolved.is_choice_resolution);
        assert_eq!(resolved.resolved_path, "valueQuantity.unit");

        Ok(())
    }

    #[tokio::test]
    async fn test_no_data_fallback() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        let resolved = resolver
            .resolve_path(
                "Observation",
                "value",
                None, // No data provided
            )
            .await?;

        assert!(resolved.is_choice_resolution);
        assert_eq!(resolved.resolved_path, "valueQuantity"); // Highest priority

        Ok(())
    }

    #[tokio::test]
    async fn test_regular_property_resolution() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        let resolved = resolver.resolve_path("Patient", "name", None).await?;

        assert!(!resolved.is_choice_resolution);
        assert_eq!(resolved.resolved_path, "name");
        assert!(resolved.alternatives.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        // First resolution - should cache
        let _resolved1 = resolver.resolve_path("Observation", "value", None).await?;

        let stats = resolver.get_cache_stats();
        assert_eq!(stats.total_entries, 1);

        // Second resolution - should use cache
        let _resolved2 = resolver.resolve_path("Observation", "value", None).await?;

        let stats2 = resolver.get_cache_stats();
        assert_eq!(stats2.total_entries, 1); // Same entry

        Ok(())
    }

    #[tokio::test]
    async fn test_requires_polymorphic_resolution() {
        let resolver = create_test_resolver().await;

        assert!(
            resolver
                .requires_polymorphic_resolution("Observation", "value")
                .await
        );
        assert!(
            resolver
                .requires_polymorphic_resolution("Observation", "value.unit")
                .await
        );
        assert!(
            !resolver
                .requires_polymorphic_resolution("Patient", "name")
                .await
        );
        assert!(
            !resolver
                .requires_polymorphic_resolution("Patient", "name.given")
                .await
        );
    }

    #[tokio::test]
    async fn test_patient_deceased_choice() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        let patient_data = FhirPathValue::resource_from_json(json!({
            "resourceType": "Patient",
            "deceasedBoolean": true
        }));

        let resolved = resolver
            .resolve_path("Patient", "deceased", Some(&patient_data))
            .await?;

        assert!(resolved.is_choice_resolution);
        assert_eq!(resolved.resolved_path, "deceasedBoolean");
        assert!(
            resolved
                .alternatives
                .contains(&"deceasedBoolean".to_string())
        );
        assert!(
            resolved
                .alternatives
                .contains(&"deceasedDateTime".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_clear_cache() -> Result<(), Box<dyn std::error::Error>> {
        let resolver = create_test_resolver().await;

        let _resolved = resolver.resolve_path("Observation", "value", None).await?;

        let stats_before = resolver.get_cache_stats();
        assert_eq!(stats_before.total_entries, 1);

        resolver.clear_cache();

        let stats_after = resolver.get_cache_stats();
        assert_eq!(stats_after.total_entries, 0);

        Ok(())
    }
}
