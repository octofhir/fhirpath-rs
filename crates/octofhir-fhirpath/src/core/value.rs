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
            JsonValue::Object(_) => FhirPathValue::resource(json),
            JsonValue::Null => FhirPathValue::Empty,
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
