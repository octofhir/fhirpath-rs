//! FHIR resource wrapper types

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a FHIR resource or complex object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FhirResource {
    /// The JSON representation of the resource
    data: Value,
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
            data,
            resource_type,
        }
    }

    /// Get the JSON representation
    pub fn to_json(&self) -> Value {
        self.data.clone()
    }

    /// Get a reference to the JSON data
    pub fn as_json(&self) -> &Value {
        &self.data
    }

    /// Get the resource type if available
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// Get a property value by path
    pub fn get_property(&self, path: &str) -> Option<&Value> {
        // Handle simple property access on JSON objects
        match &self.data {
            Value::Object(obj) => obj.get(path),
            _ => None,
        }
    }

    /// Get a property value by path, supporting nested navigation
    pub fn get_property_deep(&self, path: &str) -> Option<&Value> {
        // Handle dot notation for nested property access
        if path.contains('.') {
            let parts: Vec<&str> = path.split('.').collect();
            let mut current = &self.data;

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
        match &self.data {
            Value::Object(obj) => obj.iter().map(|(k, v)| (k.as_str(), v)).collect(),
            _ => Vec::new(),
        }
    }

    /// Check if this is a primitive extension
    pub fn is_primitive_extension(&self, property: &str) -> bool {
        if let Value::Object(obj) = &self.data {
            obj.contains_key(&format!("_{}", property))
        } else {
            false
        }
    }

    /// Get the primitive extension for a property
    pub fn get_primitive_extension(&self, property: &str) -> Option<&Value> {
        match &self.data {
            Value::Object(obj) => obj.get(&format!("_{}", property)),
            _ => None,
        }
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
}
