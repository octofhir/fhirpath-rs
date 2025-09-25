//! FHIRPath value types - Re-export of core types with JsonValue integration
//!
//! This module provides compatibility layer for value types, integrating with our
//! consolidated implementation while maintaining compatibility with serde_json::Value.

use crate::core::types::FhirPathValue;
use serde_json::Value as JsonValue;

/// Extension trait for JsonValue to add FHIRPath-specific functionality
pub trait JsonValueExt {
    /// Get the inner JSON value (compatibility method)
    fn as_inner(&self) -> &JsonValue;

    /// Get an iterator over object entries
    fn object_iter(&self) -> Option<serde_json::map::Iter>;

    /// Get an iterator over array elements  
    fn array_iter(&self) -> Option<std::slice::Iter<JsonValue>>;

    /// Get a property from an object
    fn get_property(&self, key: &str) -> Option<&JsonValue>;

    /// Check if this JSON value represents a FHIR resource
    fn is_fhir_resource(&self) -> bool;

    /// Get the resource type if this is a FHIR resource
    fn resource_type(&self) -> Option<&str>;
}

impl JsonValueExt for JsonValue {
    fn as_inner(&self) -> &JsonValue {
        self
    }

    fn object_iter(&self) -> Option<serde_json::map::Iter> {
        self.as_object().map(|obj| obj.iter())
    }

    fn array_iter(&self) -> Option<std::slice::Iter<JsonValue>> {
        self.as_array().map(|arr| arr.iter())
    }

    fn get_property(&self, key: &str) -> Option<&JsonValue> {
        self.as_object().and_then(|obj| obj.get(key))
    }

    fn is_fhir_resource(&self) -> bool {
        self.as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|rt| rt.as_str())
            .is_some()
    }

    fn resource_type(&self) -> Option<&str> {
        self.as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|rt| rt.as_str())
    }
}

/// Utility functions for working with JSON values in FHIRPath context
pub mod utils {
    use super::*;
    use crate::core::model_provider::ModelProvider;
    use std::sync::Arc;

    /// Infer FHIR type from JSON object structure
    fn infer_fhir_type_from_json(obj: &serde_json::Map<String, JsonValue>) -> Option<String> {
        // Look for Quantity pattern
        if obj.contains_key("value") && (obj.contains_key("unit") || obj.contains_key("code")) {
            return Some("Quantity".to_string());
        }

        // Could add other FHIR types here in the future
        None
    }

    /// Convert JSON object to Quantity FhirPathValue
    fn convert_json_to_quantity(obj: &serde_json::Map<String, JsonValue>) -> FhirPathValue {
        // Extract value - handle both string and number representations
        let value = obj
            .get("value")
            .and_then(|v| {
                // Try as number first
                if let Some(f) = v.as_f64() {
                    Some(f)
                } else if let Some(s) = v.as_str() {
                    // Try to parse string as number
                    s.parse::<f64>().ok()
                } else {
                    None
                }
            })
            .map(|f| rust_decimal::Decimal::try_from(f).unwrap_or_default())
            .unwrap_or_default();

        let unit_display = obj
            .get("unit")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());
        let code = obj
            .get("code")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());
        let system = obj
            .get("system")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        FhirPathValue::quantity_with_components(value, unit_display, code, system)
    }

    /// Convert a JsonValue to a FhirPathValue
    pub fn json_to_fhirpath_value(json: JsonValue) -> FhirPathValue {
        match json {
            JsonValue::Bool(b) => FhirPathValue::boolean(b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::integer(i)
                } else if let Some(f) = n.as_f64() {
                    FhirPathValue::decimal(rust_decimal::Decimal::try_from(f).unwrap_or_default())
                } else {
                    FhirPathValue::string(n.to_string())
                }
            }
            JsonValue::String(s) => FhirPathValue::string(s),
            JsonValue::Array(arr) => {
                let values: Vec<FhirPathValue> =
                    arr.into_iter().map(json_to_fhirpath_value).collect();
                FhirPathValue::collection(values)
            }
            JsonValue::Object(ref obj) => {
                // Check if this is a special FHIR type that should be converted to a specific FhirPathValue
                if let Some(fhir_type) = infer_fhir_type_from_json(obj) {
                    match fhir_type.as_str() {
                        "Quantity" => convert_json_to_quantity(obj),
                        _ => FhirPathValue::resource(json),
                    }
                } else {
                    FhirPathValue::resource(json)
                }
            }
            JsonValue::Null => FhirPathValue::Empty,
        }
    }

    /// Convert a JsonValue to a FhirPathValue with proper FHIR resource typing using ModelProvider
    pub async fn json_to_fhirpath_value_with_model_provider(
        json: JsonValue,
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
    ) -> crate::core::Result<FhirPathValue> {
        match json {
            JsonValue::Bool(b) => Ok(FhirPathValue::boolean(b)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(FhirPathValue::integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(FhirPathValue::decimal(
                        rust_decimal::Decimal::try_from(f).unwrap_or_default(),
                    ))
                } else {
                    Ok(FhirPathValue::string(n.to_string()))
                }
            }
            JsonValue::String(s) => Ok(FhirPathValue::string(s)),
            JsonValue::Array(arr) => {
                let mut values = Vec::new();
                for item in arr {
                    let value = Box::pin(json_to_fhirpath_value_with_model_provider(
                        item,
                        model_provider.clone(),
                    ))
                    .await?;
                    values.push(value);
                }
                Ok(FhirPathValue::collection(values))
            }
            JsonValue::Object(ref obj) => {
                // Check if this is a FHIR resource (has resourceType property)
                if let Some(resource_type) = obj.get("resourceType").and_then(|rt| rt.as_str()) {
                    // Extract resourceType and get proper TypeInfo from ModelProvider
                    let type_info = model_provider
                        .get_type(resource_type)
                        .await
                        .map_err(|e| {
                            crate::core::FhirPathError::evaluation_error(
                                crate::core::error_code::FP0054,
                                format!("ModelProvider error getting type '{resource_type}': {e}"),
                            )
                        })?
                        .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                            type_name: resource_type.to_string(),
                            singleton: Some(true),
                            namespace: Some("FHIR".to_string()),
                            name: Some(resource_type.to_string()),
                            is_empty: Some(false),
                        });

                    // Create properly typed resource
                    Ok(FhirPathValue::Resource(Arc::new(json), type_info, None))
                } else {
                    // Check if this is a special FHIR type that should be converted to a specific FhirPathValue
                    if let Some(fhir_type) = infer_fhir_type_from_json(obj) {
                        match fhir_type.as_str() {
                            "Quantity" => Ok(convert_json_to_quantity(obj)),
                            _ => Ok(FhirPathValue::resource(json)),
                        }
                    } else {
                        Ok(FhirPathValue::resource(json))
                    }
                }
            }
            JsonValue::Null => Ok(FhirPathValue::Empty),
        }
    }

    /// Extract a path from a JSON object (dot notation)
    pub fn extract_json_path<'a>(json: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            if let Some(obj) = current.as_object() {
                current = obj.get(part)?;
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// Check if a JSON value matches a type name
    pub fn json_value_type_name(value: &JsonValue) -> &'static str {
        match value {
            JsonValue::Bool(_) => "Boolean",
            JsonValue::Number(n) => {
                if n.is_i64() {
                    "Integer"
                } else {
                    "Decimal"
                }
            }
            JsonValue::String(_) => "String",
            JsonValue::Array(_) => "Array",
            JsonValue::Object(_) => "Object",
            JsonValue::Null => "Null",
        }
    }
}
