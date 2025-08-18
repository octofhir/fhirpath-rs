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

//! Primary JSON value implementation using sonic-rs exclusively
//!
//! This module provides the core JSON value type that uses sonic-rs for
//! high-performance SIMD-accelerated parsing and manipulation.

use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value as InternalValue};

/// High-performance JSON value wrapper using sonic-rs exclusively
///
/// This is the primary JSON value type used throughout the FHIRPath engine.
/// It leverages sonic-rs for SIMD-accelerated parsing and zero-copy operations.
#[derive(Clone, Debug)]
pub struct JsonValue {
    inner: InternalValue,
}

impl JsonValue {
    /// Parse JSON string with high-performance sonic-rs parser
    pub fn parse(input: &str) -> Result<Self, sonic_rs::Error> {
        let value = sonic_rs::from_str(input)?;
        Ok(Self { inner: value })
    }

    /// Create a new JsonValue from sonic_rs::Value
    pub fn new(value: InternalValue) -> Self {
        Self { inner: value }
    }

    /// Get a reference to the underlying sonic_rs::Value
    pub fn as_inner(&self) -> &InternalValue {
        &self.inner
    }

    /// Convert to owned sonic_rs::Value
    pub fn into_inner(self) -> InternalValue {
        self.inner
    }

    /// Zero-copy property access for objects
    pub fn get_property(&self, key: &str) -> Option<JsonValue> {
        if self.inner.is_object() {
            self.inner
                .get(key)
                .map(|value| JsonValue::new(value.clone()))
        } else {
            None
        }
    }

    /// Zero-copy array index access
    pub fn get_index(&self, index: usize) -> Option<JsonValue> {
        if self.inner.is_array() {
            self.inner
                .get(index)
                .map(|value| JsonValue::new(value.clone()))
        } else {
            None
        }
    }

    /// Get array length
    pub fn array_len(&self) -> Option<usize> {
        if self.inner.is_array() {
            self.inner.as_array().map(|arr| arr.len())
        } else {
            None
        }
    }

