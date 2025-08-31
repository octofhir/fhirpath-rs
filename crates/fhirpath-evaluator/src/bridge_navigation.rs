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

//! Bridge API-enabled navigation implementation
//!
//! This module provides navigation operations that use the bridge support
//! architecture for O(1) type operations and choice type resolution.

use crate::cache::SchemaCache;
use crate::context::EvaluationContext as LocalEvaluationContext;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::{
    ChoiceTypeResolver, FhirPathValue, JsonValue, SystemTypes, TypeResolver,
};
use octofhir_fhirschema::FhirSchemaPackageManager;
use std::sync::Arc;

/// Bridge-enabled navigation evaluator with schema integration
pub struct BridgeNavigationEvaluator {
    /// Schema manager for bridge API operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Schema cache for performance optimization
    schema_cache: SchemaCache,
    /// Choice type resolver for polymorphic properties
    choice_resolver: ChoiceTypeResolver,
    /// Type resolver for type information
    type_resolver: TypeResolver,
    /// System types for type categorization
    system_types: SystemTypes,
}

impl BridgeNavigationEvaluator {
    /// Create a new bridge navigation evaluator
    pub fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Self {
        let choice_resolver = ChoiceTypeResolver::new(schema_manager.clone());
        let type_resolver = TypeResolver::new(schema_manager.clone());
        let system_types = SystemTypes::new(schema_manager.clone());

        Self {
            schema_manager,
            schema_cache: SchemaCache::new(),
            choice_resolver,
            type_resolver,
            system_types,
        }
    }

