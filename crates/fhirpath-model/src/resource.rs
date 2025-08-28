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

//! FHIR resource wrapper types using sonic-rs exclusively

use super::json_value::JsonValue as FhirJsonValue;
use serde::{Deserialize, Serialize};
use serde_json;

/// Represents a FHIR resource or complex object using sonic-rs exclusively
#[derive(Debug, Clone, PartialEq)]
pub struct FhirResource {
    /// The JSON representation of the resource using sonic-rs
    data: FhirJsonValue,
    /// Optional resource type for optimization
    resource_type: Option<String>,
}

impl FhirResource {
    /// Create a new FHIR resource from serde_json::Value directly
    pub fn from_json(data: serde_json::Value) -> Self {
        let json_value = FhirJsonValue::new(data);

        let resource_type = json_value
            .get_property("resourceType")
            .and_then(|rt| rt.as_str().map(|s| s.to_string()));

        Self {
            data: json_value,
            resource_type,
        }
    }

    /// Create a new FHIR resource from JsonValue (zero-copy)
    pub fn from_json_value(data: FhirJsonValue) -> Self {
        let resource_type = data
            .get_property("resourceType")
            .and_then(|rt| rt.as_str().map(|s| s.to_string()));

        Self {
            data,
            resource_type,
        }
    }

    /// Create from sonic-rs value directly
    pub fn from_sonic_value(value: serde_json::Value) -> Self {
        let json_value = FhirJsonValue::new(value);
        Self::from_json_value(json_value)
    }

    /// Get the JSON representation as serde_json::Value
    pub fn to_json(&self) -> serde_json::Value {
        self.data.as_value().clone()
    }

    /// Get a reference to the JSON data as serde_json::Value
    pub fn as_json(&self) -> serde_json::Value {
        self.data.as_value().clone()
    }

    /// Get the JsonValue for efficient sharing
    pub fn as_json_value(&self) -> &FhirJsonValue {
        &self.data
    }

    /// Get the sonic-rs value directly
    pub fn as_sonic_value(&self) -> &serde_json::Value {
        self.data.as_value()
    }

    /// Get the resource type if available
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// Get the actual property name used for polymorphic "value" access
    pub fn get_value_property_name(&self) -> Option<String> {
        if !self.data.is_object() {
            return None;
        }

        // Look through all properties for value[x] patterns
        if self.data.as_value().is_object() {
            // Use sonic-rs API to iterate through object keys
            let sonic_value = self.data.as_value();
            if let Some(obj) = sonic_value.as_object() {
                for (key, _) in obj.iter() {
                    if key.starts_with("value") && key.len() > 5 {
                        return Some(key.to_string());
                    }
                }
            }
        }
        None
    }

    /// Get property value and the actual property name used (for polymorphic properties)
    pub fn get_property_with_name(&self, path: &str) -> Option<(serde_json::Value, String)> {
        if !self.data.is_object() {
            return None;
        }

        // First try direct property access
        if let Some(value) = self.data.get_property(path) {
            return Some((value.into_inner(), path.to_string()));
        }

        // Handle FHIR polymorphic properties (e.g., value -> valueString, valueInteger, etc.)
        if path == "value" && self.data.as_value().is_object() {
            let sonic_value = self.data.as_value();
            if let Some(obj) = sonic_value.as_object() {
                for (key, _) in obj.iter() {
                    if key.starts_with("value") && key.len() > 5 {
                        if let Some(value) = self.data.get_property(key) {
                            return Some((value.into_inner(), key.to_string()));
                        }
                    }
                }
            }
        }

        None
    }

    /// Get primitive extensions for a property (following _propertyName convention)
    pub fn get_primitive_extensions(&self, property_name: &str) -> Option<FhirJsonValue> {
        let extension_key = format!("_{property_name}");
        self.data.get_property(&extension_key)
    }

