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

//! Polymorphic Navigation Engine for FHIR Choice Types
//!
//! This module provides enhanced navigation for FHIR resources with polymorphic properties,
//! enabling proper resolution of choice types (value[x] patterns).

use dashmap::DashMap;
use std::sync::Arc;

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{
    FhirPathValue, JsonValue,
    polymorphic_resolver::PolymorphicPathResolver,
    provider::{ModelProvider, TypeReflectionInfo},
};

/// Enhanced navigation engine with polymorphic support
#[derive(Debug)]
pub struct PolymorphicNavigationEngine {
    /// Path resolver for handling choice types
    path_resolver: Arc<PolymorphicPathResolver>,

    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,

    /// Cache for navigation results
    navigation_cache: Arc<DashMap<String, NavigationResult>>,
}

/// Result of polymorphic navigation
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// Values found at the path
    pub values: Vec<FhirPathValue>,

    /// Type information for the values
    pub result_type: Option<TypeReflectionInfo>,

    /// The actual path that was navigated
    pub resolved_path: String,

    /// Whether choice type resolution occurred
    pub used_choice_resolution: bool,

    /// Original path requested
    pub original_path: String,

    /// Available alternatives (for choice types)
    pub alternatives: Vec<String>,
}

impl PolymorphicNavigationEngine {
    /// Create new polymorphic navigation engine
    pub fn new(
        path_resolver: Arc<PolymorphicPathResolver>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            path_resolver,
            model_provider,
            navigation_cache: Arc::new(DashMap::new()),
        }
    }

    /// Navigate a polymorphic path through data
    pub async fn navigate_path(
        &self,
        base_value: &FhirPathValue,
        path: &str,
    ) -> EvaluationResult<NavigationResult> {
        // Check cache first
        let cache_key = self.create_cache_key(base_value, path);
        if let Some(cached_result) = self.navigation_cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        // Determine base type
        let base_type = self.determine_base_type(base_value).await?;

        // Resolve the path polymorphically
        let resolved_path = self
            .path_resolver
            .resolve_path(&base_type, path, Some(base_value))
            .await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Path resolution failed: {e}"),
            })?;

        // Navigate using the resolved path
        let values = self
            .navigate_resolved_path(base_value, &resolved_path.resolved_path)
            .await?;

        let result = NavigationResult {
            values,
            result_type: resolved_path.result_type,
            resolved_path: resolved_path.resolved_path,
            used_choice_resolution: resolved_path.is_choice_resolution,
            original_path: path.to_string(),
            alternatives: resolved_path.alternatives,
        };

        // Cache the result
        self.navigation_cache.insert(cache_key, result.clone());

        Ok(result)
    }

    /// Navigate using a resolved path
    async fn navigate_resolved_path(
        &self,
        base_value: &FhirPathValue,
        path: &str,
    ) -> EvaluationResult<Vec<FhirPathValue>> {
        let path_segments: Vec<&str> = path.split('.').collect();
        let mut current_values = vec![base_value.clone()];

        for segment in path_segments {
            let mut next_values = Vec::new();

            for current_value in &current_values {
                let segment_values = self.navigate_single_segment(current_value, segment).await?;
                next_values.extend(segment_values);
            }

            current_values = next_values;

            if current_values.is_empty() {
                break; // No more values to navigate
            }
        }

        Ok(current_values)
    }

    /// Navigate a single path segment
    fn navigate_single_segment<'a>(
        &'a self,
        value: &'a FhirPathValue,
        segment: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = EvaluationResult<Vec<FhirPathValue>>> + Send + 'a>,
    > {
        Box::pin(async move {
            match value {
                FhirPathValue::JsonValue(json) => self.navigate_json_property(json, segment).await,
                FhirPathValue::Resource(resource) => {
                    let json = resource.as_json();
                    let json_value = FhirPathValue::JsonValue(JsonValue::new(json));
                    self.navigate_single_segment(&json_value, segment).await
                }
                FhirPathValue::Collection(items) => {
                    let mut result = Vec::new();
                    for item in items.iter() {
                        let item_result = self.navigate_single_segment(item, segment).await?;
                        result.extend(item_result);
                    }
                    Ok(result)
                }
                _ => {
                    // Cannot navigate further from primitive types
                    Ok(vec![])
                }
            }
        })
    }

    /// Navigate JSON object property
    async fn navigate_json_property(
        &self,
        json: &JsonValue,
        property: &str,
    ) -> EvaluationResult<Vec<FhirPathValue>> {
        // Try direct property access first
        if let Some(property_value) = json.get_property(property) {
            return Ok(vec![self.convert_json_to_fhirpath_value(property_value)]);
        }

        // If direct access failed, try FHIR choice type pattern matching
        let sonic_value = json.as_value();
        if let Some(obj) = sonic_value.as_object() {
            // Look for properties that start with the requested property name followed by an uppercase letter
            for (key, value) in obj.iter() {
                if key.starts_with(property) && key.len() > property.len() {
                    if let Some(next_char) = key.chars().nth(property.len()) {
                        if next_char.is_uppercase() {
                            return Ok(vec![
                                self.convert_json_to_fhirpath_value(JsonValue::new(value.clone())),
                            ]);
                        }
                    }
                }
            }
        }

        Ok(vec![]) // Property not found
    }

    /// Convert JsonValue to proper FhirPathValue type
    fn convert_json_to_fhirpath_value(&self, json_value: JsonValue) -> FhirPathValue {
        // Use the same conversion logic as the existing navigation evaluator
        super::navigation::NavigationEvaluator::convert_json_to_fhirpath_value(json_value)
    }

    /// Determine base type of a value
    fn determine_base_type<'a>(
        &'a self,
        value: &'a FhirPathValue,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EvaluationResult<String>> + Send + 'a>>
    {
        Box::pin(async move {
            match value {
                FhirPathValue::JsonValue(json) => {
                    let sonic_value = json.as_value();
                    if let Some(resource_type) =
                        sonic_value.get("resourceType").and_then(|rt| rt.as_str())
                    {
                        Ok(resource_type.to_string())
                    } else {
                        // Try to infer type from structure
                        Ok("Element".to_string()) // Fallback
                    }
                }
                FhirPathValue::Resource(resource) => {
                    Ok(resource.resource_type().unwrap_or("Resource").to_string())
                }
                FhirPathValue::Collection(items) => {
                    if let Some(first_item) = items.iter().next() {
                        self.determine_base_type(first_item).await
                    } else {
                        Err(EvaluationError::TypeError {
                            expected: "Resource".to_string(),
                            actual: "Empty collection".to_string(),
                        })
                    }
                }
                _ => Err(EvaluationError::TypeError {
                    expected: "Resource".to_string(),
                    actual: "Primitive value".to_string(),
                }),
            }
        })
    }

    /// Create cache key for navigation result
    fn create_cache_key(&self, value: &FhirPathValue, path: &str) -> String {
        // Create a deterministic cache key based on value type and path
        match value {
            FhirPathValue::JsonValue(json) => {
                let sonic_value = json.as_value();
                if let Some(resource_type) =
                    sonic_value.get("resourceType").and_then(|rt| rt.as_str())
                {
                    format!("{resource_type}#{path}")
                } else {
                    format!("Element#{path}")
                }
            }
            FhirPathValue::Resource(resource) => {
                let resource_type = resource.resource_type().unwrap_or("Resource");
                format!("{resource_type}#{path}")
            }
            _ => format!("Unknown#{path}"),
        }
    }

    /// Clear navigation cache
    pub fn clear_cache(&self) {
        self.navigation_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> NavigationCacheStats {
        NavigationCacheStats {
            total_entries: self.navigation_cache.len(),
            memory_usage_estimate: self.navigation_cache.len()
                * std::mem::size_of::<NavigationResult>(),
        }
    }

    /// Check if a path would benefit from polymorphic navigation
    pub async fn would_benefit_from_polymorphic_navigation(
        &self,
        base_type: &str,
        path: &str,
    ) -> bool {
        self.path_resolver
            .requires_polymorphic_resolution(base_type, path)
            .await
    }

    /// Get available alternatives for a choice type path
    pub async fn get_path_alternatives(
        &self,
        base_type: &str,
        path: &str,
    ) -> EvaluationResult<Vec<String>> {
        let resolved = self
            .path_resolver
            .resolve_path(base_type, path, None)
            .await
            .map_err(|e| EvaluationError::InvalidOperation {
                message: format!("Failed to get alternatives: {e}"),
            })?;

        Ok(resolved.alternatives)
    }
}

