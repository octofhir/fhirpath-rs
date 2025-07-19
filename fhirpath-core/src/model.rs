// FHIRPath Data Model
//
// This module defines the data model for FHIRPath values.

use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FHIRPath value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FhirPathValue {
    /// Empty value (no value)
    Empty,

    /// Boolean value
    Boolean(bool),

    /// Integer value
    Integer(i64),

    /// Decimal value
    Decimal(f64),

    /// String value
    String(String),

    /// Date value (ISO8601)
    Date(String),

    /// DateTime value (ISO8601)
    DateTime(String),

    /// Time value (ISO8601)
    Time(String),

    /// Quantity value with unit
    Quantity { value: f64, unit: String },

    /// Collection of values
    Collection(Vec<FhirPathValue>),

    /// FHIR resource or element
    Resource(FhirResource),
}

/// Representation of a FHIR resource or element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FhirResource {
    /// Resource type (e.g., "Patient", "Observation")
    pub resource_type: Option<String>,

    /// Resource properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl FhirResource {
    /// Creates a new FHIR resource from a JSON value
    pub fn from_json(json: serde_json::Value) -> Result<Self, serde_json::Error> {
        match json {
            serde_json::Value::Object(map) => {
                let mut properties = HashMap::new();
                let mut resource_type = None;

                for (key, value) in map {
                    if key == "resourceType" {
                        if let serde_json::Value::String(rt) = value {
                            resource_type = Some(rt);
                        } else {
                            properties.insert(key, value);
                        }
                    } else {
                        properties.insert(key, value);
                    }
                }

                Ok(Self {
                    resource_type,
                    properties,
                })
            }
            _ => Err(SerdeError::custom("Expected JSON object for FHIR resource")),
        }
    }

    /// Converts the FHIR resource to a JSON value
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        if let Some(rt) = &self.resource_type {
            map.insert(
                "resourceType".to_string(),
                serde_json::Value::String(rt.clone()),
            );
        }

        for (key, value) in &self.properties {
            map.insert(key.clone(), value.clone());
        }

        serde_json::Value::Object(map)
    }
}
