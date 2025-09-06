//! Core FHIRPath type definitions with comprehensive value system

use std::fmt;
use std::cmp::Ordering;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;
use octofhir_ucum::{UnitRecord, find_unit};

use super::error::{FhirPathError, Result};
use super::error_code::*;
use super::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};

/// A collection of FHIRPath values - the fundamental evaluation result type
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Collection(Vec<FhirPathValue>);

impl Collection {
    /// Create a new empty collection
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Create a collection with a single value
    pub fn single(value: FhirPathValue) -> Self {
        Self(vec![value])
    }

    /// Create a collection from a vector of values
    pub fn from_values(values: Vec<FhirPathValue>) -> Self {
        Self(values)
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of items in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the first item, if any
    pub fn first(&self) -> Option<&FhirPathValue> {
        self.0.first()
    }

    /// Get item at index
    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        self.0.get(index)
    }

    /// Iterate over values
    pub fn iter(&self) -> std::slice::Iter<FhirPathValue> {
        self.0.iter()
    }

    /// Add a value to the collection
    pub fn push(&mut self, value: FhirPathValue) {
        self.0.push(value);
    }

    /// Convert to vector
    pub fn into_vec(self) -> Vec<FhirPathValue> {
        self.0
    }

}

impl From<Vec<FhirPathValue>> for Collection {
    fn from(values: Vec<FhirPathValue>) -> Self {
        Self::from_values(values)
    }
}

impl From<FhirPathValue> for Collection {
    fn from(value: FhirPathValue) -> Self {
        Self::single(value)
    }
}

impl std::ops::Index<usize> for Collection {
    type Output = FhirPathValue;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl serde::Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as a simple array of values
        self.0.serialize(serializer)
    }
}

/// FHIRPath value type supporting core FHIR primitive types
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum FhirPathValue {
    /// Boolean value
    Boolean(bool),
    
    /// Integer value (64-bit signed)
    Integer(i64),
    
    /// Decimal value (high-precision)
    Decimal(Decimal),
    
    /// String value
    String(String),
    
    /// Date value with precision tracking
    Date(PrecisionDate),
    
    /// DateTime value with timezone and precision tracking
    DateTime(PrecisionDateTime),
    
    /// Time value with precision tracking
    Time(PrecisionTime),
    
    /// Quantity value with UCUM unit support
    Quantity {
        /// The numeric value of the quantity
        value: Decimal,
        /// The unit of measurement (e.g., "mg", "kg/m2") 
        unit: Option<String>,
        /// Parsed UCUM unit for calculations (cached for performance)
        #[serde(skip)]
        #[serde(default)]
        ucum_unit: Option<Arc<UnitRecord>>,
        /// Calendar unit for non-UCUM time units (year, month, week, day)
        calendar_unit: Option<CalendarUnit>,
    },
    
    /// Complex FHIR resource or element (JSON representation)
    /// This handles all complex FHIR types like Coding, CodeableConcept, etc.
    Resource(JsonValue),
    
    /// Raw JSON value for compatibility (distinct from Resource for type operations)
    JsonValue(JsonValue),
    
    /// UUID/identifier value
    Id(Uuid),
    
    /// Binary data (base64 encoded)
    Base64Binary(Vec<u8>),
    
    /// URI value
    Uri(String),
    
    /// URL value (subset of URI)
    Url(String),
    
    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Vec<FhirPathValue>),
    
    /// Type information object for type operations
    TypeInfoObject {
        namespace: String,
        name: String,
    },
    
    /// Null/empty value (represents absence)
    Empty,
}

