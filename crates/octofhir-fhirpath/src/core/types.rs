//! Core FHIRPath type definitions with comprehensive value system

use octofhir_ucum::{UnitRecord, find_unit};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

use super::error::{FhirPathError, Result};
use super::error_code::*;
use super::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};

/// A collection of FHIRPath values - the fundamental evaluation result type
#[derive(Debug, Clone, PartialEq)]
pub struct Collection {
    values: Vec<FhirPathValue>,
    is_ordered: bool,
}

impl Collection {
    /// Create a new empty collection (ordered by default)
    pub fn empty() -> Self {
        Self {
            values: Vec::new(),
            is_ordered: true,
        }
    }

    /// Create an empty collection with explicit ordering
    pub fn empty_with_ordering(is_ordered: bool) -> Self {
        Self {
            values: Vec::new(),
            is_ordered,
        }
    }

    /// Create a collection with a single value (ordered by default)
    pub fn single(value: FhirPathValue) -> Self {
        Self {
            values: vec![value],
            is_ordered: true,
        }
    }

    /// Create a collection from a vector of values (ordered by default)
    pub fn from_values(values: Vec<FhirPathValue>) -> Self {
        Self {
            values,
            is_ordered: true,
        }
    }

    /// Create a collection from a vector with explicit ordering
    pub fn from_values_with_ordering(values: Vec<FhirPathValue>, is_ordered: bool) -> Self {
        Self { values, is_ordered }
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the number of items in the collection
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the collection is ordered
    pub fn is_ordered(&self) -> bool {
        self.is_ordered
    }

    /// Get the first item, if any
    pub fn first(&self) -> Option<&FhirPathValue> {
        self.values.first()
    }

    /// Get the last item, if any
    pub fn last(&self) -> Option<&FhirPathValue> {
        self.values.last()
    }

    /// Get item at index
    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        self.values.get(index)
    }