    /// Evaluate property navigation with bridge API integration
    pub async fn navigate_property(
        &mut self,
        target: &FhirPathValue,
        property_name: &str,
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        match target {
            FhirPathValue::JsonValue(json) => {
                self.navigate_json_property(json, property_name, context)
                    .await
            }
            FhirPathValue::Collection(items) => {
                let mut result_items = Vec::new();
                for item in items.iter() {
                    // Create a new bridge navigator for recursion to avoid mutable borrow issues
                    let mut navigator = BridgeNavigationEvaluator::new(self.schema_manager.clone());
                    let nav_result =
                        Box::pin(navigator.navigate_property(item, property_name, context)).await?;
                    match nav_result {
                        FhirPathValue::Collection(nested_items) => {
                            result_items.extend(nested_items.iter().cloned());
                        }
                        FhirPathValue::Empty => {
                            // Skip empty results
                        }
                        value => {
                            result_items.push(value);
                        }
                    }
                }

                Ok(FhirPathValue::normalize_collection_result(result_items))
            }
            // Handle TypeInfoObject property access
            FhirPathValue::TypeInfoObject { namespace, name } => match property_name {
                "namespace" => Ok(FhirPathValue::String(namespace.clone())),
                "name" => Ok(FhirPathValue::String(name.clone())),
                _ => Ok(FhirPathValue::Empty),
            },
            // Handle Quantity property access
            FhirPathValue::Quantity(quantity) => match property_name {
                "value" => Ok(FhirPathValue::Decimal(quantity.value)),
                "unit" => {
                    if let Some(ref unit) = quantity.unit {
                        Ok(FhirPathValue::String(unit.clone().into()))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
                _ => Ok(FhirPathValue::Empty),
            },
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Navigate JSON properties with bridge API schema integration
    async fn navigate_json_property(
        &mut self,
        json: &JsonValue,
        property_name: &str,
        _context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // First try direct property access
        if let Some(value) = json.get_property(property_name) {
            return Ok(self.convert_json_to_fhirpath_value(&value));
        }

        // Get the resource/element type
        let current_type = self.get_current_type(json);

        // Check if this is a known property using schema manager
        if self
            .schema_cache
            .has_property(&self.schema_manager, &current_type, property_name)
            .await
        {
            // Property exists in schema but not in data - return empty
            Ok(FhirPathValue::Empty)
        } else {
            // Property not found in schema - check for choice type
            self.resolve_choice_property(json, &current_type, property_name)
                .await
        }
    }

    /// Resolve choice type properties using bridge API
    async fn resolve_choice_property(
        &mut self,
        json: &JsonValue,
        resource_type: &str,
        property_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // Try to infer choice base from property name
        let choice_base = self.infer_choice_base(property_name);
        let choice_path = format!("{}.{}", resource_type, choice_base);

        // Try to resolve using choice resolver
        match self
            .choice_resolver
            .resolve_choice_type(&choice_path, property_name)
            .await
        {
            Ok(choice_info) => {
                // Found valid choice type - extract value from JSON
                if let Some(value) = json.get_property(&choice_info.resolved_property) {
                    Ok(self.convert_json_to_fhirpath_value(&value))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            Err(_) => {
                // Not a valid choice type - try common choice patterns
                self.try_common_choice_patterns(json, property_name).await
            }
        }
    }

    /// Try common FHIR choice type patterns
    async fn try_common_choice_patterns(
        &self,
        json: &JsonValue,
        property_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // Common patterns like valueString -> value[x], effectiveDateTime -> effective[x]
        let patterns = [
            (
                "value",
                vec![
                    "valueString",
                    "valueInteger",
                    "valueQuantity",
                    "valueBoolean",
                    "valueDateTime",
                    "valueDecimal",
                ],
            ),
            (
                "effective",
                vec!["effectiveDateTime", "effectivePeriod", "effectiveInstant"],
            ),
            (
                "onset",
                vec![
                    "onsetDateTime",
                    "onsetAge",
                    "onsetPeriod",
                    "onsetRange",
                    "onsetString",
                ],
            ),
            ("deceased", vec!["deceasedBoolean", "deceasedDateTime"]),
            (
                "multipleBirth",
                vec!["multipleBirthBoolean", "multipleBirthInteger"],
            ),
        ];

        for (_base, variants) in &patterns {
            if variants.iter().any(|v| *v == property_name) {
                // This is a known choice variant
                if let Some(value) = json.get_property(property_name) {
                    return Ok(self.convert_json_to_fhirpath_value(&value));
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
        }

        // Not a recognized pattern - return empty
        Ok(FhirPathValue::Empty)
    }

    /// Infer choice base from concrete property name
    fn infer_choice_base(&self, concrete_property: &str) -> String {
        // Common FHIR choice patterns
        if concrete_property.starts_with("value") && concrete_property.len() > 5 {
            "value[x]".to_string()
        } else if concrete_property.starts_with("effective") && concrete_property.len() > 9 {
            "effective[x]".to_string()
        } else if concrete_property.starts_with("onset") && concrete_property.len() > 5 {
            "onset[x]".to_string()
        } else if concrete_property.starts_with("deceased") && concrete_property.len() > 8 {
            "deceased[x]".to_string()
        } else if concrete_property.starts_with("multipleBirth") && concrete_property.len() > 13 {
            "multipleBirth[x]".to_string()
        } else {
            // Fallback: assume the property itself is the base
            format!("{}[x]", concrete_property)
        }
    }

    /// Get the current type from JSON resource
    fn get_current_type(&self, json: &JsonValue) -> String {
        json.get_property("resourceType")
            .and_then(|rt| {
                if rt.is_string() {
                    rt.as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Element".to_string())
    }

    /// Convert JSON value to FhirPathValue
    fn convert_json_to_fhirpath_value(&self, json_value: &JsonValue) -> FhirPathValue {
        if json_value.is_null() {
            FhirPathValue::Empty
        } else if let Some(b) = json_value.as_bool() {
            FhirPathValue::Boolean(b)
        } else if let Some(i) = json_value.as_i64() {
            FhirPathValue::Integer(i)
        } else if let Some(f) = json_value.as_f64() {
            FhirPathValue::Decimal(rust_decimal::Decimal::try_from(f).unwrap_or_default())
        } else if let Some(s) = json_value.as_str() {
            FhirPathValue::String(s.into())
        } else if json_value.is_array() {
            // For arrays, we need to handle them differently since JsonValue doesn't have as_array()
            // This is a simplified implementation - in practice we'd need to access array elements
            FhirPathValue::JsonValue(json_value.clone())
        } else if json_value.is_object() {
            FhirPathValue::JsonValue(json_value.clone())
        } else {
            FhirPathValue::Empty
        }
    }

    /// Get schema manager reference
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> crate::cache::CacheStats {
        self.schema_cache.get_cache_stats().await
    }

    /// Clear schema cache
    pub async fn clear_cache(&self) {
        self.schema_cache.clear_all().await
    }
}

impl Clone for BridgeNavigationEvaluator {
    fn clone(&self) -> Self {
        Self {
            schema_manager: self.schema_manager.clone(),
            schema_cache: SchemaCache::new(), // Create new cache for clone
            choice_resolver: ChoiceTypeResolver::new(self.schema_manager.clone()),
            type_resolver: TypeResolver::new(self.schema_manager.clone()),
            system_types: SystemTypes::new(self.schema_manager.clone()),
        }
    }
}