impl serde::Serialize for FhirPathValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Boolean(b) => serializer.serialize_bool(*b),
            Self::Integer(i) => serializer.serialize_i64(*i),
            Self::Decimal(d) => {
                // Try to serialize as number, fall back to string if precision issues
                if let Ok(f) = d.to_string().parse::<f64>() {
                    serializer.serialize_f64(f)
                } else {
                    serializer.serialize_str(&d.to_string())
                }
            },
            Self::String(s) => serializer.serialize_str(s),
            Self::Date(date) => serializer.serialize_str(&date.to_string()),
            Self::DateTime(dt) => serializer.serialize_str(&dt.to_string()),
            Self::Time(time) => serializer.serialize_str(&time.to_string()),
            Self::Quantity { value, unit, .. } => {
                let mut map = std::collections::BTreeMap::new();
                // Convert decimal value to JSON number or string
                if let Ok(f) = value.to_string().parse::<f64>() {
                    map.insert("value", serde_json::Value::Number(
                        serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0))
                    ));
                } else {
                    map.insert("value", serde_json::Value::String(value.to_string()));
                }
                if let Some(unit) = unit {
                    map.insert("unit", serde_json::Value::String(unit.clone()));
                }
                map.serialize(serializer)
            },
            Self::Resource(json) => json.serialize(serializer),
            Self::Id(id) => serializer.serialize_str(&id.to_string()),
            Self::Base64Binary(data) => {
                // For now, serialize as string representation since we removed base64 dependency
                serializer.serialize_str(&format!("base64({} bytes)", data.len()))
            },
            Self::Uri(uri) => serializer.serialize_str(uri),
            Self::Url(url) => serializer.serialize_str(url),
            Self::Collection(values) => values.serialize(serializer),
            Self::TypeInfoObject { namespace, name } => {
                let mut map = std::collections::BTreeMap::new();
                map.insert("namespace", namespace);
                map.insert("name", name);
                map.serialize(serializer)
            },
            Self::JsonValue(json) => json.serialize(serializer),
            Self::Empty => serializer.serialize_unit(),
        }
    }
}

/// Calendar units for temporal calculations (not supported by UCUM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarUnit {
    /// Year (approximately 365.25 days)
    Year,
    /// Month (varies between 28-31 days)
    Month,
    /// Week (exactly 7 days)
    Week,
    /// Day (exactly 24 hours)
    Day,
}

impl CalendarUnit {
    /// Get the unit name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month", 
            Self::Week => "week",
            Self::Day => "day",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "year" | "years" | "a" => Some(Self::Year),
            "month" | "months" | "mo" => Some(Self::Month),
            "week" | "weeks" | "wk" => Some(Self::Week),
            "day" | "days" | "d" => Some(Self::Day),
            _ => None,
        }
    }

    /// Convert to approximate days (for rough comparisons only)
    pub fn approximate_days(&self) -> f64 {
        match self {
            Self::Year => 365.25,
            Self::Month => 30.44, // Average month length
            Self::Week => 7.0,
            Self::Day => 1.0,
        }
    }
}

