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

//! FHIR resource wrapper types

use super::json_arc::ArcJsonValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a FHIR resource or complex object
#[derive(Debug, Clone, PartialEq)]
pub struct FhirResource {
    /// The JSON representation of the resource (Arc-wrapped for efficiency)
    data: ArcJsonValue,
    /// Optional resource type for optimization
    resource_type: Option<String>,
}

impl FhirResource {
    /// Create a new FHIR resource from JSON
    pub fn from_json(data: Value) -> Self {
        let resource_type = data
            .as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|rt| rt.as_str())
            .map(|s| s.to_string());

        Self {
            data: ArcJsonValue::new(data),
            resource_type,
        }
    }

    /// Create a new FHIR resource from ArcJsonValue (zero-copy)
    pub fn from_arc_json(data: ArcJsonValue) -> Self {
        let resource_type = data
            .as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|rt| rt.as_str())
            .map(|s| s.to_string());

        Self {
            data,
            resource_type,
        }
    }

    /// Get the JSON representation (clones only if necessary)
    pub fn to_json(&self) -> Value {
        self.data.clone_inner()
    }

    /// Get a reference to the JSON data
    pub fn as_json(&self) -> &Value {
        self.data.as_json()
    }

    /// Get the ArcJsonValue for efficient sharing
    pub fn as_arc_json(&self) -> &ArcJsonValue {
        &self.data
    }

    /// Get the resource type if available
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// Get the actual property name used for polymorphic "value" access
    pub fn get_value_property_name(&self) -> Option<String> {
        match self.data.as_json() {
            Value::Object(obj) => {
                // Look for value[x] properties
                for key in obj.keys() {
                    if key.starts_with("value") && key.len() > 5 {
                        return Some(key.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Get property value and the actual property name used (for polymorphic properties)
    pub fn get_property_with_name(&self, path: &str) -> Option<(&Value, String)> {
        match self.data.as_json() {
            Value::Object(obj) => {
                // First try direct property access
                if let Some(value) = obj.get(path) {
                    return Some((value, path.to_string()));
                }

                // Handle FHIR polymorphic properties (e.g., value -> valueString, valueInteger, etc.)
                if path == "value" {
                    // Look for value[x] properties
                    for key in obj.keys() {
                        if key.starts_with("value") && key.len() > 5 {
                            // Found a value[x] property (like valueString, valueInteger, etc.)
                            if let Some(value) = obj.get(key) {
                                return Some((value, key.clone()));
                            }
                        }
                    }
                }

                None
            }
            _ => None,
        }
    }

    /// Get primitive extensions for a property (following _propertyName convention)
    pub fn get_primitive_extensions(&self, property_name: &str) -> Option<&Value> {
        let extension_key = format!("_{property_name}");
        self.get_property(&extension_key)
    }

    /// Get a property value by path
    pub fn get_property(&self, path: &str) -> Option<&Value> {
        // Handle simple property access on JSON objects
        match self.data.as_json() {
            Value::Object(obj) => {
                // First try direct property access
                if let Some(value) = obj.get(path) {
                    return Some(value);
                }

                // FHIR choice type polymorphic access
                // Check for properties that start with the requested property name
                // and have an uppercase letter immediately following (e.g., "value" matches "valueString")
                // Collect all valid matches and return the first one found
                let mut valid_matches = Vec::new();
                for (key, value) in obj.iter() {
                    if key.starts_with(path) && key.len() > path.len() {
                        // Check if the next character after the property name is uppercase
                        if let Some(next_char) = key.chars().nth(path.len()) {
                            if next_char.is_uppercase() {
                                valid_matches.push((key, value));
                            }
                        }
                    }
                }

                // Return the first valid match (deterministic based on JSON ordering)
                if let Some((_, value)) = valid_matches.first() {
                    return Some(value);
                }

                None
            }
            _ => None,
        }
    }

    /// Get a property value by path (Arc-optimized version)
    pub fn get_property_arc(&self, path: &str) -> Option<ArcJsonValue> {
        // Use the efficient Arc-based property access
        let result = self.data.get_property(path);

        // Handle FHIR polymorphic properties if direct access failed
        if result.is_none() {
            if let Value::Object(obj) = self.data.as_json() {
                // FHIR choice type polymorphic access
                let mut valid_matches = Vec::new();
                for key in obj.keys() {
                    if key.starts_with(path) && key.len() > path.len() {
                        // Check if the next character after the property name is uppercase
                        if let Some(next_char) = key.chars().nth(path.len()) {
                            if next_char.is_uppercase() {
                                valid_matches.push(key);
                            }
                        }
                    }
                }

                // Return the first valid match
                if let Some(key) = valid_matches.first() {
                    return self.data.get_property(key);
                }
            }
        }

        result
    }

    /// Get a property value by path, supporting nested navigation
    pub fn get_property_deep(&self, path: &str) -> Option<&Value> {
        // Handle dot notation for nested property access
        if path.contains('.') {
            let parts: Vec<&str> = path.split('.').collect();
            let mut current = self.data.as_json();

            for part in parts {
                match current {
                    Value::Object(obj) => {
                        current = obj.get(part)?;
                    }
                    Value::Array(_) => {
                        // For arrays, this is handled by the caller
                        return None;
                    }
                    _ => return None,
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
    pub fn properties(&self) -> Vec<(&str, &Value)> {
        match self.data.as_json() {
            Value::Object(obj) => obj.iter().map(|(k, v)| (k.as_str(), v)).collect(),
            _ => Vec::new(),
        }
    }

    /// Check if this is a primitive extension
    pub fn is_primitive_extension(&self, property: &str) -> bool {
        if let Value::Object(obj) = self.data.as_json() {
            obj.contains_key(&format!("_{property}"))
        } else {
            false
        }
    }

    /// Get the primitive extension for a property
    pub fn get_primitive_extension(&self, property: &str) -> Option<&Value> {
        match self.data.as_json() {
            Value::Object(obj) => obj.get(&format!("_{property}")),
            _ => None,
        }
    }
}

// Custom Serialize implementation
impl Serialize for FhirResource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.as_json().serialize(serializer)
    }
}

// Custom Deserialize implementation
impl<'de> Deserialize<'de> for FhirResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(FhirResource::from_json(value))
    }
}

impl From<ArcJsonValue> for FhirResource {
    fn from(data: ArcJsonValue) -> Self {
        Self::from_arc_json(data)
    }
}

impl From<Value> for FhirResource {
    fn from(data: Value) -> Self {
        Self::from_json(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resource_creation() {
        let json = json!({
            "resourceType": "Patient",
            "id": "123",
            "name": [{
                "given": ["John"],
                "family": "Doe"
            }]
        });

        let resource = FhirResource::from_json(json.clone());
        assert_eq!(resource.resource_type(), Some("Patient"));
        assert_eq!(resource.to_json(), json);
    }

    #[test]
    fn test_property_access() {
        let json = json!({
            "resourceType": "Patient",
            "id": "123",
            "active": true,
            "name": [{
                "given": ["John"],
                "family": "Doe"
            }]
        });

        let resource = FhirResource::from_json(json);

        assert_eq!(resource.get_property("id"), Some(&json!("123")));
        assert_eq!(resource.get_property("active"), Some(&json!(true)));
        assert!(resource.has_property("name"));
        assert!(!resource.has_property("nonexistent"));
    }

    #[test]
    fn test_nested_property_access() {
        let json = json!({
            "resourceType": "Observation",
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "12345"
                }],
                "text": "Test Code"
            }
        });

        let resource = FhirResource::from_json(json);

        assert_eq!(
            resource.get_property_deep("code.text"),
            Some(&json!("Test Code"))
        );
    }

    #[test]
    fn test_primitive_extensions() {
        let json = json!({
            "resourceType": "Patient",
            "id": "123",
            "_id": {
                "extension": [{
                    "url": "http://example.com/ext",
                    "valueString": "extended"
                }]
            }
        });

        let resource = FhirResource::from_json(json);

        assert!(resource.is_primitive_extension("id"));
        assert!(resource.get_primitive_extension("id").is_some());
        assert!(!resource.is_primitive_extension("resourceType"));
    }

    #[test]
    fn test_fhir_choice_type_polymorphic_access() {
        // Test with Observation.value[x] polymorphic access
        let observation_string = json!({
            "resourceType": "Observation",
            "id": "obs1",
            "valueString": "test result"
        });

        let resource = FhirResource::from_json(observation_string);

        // Both direct and polymorphic access should work
        assert_eq!(
            resource.get_property("valueString"),
            Some(&json!("test result"))
        );
        assert_eq!(resource.get_property("value"), Some(&json!("test result")));
    }

    #[test]
    fn test_fhir_choice_type_different_types() {
        // Test with valueQuantity
        let observation_quantity = json!({
            "resourceType": "Observation",
            "id": "obs2",
            "valueQuantity": {
                "value": 120,
                "unit": "mmHg"
            }
        });

        let resource = FhirResource::from_json(observation_quantity);
        let expected_quantity = json!({
            "value": 120,
            "unit": "mmHg"
        });

        assert_eq!(
            resource.get_property("valueQuantity"),
            Some(&expected_quantity)
        );
        assert_eq!(resource.get_property("value"), Some(&expected_quantity));

        // Test with valueBoolean
        let observation_boolean = json!({
            "resourceType": "Observation",
            "id": "obs3",
            "valueBoolean": true
        });

        let resource = FhirResource::from_json(observation_boolean);
        assert_eq!(resource.get_property("valueBoolean"), Some(&json!(true)));
        assert_eq!(resource.get_property("value"), Some(&json!(true)));
    }

    #[test]
    fn test_fhir_choice_type_case_sensitivity() {
        // Test that only uppercase letters after the base property name match
        let test_resource = json!({
            "resourceType": "TestResource",
            "valueString": "should_match",
            "valuetype": "should_not_match",
            "valuelowercase": "should_not_match_either"
        });

        let resource = FhirResource::from_json(test_resource);

        // Should match valueString (uppercase S), not the lowercase variants
        let result = resource.get_property("value");
        assert!(result.is_some());
        assert_eq!(result, Some(&json!("should_match")));

        // Direct access to non-matching properties should still work
        assert_eq!(
            resource.get_property("valuetype"),
            Some(&json!("should_not_match"))
        );
        assert_eq!(
            resource.get_property("valuelowercase"),
            Some(&json!("should_not_match_either"))
        );

        // Test with only lowercase variants (should return None)
        let test_resource_lowercase_only = json!({
            "resourceType": "TestResource",
            "valuetype": "lowercase_only",
            "valuelowercase": "also_lowercase"
        });

        let resource2 = FhirResource::from_json(test_resource_lowercase_only);
        assert_eq!(resource2.get_property("value"), None);
    }

    #[test]
    fn test_fhir_choice_type_medication_property() {
        // Test other common FHIR choice types beyond just "value"
        let medication_request = json!({
            "resourceType": "MedicationRequest",
            "id": "med1",
            "medicationCodeableConcept": {
                "coding": [{
                    "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                    "code": "582620",
                    "display": "Nizatidine"
                }]
            }
        });

        let resource = FhirResource::from_json(medication_request);
        let expected_medication = json!({
            "coding": [{
                "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                "code": "582620",
                "display": "Nizatidine"
            }]
        });

        // Both direct and polymorphic access should work
        assert_eq!(
            resource.get_property("medicationCodeableConcept"),
            Some(&expected_medication)
        );
        assert_eq!(
            resource.get_property("medication"),
            Some(&expected_medication)
        );
    }

    #[test]
    fn test_fhir_choice_type_direct_property_priority() {
        // Test that direct property access takes priority over polymorphic
        let test_resource = json!({
            "resourceType": "TestResource",
            "value": "direct_value",
            "valueString": "polymorphic_value"
        });

        let resource = FhirResource::from_json(test_resource);

        // Should get direct value, not polymorphic
        assert_eq!(resource.get_property("value"), Some(&json!("direct_value")));
        assert_eq!(
            resource.get_property("valueString"),
            Some(&json!("polymorphic_value"))
        );
    }

    #[test]
    fn test_fhir_choice_type_no_match() {
        // Test that accessing a choice type property that doesn't exist returns None
        let observation_no_value = json!({
            "resourceType": "Observation",
            "id": "obs4",
            "status": "final"
        });

        let resource = FhirResource::from_json(observation_no_value);
        assert_eq!(resource.get_property("value"), None);
        assert_eq!(resource.get_property("medication"), None);
    }

    #[test]
    fn test_fhir_choice_type_arc_access() {
        // Test that Arc-optimized access also supports polymorphic access
        let observation = json!({
            "resourceType": "Observation",
            "valueString": "test via arc"
        });

        let resource = FhirResource::from_json(observation);

        // Both get_property and get_property_arc should work for polymorphic access
        assert_eq!(resource.get_property("value"), Some(&json!("test via arc")));

        let arc_result = resource.get_property_arc("value");
        assert!(arc_result.is_some());
        if let Some(arc_json) = arc_result {
            assert_eq!(arc_json.as_json(), &json!("test via arc"));
        }
    }
}