    /// Get an iterator over the values in the collection
    pub fn iter(&self) -> std::slice::Iter<'_, FhirPathValue> {
        self.values.iter()
    }

    /// Get the underlying values as a slice
    pub fn values(&self) -> &[FhirPathValue] {
        &self.values
    }

    /// Add a value to the collection
    pub fn push(&mut self, value: FhirPathValue) {
        self.values.push(value);
    }

    /// Convert to vector
    pub fn into_vec(self) -> Vec<FhirPathValue> {
        self.values
    }

    /// Convert to serde_json::Value
    pub fn to_json_value(&self) -> JsonValue {
        match self.values.len() {
            0 => JsonValue::Null,
            1 => self.values[0].to_json_value(),
            _ => JsonValue::Array(self.values.iter().map(|v| v.to_json_value()).collect()),
        }
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

impl IntoIterator for Collection {
    type Item = FhirPathValue;
    type IntoIter = std::vec::IntoIter<FhirPathValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl std::iter::FromIterator<FhirPathValue> for Collection {
    fn from_iter<T: IntoIterator<Item = FhirPathValue>>(iter: T) -> Self {
        Self::from_values(iter.into_iter().collect())
    }
}

impl std::ops::Index<usize> for Collection {
    type Output = FhirPathValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl serde::Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as a simple array of values
        self.values.serialize(serializer)
    }
}

/// FHIRPath value type supporting core FHIR primitive types
#[derive(Debug, Clone, PartialEq)]
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
        ucum_unit: Option<Arc<UnitRecord>>,
        /// Calendar unit for non-UCUM time units (year, month, week, day)
        calendar_unit: Option<CalendarUnit>,
    },

    /// Complex FHIR resource or element (JSON representation)
    /// This handles all complex FHIR types like Coding, CodeableConcept, etc.
    Resource(Arc<JsonValue>),

    /// Raw JSON value for compatibility (distinct from Resource for type operations)
    JsonValue(Arc<JsonValue>),

    /// UUID/identifier value
    Id(Uuid),

    /// Binary data (base64 encoded)
    Base64Binary(Vec<u8>),

    /// URI value
    Uri(String),

    /// URL value (subset of URI)
    Url(String),

    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Collection),

    /// Type information object for type operations
    TypeInfoObject { namespace: String, name: String },

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
            }
            Self::String(s) => serializer.serialize_str(s),
            Self::Date(date) => serializer.serialize_str(&date.to_string()),
            Self::DateTime(dt) => serializer.serialize_str(&dt.to_string()),
            Self::Time(time) => serializer.serialize_str(&time.to_string()),
            Self::Quantity { value, unit, .. } => {
                let mut map = std::collections::BTreeMap::new();
                // Convert decimal value to JSON number or string
                if let Ok(f) = value.to_string().parse::<f64>() {
                    map.insert(
                        "value",
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
                        ),
                    );
                } else {
                    map.insert("value", serde_json::Value::String(value.to_string()));
                }
                if let Some(unit) = unit {
                    map.insert("unit", serde_json::Value::String(unit.clone()));
                }
                map.serialize(serializer)
            }
            Self::Resource(json) => json.serialize(serializer),
            Self::Id(id) => serializer.serialize_str(&id.to_string()),
            Self::Base64Binary(data) => {
                // For now, serialize as string representation since we removed base64 dependency
                serializer.serialize_str(&format!("base64({} bytes)", data.len()))
            }
            Self::Uri(uri) => serializer.serialize_str(uri),
            Self::Url(url) => serializer.serialize_str(url),
            Self::Collection(collection) => collection.serialize(serializer),
            Self::TypeInfoObject { namespace, name } => {
                let mut map = std::collections::BTreeMap::new();
                map.insert("namespace", namespace);
                map.insert("name", name);
                map.serialize(serializer)
            }
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
        match self {
            Self::Empty => true,
            Self::Collection(c) => c.is_empty(),
            _ => false,
        }
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
        Self::Resource(Arc::new(json))
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
        Self::JsonValue(Arc::new(json))
    }

    /// Create a collection value
    pub fn collection(values: Vec<FhirPathValue>) -> Self {
        if values.is_empty() {
            Self::Empty
        } else if values.len() == 1 {
            values.into_iter().next().unwrap()
        } else {
            Self::Collection(Collection::from_values(values))
        }
    }

    /// Create a collection with explicit ordering
    pub fn collection_with_ordering(values: Vec<FhirPathValue>, is_ordered: bool) -> Self {
        if values.is_empty() {
            Self::Empty
        } else if values.len() == 1 && is_ordered {
            // Only return single value for ordered collections (FHIRPath optimization)
            values.into_iter().next().unwrap()
        } else {
            // Always return Collection for unordered collections to preserve ordering info
            Self::Collection(Collection::from_values_with_ordering(values, is_ordered))
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
                Self::Quantity {
                    ucum_unit: Some(u1),
                    ..
                },
                Self::Quantity {
                    ucum_unit: Some(u2),
                    ..
                },
            ) => {
                // Both have UCUM units - check if they're compatible (same dimension)
                u1.dim == u2.dim
            }
            (
                Self::Quantity {
                    calendar_unit: Some(c1),
                    ucum_unit: None,
                    ..
                },
                Self::Quantity {
                    calendar_unit: Some(c2),
                    ucum_unit: None,
                    ..
                },
            ) => {
                // Both have calendar units - same type is compatible
                c1 == c2
            }
            (Self::Quantity { unit: None, .. }, Self::Quantity { unit: None, .. }) => {
                // Both dimensionless quantities are compatible
                true
            }
            _ => false,
        }
    }

    /// Get the length of the value (1 for single values, n for collections, 0 for empty)
    pub fn len(&self) -> usize {
        match self {
            Self::Collection(col) => col.len(),
            Self::Empty => 0,
            _ => 1,
        }
    }

    /// Get the first item from a collection, or the value itself if single
    pub fn first(&self) -> Option<&FhirPathValue> {
        match self {
            Self::Collection(col) => col.first(),
            Self::Empty => None,
            single => Some(single),
        }
    }

    /// Get the last item from a collection, or the value itself if single
    pub fn last(&self) -> Option<&FhirPathValue> {
        match self {
            Self::Collection(col) => col.last(),
            Self::Empty => None,
            single => Some(single),
        }
    }

    /// Get an item at a specific index
    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        match self {
            Self::Collection(col) => col.get(index),
            Self::Empty => None,
            single if index == 0 => Some(single),
            _ => None,
        }
    }

    /// Iterate over collection items, or a single item if not a collection
    pub fn iter(&self) -> Box<dyn Iterator<Item = &FhirPathValue> + '_> {
        match self {
            Self::Collection(col) => Box::new(col.iter()),
            Self::Empty => Box::new(std::iter::empty()),
            single => Box::new(std::iter::once(single)),
        }
    }

    /// Convert to a collection (wrapping single values)
    pub fn to_collection(self) -> Vec<FhirPathValue> {
        match self {
            Self::Collection(col) => col.into_vec(),
            Self::Empty => Vec::new(),
            single => vec![single],
        }
    }

    /// Clone the collection items into a Vec
    pub fn cloned_collection(&self) -> Vec<FhirPathValue> {
        self.iter().cloned().collect()
    }

    /// Check if this is an ordered collection
    pub fn is_ordered_collection(&self) -> bool {
        match self {
            Self::Collection(col) => col.is_ordered(),
            _ => true, // Single values and empty are considered ordered
        }
    }

    /// Convert to Collection struct
    pub fn as_collection(&self) -> Option<&Collection> {
        match self {
            Self::Collection(col) => Some(col),
            _ => None,
        }
    }

    /// Convert to serde_json::Value
    pub fn to_json_value(&self) -> JsonValue {
        match self {
            Self::Boolean(b) => JsonValue::Bool(*b),
            Self::Integer(i) => JsonValue::Number(serde_json::Number::from(*i)),
            Self::Decimal(d) => {
                if let Ok(f) = d.to_string().parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        JsonValue::Number(n)
                    } else {
                        JsonValue::String(d.to_string())
                    }
                } else {
                    JsonValue::String(d.to_string())
                }
            }
            Self::String(s) => JsonValue::String(s.clone()),
            Self::Date(date) => JsonValue::String(date.to_string()),
            Self::DateTime(dt) => JsonValue::String(dt.to_string()),
            Self::Time(time) => JsonValue::String(time.to_string()),
            Self::Quantity { value, unit, .. } => {
                let mut map = serde_json::Map::new();
                if let Ok(f) = value.to_string().parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        map.insert("value".to_string(), JsonValue::Number(n));
                    } else {
                        map.insert("value".to_string(), JsonValue::String(value.to_string()));
                    }
                } else {
                    map.insert("value".to_string(), JsonValue::String(value.to_string()));
                }
                if let Some(unit) = unit {
                    map.insert("unit".to_string(), JsonValue::String(unit.clone()));
                }
                JsonValue::Object(map)
            }
            Self::Resource(json) => (**json).clone(),
            Self::JsonValue(json) => (**json).clone(),
            Self::Id(id) => JsonValue::String(id.to_string()),
            Self::Base64Binary(data) => JsonValue::String(format!("base64({} bytes)", data.len())),
            Self::Uri(uri) => JsonValue::String(uri.clone()),
            Self::Url(url) => JsonValue::String(url.clone()),
            Self::Collection(collection) => collection.to_json_value(),
            Self::TypeInfoObject { namespace, name } => {
                let mut map = serde_json::Map::new();
                map.insert(
                    "namespace".to_string(),
                    JsonValue::String(namespace.clone()),
                );
                map.insert("name".to_string(), JsonValue::String(name.clone()));
                JsonValue::Object(map)
            }
            Self::Empty => JsonValue::Null,
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
            }
            Self::Resource(json) => {
                // Try to extract resource type for better display
                if let Some(resource_type) = json.get("resourceType").and_then(|rt| rt.as_str()) {
                    write!(f, "{}({})", resource_type, json)
                } else {
                    write!(f, "Resource({})", json)
                }
            }
            Self::Id(id) => write!(f, "{}", id),
            Self::Base64Binary(data) => write!(f, "base64({} bytes)", data.len()),
            Self::Uri(u) => write!(f, "{}", u),
            Self::Url(u) => write!(f, "{}", u),
            Self::Collection(collection) => {
                write!(f, "Collection[")?;
                for (i, val) in collection.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            Self::TypeInfoObject { namespace, name } => {
                if namespace.is_empty() {
                    write!(f, "TypeInfo({})", name)
                } else {
                    write!(f, "TypeInfo({}.{})", namespace, name)
                }
            }
            Self::JsonValue(json) => write!(f, "JsonValue({})", json),
            Self::Empty => write!(f, "{{}}"),
        }
    }
}

