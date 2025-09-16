//! FhirPathWrapped system for type-preserving Arc-based value sharing
//!
//! This module implements the FhirPathWrapped<T> system that preserves type information
//! throughout the evaluation pipeline while enabling zero-copy operations through Arc sharing.

use crate::core::model_provider::TypeInfo;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// FHIR extension element  
#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    /// Extension URL identifier
    pub url: String,
    /// Extension value (can be any JSON value)
    pub value: JsonValue,
}

/// Primitive element containing extensions and id for FHIR primitive types
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveElement {
    /// Element id
    pub id: Option<String>,
    /// Extensions on this primitive element
    pub extensions: Vec<Extension>,
}

/// FhirPathWrapped value with Arc-based sharing and type preservation
#[derive(Debug, Clone)]
pub struct FhirPathWrapped<T> {
    /// Arc-shared value for zero-copy operations
    pub value: Arc<T>,
    /// Type information from ModelProvider
    pub type_info: Option<TypeInfo>,
    /// Primitive element extensions (for FHIR primitive types)
    pub primitive_element: Option<PrimitiveElement>,
}

impl Extension {
    /// Create a new extension
    pub fn new(url: String, value: JsonValue) -> Self {
        Self { url, value }
    }

    /// Create extension from JSON object (FHIR extension format)
    pub fn from_json(json: &JsonValue) -> crate::core::error::Result<Self> {
        let obj = json.as_object().ok_or_else(|| {
            crate::core::error::FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "Extension must be JSON object".to_string(),
            )
        })?;

        let url = obj
            .get("url")
            .and_then(|u| u.as_str())
            .ok_or_else(|| {
                crate::core::error::FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    "Extension must have 'url' field".to_string(),
                )
            })?
            .to_string();

        // Extract value* field (valueString, valueInteger, etc.)
        let value = obj
            .iter()
            .find(|(key, _)| key.starts_with("value"))
            .map(|(_, val)| val.clone())
            .unwrap_or(JsonValue::Null);

        Ok(Self { url, value })
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> JsonValue {
        let mut obj = serde_json::Map::new();
        obj.insert("url".to_string(), JsonValue::String(self.url.clone()));

        // Determine the value type and create appropriate value[x] field
        match &self.value {
            JsonValue::String(s) => {
                obj.insert("valueString".to_string(), JsonValue::String(s.clone()));
            }
            JsonValue::Number(n) => {
                obj.insert("valueDecimal".to_string(), JsonValue::Number(n.clone()));
            }
            JsonValue::Bool(b) => {
                obj.insert("valueBoolean".to_string(), JsonValue::Bool(*b));
            }
            other => {
                obj.insert(
                    "valueString".to_string(),
                    JsonValue::String(other.to_string()),
                );
            }
        }

        JsonValue::Object(obj)
    }
}

impl PrimitiveElement {
    /// Create a new primitive element
    pub fn new() -> Self {
        Self {
            id: None,
            extensions: Vec::new(),
        }
    }

    /// Create with ID
    pub fn with_id(id: String) -> Self {
        Self {
            id: Some(id),
            extensions: Vec::new(),
        }
    }

    /// Add an extension
    pub fn with_extension(mut self, extension: Extension) -> Self {
        self.extensions.push(extension);
        self
    }

    /// Create from JSON object (FHIR primitive element format)
    pub fn from_json(json: &JsonValue) -> crate::core::error::Result<Self> {
        let obj = json.as_object().ok_or_else(|| {
            crate::core::error::FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "PrimitiveElement must be JSON object".to_string(),
            )
        })?;

        let id = obj.get("id").and_then(|i| i.as_str()).map(String::from);

