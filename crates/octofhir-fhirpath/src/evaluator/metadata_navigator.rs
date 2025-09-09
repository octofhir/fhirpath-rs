//! Metadata-aware navigator implementation for FHIRPath evaluation
//!
//! This module provides navigation capabilities that maintain rich metadata
//! throughout property access and indexing operations.

use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    core::{FhirPathError, FhirPathValue, Result},
    evaluator::traits::MetadataAwareNavigator,
    path::CanonicalPath,
    typing::TypeResolver,
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue, collection_utils},
};

/// Global cache for property type lookups to avoid expensive ModelProvider calls
static PROPERTY_TYPE_CACHE: Lazy<RwLock<HashMap<(String, String), String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Metadata-aware navigator that maintains metadata during navigation
#[derive(Debug, Clone)]
pub struct MetadataNavigator;

impl MetadataNavigator {
    /// Create a new metadata-aware navigator
    pub fn new() -> Self {
        Self
    }

    /// Cached property type resolution to avoid expensive ModelProvider calls
    async fn get_property_type_cached(
        &self,
        resource_type: &str,
        property: &str,
        resolver: &TypeResolver,
    ) -> Result<String> {
        let cache_key = (resource_type.to_string(), property.to_string());

        // Check cache first (fast O(1) lookup)
        {
            let cache = PROPERTY_TYPE_CACHE.read();
            if let Some(cached_type) = cache.get(&cache_key) {
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 CACHE HIT: {}::{} -> {}",
                        resource_type, property, cached_type
                    );
                }
                return Ok(cached_type.clone());
            }
        }

        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
            eprintln!("🔍 CACHE MISS: Resolving {}::{}", resource_type, property);
        }

        // Not in cache - resolve and cache the result
        let resolve_start = std::time::Instant::now();
        // IMPORTANT: Don't fall back to "unknown" - this breaks polymorphic/choice type resolution
        match resolver
            .resolve_property_type(resource_type, property)
            .await
        {
            Ok(property_type) => {
                let resolve_time = resolve_start.elapsed();
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 RESOLVE SUCCESS: {}::{} -> {} ({}ms)",
                        resource_type,
                        property,
                        property_type,
                        resolve_time.as_millis()
                    );
                }
                // Cache successful resolution
                {
                    let mut cache = PROPERTY_TYPE_CACHE.write();
                    cache.insert(cache_key, property_type.clone());
                }
                Ok(property_type)
            }
            Err(e) => {
                let resolve_time = resolve_start.elapsed();
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 RESOLVE ERROR: {}::{} -> ERROR ({}ms): {}",
                        resource_type,
                        property,
                        resolve_time.as_millis(),
                        e
                    );
                }
                // Don't cache errors - let them bubble up for proper error handling
                // This is important for polymorphic fields like medicationReference
                Err(e)
            }
        }
    }

    /// Extract property value from JSON with metadata awareness
    async fn extract_property_from_json(
        &self,
        json: &JsonValue,
        property: &str,
        source_metadata: &ValueMetadata,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // OPTIMIZATION: First check if property exists in JSON (fast operation)
        match json.get(property) {
            Some(property_value) => {
                // AGGRESSIVE TYPE CASTING OPTIMIZATION: If metadata indicates strong typing from resourceType filtering,
                // first try to get the proper type from ModelProvider, fall back to JSON-direct property access
                let property_type = if source_metadata.resource_type.is_some()
                    && source_metadata.fhir_type != "unknown"
                    && source_metadata.fhir_type != "generic"
                    && source_metadata.fhir_type != "BackboneElement"
                {
                    if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                        eprintln!(
                            "🚀 AGGRESSIVE TYPE CAST OPTIMIZATION: {}.{} - trying ModelProvider first",
                            source_metadata.fhir_type, property
                        );
                    }
                    // First try to get proper type from ModelProvider for array elements
                    match self.get_property_type_cached(&source_metadata.fhir_type, property, resolver).await {
                        Ok(resolved_type) => {
                            if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                                eprintln!(
                                    "🚀 AGGRESSIVE TYPE CAST: ModelProvider returned {} for {}.{}",
                                    resolved_type, source_metadata.fhir_type, property
                                );
                            }
                            resolved_type
                        },
                        Err(_) => {
                            if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                                eprintln!(
                                    "🚀 AGGRESSIVE TYPE CAST: ModelProvider failed, using JSON inference for {}.{}",
                                    source_metadata.fhir_type, property
                                );
                            }
                            // Fall back to JSON inference for aggressively typed resources
                            match property_value {
                                JsonValue::String(_) => "string".to_string(),
                                JsonValue::Number(_) => "decimal".to_string(),
                                JsonValue::Bool(_) => "boolean".to_string(),
                                JsonValue::Array(_) => "array".to_string(),
                                JsonValue::Object(obj) => {
                                    // If it has a resourceType, it's a resource reference
                                    if obj.contains_key("resourceType") {
                                        obj.get("resourceType")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("Resource")
                                            .to_string()
                                    } else if obj.contains_key("reference") {
                                        "Reference".to_string()
                                    } else {
                                        "BackboneElement".to_string()
                                    }
                                }
                                JsonValue::Null => "unknown".to_string(),
                            }
                        }
                    }
                }
                // SPECIAL CASES for Bundle navigation to avoid expensive type resolution
                else if source_metadata.path.to_string().contains("Bundle") {
                    if source_metadata.fhir_type == "Bundle" && property == "entry" {
                        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                            eprintln!("🔍 SPECIAL CASE: Bundle.entry - using BackboneElement type");
                        }
                        "BackboneElement".to_string() // Bundle entry type
                    } else if property == "resource"
                        && source_metadata.path.to_string().contains("Bundle.entry")
                    {
                        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                            eprintln!(
                                "🔍 SPECIAL CASE: {}.resource - detecting actual resource type from JSON",
                                source_metadata.fhir_type
                            );
                        }
                        // Let json_to_wrapped_collection detect the actual resource type from JSON
                        "Resource".to_string() // Generic type, but will be overridden by actual detection
                    } else {
                        // Other Bundle properties - use normal resolution but with fallback
                        self.get_property_type_cached(
                            &source_metadata.fhir_type,
                            property,
                            resolver,
                        )
                        .await
                        .unwrap_or_else(|_| {
                            if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                                eprintln!(
                                    "🔍 BUNDLE FALLBACK: {}.{} -> BackboneElement",
                                    source_metadata.fhir_type, property
                                );
                            }
                            "BackboneElement".to_string() // Better fallback for Bundle components
                        })
                    }
                } else {
                    // Property exists - use cached metadata resolution
                    if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                        eprintln!(
                            "🔍 NORMAL CASE: Resolving {}.{}",
                            source_metadata.fhir_type, property
                        );
                    }
                    self.get_property_type_cached(&source_metadata.fhir_type, property, resolver)
                        .await
                        .unwrap_or_else(|_| "unknown".to_string())
                };

                // Create new path for the property
                let property_path = source_metadata.path.append_property(property);

                // Convert JSON value to FhirPathValue and wrap with metadata
                Ok(self.json_to_wrapped_collection(property_value, property_path, property_type))
            }
            None => {
                // Property not found in JSON - check for polymorphic choice types
                if let Some(choice_property) = self.check_choice_type_properties(json, property, source_metadata, resolver).await? {
                    Ok(choice_property)
                } else {
                    // PERFORMANCE OPTIMIZATION: Property not found and no polymorphic match
                    // Instead of doing expensive schema validation for missing properties,
                    // we assume missing properties are valid (they just have no value in this instance)
                    // This matches the FHIRPath spec behavior and avoids the bottleneck
                    Ok(collection_utils::empty())
                }
            }
        }
    }

    /// Check for FHIR choice type properties (e.g., value[x]) using ModelProvider
    async fn check_choice_type_properties(
        &self,
        json: &JsonValue,
        property: &str,
        source_metadata: &ValueMetadata,
        resolver: &TypeResolver,
    ) -> Result<Option<WrappedCollection>> {
        let JsonValue::Object(obj) = json else {
            return Ok(None);
        };

        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
            eprintln!(
                "🔍 POLYMORPHIC CHECK: Looking for {}[x] choices in {}",
                property, source_metadata.fhir_type
            );
        }

        // Use ModelProvider to get type reflection and find choice type properties
        let model_provider = resolver.model_provider().clone();
        match model_provider.get_type_reflection(&source_metadata.fhir_type).await {
            Ok(Some(octofhir_fhir_model::TypeReflectionInfo::ClassInfo { elements, .. })) => {
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 POLYMORPHIC SCHEMA: Got {} elements for {}",
                        elements.len(), source_metadata.fhir_type
                    );
                }
                
                // Look for schema elements that start with the requested property name
                // This correctly handles choice types like value[x] -> valueQuantity, valueString, etc.
                for element in elements {
                    if element.name.starts_with(property) && element.name.len() > property.len() {
                        // Check if this element exists in the JSON
                        if let Some(value) = obj.get(&element.name) {
                            if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                                eprintln!(
                                    "🔍 POLYMORPHIC SUCCESS: {}.{} -> {} (via schema choice type)",
                                    source_metadata.fhir_type, element.name, element.type_info.name()
                                );
                            }
                            
                            // Found matching choice type - use it
                            let property_path = source_metadata.path.append_property(&element.name);
                            let property_type = element.type_info.name().to_string();
                            let wrapped_collection = self.json_to_wrapped_collection(value, property_path, property_type);
                            return Ok(Some(wrapped_collection));
                        }
                    }
                }
            }
            Ok(Some(_)) => {
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 POLYMORPHIC SKIP: {} is not a ClassInfo type",
                        source_metadata.fhir_type
                    );
                }
            }
            Ok(None) => {
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 POLYMORPHIC UNKNOWN: Type {} not found in schema",
                        source_metadata.fhir_type
                    );
                }
            }
            Err(e) => {
                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                    eprintln!(
                        "🔍 POLYMORPHIC ERROR: Failed to get type reflection for {}: {}",
                        source_metadata.fhir_type, e
                    );
                }
            }
        }

        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
            eprintln!(
                "🔍 POLYMORPHIC NONE: No valid {}[x] choice found in {}",
                property, source_metadata.fhir_type
            );
        }

        Ok(None)
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
                array
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let indexed_path = path.append_index(i);
                        let fhir_path_value = self.json_to_fhir_path_value(item);

                        // IMPORTANT FIX: Detect actual FHIR resource type from JSON
                        let actual_fhir_type = if let JsonValue::Object(obj) = item {
                            if let Some(resource_type) =
                                obj.get("resourceType").and_then(|v| v.as_str())
                            {
                                resource_type.to_string()
                            } else {
                                fhir_type.clone()
                            }
                        } else {
                            fhir_type.clone()
                        };

                        let metadata = ValueMetadata {
                            fhir_type: actual_fhir_type.clone(),
                            resource_type: if actual_fhir_type != "unknown" {
                                Some(actual_fhir_type)
                            } else {
                                None
                            },
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

                // IMPORTANT FIX: Detect actual FHIR resource type from JSON
                let actual_fhir_type = if let JsonValue::Object(obj) = json {
                    if let Some(resource_type) = obj.get("resourceType").and_then(|v| v.as_str()) {
                        resource_type.to_string()
                    } else {
                        fhir_type
                    }
                } else {
                    fhir_type
                };

                let metadata = ValueMetadata {
                    fhir_type: actual_fhir_type.clone(),
                    resource_type: if actual_fhir_type != "unknown" {
                        Some(actual_fhir_type)
                    } else {
                        None
                    },
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
                    FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64_retain(f)
                            .unwrap_or_else(|| rust_decimal::Decimal::new(0, 0)),
                    )
                } else {
                    FhirPathValue::String(n.to_string())
                }
            }
            JsonValue::String(s) => FhirPathValue::String(s.clone()),
            JsonValue::Array(_) => {
                // Array values remain as JSON
                FhirPathValue::JsonValue(Arc::new(json.clone()))
            }
            JsonValue::Object(obj) => {
                // IMPORTANT FIX: Check if this is a FHIR resource and preserve type info
                if let Some(_resource_type) = obj.get("resourceType").and_then(|v| v.as_str()) {
                    // This is a FHIR resource - create a Resource value to preserve type information
                    FhirPathValue::Resource(Arc::new(json.clone()))
                } 
                // CRITICAL FIX: Check if this is a FHIR Quantity data type
                else if let (Some(value), Some(code)) = (
                    obj.get("value").and_then(|v| v.as_f64()),
                    obj.get("code").and_then(|c| c.as_str())
                ) {
                    if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                        eprintln!(
                            "🔍 QUANTITY CONVERSION: Converting JSON to FhirPathValue::Quantity - value: {}, code: {}",
                            value, code
                        );
                    }
                    // Convert JSON Quantity object to proper FhirPathValue::Quantity
                    let decimal_value = rust_decimal::Decimal::from_f64_retain(value)
                        .unwrap_or_else(|| rust_decimal::Decimal::new(0, 0));
                    FhirPathValue::quantity(decimal_value, Some(code.to_string()))
                }
                else {
                    // Regular JSON object
                    FhirPathValue::JsonValue(Arc::new(json.clone()))
                }
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
                    // Determine the actual type of this specific element
                    let element_type = crate::typing::type_utils::fhirpath_value_to_fhir_type(value);

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
            FhirPathValue::JsonValue(json_value) if json_value.is_array() => {
                if let JsonValue::Array(array) = json_value.as_ref() {
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
                self.extract_property_from_json(json, property, &source.metadata, resolver)
                    .await
            }
            FhirPathValue::Collection(values) => {
                // Navigate property on each element in collection
                let mut result = Vec::new();

                for (i, value) in values.iter().enumerate() {
                    // Create temporary wrapped value for each collection element
                    // Determine the actual type of this element
                    let element_type = crate::typing::type_utils::fhirpath_value_to_fhir_type(value);
                    let element_metadata = source.metadata.derive_index(i, Some(element_type));
                    let wrapped_element = WrappedValue::new(value.clone(), element_metadata);

                    // Navigate property on this element - silently ignore errors for primitive types
                    match self
                        .navigate_property_with_metadata(&wrapped_element, property, resolver)
                        .await
                    {
                        Ok(property_results) => result.extend(property_results),
                        Err(err) => {
                            // Check if this is a property access error on primitive type (FP0052)
                            if err.error_code() == &crate::core::error_code::FP0052 {
                                // Silently ignore - FHIRPath specification allows property access 
                                // on mixed collections to ignore non-navigable items
                                continue;
                            } else {
                                // Re-raise other types of errors
                                return Err(err);
                            }
                        }
                    }
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
                        property, source.metadata.fhir_type, source.metadata.path
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
        assert_eq!(
            indexed_value.metadata.path.to_string(),
            "Patient.name.given[1]"
        );
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
        assert_eq!(
            results[0].metadata.path.to_string(),
            "Patient.name[0].given[0]"
        );
        assert_eq!(
            results[1].metadata.path.to_string(),
            "Patient.name[0].given[1]"
        );
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
        let source = WrappedValue::new(FhirPathValue::Resource(patient), source_metadata);

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
        let source = WrappedValue::new(FhirPathValue::String("Doe".to_string()), source_metadata);

        let result = navigator
            .navigate_property_with_metadata(&source, "nonexistent", &resolver)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("primitive type"));
    }
}