/// Evaluation result that preserves type information and metadata
/// Used for advanced tooling and API compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct ResultWithMetadata {
    /// The actual FHIRPath value
    pub value: FhirPathValue,
    /// Type information for this value
    pub type_info: ValueTypeInfo,
    /// Source location information (if available)
    pub source_location: Option<ValueSourceLocation>,
    /// Additional metadata for this result
    pub metadata: Option<JsonValue>,
}

/// Type information for FHIRPath values
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueTypeInfo {
    /// The primary type name (e.g., "String", "Integer", "Patient")
    pub type_name: String,
    /// Expected return type from static analysis
    pub expected_return_type: Option<String>,
    /// Cardinality information (0..1, 0..*, 1..1, etc.)
    pub cardinality: Option<String>,
    /// Type constraints or additional type information
    pub constraints: Vec<String>,
    /// Whether this is a FHIR resource type
    pub is_fhir_type: bool,
    /// Namespace for the type (e.g., "FHIR" for FHIR types)
    pub namespace: Option<String>,
}

/// Source location information for values (distinct from diagnostics SourceLocation)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueSourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Character offset in source
    pub offset: usize,
    /// Optional source identifier (e.g., expression name)
    pub source: Option<String>,
}

/// Collection of results with type information preserved
#[derive(Debug, Clone, PartialEq)]
pub struct CollectionWithMetadata {
    /// The results with metadata
    results: Vec<ResultWithMetadata>,
    /// Whether the collection is ordered
    is_ordered: bool,
    /// Collection-level type information
    collection_type: Option<ValueTypeInfo>,
}