        let extensions = if let Some(ext_array) = obj.get("extension") {
            if let Some(array) = ext_array.as_array() {
                array
                    .iter()
                    .map(Extension::from_json)
                    .collect::<crate::core::error::Result<Vec<_>>>()?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(Self { id, extensions })
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> JsonValue {
        let mut obj = serde_json::Map::new();

        if let Some(id) = &self.id {
            obj.insert("id".to_string(), JsonValue::String(id.clone()));
        }

        if !self.extensions.is_empty() {
            let ext_array: Vec<JsonValue> =
                self.extensions.iter().map(|ext| ext.to_json()).collect();
            obj.insert("extension".to_string(), JsonValue::Array(ext_array));
        }

        JsonValue::Object(obj)
    }

    /// Check if this primitive element has any extensions
    pub fn has_extensions(&self) -> bool {
        !self.extensions.is_empty()
    }
}

impl Default for PrimitiveElement {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FhirPathWrapped<T> {
    /// Create a new wrapped value
    pub fn new(value: T, type_info: Option<TypeInfo>) -> Self {
        Self {
            value: Arc::new(value),
            type_info,
            primitive_element: None,
        }
    }

    /// Create with primitive element extensions
    pub fn with_primitive_element(
        value: T,
        type_info: Option<TypeInfo>,
        primitive_element: PrimitiveElement,
    ) -> Self {
        Self {
            value: Arc::new(value),
            type_info,
            primitive_element: Some(primitive_element),
        }
    }

    /// Get a reference to the wrapped value
    pub fn unwrap(&self) -> &T {
        &self.value
    }

    /// Get the Arc for zero-copy sharing
    pub fn arc(&self) -> &Arc<T> {
        &self.value
    }

    /// Get type information
    pub fn get_type_info(&self) -> Option<&TypeInfo> {
        self.type_info.as_ref()
    }

    /// Get primitive element (if any)
    pub fn get_primitive_element(&self) -> Option<&PrimitiveElement> {
        self.primitive_element.as_ref()
    }

    /// Check if this value has primitive extensions
    pub fn has_extensions(&self) -> bool {
        self.primitive_element
            .as_ref()
            .map(|pe| pe.has_extensions())
            .unwrap_or(false)
    }

    /// Clone the wrapped value with different type info
    pub fn with_type_info(&self, type_info: Option<TypeInfo>) -> Self {
        Self {
            value: self.value.clone(), // Arc clone - no data copy
            type_info,
            primitive_element: self.primitive_element.clone(),
        }
    }

    /// Add or update primitive element
    pub fn with_primitive(&self, primitive_element: Option<PrimitiveElement>) -> Self {
        Self {
            value: self.value.clone(), // Arc clone - no data copy
            type_info: self.type_info.clone(),
            primitive_element,
        }
    }
}

impl FhirPathWrapped<JsonValue> {
    /// Create wrapped JSON value for FHIR resource
    pub fn resource(json: JsonValue) -> Self {
        // Extract resource type for type information
        let resource_type = json
            .get("resourceType")
            .and_then(|rt| rt.as_str())
            .map(String::from);

        let type_info = resource_type.map(|rt| TypeInfo {
            type_name: rt.clone(),
            singleton: true,
            namespace: Some("FHIR".to_string()),
            name: Some(rt),
            is_empty: Some(false),
            is_union_type: Some(false),
            union_choices: None,
        });

        Self::new(json, type_info)
    }

    /// Create wrapped JSON value with explicit type info
    pub fn typed(json: JsonValue, type_info: TypeInfo) -> Self {
        Self::new(json, Some(type_info))
    }

    /// Create wrapped JSON value for primitive with extensions
    pub fn primitive_with_extensions(
        json: JsonValue,
        type_info: Option<TypeInfo>,
        primitive_element: PrimitiveElement,
    ) -> Self {
        Self::with_primitive_element(json, type_info, primitive_element)
    }

    /// Get property from wrapped JSON object with type preservation
    pub fn get_property(&self, property: &str) -> Option<FhirPathWrapped<JsonValue>> {
        if let Some(prop_value) = self.value.get(property) {
            // For collections, preserve as collection wrapper
            if prop_value.is_array() {
                return Some(FhirPathWrapped::new(
                    prop_value.clone(),
                    Some(TypeInfo {
                        type_name: "Collection".to_string(),
                        singleton: false,
                        namespace: None,
                        name: None,
                        is_empty: Some(prop_value.as_array().unwrap().is_empty()),
                        is_union_type: Some(false),
                        union_choices: None,
                    }),
                ));
            }

            // Check for primitive element (_{property} field)
            let primitive_key = format!("_{property}");
            let primitive_element = self
                .value
                .get(&primitive_key)
                .and_then(|pe_json| PrimitiveElement::from_json(pe_json).ok());

            // Create appropriate type info based on value type
            let type_info = match prop_value {
                JsonValue::String(_) => Some(TypeInfo {
                    type_name: "String".to_string(),
                    singleton: true,
                    namespace: Some("System".to_string()),
                    name: Some("String".to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }),
                JsonValue::Number(_) => Some(TypeInfo {
                    type_name: "Decimal".to_string(), // Could be Integer, but Decimal is safer
                    singleton: true,
                    namespace: Some("System".to_string()),
                    name: Some("Decimal".to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }),
                JsonValue::Bool(_) => Some(TypeInfo {
                    type_name: "Boolean".to_string(),
                    singleton: true,
                    namespace: Some("System".to_string()),
                    name: Some("Boolean".to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }),
                JsonValue::Object(_) => Some(TypeInfo {
                    type_name: "Object".to_string(),
                    singleton: true,
                    namespace: Some("FHIR".to_string()),
                    name: Some("Element".to_string()), // Generic FHIR element
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }),
                JsonValue::Null => Some(TypeInfo {
                    type_name: "Empty".to_string(),
                    singleton: true,
                    namespace: None,
                    name: None,
                    is_empty: Some(true),
                    is_union_type: Some(false),
                    union_choices: None,
                }),
                JsonValue::Array(_) => Some(TypeInfo {
                    type_name: "Collection".to_string(),
                    singleton: false,
                    namespace: None,
                    name: None,
                    is_empty: Some(prop_value.as_array().unwrap().is_empty()),
                    is_union_type: Some(false),
                    union_choices: None,
                }),
            };

            Some(FhirPathWrapped {
                value: Arc::new(prop_value.clone()),
                type_info,
                primitive_element,
            })
        } else {
            None
        }
    }

    /// Detect choice properties (value[x] pattern)
    pub fn detect_choice_properties(&self, base_property: &str) -> Vec<ChoiceProperty> {
        let mut choices = Vec::new();

        if let Some(obj) = self.value.as_object() {
            for (key, value) in obj {
                if key.starts_with(base_property) && key.len() > base_property.len() {
                    let suffix = &key[base_property.len()..];
                    if let Some(first_char) = suffix.chars().next() {
                        if first_char.is_uppercase() {
                            // Check for primitive element
                            let primitive_key = format!("_{key}");
                            let primitive_element = obj
                                .get(&primitive_key)
                                .and_then(|pe_json| PrimitiveElement::from_json(pe_json).ok());

                            choices.push(ChoiceProperty {
                                key: key.clone(),
                                value: value.clone(),
                                type_suffix: suffix.to_string(),
                                primitive_element,
                            });
                        }
                    }
                }
            }
        }

        choices
    }

    /// Check if this is a FHIR resource
    pub fn is_resource(&self) -> bool {
        self.value.get("resourceType").is_some()
    }

    /// Get resource type if this is a FHIR resource
    pub fn resource_type(&self) -> Option<&str> {
        self.value.get("resourceType").and_then(|rt| rt.as_str())
    }
}

/// Choice property detected during navigation
#[derive(Debug, Clone)]
pub struct ChoiceProperty {
    /// Full property key (e.g., "valueString", "valueInteger")
    pub key: String,
    /// Property value
    pub value: JsonValue,
    /// Type suffix (e.g., "String", "Integer")
    pub type_suffix: String,
    /// Associated primitive element (if any)
    pub primitive_element: Option<PrimitiveElement>,
}

impl ChoiceProperty {
    /// Convert to wrapped value with appropriate type info
    pub fn to_wrapped(&self) -> FhirPathWrapped<JsonValue> {
        let type_name = &self.type_suffix;
        let type_info = Some(TypeInfo {
            type_name: type_name.clone(),
            singleton: true,
            namespace: Some("System".to_string()),
            name: Some(type_name.clone()),
            is_empty: Some(false),
            is_union_type: Some(false),
            union_choices: None,
        });

        FhirPathWrapped {
            value: Arc::new(self.value.clone()),
            type_info,
            primitive_element: self.primitive_element.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for FhirPathWrapped<T> {
    fn eq(&self, other: &Self) -> bool {
        // Compare by Arc contents for efficiency
        Arc::ptr_eq(&self.value, &other.value) || *self.value == *other.value
    }
}

impl<T: std::fmt::Display> std::fmt::Display for FhirPathWrapped<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(type_info) = &self.type_info {
            if let Some(name) = &type_info.name {
                write!(f, "{}({})", name, self.value)
            } else {
                write!(f, "Wrapped({})", self.value)
            }
        } else {
            write!(f, "Wrapped({})", self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_wrapped_value_creation() {
        let data = json!({
            "resourceType": "Patient",
            "name": [{"family": "Smith"}]
        });

        let wrapped = FhirPathWrapped::resource(data);

        assert!(wrapped.get_type_info().is_some());
        let type_info = wrapped.get_type_info().unwrap();
        assert_eq!(type_info.type_name, "Patient");
        assert_eq!(type_info.namespace, Some("FHIR".to_string()));
        assert!(wrapped.is_resource());
        assert_eq!(wrapped.resource_type(), Some("Patient"));
    }

    #[test]
    fn test_primitive_element_creation() {
        let ext_json = json!({
            "id": "test-id",
            "extension": [{
                "url": "http://example.com/ext",
                "valueString": "test value"
            }]
        });

        let primitive_element = PrimitiveElement::from_json(&ext_json).unwrap();

        assert_eq!(primitive_element.id, Some("test-id".to_string()));
        assert_eq!(primitive_element.extensions.len(), 1);
        assert_eq!(
            primitive_element.extensions[0].url,
            "http://example.com/ext"
        );
        assert!(primitive_element.has_extensions());
    }

    #[test]
    fn test_property_access_with_extensions() {
        let data = json!({
            "status": "active",
            "_status": {
                "id": "status-id",
                "extension": [{
                    "url": "http://example.com/status-ext",
                    "valueString": "additional info"
                }]
            }
        });

        let wrapped = FhirPathWrapped::new(data, None);
        let status_prop = wrapped.get_property("status").unwrap();

        assert_eq!(*status_prop.unwrap(), json!("active"));
        assert!(status_prop.has_extensions());

        let pe = status_prop.get_primitive_element().unwrap();
        assert_eq!(pe.id, Some("status-id".to_string()));
        assert_eq!(pe.extensions.len(), 1);
    }

    #[test]
    fn test_choice_property_detection() {
        let data = json!({
            "valueString": "test string",
            "valueInteger": 42,
            "_valueString": {
                "extension": [{
                    "url": "http://example.com/ext",
                    "valueBoolean": true
                }]
            }
        });

        let wrapped = FhirPathWrapped::new(data, None);
        let choices = wrapped.detect_choice_properties("value");

        assert_eq!(choices.len(), 2);

        let string_choice = choices.iter().find(|c| c.type_suffix == "String").unwrap();
        assert_eq!(string_choice.value, json!("test string"));
        assert!(string_choice.primitive_element.is_some());

        let int_choice = choices.iter().find(|c| c.type_suffix == "Integer").unwrap();
        assert_eq!(int_choice.value, json!(42));
        assert!(int_choice.primitive_element.is_none());
    }

    #[test]
    fn test_arc_sharing() {
        let data = json!({"large": "data".repeat(1000)});
        let wrapped1 = FhirPathWrapped::new(data.clone(), None);
        let wrapped2 = wrapped1.clone();

        // Verify Arc sharing (same memory location)
        assert!(Arc::ptr_eq(&wrapped1.value, &wrapped2.value));
    }

    #[test]
    fn test_type_info_preservation() {
        let type_info = TypeInfo {
            type_name: "HumanName".to_string(),
            singleton: false,
            namespace: Some("FHIR".to_string()),
            name: Some("HumanName".to_string()),
            is_empty: Some(false),
        };

        let data = json!([{"family": "Smith"}, {"family": "Jones"}]);
        let wrapped = FhirPathWrapped::typed(data, type_info.clone());

        assert_eq!(wrapped.get_type_info().unwrap(), &type_info);
        assert!(!wrapped.get_type_info().unwrap().singleton);
    }
}
