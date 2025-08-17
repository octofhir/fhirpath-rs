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

//! Runtime value boxing implementation
//!
//! This module implements universal value boxing to preserve metadata
//! and type information throughout FHIRPath evaluation.

use crate::FhirPathValue;
use serde_json::Value;

/// Type information for boxed values
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    /// Type namespace (e.g., "FHIR", "System")
    pub namespace: String,
    /// Type name (e.g., "boolean", "string", "Patient")
    pub name: String,
}

/// Primitive element metadata for FHIR primitive extensions
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveElement {
    /// Element ID
    pub id: Option<String>,
    /// Extensions on the primitive value
    pub extensions: Vec<Extension>,
}

/// FHIR Extension
#[derive(Debug, Clone, PartialEq)]
pub struct Extension {
    /// Extension URL
    pub url: String,
    /// Extension value
    pub value: Option<Value>,
}

/// Universal boxed value interface
#[derive(Debug, Clone, PartialEq)]
pub struct BoxedValue {
    /// The actual value
    pub value: FhirPathValue,
    /// Type information for this value
    pub type_info: Option<TypeInfo>,
    /// Primitive element metadata (for FHIR primitives)
    pub primitive_element: Option<PrimitiveElement>,
    /// Source property name (for polymorphic properties)
    pub source_property: Option<String>,
}

impl BoxedValue {
    /// Create a new boxed value with just the value
    pub fn new(value: FhirPathValue) -> Self {
        Self {
            value,
            type_info: None,
            primitive_element: None,
            source_property: None,
        }
    }

    /// Create a boxed value with type information
    pub fn with_type(
        value: FhirPathValue,
        namespace: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            value,
            type_info: Some(TypeInfo {
                namespace: namespace.into(),
                name: name.into(),
            }),
            primitive_element: None,
            source_property: None,
        }
    }

    /// Create a boxed FHIR primitive with extensions
    pub fn fhir_primitive(
        value: FhirPathValue,
        type_name: impl Into<String>,
        primitive_element: Option<PrimitiveElement>,
        source_property: Option<String>,
    ) -> Self {
        Self {
            value,
            type_info: Some(TypeInfo {
                namespace: "FHIR".to_string(),
                name: type_name.into(),
            }),
            primitive_element,
            source_property,
        }
    }

    /// Create a boxed System primitive
    pub fn system_primitive(value: FhirPathValue, type_name: impl Into<String>) -> Self {
        Self {
            value,
            type_info: Some(TypeInfo {
                namespace: "System".to_string(),
                name: type_name.into(),
            }),
            primitive_element: None,
            source_property: None,
        }
    }

    /// Unwrap the value for computation
    pub fn unwrap_value(&self) -> &FhirPathValue {
        &self.value
    }

    /// Get the effective type name (inferred if not explicitly set)
    pub fn get_type_name(&self) -> String {
        if let Some(ref type_info) = self.type_info {
            if type_info.namespace.is_empty() {
                type_info.name.clone()
            } else {
                format!("{}.{}", type_info.namespace, type_info.name)
            }
        } else {
            // Infer type from value
            match &self.value {
                FhirPathValue::Boolean(_) => "System.Boolean".to_string(),
                FhirPathValue::Integer(_) => "System.Integer".to_string(),
                FhirPathValue::Decimal(_) => "System.Decimal".to_string(),
                FhirPathValue::String(_) => "System.String".to_string(),
                FhirPathValue::Date(_) => "System.Date".to_string(),
                FhirPathValue::DateTime(_) => "System.DateTime".to_string(),
                FhirPathValue::Time(_) => "System.Time".to_string(),
                FhirPathValue::Quantity(_) => "System.Quantity".to_string(),
                FhirPathValue::Resource(resource) => {
                    if let Some(resource_type) = resource.resource_type() {
                        format!("FHIR.{resource_type}")
                    } else {
                        "FHIR.Resource".to_string()
                    }
                }
                FhirPathValue::Collection(_) => "System.Collection".to_string(),
                FhirPathValue::TypeInfoObject { .. } => "System.TypeInfo".to_string(),
                FhirPathValue::JsonValue(_) => "System.JsonValue".to_string(),
                FhirPathValue::Empty => "System.Empty".to_string(),
            }
        }
    }

    /// Check if this value has primitive extensions
    pub fn has_extensions(&self) -> bool {
        self.primitive_element
            .as_ref()
            .map(|pe| !pe.extensions.is_empty())
            .unwrap_or(false)
    }

    /// Get extensions (for children() function support)
    pub fn get_extensions(&self) -> Vec<Extension> {
        self.primitive_element
            .as_ref()
            .map(|pe| pe.extensions.clone())
            .unwrap_or_default()
    }
}

/// Boxing utilities for navigation and operations
pub struct Boxing;