/// Navigation cache statistics
#[derive(Debug, Clone)]
pub struct NavigationCacheStats {
    /// Total number of cached entries
    pub total_entries: usize,
    /// Estimated memory usage in bytes
    pub memory_usage_estimate: usize,
}

/// Factory for creating polymorphic navigation engines
pub struct PolymorphicNavigationFactory;

impl PolymorphicNavigationFactory {
    /// Create a navigation engine with dynamic choice type discovery from FHIRSchema
    pub fn create_r4_navigation_engine(
        model_provider: Arc<dyn ModelProvider>,
    ) -> PolymorphicNavigationEngine {
        use octofhir_fhirpath_model::polymorphic_resolver::PolymorphicResolverFactory;

        let path_resolver = Arc::new(PolymorphicResolverFactory::create_dynamic_resolver(
            model_provider.clone(),
        ));
        PolymorphicNavigationEngine::new(path_resolver, model_provider)
    }

    /// Create a navigation engine with custom resolver
    pub fn create_custom_navigation_engine(
        path_resolver: Arc<PolymorphicPathResolver>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> PolymorphicNavigationEngine {
        PolymorphicNavigationEngine::new(path_resolver, model_provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::mock_provider::MockModelProvider;
    use serde_json::json;

    async fn create_test_engine() -> PolymorphicNavigationEngine {
        let model_provider = Arc::new(MockModelProvider::new());
        PolymorphicNavigationFactory::create_r4_navigation_engine(model_provider)
    }

    #[tokio::test]
    async fn test_observation_value_navigation() -> Result<(), Box<dyn std::error::Error>> {
        let engine = create_test_engine().await;

        let observation = FhirPathValue::JsonValue(JsonValue::new(json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        })));

        let result = engine.navigate_path(&observation, "value").await?;

        assert!(result.used_choice_resolution);
        assert_eq!(result.resolved_path, "valueQuantity");
        assert!(!result.values.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_observation_value_unit_navigation() -> Result<(), Box<dyn std::error::Error>> {
        let engine = create_test_engine().await;

        let observation = FhirPathValue::JsonValue(JsonValue::new(json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        })));

        let result = engine.navigate_path(&observation, "value.unit").await?;

        assert!(result.used_choice_resolution);
        assert_eq!(result.resolved_path, "valueQuantity.unit");
        assert_eq!(result.values.len(), 1);

        match &result.values[0] {
            FhirPathValue::String(s) => assert_eq!(s.as_ref(), "lbs"),
            _ => panic!("Expected string result"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_direct_choice_property_navigation() -> Result<(), Box<dyn std::error::Error>> {
        let engine = create_test_engine().await;

        let observation = FhirPathValue::JsonValue(JsonValue::new(json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        })));

        let result = engine
            .navigate_path(&observation, "valueQuantity.unit")
            .await?;

        assert!(!result.used_choice_resolution); // Direct access doesn't need choice resolution
        assert_eq!(result.resolved_path, "valueQuantity.unit");
        assert_eq!(result.values.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_patient_deceased_choice() -> Result<(), Box<dyn std::error::Error>> {
        let engine = create_test_engine().await;

        let patient = FhirPathValue::JsonValue(JsonValue::new(json!({
            "resourceType": "Patient",
            "deceasedBoolean": true
        })));

        let result = engine.navigate_path(&patient, "deceased").await?;

        assert!(result.used_choice_resolution);
        assert_eq!(result.resolved_path, "deceasedBoolean");
        assert!(!result.alternatives.is_empty());
        assert!(result.alternatives.contains(&"deceasedBoolean".to_string()));
        assert!(
            result
                .alternatives
                .contains(&"deceasedDateTime".to_string())
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_functionality() -> Result<(), Box<dyn std::error::Error>> {
        let engine = create_test_engine().await;

        let observation = FhirPathValue::JsonValue(JsonValue::new(json!({
            "resourceType": "Observation",
            "valueQuantity": {"value": 185}
        })));

        // First navigation - should populate cache
        let _result1 = engine.navigate_path(&observation, "value").await?;

        let stats = engine.get_cache_stats();
        assert_eq!(stats.total_entries, 1);

        // Second navigation - should use cache
        let _result2 = engine.navigate_path(&observation, "value").await?;

        let stats2 = engine.get_cache_stats();
        assert_eq!(stats2.total_entries, 1); // Same entry

        Ok(())
    }

    #[tokio::test]
    async fn test_get_path_alternatives() -> Result<(), Box<dyn std::error::Error>> {
        let engine = create_test_engine().await;

        let alternatives = engine.get_path_alternatives("Observation", "value").await?;

        assert!(alternatives.contains(&"valueQuantity".to_string()));
        assert!(alternatives.contains(&"valueString".to_string()));
        assert!(alternatives.contains(&"valueBoolean".to_string()));

        Ok(())
    }
}