    /// Get a property value by path using sonic-rs
    pub fn get_property(&self, path: &str) -> Option<serde_json::Value> {
        if !self.data.is_object() {
            return None;
        }

        // First try direct property access
        if let Some(json_value) = self.data.get_property(path) {
            return Some(json_value.into_inner());
        }

        // FHIR choice type polymorphic access
        if self.data.as_value().is_object() {
            let sonic_value = self.data.as_value();
            if let Some(obj) = sonic_value.as_object() {
                let mut valid_matches = Vec::new();
                for (key, _) in obj.iter() {
                    if key.starts_with(path) && key.len() > path.len() {
                        // Check if the next character after the property name is uppercase
                        if let Some(next_char) = key.chars().nth(path.len()) {
                            if next_char.is_uppercase() {
                                if let Some(value) = self.data.get_property(key) {
                                    valid_matches.push(value.into_inner());
                                }
                            }
                        }
                    }
                }

                // Return the first valid match
                valid_matches.into_iter().next()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a property value by path (JsonValue-optimized version)
    pub fn get_property_json_value(&self, path: &str) -> Option<FhirJsonValue> {
        self.data.get_property(path)
    }

    /// Get a property value by path, supporting nested navigation
    pub fn get_property_deep(&self, path: &str) -> Option<serde_json::Value> {
        // Handle dot notation for nested property access
        if path.contains('.') {
            let parts: Vec<&str> = path.split('.').collect();
            let mut current = self.data.as_value().clone();

            for part in parts {
                if current.is_object() {
                    if let Some(next_value) = current.get(part) {
                        current = next_value.clone();
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            Some(current)
        } else {
            // Simple property access
            self.get_property(path)
        }
    }

    /// Check if this resource has a specific property
    pub fn has_property(&self, path: &str) -> bool {
        self.get_property(path).is_some()
    }

    /// Get all properties as a vector of (key, value) pairs
    pub fn properties(&self) -> Vec<(String, serde_json::Value)> {
        if self.data.as_value().is_object() {
            let sonic_value = self.data.as_value();
            if let Some(obj) = sonic_value.as_object() {
                obj.iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Check if this is a primitive extension
    pub fn is_primitive_extension(&self, property: &str) -> bool {
        if self.data.as_value().is_object() {
            let key = format!("_{property}");
            self.data.as_value().get(&key).is_some()
        } else {
            false
        }
    }

    /// Get the primitive extension for a property
    pub fn get_primitive_extension(&self, property: &str) -> Option<serde_json::Value> {
        if self.data.as_value().is_object() {
            let key = format!("_{property}");
            self.data.as_value().get(&key).cloned()
        } else {
            None
        }
    }
}

// Custom Serialize implementation
impl Serialize for FhirResource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the sonic-rs value using sonic-rs's serde implementation
        self.data.as_value().serialize(serializer)
    }
}

// Custom Deserialize implementation
impl<'de> Deserialize<'de> for FhirResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(FhirResource::from_sonic_value(value))
    }
}

impl From<FhirJsonValue> for FhirResource {
    fn from(data: FhirJsonValue) -> Self {
        Self::from_json_value(data)
    }
}
impl From<serde_json::Value> for FhirResource {
    fn from(data: serde_json::Value) -> Self {
        Self::from_sonic_value(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resource_creation() {
        let sonic_json = json!({
            "resourceType": "Patient",
            "id": "123",
            "name": [{
                "given": ["John"],
                "family": "Doe"
            }]
        });

        let resource = FhirResource::from_sonic_value(sonic_json);
        assert_eq!(resource.resource_type(), Some("Patient"));
    }

    #[test]
    fn test_property_access() {
        let sonic_json = json!({
            "resourceType": "Patient",
            "id": "123",
            "active": true,
            "name": [{
                "given": ["John"],
                "family": "Doe"
            }]
        });

        let resource = FhirResource::from_sonic_value(sonic_json);

        assert!(resource.get_property("id").is_some());
        assert!(resource.get_property("active").is_some());
        assert!(resource.has_property("name"));
        assert!(!resource.has_property("nonexistent"));
    }

    #[test]
    fn test_primitive_extensions() {
        let sonic_json = json!({
            "resourceType": "Patient",
            "id": "123",
            "_id": {
                "extension": [{
                    "url": "http://example.com/ext",
                    "valueString": "extended"
                }]
            }
        });

        let resource = FhirResource::from_sonic_value(sonic_json);

        assert!(resource.is_primitive_extension("id"));
        assert!(resource.get_primitive_extension("id").is_some());
        assert!(!resource.is_primitive_extension("resourceType"));
    }

    #[test]
    fn test_fhir_choice_type_polymorphic_access() {
        let sonic_json = json!({
            "resourceType": "Observation",
            "id": "obs1",
            "valueString": "test result"
        });

        let resource = FhirResource::from_sonic_value(sonic_json);

        // Both direct and polymorphic access should work
        assert!(resource.get_property("valueString").is_some());
        assert!(resource.get_property("value").is_some());
    }

    #[test]
    fn test_deep_property_access() {
        let sonic_json = json!({
            "resourceType": "Observation",
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "12345"
                }],
                "text": "Test Code"
            }
        });

        let resource = FhirResource::from_sonic_value(sonic_json);

        assert!(resource.get_property_deep("code.text").is_some());
    }
}