impl ResultWithMetadata {
    /// Create a new result with metadata
    pub fn new(value: FhirPathValue, type_info: ValueTypeInfo) -> Self {
        Self {
            value,
            type_info,
            source_location: None,
            metadata: None,
        }
    }

    /// Create a result with source location
    pub fn with_location(
        value: FhirPathValue,
        type_info: ValueTypeInfo,
        location: ValueSourceLocation,
    ) -> Self {
        Self {
            value,
            type_info,
            source_location: Some(location),
            metadata: None,
        }
    }

    /// Add metadata to this result
    pub fn with_metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Convert to JSON representation for APIs - should return the actual value with metadata as separate structure
    pub fn to_json_parts(&self) -> JsonValue {
        // For FHIRPath Lab API, we want the actual value with metadata enrichment
        // The value should be the primary result, with type info as additional structure
        self.value.to_json_value()
    }

    /// Get the metadata as a separate structure for API responses
    pub fn get_type_metadata(&self) -> JsonValue {
        use serde_json::json;

        let mut metadata = serde_json::Map::new();

        // Add type information
        metadata.insert(
            "type".to_string(),
            JsonValue::String(self.type_info.type_name.clone()),
        );

        // Add expected return type if available
        if let Some(expected_type) = &self.type_info.expected_return_type {
            metadata.insert(
                "expectedReturnType".to_string(),
                JsonValue::String(expected_type.clone()),
            );
        }

        // Add cardinality if available
        if let Some(cardinality) = &self.type_info.cardinality {
            metadata.insert(
                "cardinality".to_string(),
                JsonValue::String(cardinality.clone()),
            );
        }

        // Add namespace if available
        if let Some(namespace) = &self.type_info.namespace {
            metadata.insert(
                "namespace".to_string(),
                JsonValue::String(namespace.clone()),
            );
        }

        // Add constraints
        if !self.type_info.constraints.is_empty() {
            metadata.insert(
                "constraints".to_string(),
                JsonValue::Array(
                    self.type_info
                        .constraints
                        .iter()
                        .map(|c| JsonValue::String(c.clone()))
                        .collect(),
                ),
            );
        }

        JsonValue::Object(metadata)
    }
}