impl fmt::Display for CalendarUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FhirPathValue {
    /// Get the FHIRPath type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Boolean(_) => "Boolean",
            Self::Integer(_) => "Integer", 
            Self::Decimal(_) => "Decimal",
            Self::String(_) => "String",
            Self::Date(_) => "Date",
            Self::DateTime(_) => "DateTime",
            Self::Time(_) => "Time",
            Self::Quantity { .. } => "Quantity",
            Self::Resource(_) => "Resource",
            Self::Id(_) => "id",
            Self::Base64Binary(_) => "base64Binary",
            Self::Uri(_) => "uri",
            Self::Url(_) => "url",
            Self::Collection(_) => "Collection",
            Self::TypeInfoObject { .. } => "TypeInfo",
            Self::JsonValue(_) => "JsonValue",
            Self::Empty => "empty",
        }
    }

    /// Check if this value is empty/null
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Convert to boolean (FHIRPath boolean conversion rules)
    pub fn to_boolean(&self) -> Result<bool> {
        match self {
            Self::Boolean(b) => Ok(*b),
            Self::Integer(i) => Ok(*i != 0),
            Self::Decimal(d) => Ok(!d.is_zero()),
            Self::String(s) => Ok(!s.is_empty()),
            Self::Empty => Ok(false),
            _ => Err(FhirPathError::evaluation_error(
                FP0051,
                format!("Cannot convert {} to boolean", self.type_name()),
            )),
        }
    }

    /// Convert to string (FHIRPath string conversion rules)
    pub fn to_string(&self) -> Result<String> {
        match self {
            Self::String(s) => Ok(s.clone()),
            Self::Integer(i) => Ok(i.to_string()),
            Self::Decimal(d) => Ok(d.to_string()),
            Self::Boolean(b) => Ok(b.to_string()),
            Self::Date(d) => Ok(d.to_string()),
            Self::DateTime(dt) => Ok(dt.to_string()),
            Self::Time(t) => Ok(t.to_string()),
            Self::Uri(u) => Ok(u.clone()),
            Self::Url(u) => Ok(u.clone()),
            Self::Id(id) => Ok(id.to_string()),
            Self::Empty => Ok(String::new()),
            Self::JsonValue(json) => Ok(json.to_string()),
            _ => Err(FhirPathError::evaluation_error(
                FP0051,
                format!("Cannot convert {} to string", self.type_name()),
            )),
        }
    }

    /// Create a string value
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Create an integer value
    pub fn integer(i: i64) -> Self {
        Self::Integer(i)
    }

    /// Create a decimal value
    pub fn decimal(d: impl Into<Decimal>) -> Self {
        Self::Decimal(d.into())
    }

    /// Create a boolean value
    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }

    /// Create a resource value from JSON
    pub fn resource(json: JsonValue) -> Self {
        Self::Resource(json)
    }

    /// Create a quantity value with UCUM unit parsing
    pub fn quantity(value: Decimal, unit: Option<String>) -> Self {
        let (ucum_unit, calendar_unit) = if let Some(ref unit_str) = unit {
            // Try parsing as calendar unit first (year, month, week, day)
            if let Some(cal_unit) = CalendarUnit::from_str(unit_str) {
                (None, Some(cal_unit))
            } else {
                // Try parsing as UCUM unit
                match find_unit(unit_str) {
                    Some(ucum) => (Some(Arc::new(ucum.clone())), None),
                    None => (None, None), // Invalid unit, keep as string only
                }
            }
        } else {
            (None, None)
        };

        Self::Quantity {
            value,
            unit,
            ucum_unit,
            calendar_unit,
        }
    }

    /// Create a quantity value for quoted units (UCUM only, no calendar units)
    pub fn quoted_quantity(value: Decimal, unit: Option<String>) -> Self {
        let (ucum_unit, calendar_unit) = if let Some(ref unit_str) = unit {
            // For quoted units, only try UCUM parsing, not calendar units
            match find_unit(unit_str) {
                Some(ucum) => (Some(Arc::new(ucum.clone())), None),
                None => (None, None), // Invalid unit, keep as string only
            }
        } else {
            (None, None)
        };

        Self::Quantity {
            value,
            unit,
            ucum_unit,
            calendar_unit,
        }
    }

    /// Create a quantity value with explicit calendar unit
    pub fn calendar_quantity(value: Decimal, calendar_unit: CalendarUnit) -> Self {
        Self::Quantity {
            value,
            unit: Some(calendar_unit.to_string()),
            ucum_unit: None,
            calendar_unit: Some(calendar_unit),
        }
    }

    /// Create a date value
    pub fn date(date: PrecisionDate) -> Self {
        Self::Date(date)
    }

    /// Create a datetime value
    pub fn datetime(datetime: PrecisionDateTime) -> Self {
        Self::DateTime(datetime)
    }

    /// Create a time value
    pub fn time(time: PrecisionTime) -> Self {
        Self::Time(time)
    }

    /// Create a JSON value (for compatibility)
    pub fn json_value(json: JsonValue) -> Self {
        Self::JsonValue(json)
    }

    /// Create a collection value
    pub fn collection(values: Vec<FhirPathValue>) -> Self {
        if values.is_empty() {
            Self::Empty
        } else if values.len() == 1 {
            values.into_iter().next().unwrap()
        } else {
            Self::Collection(values)
        }
    }

    /// Create an empty value
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Check if this quantity is compatible with another for arithmetic operations
    pub fn is_quantity_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Quantity { ucum_unit: Some(u1), .. },
                Self::Quantity { ucum_unit: Some(u2), .. }
            ) => {
                // Both have UCUM units - check if they're compatible (same dimension)
                u1.dim == u2.dim
            },
            (
                Self::Quantity { calendar_unit: Some(c1), ucum_unit: None, .. },
                Self::Quantity { calendar_unit: Some(c2), ucum_unit: None, .. }
            ) => {
                // Both have calendar units - same type is compatible
                c1 == c2
            },
            (
                Self::Quantity { unit: None, .. },
                Self::Quantity { unit: None, .. }
            ) => {
                // Both dimensionless quantities are compatible
                true
            },
            _ => false,
        }
    }

}