    /// Get object keys count
    pub fn object_len(&self) -> Option<usize> {
        if self.inner.is_object() {
            self.inner.as_object().map(|obj| obj.len())
        } else {
            None
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }

    /// Check if value is boolean
    pub fn is_boolean(&self) -> bool {
        self.inner.is_boolean()
    }

    /// Check if value is number
    pub fn is_number(&self) -> bool {
        self.inner.is_number()
    }

    /// Check if value is string
    pub fn is_string(&self) -> bool {
        self.inner.is_str()
    }

    /// Check if value is array
    pub fn is_array(&self) -> bool {
        self.inner.is_array()
    }

    /// Check if value is object
    pub fn is_object(&self) -> bool {
        self.inner.is_object()
    }

    /// Get boolean value if this is a boolean
    pub fn as_bool(&self) -> Option<bool> {
        self.inner.as_bool()
    }

    /// Get string value if this is a string
    pub fn as_str(&self) -> Option<&str> {
        self.inner.as_str()
    }

    /// Get number value as f64 if this is a number
    pub fn as_f64(&self) -> Option<f64> {
        self.inner.as_f64()
    }

    /// Get number value as i64 if this is a number
    pub fn as_i64(&self) -> Option<i64> {
        self.inner.as_i64()
    }

    /// Get number value as u64 if this is a number
    pub fn as_u64(&self) -> Option<u64> {
        self.inner.as_u64()
    }

    /// Iterate over array elements
    pub fn array_iter(&self) -> Option<ArrayIter> {
        if self.inner.is_array() {
            let len = self.array_len().unwrap_or(0);
            Some(ArrayIter {
                value: &self.inner,
                index: 0,
                len,
            })
        } else {
            None
        }
    }

    /// Iterate over object properties (key-value pairs)
    pub fn object_iter(&self) -> Option<ObjectIter> {
        if self.inner.is_object() {
            Some(ObjectIter {
                value: &self.inner,
                keys: Vec::new(), // Will be populated on first iteration
                index: 0,
                initialized: false,
            })
        } else {
            None
        }
    }

    /// Check if this represents a FHIR Bundle  
    pub fn is_bundle(&self) -> bool {
        if let Some(resource_type) = self.get_property("resourceType") {
            resource_type.as_str() == Some("Bundle")
        } else {
            false
        }
    }

    /// Get Bundle entries (no conversion)
    pub fn get_bundle_entries(&self) -> Option<Vec<JsonValue>> {
        if !self.is_bundle() {
            return None;
        }

        let entries = self.get_property("entry")?;
        entries.array_iter().map(|iter| iter.collect())
    }

    /// Filter Bundle entries by resource type
    pub fn get_bundle_entries_by_type(&self, resource_type: &str) -> Option<Vec<JsonValue>> {
        let entries = self.get_bundle_entries()?;
        let filtered: Vec<JsonValue> = entries
            .into_iter()
            .filter_map(|entry| {
                let resource = entry.get_property("resource")?;
                let res_type = resource.get_property("resourceType")?;
                if res_type.as_str() == Some(resource_type) {
                    Some(resource) // Return the resource, not the entry
                } else {
                    None
                }
            })
            .collect();

        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    }

    /// Convert to JSON string
    pub fn to_string(&self) -> Result<String, sonic_rs::Error> {
        sonic_rs::to_string(&self.inner)
    }

    /// Convert to pretty-printed JSON string
    pub fn to_string_pretty(&self) -> Result<String, sonic_rs::Error> {
        sonic_rs::to_string_pretty(&self.inner)
    }

    // === Compatibility Methods for Migration ===

    /// Get as sonic_rs::Value directly (zero-copy)
    pub fn as_sonic_value(&self) -> &InternalValue {
        &self.inner
    }

    /// Create from any serializable type using sonic-rs
    pub fn from_value<T: serde::Serialize>(value: &T) -> Result<Self, String> {
        let inner =
            sonic_rs::to_value(value).map_err(|e| format!("sonic-rs serialization error: {e}"))?;
        Ok(Self { inner })
    }

    /// Convert to any deserializable type using sonic-rs
    pub fn to_value<T: serde::de::DeserializeOwned>(&self) -> Result<T, String> {
        sonic_rs::from_value(&self.inner)
            .map_err(|e| format!("sonic-rs deserialization error: {e}"))
    }
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<InternalValue> for JsonValue {
    fn from(value: InternalValue) -> Self {
        Self::new(value)
    }
}

/// Iterator over array elements
pub struct ArrayIter<'a> {
    value: &'a InternalValue,
    index: usize,
    len: usize,
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = JsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let result = self
                .value
                .get(self.index)
                .map(|value| JsonValue::new(value.clone()));
            self.index += 1;
            result
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

/// Iterator over object properties (key-value pairs)
pub struct ObjectIter<'a> {
    value: &'a InternalValue,
    keys: Vec<String>,
    index: usize,
    initialized: bool,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = (String, JsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        use sonic_rs::{JsonContainerTrait, JsonValueTrait};

        // Initialize keys on first iteration
        if !self.initialized {
            if let Some(obj) = self.value.as_object() {
                self.keys = obj.iter().map(|(k, _)| k.to_string()).collect();
            }
            self.initialized = true;
        }

        if self.index < self.keys.len() {
            let key = &self.keys[self.index];
            let result = self
                .value
                .get(key)
                .map(|value| (key.clone(), JsonValue::new(value.clone())));
            self.index += 1;
            result
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if !self.initialized {
            (0, None) // Unknown until initialized
        } else {
            let remaining = self.keys.len() - self.index;
            (remaining, Some(remaining))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parsing() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let json_value = JsonValue::parse(json_str).unwrap();

        assert!(json_value.is_object());
        assert_eq!(json_value.object_len(), Some(2));
    }

    #[test]
    fn test_property_access() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let json_value = JsonValue::parse(json_str).unwrap();

        let name = json_value.get_property("name").unwrap();
        assert_eq!(name.as_str(), Some("John"));

        let age = json_value.get_property("age").unwrap();
        assert_eq!(age.as_i64(), Some(30));
    }

    #[test]
    fn test_array_access() {
        let json_str = r#"[1, 2, 3, 4, 5]"#;
        let json_value = JsonValue::parse(json_str).unwrap();

        assert!(json_value.is_array());
        assert_eq!(json_value.array_len(), Some(5));

        let first = json_value.get_index(0).unwrap();
        assert_eq!(first.as_i64(), Some(1));

        let third = json_value.get_index(2).unwrap();
        assert_eq!(third.as_i64(), Some(3));
    }

    #[test]
    fn test_array_iteration() {
        let json_str = r#"[10, 20, 30]"#;
        let json_value = JsonValue::parse(json_str).unwrap();

        let values: Vec<i64> = json_value
            .array_iter()
            .unwrap()
            .map(|val| val.as_i64().unwrap())
            .collect();

        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn test_type_checks() {
        let test_cases = vec![
            (r#"null"#, "null"),
            (r#"true"#, "boolean"),
            (r#"42"#, "number"),
            (r#""hello""#, "string"),
            (r#"[]"#, "array"),
            (r#"{}"#, "object"),
        ];

        for (json_str, expected_type) in test_cases {
            let json_value = JsonValue::parse(json_str).unwrap();

            match expected_type {
                "null" => assert!(json_value.is_null()),
                "boolean" => assert!(json_value.is_boolean()),
                "number" => assert!(json_value.is_number()),
                "string" => assert!(json_value.is_string()),
                "array" => assert!(json_value.is_array()),
                "object" => assert!(json_value.is_object()),
                _ => panic!("Unknown type: {expected_type}"),
            }
        }
    }

    #[test]
    fn test_from_to_value() {
        let original = sonic_rs::json!({"name": "John", "age": 30});
        let json_value = JsonValue::from_value(&original).unwrap();

        assert!(json_value.is_object());
        let name = json_value.get_property("name").unwrap();
        assert_eq!(name.as_str(), Some("John"));

        let age = json_value.get_property("age").unwrap();
        assert_eq!(age.as_i64(), Some(30));
    }
}