impl ValueTypeInfo {
    /// Create type info from a FhirPathValue
    pub fn from_value(value: &FhirPathValue) -> Self {
        let type_name = value.type_name().to_string();
        let is_fhir_type = matches!(value, FhirPathValue::Resource(_));

        Self {
            type_name,
            expected_return_type: None,
            cardinality: Some("0..1".to_string()),
            constraints: Vec::new(),
            is_fhir_type,
            namespace: if is_fhir_type {
                Some("FHIR".to_string())
            } else {
                None
            },
        }
    }

    /// Create type info with expected return type
    pub fn with_expected_type(value: &FhirPathValue, expected_type: String) -> Self {
        let mut info = Self::from_value(value);
        info.expected_return_type = Some(expected_type);
        info
    }

    /// Add a constraint to this type info
    pub fn add_constraint(mut self, constraint: String) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set cardinality information
    pub fn with_cardinality(mut self, cardinality: String) -> Self {
        self.cardinality = Some(cardinality);
        self
    }
}

impl CollectionWithMetadata {
    /// Create a new empty collection with metadata
    pub fn empty() -> Self {
        Self {
            results: Vec::new(),
            is_ordered: true,
            collection_type: None,
        }
    }

    /// Create a collection from a single result
    pub fn single(result: ResultWithMetadata) -> Self {
        Self {
            results: vec![result],
            is_ordered: true,
            collection_type: None,
        }
    }

    /// Create from multiple results
    pub fn from_results(results: Vec<ResultWithMetadata>) -> Self {
        Self {
            results,
            is_ordered: true,
            collection_type: None,
        }
    }

    /// Convert to regular Collection (loses type information)
    pub fn to_collection(&self) -> Collection {
        let values = self.results.iter().map(|r| r.value.clone()).collect();
        Collection::from_values_with_ordering(values, self.is_ordered)
    }

    /// Add a result to the collection
    pub fn push(&mut self, result: ResultWithMetadata) {
        self.results.push(result);
    }

    /// Get iterator over results
    pub fn iter(&self) -> std::slice::Iter<'_, ResultWithMetadata> {
        self.results.iter()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get length of collection
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Get access to the individual results with their metadata
    pub fn results(&self) -> &[ResultWithMetadata] {
        &self.results
    }

    /// Convert to JSON representation for APIs - returns just the values
    pub fn to_json_parts(&self) -> JsonValue {
        let values: Vec<JsonValue> = self
            .results
            .iter()
            .map(|result| result.value.to_json_value())
            .collect();

        // Return single value if collection has only one item
        match values.len() {
            0 => JsonValue::Null,
            1 => values.into_iter().next().unwrap(),
            _ => JsonValue::Array(values),
        }
    }

    /// Get type metadata for all results in the collection
    pub fn get_type_metadata_array(&self) -> JsonValue {
        let metadata_array: Vec<JsonValue> = self
            .results
            .iter()
            .map(|result| result.get_type_metadata())
            .collect();

        JsonValue::Array(metadata_array)
    }
}

impl From<Collection> for CollectionWithMetadata {
    fn from(collection: Collection) -> Self {
        let results = collection
            .into_iter()
            .map(|value| {
                let type_info = ValueTypeInfo::from_value(&value);
                ResultWithMetadata::new(value, type_info)
            })
            .collect();

        Self::from_results(results)
    }
}

impl From<CollectionWithMetadata> for Collection {
    fn from(collection_with_metadata: CollectionWithMetadata) -> Self {
        collection_with_metadata.to_collection()
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
            }

            // ID comparisons
            (Self::Id(a), Self::Id(b)) => a.partial_cmp(b),

            _ => None, // Different types are not comparable
        }
    }
}