impl fmt::Display for FhirPathValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Decimal(d) => write!(f, "{}", d),
            Self::String(s) => write!(f, "'{}'", s),
            Self::Date(d) => write!(f, "@{}", d),
            Self::DateTime(dt) => write!(f, "@{}", dt),
            Self::Time(t) => write!(f, "@T{}", t),
            Self::Quantity { value, unit, .. } => {
                if let Some(unit) = unit {
                    write!(f, "{} '{}'", value, unit)
                } else {
                    write!(f, "{}", value)
                }
            },
            Self::Resource(json) => {
                // Try to extract resource type for better display
                if let Some(resource_type) = json.get("resourceType").and_then(|rt| rt.as_str()) {
                    write!(f, "{}({})", resource_type, json)
                } else {
                    write!(f, "Resource({})", json)
                }
            },
            Self::Id(id) => write!(f, "{}", id),
            Self::Base64Binary(data) => write!(f, "base64({} bytes)", data.len()),
            Self::Uri(u) => write!(f, "{}", u),
            Self::Url(u) => write!(f, "{}", u),
            Self::Collection(values) => {
                write!(f, "Collection[")?;
                for (i, val) in values.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            },
            Self::TypeInfoObject { namespace, name } => {
                if namespace.is_empty() {
                    write!(f, "TypeInfo({})", name)
                } else {
                    write!(f, "TypeInfo({}.{})", namespace, name)
                }
            },
            Self::JsonValue(json) => write!(f, "JsonValue({})", json),
            Self::Empty => write!(f, "{{}}"),
        }
    }
}

impl PartialOrd for FhirPathValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // Numeric comparisons
            (Self::Integer(a), Self::Integer(b)) => a.partial_cmp(b),
            (Self::Decimal(a), Self::Decimal(b)) => a.partial_cmp(b),
            (Self::Integer(a), Self::Decimal(b)) => Decimal::from(*a).partial_cmp(b),
            (Self::Decimal(a), Self::Integer(b)) => a.partial_cmp(&Decimal::from(*b)),
            
            // String comparisons
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            (Self::Uri(a), Self::Uri(b)) => a.partial_cmp(b),
            (Self::Url(a), Self::Url(b)) => a.partial_cmp(b),
            
            // Boolean comparison
            (Self::Boolean(a), Self::Boolean(b)) => a.partial_cmp(b),
            
            // Temporal comparisons 
            (Self::Date(a), Self::Date(b)) => a.partial_cmp(b),
            (Self::DateTime(a), Self::DateTime(b)) => a.partial_cmp(b),
            (Self::Time(a), Self::Time(b)) => a.partial_cmp(b),
            
            // Quantity comparisons (only if compatible units)
            (Self::Quantity { value: v1, .. }, Self::Quantity { value: v2, .. }) => {
                if self.is_quantity_compatible(other) {
                    v1.partial_cmp(v2)
                } else {
                    None // Incompatible units
                }
            },
            
            // ID comparisons
            (Self::Id(a), Self::Id(b)) => a.partial_cmp(b),
            
            _ => None, // Different types are not comparable
        }
    }
}