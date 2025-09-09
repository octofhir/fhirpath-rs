//! FHIR utilities for registry functions

use serde_json::{Map, Value as JsonValue};
use std::sync::Arc;
use crate::core::types::Collection;

use crate::core::{FhirPathValue, Result};

/// Utilities for FHIR-specific operations on JSON-backed values
pub struct FhirUtils;

impl FhirUtils {
    /// Extract primitive-ish value from a FHIR element object.
    ///
    /// Heuristics:
    /// - If field "value" exists, use it
    /// - If a key starts with "value" and is followed by uppercase (valueString, valueInteger, ...), use it
    /// Returns None if no such field is present.
    pub fn extract_primitive_value(obj: &Map<String, JsonValue>) -> Result<Option<FhirPathValue>> {
        if let Some(v) = obj.get("value") {
            return Ok(Some(json_to_value(v.clone())));
        }

        if let Some((_, v)) = obj.iter().find(|(k, _)| Self::is_value_x_key(k)) {
            return Ok(Some(json_to_value(v.clone())));
        }

        Ok(None)
    }

    /// Return true if key is a FHIR valueX selection, e.g., valueString, valueCodeableConcept
    fn is_value_x_key(key: &str) -> bool {
        key.starts_with("value")
            && key
                .chars()
                .nth(5)
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
    }

    /// Collect all descendant elements recursively
    pub fn collect_descendants(value: &FhirPathValue, results: &mut Vec<FhirPathValue>) {
        match value {
            FhirPathValue::Resource(json) | FhirPathValue::JsonValue(json) => {
                if let Some(obj) = json.as_object() {
                    for (_, child) in obj.iter() {
                        // Flatten arrays
                        if let Some(arr) = child.as_array() {
                            for item in arr {
                                let v = json_to_value(item.clone());
                                results.push(v.clone());
                                Self::collect_descendants(&v, results);
                            }
                        } else {
                            let v = json_to_value(child.clone());
                            results.push(v.clone());
                            Self::collect_descendants(&v, results);
                        }
                    }
                }
            }
            // Primitive FHIRPath values have no descendants
            _ => {}
        }
    }

    /// Get all direct children of a JSON-backed value
    pub fn collect_children(value: &FhirPathValue) -> Vec<FhirPathValue> {
        let mut out = Vec::new();
        match value {
            FhirPathValue::Resource(json) | FhirPathValue::JsonValue(json) => {
                if let Some(obj) = json.as_object() {
                    for (_, child) in obj.iter() {
                        if let Some(arr) = child.as_array() {
                            for item in arr {
                                out.push(json_to_value(item.clone()));
                            }
                        } else {
                            out.push(json_to_value(child.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
        out
    }

    /// Get all extensions and modifierExtensions from a value
    pub fn get_extensions(value: &FhirPathValue) -> Vec<FhirPathValue> {
        let mut res = Vec::new();
        let json = match value {
            FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => j,
            _ => return res,
        };

        if let Some(obj) = json.as_object() {
            if let Some(ext) = obj.get("extension").and_then(|v| v.as_array()) {
                for e in ext.iter().cloned() {
                    res.push(json_to_value(e));
                }
            }
            if let Some(ext) = obj.get("modifierExtension").and_then(|v| v.as_array()) {
                for e in ext.iter().cloned() {
                    res.push(json_to_value(e));
                }
            }
        }
        res
    }

    /// Filter extensions by `url`
    pub fn filter_extensions_by_url(value: &FhirPathValue, url: &str) -> Vec<FhirPathValue> {
        Self::get_extensions(value)
            .into_iter()
            .filter(|ext| match ext {
                FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => j
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(|s| s == url)
                    .unwrap_or(false),
                _ => false,
            })
            .collect()
    }
}

fn json_to_value(v: JsonValue) -> FhirPathValue {
    match v {
        JsonValue::Bool(b) => FhirPathValue::boolean(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                FhirPathValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                FhirPathValue::decimal(rust_decimal::Decimal::try_from(f).unwrap_or_default())
            } else {
                FhirPathValue::string(n.to_string())
            }
        }
        JsonValue::String(s) => FhirPathValue::string(s),
        JsonValue::Array(arr) => {
            let values = arr.into_iter().map(json_to_value).collect();
            FhirPathValue::Collection(Collection::from_values(values))
        }
        JsonValue::Object(_) => FhirPathValue::Resource(Arc::new(v)),
        JsonValue::Null => FhirPathValue::Empty,
    }
}