impl Boxing {
    /// Box a value during property navigation from a FHIR resource
    pub fn box_from_fhir_property(
        value: &Value,
        property_name: &str,
        primitive_extensions: Option<Value>,
    ) -> BoxedValue {
        // Determine FHIR type from property name for polymorphic properties
        let (fhir_type, actual_property) = if property_name == "value" {
            // This should be resolved to the actual property like "valueDecimal"
            // For now, infer from the value type
            Self::infer_fhir_type_from_value(value)
        } else if property_name.starts_with("value") && property_name.len() > 5 {
            // Direct value[x] property access
            let type_suffix = &property_name[5..]; // Remove "value" prefix
            (
                Self::fhir_type_from_suffix(type_suffix),
                Some(property_name.to_string()),
            )
        } else {
            // Regular property - infer type from value
            Self::infer_fhir_type_from_value(value)
        };

        let fhir_value = Self::json_to_fhirpath_value(value);
        let primitive_element = Self::parse_primitive_extensions(primitive_extensions);

        BoxedValue::fhir_primitive(fhir_value, fhir_type, primitive_element, actual_property)
    }

    /// Box a value for system operations (not from FHIR context)
    pub fn box_system_value(value: FhirPathValue) -> BoxedValue {
        let type_name = match &value {
            FhirPathValue::Boolean(_) => "Boolean",
            FhirPathValue::Integer(_) => "Integer",
            FhirPathValue::Decimal(_) => "Decimal",
            FhirPathValue::String(_) => "String",
            FhirPathValue::Date(_) => "Date",
            FhirPathValue::DateTime(_) => "DateTime",
            FhirPathValue::Time(_) => "Time",
            FhirPathValue::Quantity(_) => "Quantity",
            _ => "Any",
        };

        BoxedValue::system_primitive(value, type_name)
    }

    /// Convert JSON value to FhirPathValue
    fn json_to_fhirpath_value(value: &Value) -> FhirPathValue {
        match value {
            Value::Null => FhirPathValue::Empty,
            Value::Bool(b) => FhirPathValue::Boolean(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    match rust_decimal::Decimal::try_from(f) {
                        Ok(d) => FhirPathValue::Decimal(d),
                        Err(_) => FhirPathValue::JsonValue(crate::json_arc::ArcJsonValue::new(
                            value.clone(),
                        )),
                    }
                } else {
                    FhirPathValue::JsonValue(crate::json_arc::ArcJsonValue::new(value.clone()))
                }
            }
            Value::String(s) => FhirPathValue::String(s.as_str().into()),
            Value::Array(_) | Value::Object(_) => {
                FhirPathValue::JsonValue(crate::json_arc::ArcJsonValue::new(value.clone()))
            }
        }
    }

    /// Infer FHIR type from JSON value
    fn infer_fhir_type_from_value(value: &Value) -> (String, Option<String>) {
        match value {
            Value::Bool(_) => ("boolean".to_string(), None),
            Value::Number(n) => {
                if n.is_f64() {
                    ("decimal".to_string(), None)
                } else {
                    ("integer".to_string(), None)
                }
            }
            Value::String(s) => {
                // Try to infer more specific string types
                if s.starts_with("urn:uuid:") {
                    ("uuid".to_string(), None)
                } else if s.starts_with("http://")
                    || s.starts_with("https://")
                    || s.starts_with("urn:")
                {
                    ("uri".to_string(), None)
                } else {
                    ("string".to_string(), None)
                }
            }
            _ => ("Resource".to_string(), None),
        }
    }

    /// Get FHIR type from value[x] property suffix
    fn fhir_type_from_suffix(suffix: &str) -> String {
        match suffix {
            "String" => "string".to_string(),
            "Integer" => "integer".to_string(),
            "Decimal" => "decimal".to_string(),
            "Boolean" => "boolean".to_string(),
            "Date" => "date".to_string(),
            "DateTime" => "dateTime".to_string(),
            "Time" => "time".to_string(),
            "Uuid" => "uuid".to_string(),
            "Uri" => "uri".to_string(),
            "Code" => "code".to_string(),
            _ => suffix.to_lowercase(),
        }
    }

    /// Parse primitive extensions from JSON
    fn parse_primitive_extensions(extensions_json: Option<Value>) -> Option<PrimitiveElement> {
        if let Some(Value::Object(obj)) = extensions_json {
            let extensions: Vec<Extension> = obj
                .get("extension")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|ext| {
                            if let Value::Object(ext_obj) = ext {
                                Some(Extension {
                                    url: ext_obj
                                        .get("url")
                                        .and_then(|u| u.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    value: ext_obj.get("value").cloned(),
                                })
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let id = obj.get("id").and_then(|v| v.as_str()).map(String::from);

            if !extensions.is_empty() || id.is_some() {
                Some(PrimitiveElement { id, extensions })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_boxed_value_creation() {
        let value = FhirPathValue::Boolean(true);
        let boxed = BoxedValue::with_type(value, "FHIR", "boolean");

        assert_eq!(boxed.get_type_name(), "FHIR.boolean");
        assert!(!boxed.has_extensions());
    }

    #[test]
    fn test_boxing_from_fhir_property() {
        let json_value = json!(1);
        let boxed = Boxing::box_from_fhir_property(&json_value, "valueDecimal", None);

        assert_eq!(boxed.get_type_name(), "FHIR.decimal");
        assert_eq!(boxed.source_property, Some("valueDecimal".to_string()));
    }

    #[test]
    fn test_primitive_extensions() {
        let ext_json = json!({
            "extension": [{
                "url": "http://example.com/extension",
                "valueString": "test"
            }]
        });

        let json_value = json!(true);
        let boxed = Boxing::box_from_fhir_property(&json_value, "active", Some(ext_json));

        assert!(boxed.has_extensions());
        assert_eq!(boxed.get_extensions().len(), 1);
    }
}
