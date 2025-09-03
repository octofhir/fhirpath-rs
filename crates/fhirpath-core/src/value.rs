//! FHIRPath value types
//!
//! This module defines the core value types used throughout the FHIRPath implementation.

use crate::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Core value type for FHIRPath expressions
///
/// This enum represents all possible values that can be produced by FHIRPath expressions.
/// All values in FHIRPath are conceptual collections, but single values are represented
/// directly for performance reasons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FhirPathValue {
    /// Boolean value
    Boolean(bool),

    /// Integer value (64-bit signed)
    Integer(i64),

    /// Decimal value with arbitrary precision
    Decimal(Decimal),

    /// String value
    String(String),

    /// Date value with precision tracking
    Date(PrecisionDate),

    /// DateTime value with timezone and precision tracking
    DateTime(PrecisionDateTime),

    /// Time value with precision tracking
    Time(PrecisionTime),

    /// Quantity value with optional unit
    Quantity {
        /// The numeric value of the quantity
        value: Decimal,
        /// The unit of measurement (e.g., "mg", "kg/m2")
        unit: Option<String>,
        /// Cached parsed UCUM unit expression for performance
        #[serde(skip)]
        #[serde(default)]
        ucum_expr: Option<Arc<String>>, // Simplified - no UCUM dependency in core
    },

    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Vec<FhirPathValue>),

    /// JSON value (for FHIR Resources or complex objects)
    JsonValue(Value),

    /// Type information object for type operations
    TypeInfoObject {
        namespace: String,
        name: String,
    },

    /// FHIR Resource wrapper
    Resource(FhirResource),

    /// Empty value (equivalent to an empty collection)
    Empty,
}

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
    pub fn new(data: Value) -> Self {
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

    /// Get the resource type
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// Get the JSON data
    pub fn as_json(&self) -> &Value {
        &self.data
    }

    /// Get the JSON value (for compatibility)
    pub fn as_json_value(&self) -> &Value {
        &self.data
    }
}

impl FhirPathValue {
    /// Get the type name of this value
    pub fn type_name(&self) -> String {
        match self {
            FhirPathValue::Boolean(_) => "Boolean".to_string(),
            FhirPathValue::Integer(_) => "Integer".to_string(),
            FhirPathValue::Decimal(_) => "Decimal".to_string(),
            FhirPathValue::String(_) => "String".to_string(),
            FhirPathValue::Date(_) => "Date".to_string(),
            FhirPathValue::DateTime(_) => "DateTime".to_string(),
            FhirPathValue::Time(_) => "Time".to_string(),
            FhirPathValue::Quantity { .. } => "Quantity".to_string(),
            FhirPathValue::Collection(_) => "Collection".to_string(),
            FhirPathValue::JsonValue(v) => {
                if let Some(resource_type) = v.get("resourceType").and_then(|rt| rt.as_str()) {
                    resource_type.to_string()
                } else {
                    "Object".to_string()
                }
            },
            FhirPathValue::TypeInfoObject { name, .. } => name.clone(),
            FhirPathValue::Resource(r) => {
                r.resource_type().unwrap_or("Resource").to_string()
            },
            FhirPathValue::Empty => "Empty".to_string(),
        }
    }

    /// Create a collection from a vector of values
    pub fn collection(values: Vec<FhirPathValue>) -> Self {
        if values.is_empty() {
            FhirPathValue::Empty
        } else if values.len() == 1 {
            values.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(values)
        }
    }

    /// Get string value if this is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            FhirPathValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get integer value if this is an integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            FhirPathValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string_value(&self) -> String {
        match self {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Date(d) => d.to_string(),
            FhirPathValue::DateTime(dt) => dt.to_string(),
            FhirPathValue::Time(t) => t.to_string(),
            _ => format!("{:?}", self),
        }
    }

    /// Create a quantity value
    pub fn quantity(value: Decimal, unit: Option<String>) -> Self {
        FhirPathValue::Quantity {
            value,
            unit,
            ucum_expr: None,
        }
    }
}

/// Extension trait for Value to add missing methods
pub trait JsonValueExt {
    /// Get the inner JSON value (compatibility method)
    fn as_inner(&self) -> &Value;
    
    /// Get an iterator over object entries
    fn object_iter(&self) -> Option<serde_json::map::Iter>;
    
    /// Get an iterator over array elements  
    fn array_iter(&self) -> Option<std::slice::Iter<Value>>;
    
    /// Get a property from an object
    fn get_property(&self, key: &str) -> Option<&Value>;
}

impl JsonValueExt for Value {
    fn as_inner(&self) -> &Value {
        self
    }
    
    fn object_iter(&self) -> Option<serde_json::map::Iter> {
        self.as_object().map(|obj| obj.iter())
    }
    
    fn array_iter(&self) -> Option<std::slice::Iter<Value>> {
        self.as_array().map(|arr| arr.iter())
    }
    
    fn get_property(&self, key: &str) -> Option<&Value> {
        self.as_object().and_then(|obj| obj.get(key))
    }
}

/// Collection wrapper for compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Collection(Vec<FhirPathValue>);

impl Collection {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from(values: Vec<FhirPathValue>) -> Self {
        Self(values)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<FhirPathValue> {
        self.0.iter()
    }

    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        self.0.get(index)
    }

    pub fn first(&self) -> Option<&FhirPathValue> {
        self.0.first()
    }
}

impl From<Vec<FhirPathValue>> for Collection {
    fn from(values: Vec<FhirPathValue>) -> Self {
        Self(values)
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}