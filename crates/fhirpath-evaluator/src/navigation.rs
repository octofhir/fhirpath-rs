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

//! Type-aware navigation for FHIRPath expressions
//!
//! This module provides navigation capabilities that use the async ModelProvider
//! for type validation and enhanced error reporting.

use super::context::EvaluationContext;
use fhirpath_core::{EvaluationError, EvaluationResult};
use fhirpath_model::{
    json_arc::ArcJsonValue,
    provider::{ModelProvider, TypeReflectionInfo},
    resource::FhirResource,
    Collection, FhirPathValue,
};
use std::sync::Arc;

/// Type-aware navigator that validates property access using ModelProvider
pub struct TypeAwareNavigator {
    /// Reference to the async ModelProvider
    provider: Arc<dyn ModelProvider>,
}

impl TypeAwareNavigator {
    /// Create a new type-aware navigator
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self { provider }
    }

    /// Navigate to a property with async type validation
    pub async fn navigate_property(
        &self,
        _context: &EvaluationContext,
        input: &FhirPathValue,
        property_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // First, perform the actual navigation (sync operation)
        let result = self.navigate_property_sync(input, property_name)?;

        // Always validate navigation using the ModelProvider
        self.validate_navigation_async(input, property_name, &result)
            .await?;

        Ok(result)
    }

    /// Synchronous property navigation (core logic)
    fn navigate_property_sync(
        &self,
        input: &FhirPathValue,
        property_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        match input {
            FhirPathValue::Resource(resource) => {
                match resource.get_property(property_name) {
                    Some(value) => {
                        // For FHIR resources, primitive values should retain FHIR context
                        let result = self.json_value_to_fhirpath_value_with_context(value, true);
                        Ok(result)
                    }
                    None => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::JsonValue(json) => {
                match json.get(property_name) {
                    Some(value) => {
                        // For JsonValue, check if it has resourceType to determine if it's FHIR
                        let is_fhir = json.get("resourceType").is_some();
                        Ok(self.json_value_to_fhirpath_value_with_context(value, is_fhir))
                    }
                    None => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Collection(collection) => {
                // Navigate each element in the collection
                let results = collection
                    .iter()
                    .filter_map(|item| self.navigate_property_sync(item, property_name).ok())
                    .filter(|result| !result.is_empty())
                    .collect::<Vec<_>>();

                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Collection(Collection::from_vec(results)))
                }
            }
            _ => {
                // For primitive types, no properties exist
                Ok(FhirPathValue::Empty)
            }
        }
    }

    /// Async validation of navigation paths
    async fn validate_navigation_async(
        &self,
        input: &FhirPathValue,
        property_name: &str,
        _result: &FhirPathValue,
    ) -> EvaluationResult<()> {
        // Get the input type for validation
        let input_type = self.infer_type_from_value(input);

        if let Some(type_name) = input_type {
            // Check if the navigation path is valid using ModelProvider
            match self
                .provider
                .validate_navigation_path(&type_name, property_name)
                .await
            {
                Ok(validation) => {
                    if !validation.is_valid {
                        return Err(EvaluationError::PropertyNotFound {
                            property: property_name.to_string(),
                            type_name: type_name.clone(),
                        });
                    }

                    // Store successful validation in cache for performance
                    // Note: This would be implemented with the context type annotations
                }
                Err(model_error) => {
                    // Model provider error - log but don't fail the navigation
                    eprintln!("ModelProvider error during navigation validation: {model_error:?}");
                }
            }
        }

        Ok(())
    }

    /// Get async property type information with caching
    pub async fn get_property_type_async(
        &self,
        context: &EvaluationContext,
        parent_type: &str,
        property_name: &str,
    ) -> Option<TypeReflectionInfo> {
        // First check the cache
        let cache_key = format!("{parent_type}.{property_name}");
        if let Some(cached_type) = context.get_type_annotation(&cache_key) {
            return Some(cached_type);
        }

        // Query the ModelProvider
        if let Some(type_info) = self
            .provider
            .get_property_type(parent_type, property_name)
            .await
        {
            // Cache the result for future use
            context.set_type_annotation(cache_key, type_info.clone());
            Some(type_info)
        } else {
            None
        }
    }

    /// Navigate with choice type resolution
    pub async fn navigate_with_choice_types(
        &self,
        context: &EvaluationContext,
        input: &FhirPathValue,
        property_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // First perform regular navigation
        let result = self
            .navigate_property(context, input, property_name)
            .await?;

        // If the result is empty, try choice type resolution
        if result.is_empty() {
            if let Some(input_type) = self.infer_type_from_value(input) {
                // Try resolving choice types (e.g., value[x] -> valueString, valueInteger, etc.)
                let resolved_result = self
                    .try_choice_type_resolution(context, input, &input_type, property_name)
                    .await?;
                if !resolved_result.is_empty() {
                    return Ok(resolved_result);
                }
            }
        }

        Ok(result)
    }

    /// Attempt to resolve choice types for navigation
    async fn try_choice_type_resolution(
        &self,
        _context: &EvaluationContext,
        input: &FhirPathValue,
        input_type: &str,
        property_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // Get type reflection to understand choice types
        if let Some(_type_reflection) = self.provider.get_type_reflection(input_type).await {
            // Simplified choice type resolution - try common patterns
            if property_name.contains("value") {
                // Try the navigation with the original choice type property
                return self.navigate_property_sync(input, property_name);
            }
        }

        Ok(FhirPathValue::Empty)
    }

    /// Check if a choice type suffix is valid for a property type
    #[allow(dead_code)]
    fn is_valid_choice_type(&self, _prop_type: &TypeReflectionInfo, _choice_suffix: &str) -> bool {
        // This would check against the actual choice type constraints
        // For now, we'll allow common FHIR choice types
        true // Simplified implementation
    }

    /// Convert serde_json::Value to FhirPathValue with FHIR context preservation
    fn json_value_to_fhirpath_value_with_context(
        &self,
        value: &serde_json::Value,
        is_fhir_context: bool,
    ) -> FhirPathValue {
        match value {
            serde_json::Value::Null => FhirPathValue::Empty,
            serde_json::Value::Bool(b) => {
                if is_fhir_context {
                    // Wrap as FHIR resource to preserve FHIR boolean type
                    FhirPathValue::Resource(Arc::new(FhirResource::from_json(value.clone())))
                } else {
                    FhirPathValue::Boolean(*b)
                }
            }
            serde_json::Value::Number(n) => {
                if is_fhir_context {
                    // Wrap as FHIR resource to preserve FHIR integer/decimal type
                    FhirPathValue::Resource(Arc::new(FhirResource::from_json(value.clone())))
                } else if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    if let Ok(decimal) = rust_decimal::Decimal::try_from(f) {
                        FhirPathValue::Decimal(decimal)
                    } else {
                        // Fallback to JSON value if decimal conversion fails
                        FhirPathValue::JsonValue(ArcJsonValue::new(value.clone()))
                    }
                } else {
                    FhirPathValue::JsonValue(ArcJsonValue::new(value.clone()))
                }
            }
            serde_json::Value::String(s) => {
                if is_fhir_context {
                    // Wrap as FHIR resource to preserve FHIR string type
                    FhirPathValue::Resource(Arc::new(FhirResource::from_json(value.clone())))
                } else {
                    FhirPathValue::String(s.as_str().into())
                }
            }
            serde_json::Value::Array(arr) => {
                let collection_items = arr
                    .iter()
                    .map(|v| self.json_value_to_fhirpath_value_with_context(v, is_fhir_context))
                    .collect();
                FhirPathValue::Collection(Collection::from_vec(collection_items))
            }
            serde_json::Value::Object(_) => {
                // Objects are always wrapped - check if it's a FHIR resource
                if value.get("resourceType").is_some() {
                    FhirPathValue::Resource(Arc::new(FhirResource::from_json(value.clone())))
                } else {
                    FhirPathValue::JsonValue(ArcJsonValue::new(value.clone()))
                }
            }
        }
    }

    /// Convert serde_json::Value to FhirPathValue
    fn json_value_to_fhirpath_value(&self, value: &serde_json::Value) -> FhirPathValue {
        match value {
            serde_json::Value::Null => FhirPathValue::Empty,
            serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    if let Ok(decimal) = rust_decimal::Decimal::try_from(f) {
                        FhirPathValue::Decimal(decimal)
                    } else {
                        // Fallback to JSON value if decimal conversion fails
                        FhirPathValue::JsonValue(ArcJsonValue::new(value.clone()))
                    }
                } else {
                    FhirPathValue::JsonValue(ArcJsonValue::new(value.clone()))
                }
            }
            serde_json::Value::String(s) => FhirPathValue::String(s.as_str().into()),
            serde_json::Value::Array(arr) => {
                let collection_items = arr
                    .iter()
                    .map(|v| self.json_value_to_fhirpath_value_with_context(v, false))
                    .collect();
                FhirPathValue::Collection(Collection::from_vec(collection_items))
            }
            serde_json::Value::Object(_) => {
                FhirPathValue::JsonValue(ArcJsonValue::new(value.clone()))
            }
        }
    }

    /// Infer FHIR type from a FhirPathValue
    fn infer_type_from_value(&self, value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::Resource(resource) => resource.resource_type().map(|rt| rt.to_string()),
            FhirPathValue::JsonValue(json) => {
                // Try to infer from resourceType property
                json.get("resourceType").and_then(|rt| match rt {
                    serde_json::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
            }
            FhirPathValue::String(_) => Some("string".to_string()),
            FhirPathValue::Integer(_) => Some("integer".to_string()),
            FhirPathValue::Decimal(_) => Some("decimal".to_string()),
            FhirPathValue::Boolean(_) => Some("boolean".to_string()),
            FhirPathValue::Date(_) => Some("date".to_string()),
            FhirPathValue::DateTime(_) => Some("dateTime".to_string()),
            FhirPathValue::Time(_) => Some("time".to_string()),
            FhirPathValue::Collection(coll) => {
                // Infer from first element if collection is not empty
                if let Some(first) = coll.iter().next() {
                    self.infer_type_from_value(first)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get enhanced error message using async ModelProvider
    pub async fn get_enhanced_property_error(
        &self,
        input_type: &str,
        property_name: &str,
    ) -> String {
        // Get properties for suggestions
        let properties = self.provider.get_properties(input_type).await;

        let suggestions = properties
            .iter()
            .map(|(name, _)| name.as_str())
            .filter(|name| {
                // Simple similarity check
                let similarity = jaro_winkler_similarity(property_name, name);
                similarity > 0.6
            })
            .take(3)
            .collect::<Vec<_>>();

        if suggestions.is_empty() {
            format!(
                "Property '{}' not found on type '{}'. Available properties: {}",
                property_name,
                input_type,
                properties
                    .iter()
                    .take(5)
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!(
                "Property '{}' not found on type '{}'. Did you mean: {}?",
                property_name,
                input_type,
                suggestions.join(", ")
            )
        }
    }
}

/// Enhanced similarity function for property name suggestions
/// Uses a combination of prefix matching and substring matching for FHIR properties
fn jaro_winkler_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let s1_len = s1.len();
    let s2_len = s2.len();

    if s1_len == 0 || s2_len == 0 {
        return 0.0;
    }

    // Case-insensitive comparison
    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();

    // Perfect case-insensitive match
    if s1_lower == s2_lower {
        return 0.9;
    }

    // Calculate prefix similarity
    let common_prefix = s1_lower
        .chars()
        .zip(s2_lower.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .count();

    let prefix_score = if common_prefix > 0 {
        (common_prefix as f64 / s1_len.max(s2_len) as f64) * 0.8
    } else {
        0.0
    };

    // Calculate substring similarity (for cases like "given" in "givenName")
    let substring_score = if s1_lower.len() <= s2_lower.len() && s2_lower.contains(&s1_lower) {
        // Higher score for substring matches, especially when the shorter string is a meaningful prefix/root
        let ratio = s1_lower.len() as f64 / s2_lower.len() as f64;
        if ratio > 0.6 {
            0.8 // High score for substantial substrings
        } else if ratio > 0.4 {
            0.7 // Good score for reasonable substrings
        } else {
            ratio * 0.6
        }
    } else if s2_lower.len() <= s1_lower.len() && s1_lower.contains(&s2_lower) {
        let ratio = s2_lower.len() as f64 / s1_lower.len() as f64;
        if ratio > 0.6 {
            0.8
        } else if ratio > 0.4 {
            0.7
        } else {
            ratio * 0.6
        }
    } else {
        0.0
    };

    // Take the maximum of prefix and substring similarity
    let base_score = prefix_score.max(substring_score);

    // Reduce length penalty for meaningful matches
    if base_score > 0.6 {
        base_score // Don't penalize good matches
    } else {
        let length_diff = s1_len.abs_diff(s2_len) as f64;
        let max_len = s1_len.max(s2_len) as f64;
        let length_penalty = length_diff / max_len * 0.05; // Reduced penalty
        (base_score - length_penalty).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fhirpath_model::mock_provider::MockModelProvider;
    use tokio;

    #[tokio::test]
    async fn test_sync_navigation() {
        let provider = Arc::new(MockModelProvider::empty());
        let navigator = TypeAwareNavigator::new(provider);

        // Test simple object navigation
        let mut obj = serde_json::Map::new();
        obj.insert(
            "name".to_string(),
            serde_json::Value::String("Test".to_string()),
        );
        let json_obj = serde_json::Value::Object(obj);
        let input = FhirPathValue::JsonValue(ArcJsonValue::new(json_obj));

        let result = navigator.navigate_property_sync(&input, "name").unwrap();
        match result {
            FhirPathValue::String(s) => assert_eq!(s.as_ref(), "Test"),
            _ => panic!("Expected string result"),
        }
    }

    #[tokio::test]
    async fn test_similarity_function() {
        assert_eq!(jaro_winkler_similarity("name", "name"), 1.0);
        assert!(jaro_winkler_similarity("name", "Name") > 0.8);
        assert!(jaro_winkler_similarity("given", "givenName") > 0.6);
        assert!(jaro_winkler_similarity("family", "familyName") > 0.6);
        assert_eq!(jaro_winkler_similarity("", "test"), 0.0);
        assert_eq!(jaro_winkler_similarity("test", ""), 0.0);
    }

    #[tokio::test]
    async fn test_type_inference() {
        let provider = Arc::new(MockModelProvider::empty());
        let navigator = TypeAwareNavigator::new(provider);

        assert_eq!(
            navigator.infer_type_from_value(&FhirPathValue::String("test".to_string().into())),
            Some("string".to_string())
        );
        assert_eq!(
            navigator.infer_type_from_value(&FhirPathValue::Integer(42)),
            Some("integer".to_string())
        );
        assert_eq!(
            navigator.infer_type_from_value(&FhirPathValue::Boolean(true)),
            Some("boolean".to_string())
        );
    }
}
