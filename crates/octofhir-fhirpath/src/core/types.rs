//! Core FHIRPath type definitions with comprehensive value system

use octofhir_ucum::{UnitRecord, find_unit};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::sync::{Arc, LazyLock};

use super::error::{FhirPathError, Result};
use super::error_code::FP0051;
use super::model_provider::utils::extract_resource_type;
use super::model_provider::{ModelProvider, TypeInfo};
use super::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};

// Import the new evaluation types from fhir-model-rs
pub use octofhir_fhir_model::{EvaluationResult, IntoEvaluationResult, TypeInfoResult};
// New wrapped primitive element for Phase 1 implementation

/// A collection of FHIRPath values - the fundamental evaluation result type.
/// Uses Arc<Vec<>> for cheap cloning when creating child evaluation contexts.
#[derive(Debug, Clone, PartialEq)]
pub struct Collection {
    values: Arc<Vec<FhirPathValue>>,
    is_ordered: bool,
    mixed: bool, // Flag indicating this collection contains mixed types
}

impl Collection {
    /// Create a new empty collection (ordered by default)
    pub fn empty() -> Self {
        Self {
            values: Arc::new(Vec::new()),
            is_ordered: true,
            mixed: false,
        }
    }

    /// Create an empty collection with explicit ordering
    pub fn empty_with_ordering(is_ordered: bool) -> Self {
        Self {
            values: Arc::new(Vec::new()),
            is_ordered,
            mixed: false,
        }
    }

    /// Create a collection with a single value (ordered by default)
    pub fn single(value: FhirPathValue) -> Self {
        Self {
            values: Arc::new(vec![value]),
            is_ordered: true,
            mixed: false,
        }
    }

    /// Create a collection from a vector of values (ordered by default)
    pub fn from_values(values: Vec<FhirPathValue>) -> Self {
        Self {
            values: Arc::new(values),
            is_ordered: true,
            mixed: false,
        }
    }

    /// Create a collection from JSON resource with ModelProvider-aware typing
    pub async fn from_json_resource(
        json: JsonValue,
        model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    ) -> crate::core::Result<Self> {
        let resource_value =
            FhirPathValue::resource_with_model_provider(json, model_provider).await?;
        Ok(Self::from_values(vec![resource_value]))
    }

    /// Create a collection from Arc-wrapped JSON resource (avoids deep clone)
    pub async fn from_json_resource_arc(
        json: Arc<JsonValue>,
        model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    ) -> crate::core::Result<Self> {
        let resource_value =
            FhirPathValue::resource_with_model_provider_arc(json, model_provider).await?;
        Ok(Self::from_values(vec![resource_value]))
    }

    /// Create a collection from a vector with explicit ordering
    pub fn from_values_with_ordering(values: Vec<FhirPathValue>, is_ordered: bool) -> Self {
        Self {
            values: Arc::new(values),
            is_ordered,
            mixed: false,
        }
    }

    /// Create a mixed collection (for collections containing different types)
    pub fn from_values_mixed(values: Vec<FhirPathValue>, is_ordered: bool) -> Self {
        Self {
            values: Arc::new(values),
            is_ordered,
            mixed: true,
        }
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

    /// Check if the collection contains mixed types
    pub fn is_mixed(&self) -> bool {
        self.mixed
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
        Arc::make_mut(&mut self.values).push(value);
    }

    /// Convert to vector
    pub fn into_vec(self) -> Vec<FhirPathValue> {
        Arc::try_unwrap(self.values).unwrap_or_else(|arc| (*arc).clone())
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
        Arc::try_unwrap(self.values)
            .unwrap_or_else(|arc| (*arc).clone())
            .into_iter()
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

/// Wrapped primitive element for FHIR primitive type extensions
/// Replaces the old wrapped module approach with direct embedding
#[derive(Debug, Clone, PartialEq)]
pub struct WrappedPrimitiveElement {
    /// Element ID for FHIR primitive types
    pub id: Option<String>,
    /// Extensions for primitive types
    pub extensions: Vec<WrappedExtension>,
}

/// Extension for wrapped primitive elements
#[derive(Debug, Clone, PartialEq)]
pub struct WrappedExtension {
    /// Extension URL
    pub url: String,
    /// Extension value key (e.g., "valueDateTime", "valueString")
    pub value_key: Option<String>,
    /// Extension value
    pub value: Option<JsonValue>,
    /// Nested extensions
    pub extensions: Vec<WrappedExtension>,
}

impl Default for WrappedPrimitiveElement {
    fn default() -> Self {
        Self::new()
    }
}

impl WrappedPrimitiveElement {
    /// Create new wrapped primitive element
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

    /// Add extension
    pub fn add_extension(mut self, extension: WrappedExtension) -> Self {
        self.extensions.push(extension);
        self
    }

    /// Check if has any content
    pub fn is_empty(&self) -> bool {
        self.id.is_none() && self.extensions.is_empty()
    }
}

impl WrappedExtension {
    /// Create new extension
    pub fn new(url: String) -> Self {
        Self {
            url,
            value_key: None,
            value: None,
            extensions: Vec::new(),
        }
    }

    /// Create with value and key name
    pub fn with_value(url: String, value_key: String, value: JsonValue) -> Self {
        Self {
            url,
            value_key: Some(value_key),
            value: Some(value),
            extensions: Vec::new(),
        }
    }

    /// Convert to a complete JSON representation of the extension
    pub fn to_json(&self) -> JsonValue {
        let mut map = serde_json::Map::new();

        // Add URL
        map.insert("url".to_string(), JsonValue::String(self.url.clone()));

        // Add value with its original key name (e.g., valueDateTime, valueString)
        if let (Some(key), Some(value)) = (&self.value_key, &self.value) {
            map.insert(key.clone(), value.clone());
        }

        // Add nested extensions if present
        if !self.extensions.is_empty() {
            let nested: Vec<JsonValue> = self.extensions.iter().map(|e| e.to_json()).collect();
            map.insert("extension".to_string(), JsonValue::Array(nested));
        }

        JsonValue::Object(map)
    }
}

/// FHIRPath value type with wrapped terminology and TypeInfo integration
/// Phase 1: New design with wrapped value system and ModelProvider integration
#[derive(Debug, Clone, PartialEq)]
pub enum FhirPathValue {
    /// Boolean value with TypeInfo and optional wrapped primitive element
    Boolean(bool, TypeInfo, Option<WrappedPrimitiveElement>),

    /// Integer value (64-bit signed) with TypeInfo and optional wrapped primitive element
    Integer(i64, TypeInfo, Option<WrappedPrimitiveElement>),

    /// Decimal value (high-precision) with TypeInfo and optional wrapped primitive element
    Decimal(Decimal, TypeInfo, Option<WrappedPrimitiveElement>),

    /// String value with TypeInfo and optional wrapped primitive element
    String(String, TypeInfo, Option<WrappedPrimitiveElement>),

    /// Date value with precision tracking, TypeInfo and optional wrapped primitive element
    Date(PrecisionDate, TypeInfo, Option<WrappedPrimitiveElement>),

    /// DateTime value with timezone and precision tracking, TypeInfo and optional wrapped primitive element
    DateTime(PrecisionDateTime, TypeInfo, Option<WrappedPrimitiveElement>),

    /// Time value with precision tracking, TypeInfo and optional wrapped primitive element
    Time(PrecisionTime, TypeInfo, Option<WrappedPrimitiveElement>),

    /// Quantity value with UCUM unit support, TypeInfo and optional wrapped primitive element
    Quantity {
        /// The numeric value of the quantity
        value: Decimal,
        /// Human-readable unit string (FHIR `unit`)
        unit: Option<String>,
        /// Canonical unit code (FHIR `code`)
        code: Option<String>,
        /// System URI (FHIR `system`)
        system: Option<String>,
        /// Parsed UCUM unit for calculations (cached for performance)
        ucum_unit: Option<Arc<UnitRecord>>,
        /// Calendar unit for non-UCUM time units (year, month, week, day)
        calendar_unit: Option<CalendarUnit>,
        /// Type information from ModelProvider
        type_info: TypeInfo,
        /// Optional wrapped primitive element
        primitive_element: Option<WrappedPrimitiveElement>,
    },

    /// Resource/complex type with TypeInfo for FHIR schema validation
    Resource(Arc<JsonValue>, TypeInfo, Option<WrappedPrimitiveElement>),

    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Collection),

    /// Null/empty value (represents absence)
    Empty,
}

impl serde::Serialize for FhirPathValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Boolean(b, _, _) => serializer.serialize_bool(*b),
            Self::Integer(i, _, _) => serializer.serialize_i64(*i),
            Self::Decimal(d, _, _) => {
                // Try to serialize as number, fall back to string if precision issues
                if let Ok(f) = d.to_string().parse::<f64>() {
                    serializer.serialize_f64(f)
                } else {
                    serializer.serialize_str(&d.to_string())
                }
            }
            Self::String(s, _, _) => serializer.serialize_str(s),
            Self::Date(date, _, _) => serializer.serialize_str(&date.to_string()),
            Self::DateTime(dt, _, _) => serializer.serialize_str(&dt.to_string()),
            Self::Time(time, _, _) => serializer.serialize_str(&time.to_string()),
            Self::Quantity { value, unit, .. } => {
                // Serialize Quantity as its FHIRPath literal string form: "<value> '<unit>'" or just "<value>"
                let s = if let Some(unit) = unit {
                    format!("{value} '{unit}'")
                } else {
                    value.to_string()
                };
                serializer.serialize_str(&s)
            }
            Self::Resource(json, _, _) => json.serialize(serializer),
            Self::Collection(collection) => collection.serialize(serializer),
            Self::Empty => serializer.serialize_unit(),
        }
    }
}

/// Calendar units for temporal calculations (not supported by UCUM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarUnit {
    /// One millisecond - 1/1000 of a calendar second
    Millisecond,
    /// One second
    Second,
    /// One minute - exactly 60 seconds
    Minute,
    /// One hour - exactly 60 minutes
    Hour,
    /// One day - exactly 24 hours
    Day,
    /// One week - exactly 7 days
    Week,
    /// A calendar month - Month is not a fixed size in days
    Month,
    /// A calendar year - exactly 12 months
    Year,
}

impl CalendarUnit {
    /// Get the unit name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Millisecond => "millisecond",
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }

    /// Parse from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "millisecond" | "milliseconds" | "ms" => Some(Self::Millisecond),
            "second" | "seconds" | "sec" | "secs" | "s" => Some(Self::Second),
            "minute" | "minutes" | "min" | "mins" | "m" => Some(Self::Minute),
            "hour" | "hours" | "hr" | "hrs" | "h" => Some(Self::Hour),
            "day" | "days" | "d" => Some(Self::Day),
            "week" | "weeks" | "wk" | "wks" | "w" => Some(Self::Week),
            "month" | "months" | "mos" => Some(Self::Month), // Removed "mo" - UCUM only
            "year" | "years" | "yr" | "yrs" => Some(Self::Year), // Removed "a" - UCUM only
            _ => None,
        }
    }

    /// Get the human-readable description of the unit
    pub fn description(&self) -> &'static str {
        match self {
            Self::Millisecond => "One millisecond - 1/1000 of a calendar second",
            Self::Second => "One second",
            Self::Minute => "One minute - exactly 60 seconds",
            Self::Hour => "One hour - exactly 60 minutes",
            Self::Day => "One day - exactly 24 hours",
            Self::Week => "One week - exactly 7 days",
            Self::Month => "A calendar month - Month is not a fixed size in days",
            Self::Year => "A calendar year - exactly 12 months",
        }
    }

    /// Check if this unit has a fixed duration in lower units
    /// Returns true for units that always have the same duration
    pub fn is_fixed_duration(&self) -> bool {
        match self {
            Self::Millisecond => true,
            Self::Second => true,
            Self::Minute => true,
            Self::Hour => true,
            Self::Day => true,
            Self::Week => true,
            Self::Month => false, // Varies between 28-31 days
            Self::Year => false,  // Varies between 365-366 days
        }
    }

    /// Get the conversion factor to milliseconds for fixed-duration units
    /// Returns None for variable-duration units (month, year)
    pub fn to_milliseconds(&self) -> Option<i64> {
        match self {
            Self::Millisecond => Some(1),
            Self::Second => Some(1_000),
            Self::Minute => Some(60_000),
            Self::Hour => Some(3_600_000),
            Self::Day => Some(86_400_000),
            Self::Week => Some(604_800_000),
            Self::Month => None, // Variable duration
            Self::Year => None,  // Variable duration
        }
    }

    /// Get all calendar units in order from smallest to largest
    pub fn all_units() -> &'static [CalendarUnit] {
        &[
            CalendarUnit::Millisecond,
            CalendarUnit::Second,
            CalendarUnit::Minute,
            CalendarUnit::Hour,
            CalendarUnit::Day,
            CalendarUnit::Week,
            CalendarUnit::Month,
            CalendarUnit::Year,
        ]
    }

    /// Check if this unit is compatible for arithmetic with another unit
    /// Generally, only units of the same "type" can be safely combined
    pub fn is_compatible_with(&self, other: &CalendarUnit) -> bool {
        let time_units = [
            CalendarUnit::Millisecond,
            CalendarUnit::Second,
            CalendarUnit::Minute,
            CalendarUnit::Hour,
        ];
        let date_units = [CalendarUnit::Day, CalendarUnit::Week];
        let calendar_units = [CalendarUnit::Month, CalendarUnit::Year];

        let self_is_time = time_units.contains(self);
        let self_is_date = date_units.contains(self);
        let self_is_calendar = calendar_units.contains(self);

        let other_is_time = time_units.contains(other);
        let other_is_date = date_units.contains(other);
        let other_is_calendar = calendar_units.contains(other);

        // Same category units are compatible
        (self_is_time && other_is_time)
            || (self_is_date && other_is_date)
            || (self_is_calendar && other_is_calendar)
            // Time units are compatible with date units
            || (self_is_time && other_is_date)
            || (self_is_date && other_is_time)
    }

    /// Convert to approximate days (for rough comparisons only)
    pub fn approximate_days(&self) -> f64 {
        match self {
            Self::Millisecond => 1.0 / 86_400_000.0,
            Self::Second => 1.0 / 86_400.0,
            Self::Minute => 1.0 / 1_440.0,
            Self::Hour => 1.0 / 24.0,
            Self::Day => 1.0,
            Self::Week => 7.0,
            Self::Month => 30.44, // Average month length
            Self::Year => 365.25,
        }
    }
}

impl fmt::Display for CalendarUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

fn normalize_ucum_unit(unit: &str) -> Cow<'_, str> {
    match unit.trim() {
        "°C" => Cow::Borrowed("Cel"),
        "°F" => Cow::Borrowed("[degF]"),
        _ => Cow::Borrowed(unit),
    }
}

impl FhirPathValue {
    /// Create an integer value with default TypeInfo
    pub fn integer(value: i64) -> Self {
        let type_info = TypeInfo {
            type_name: "Integer".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Integer".to_string()),
            is_empty: Some(false),
        };
        Self::Integer(value, type_info, None)
    }

    /// Create a new Long value with default TypeInfo (FHIRPath v3.0.0-ballot STU)
    pub fn long(value: i64) -> Self {
        let type_info = TypeInfo {
            type_name: "Long".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Long".to_string()),
            is_empty: Some(false),
        };
        // Long is stored in the Integer variant with Long type info
        Self::Integer(value, type_info, None)
    }

    /// Get TypeInfo for this value (always available in Phase 1)
    pub fn type_info(&self) -> &TypeInfo {
        match self {
            Self::Boolean(_, type_info, _)
            | Self::Integer(_, type_info, _)
            | Self::Decimal(_, type_info, _)
            | Self::String(_, type_info, _)
            | Self::Date(_, type_info, _)
            | Self::DateTime(_, type_info, _)
            | Self::Time(_, type_info, _)
            | Self::Resource(_, type_info, _) => type_info,
            Self::Quantity { type_info, .. } => type_info,
            Self::Collection(_) => {
                // Return a reference to a static lazy TypeInfo
                static COLLECTION_TYPE: LazyLock<TypeInfo> = LazyLock::new(|| TypeInfo {
                    type_name: "Collection".to_string(),
                    singleton: Some(false),
                    namespace: Some("System".to_string()),
                    name: Some("Collection".to_string()),
                    is_empty: Some(false),
                });
                &COLLECTION_TYPE
            }
            Self::Empty => {
                // Return a reference to a static lazy TypeInfo
                static EMPTY_TYPE: LazyLock<TypeInfo> = LazyLock::new(|| TypeInfo {
                    type_name: "Empty".to_string(),
                    singleton: Some(true),
                    namespace: Some("System".to_string()),
                    name: Some("Empty".to_string()),
                    is_empty: Some(true),
                });
                &EMPTY_TYPE
            }
        }
    }

    /// Get wrapped primitive element if available
    pub fn wrapped_primitive_element(&self) -> Option<&WrappedPrimitiveElement> {
        match self {
            Self::Boolean(_, _, element)
            | Self::Integer(_, _, element)
            | Self::Decimal(_, _, element)
            | Self::String(_, _, element)
            | Self::Date(_, _, element)
            | Self::DateTime(_, _, element)
            | Self::Time(_, _, element)
            | Self::Resource(_, _, element) => element.as_ref(),
            Self::Quantity {
                primitive_element, ..
            } => primitive_element.as_ref(),
            _ => None,
        }
    }

    /// Check if this value has primitive extensions
    pub fn has_primitive_extensions(&self) -> bool {
        self.wrapped_primitive_element()
            .is_some_and(|pe| !pe.extensions.is_empty())
    }

    /// Wrap a value with TypeInfo and optional primitive element
    pub fn wrap_value(
        value: Self,
        type_info: TypeInfo,
        primitive_element: Option<WrappedPrimitiveElement>,
    ) -> Self {
        match value {
            Self::Boolean(v, _, _) => Self::Boolean(v, type_info, primitive_element),
            Self::Integer(v, _, _) => Self::Integer(v, type_info, primitive_element),
            Self::Decimal(v, _, _) => Self::Decimal(v, type_info, primitive_element),
            Self::String(v, _, _) => Self::String(v, type_info, primitive_element),
            Self::Date(v, _, _) => Self::Date(v, type_info, primitive_element),
            Self::DateTime(v, _, _) => Self::DateTime(v, type_info, primitive_element),
            Self::Time(v, _, _) => Self::Time(v, type_info, primitive_element),
            Self::Resource(v, _, _) => Self::Resource(v, type_info, primitive_element),
            Self::Quantity {
                value,
                unit,
                code,
                system,
                ucum_unit,
                calendar_unit,
                ..
            } => {
                // Ensure system field has default value for UCUM units when not already set
                let resolved_system = if system.is_none() && ucum_unit.is_some() {
                    Some("http://unitsofmeasure.org".to_string())
                } else {
                    system
                };

                Self::Quantity {
                    value,
                    unit,
                    code,
                    system: resolved_system,
                    ucum_unit,
                    calendar_unit,
                    type_info,
                    primitive_element,
                }
            }
            other => other, // Collection and Empty don't change
        }
    }

    /// Unwrap the raw value without TypeInfo or primitive element
    pub fn unwrap_value(&self) -> Self {
        let default_type_info = TypeInfo {
            type_name: "Unknown".to_string(),
            singleton: Some(true),
            namespace: None,
            name: Some("Unknown".to_string()),
            is_empty: Some(false),
        };

        match self {
            Self::Boolean(v, _, _) => Self::Boolean(*v, default_type_info.clone(), None),
            Self::Integer(v, _, _) => Self::Integer(*v, default_type_info.clone(), None),
            Self::Decimal(v, _, _) => Self::Decimal(*v, default_type_info.clone(), None),
            Self::String(v, _, _) => Self::String(v.clone(), default_type_info.clone(), None),
            Self::Date(v, _, _) => Self::Date(v.clone(), default_type_info.clone(), None),
            Self::DateTime(v, _, _) => Self::DateTime(v.clone(), default_type_info.clone(), None),
            Self::Time(v, _, _) => Self::Time(v.clone(), default_type_info.clone(), None),
            Self::Resource(v, _, _) => Self::Resource(v.clone(), default_type_info.clone(), None),
            Self::Quantity {
                value,
                unit,
                code,
                system,
                ucum_unit,
                calendar_unit,
                ..
            } => {
                // Ensure system field has default value for UCUM units when not already set
                let resolved_system = if system.is_none() && ucum_unit.is_some() {
                    Some("http://unitsofmeasure.org".to_string())
                } else {
                    system.clone()
                };

                Self::Quantity {
                    value: *value,
                    unit: unit.clone(),
                    code: code.clone(),
                    system: resolved_system,
                    ucum_unit: ucum_unit.clone(),
                    calendar_unit: *calendar_unit,
                    type_info: default_type_info,
                    primitive_element: None,
                }
            }
            other => other.clone(),
        }
    }

    /// Ensure value is wrapped with proper TypeInfo (from ModelProvider)
    pub fn ensure_wrapped(&self, _model_provider: &dyn crate::core::ModelProvider) -> Self {
        // In a real implementation, this would use the ModelProvider to get proper TypeInfo
        // For now, return a clone with the existing TypeInfo
        self.clone()
    }

    /// Get the FHIRPath type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Boolean(_, _, _) => "Boolean",
            Self::Integer(_, _, _) => "Integer",
            Self::Decimal(_, _, _) => "Decimal",
            Self::String(_, _, _) => "String",
            Self::Date(_, _, _) => "Date",
            Self::DateTime(_, _, _) => "DateTime",
            Self::Time(_, _, _) => "Time",
            Self::Quantity { .. } => "Quantity",
            Self::Resource(_, _, _) => "Resource",
            Self::Collection(_) => "Collection",
            Self::Empty => "empty",
        }
    }

    /// Get the display type name, using TypeInfo when available for Resources
    pub fn display_type_name(&self) -> String {
        match self {
            // For all primitive variants, prefer the embedded TypeInfo's type_name.
            // This allows FHIR primitive subtypes (e.g., 'code') to be displayed instead of generic 'String'.
            Self::Boolean(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::Integer(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::Decimal(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::String(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::Date(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::DateTime(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::Time(_, type_info, _) => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::Quantity { type_info, .. } => type_info
                .name
                .as_deref()
                .unwrap_or(&type_info.type_name)
                .to_string(),
            Self::Resource(_, type_info, _) => {
                // Use the corrected specific name if present (e.g., HumanName), otherwise type_name
                type_info
                    .name
                    .as_deref()
                    .unwrap_or(&type_info.type_name)
                    .to_string()
            }
            Self::Collection(_) => "Collection".to_string(),
            Self::Empty => "empty".to_string(),
        }
    }

    /// Check if this value is a singleton (not a collection)
    pub fn is_singleton(&self) -> bool {
        !matches!(self, Self::Collection(_) | Self::Empty)
    }

    // TODO: has_type method will be reimplemented when registry is redesigned
    // /// Check if this value matches a specific FHIRPath type
    // pub fn has_type(&self, fhir_type: &FhirPathType) -> bool { ... }

    /// Get the length of this value (1 for singletons, actual length for collections)
    pub fn len(&self) -> usize {
        match self {
            Self::Collection(collection) => collection.len(),
            Self::Empty => 0,
            _ => 1,
        }
    }

    /// Check if this value is empty
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
            Self::Boolean(b, _, _) => Ok(*b),
            Self::Integer(i, _, _) => Ok(*i != 0),
            Self::Decimal(d, _, _) => Ok(!d.is_zero()),
            Self::String(s, _, _) => Ok(!s.is_empty()),
            Self::Empty => Ok(false),
            _ => Err(FhirPathError::evaluation_error(
                FP0051,
                format!("Cannot convert {} to boolean", self.type_name()),
            )),
        }
    }

    /// Extract boolean value directly (returns None if not a boolean or conversion fails)
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b, _, _) => Some(*b),
            _ => None,
        }
    }

    /// Extract integer value directly (returns None if not an integer)
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i, _, _) => Some(*i),
            _ => None,
        }
    }

    /// Extract decimal value directly (returns None if not a decimal)
    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Decimal(d, _, _) => Some(*d),
            _ => None,
        }
    }

    /// Extract string value directly (returns None if not a string)
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s, _, _) => Some(s),
            _ => None,
        }
    }

    /// Convert to string (FHIRPath string conversion rules)
    pub fn to_string(&self) -> Result<String> {
        match self {
            Self::String(s, _, _) => Ok(s.clone()),
            Self::Integer(i, _, _) => Ok(i.to_string()),
            Self::Decimal(d, _, _) => Ok(d.to_string()),
            Self::Boolean(b, _, _) => Ok(b.to_string()),
            Self::Date(d, _, _) => Ok(d.to_string()),
            Self::DateTime(dt, _, _) => Ok(dt.to_string()),
            Self::Time(t, _, _) => Ok(t.to_string()),
            Self::Empty => Ok(String::new()),
            _ => Err(FhirPathError::evaluation_error(
                FP0051,
                format!("Cannot convert {} to string", self.type_name()),
            )),
        }
    }

    /// Create a string value with default TypeInfo
    pub fn string(s: impl Into<String>) -> Self {
        let type_info = TypeInfo {
            type_name: "String".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("String".to_string()),
            is_empty: Some(false),
        };
        Self::String(s.into(), type_info, None)
    }

    /// Create a decimal value with default TypeInfo
    pub fn decimal(d: impl Into<Decimal>) -> Self {
        let type_info = TypeInfo {
            type_name: "Decimal".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Decimal".to_string()),
            is_empty: Some(false),
        };
        Self::Decimal(d.into(), type_info, None)
    }

    /// Create a boolean value with default TypeInfo
    pub fn boolean(b: bool) -> Self {
        let type_info = TypeInfo {
            type_name: "Boolean".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Boolean".to_string()),
            is_empty: Some(false),
        };
        Self::Boolean(b, type_info, None)
    }

    /// Create a resource value from JSON with default TypeInfo
    pub fn resource(json: JsonValue) -> Self {
        // Try to infer concrete FHIR resource type from the JSON payload
        let inferred_type = json
            .as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|rt| rt.as_str())
            .map(|s| s.to_string());

        let type_info = if let Some(resource_type) = inferred_type {
            // Set precise FHIR resource typing when resourceType is present
            TypeInfo {
                type_name: resource_type.clone(),
                singleton: Some(true),
                namespace: Some("FHIR".to_string()),
                name: Some(resource_type),
                is_empty: Some(false),
            }
        } else {
            // Fallback to generic Resource typing
            TypeInfo {
                type_name: "Resource".to_string(),
                singleton: Some(true),
                namespace: Some("FHIR".to_string()),
                name: Some("Resource".to_string()),
                is_empty: Some(false),
            }
        };

        Self::Resource(Arc::new(json), type_info, None)
    }

    /// Create a resource value from an already Arc-wrapped JSON (avoids clone)
    pub fn resource_from_arc(json: Arc<JsonValue>) -> Self {
        let inferred_type = json
            .as_object()
            .and_then(|obj| obj.get("resourceType"))
            .and_then(|rt| rt.as_str())
            .map(|s| s.to_string());

        let type_info = if let Some(resource_type) = inferred_type {
            TypeInfo {
                type_name: resource_type.clone(),
                singleton: Some(true),
                namespace: Some("FHIR".to_string()),
                name: Some(resource_type),
                is_empty: Some(false),
            }
        } else {
            TypeInfo {
                type_name: "Resource".to_string(),
                singleton: Some(true),
                namespace: Some("FHIR".to_string()),
                name: Some("Resource".to_string()),
                is_empty: Some(false),
            }
        };

        Self::Resource(json, type_info, None)
    }

    /// Create a resource value with ModelProvider-aware type resolution
    pub async fn resource_with_model_provider(
        json: JsonValue,
        model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    ) -> crate::core::Result<Self> {
        if let Some(provider) = model_provider {
            // Try to extract resourceType and get proper TypeInfo
            if let Some(resource_type) = extract_resource_type(&json) {
                match provider.get_type(&resource_type).await {
                    Ok(Some(type_info)) => {
                        return Ok(Self::Resource(Arc::new(json), type_info, None));
                    }
                    Ok(None) => {
                        // Type not found in model provider, fall through to default
                    }
                    Err(_) => {
                        // Error getting type, fall through to default
                    }
                }
            }
        }

        // Fallback to default resource TypeInfo
        Ok(Self::resource(json))
    }

    /// Create a resource value with ModelProvider-aware type resolution from Arc (avoids deep clone)
    pub async fn resource_with_model_provider_arc(
        json: Arc<JsonValue>,
        model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    ) -> crate::core::Result<Self> {
        if let Some(provider) = model_provider
            && let Some(resource_type) = extract_resource_type(&json)
        {
            match provider.get_type(&resource_type).await {
                Ok(Some(type_info)) => {
                    return Ok(Self::Resource(json, type_info, None));
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }

        // Fallback to default resource TypeInfo
        Ok(Self::resource_from_arc(json))
    }

    /// Create a resource value with TypeInfo
    pub fn resource_wrapped(json: JsonValue, type_info: TypeInfo) -> Self {
        Self::Resource(Arc::new(json), type_info, None)
    }

    /// Create a wrapped value with type information
    pub fn wrapped_with_type_info(data: JsonValue, type_info: TypeInfo) -> Self {
        Self::Resource(Arc::new(data), type_info, None)
    }

    /// Create a quantity value with full component control
    pub fn quantity_with_components(
        value: Decimal,
        unit: Option<String>,
        code: Option<String>,
        system: Option<String>,
    ) -> Self {
        let mut resolved_system = system;

        let (ucum_unit, calendar_unit) = if let Some(ref code_str) = code {
            let normalized = normalize_ucum_unit(code_str);
            if let Some(cal_unit) = CalendarUnit::from_str(normalized.as_ref()) {
                (None, Some(cal_unit))
            } else {
                match find_unit(normalized.as_ref()) {
                    Some(ucum) => {
                        if resolved_system.is_none() {
                            resolved_system = Some("http://unitsofmeasure.org".to_string());
                        }
                        (Some(Arc::new(ucum.clone())), None)
                    }
                    None => (None, None),
                }
            }
        } else if let Some(ref unit_str) = unit {
            let normalized = normalize_ucum_unit(unit_str);
            if let Some(cal_unit) = CalendarUnit::from_str(normalized.as_ref()) {
                (None, Some(cal_unit))
            } else {
                match find_unit(normalized.as_ref()) {
                    Some(ucum) => {
                        if resolved_system.is_none() {
                            resolved_system = Some("http://unitsofmeasure.org".to_string());
                        }
                        (Some(Arc::new(ucum.clone())), None)
                    }
                    None => (None, None),
                }
            }
        } else {
            (None, None)
        };

        let type_info = TypeInfo {
            type_name: "Quantity".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Quantity".to_string()),
            is_empty: Some(false),
        };

        Self::Quantity {
            value,
            unit,
            code,
            system: resolved_system,
            ucum_unit,
            calendar_unit,
            type_info,
            primitive_element: None,
        }
    }

    /// Create a quantity value with UCUM unit parsing and default TypeInfo
    pub fn quantity(value: Decimal, unit: Option<String>) -> Self {
        Self::quantity_with_components(value, unit.clone(), unit, None)
    }

    /// Create a quantity value for quoted units (UCUM only, no calendar units)
    pub fn quoted_quantity(value: Decimal, unit: Option<String>) -> Self {
        Self::quantity_with_components(value, unit.clone(), unit, None)
    }

    /// Create a quantity value with explicit calendar unit
    pub fn calendar_quantity(value: Decimal, calendar_unit: CalendarUnit) -> Self {
        let type_info = TypeInfo {
            type_name: "Quantity".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Quantity".to_string()),
            is_empty: Some(false),
        };

        Self::Quantity {
            value,
            unit: Some(calendar_unit.to_string()),
            code: Some(calendar_unit.to_string()),
            system: None,
            ucum_unit: None,
            calendar_unit: Some(calendar_unit),
            type_info,
            primitive_element: None,
        }
    }

    /// Create a date value with default TypeInfo
    pub fn date(date: PrecisionDate) -> Self {
        let type_info = TypeInfo {
            type_name: "Date".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Date".to_string()),
            is_empty: Some(false),
        };
        Self::Date(date, type_info, None)
    }

    /// Create a datetime value with default TypeInfo
    pub fn datetime(datetime: PrecisionDateTime) -> Self {
        let type_info = TypeInfo {
            type_name: "DateTime".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("DateTime".to_string()),
            is_empty: Some(false),
        };
        Self::DateTime(datetime, type_info, None)
    }

    /// Create a time value with default TypeInfo
    pub fn time(time: PrecisionTime) -> Self {
        let type_info = TypeInfo {
            type_name: "Time".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Time".to_string()),
            is_empty: Some(false),
        };
        Self::Time(time, type_info, None)
    }

    /// Create a JSON value (for compatibility) with default TypeInfo
    pub fn json_value(json: JsonValue) -> Self {
        let type_info = TypeInfo {
            type_name: "Json".to_string(),
            singleton: Some(true),
            namespace: Some("System".to_string()),
            name: Some("Json".to_string()),
            is_empty: Some(false),
        };
        Self::Resource(Arc::new(json), type_info, None)
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

    /// Get the wrapped JSON value if this is a resource value
    pub fn unwrap_json(&self) -> Option<&JsonValue> {
        match self {
            Self::Resource(json, _, _) => Some(json),
            _ => None,
        }
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
            Self::Boolean(b, _, _) => JsonValue::Bool(*b),
            Self::Integer(i, _, _) => JsonValue::Number(serde_json::Number::from(*i)),
            Self::Decimal(d, _, _) => {
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
            Self::String(s, _, _) => JsonValue::String(s.clone()),
            Self::Date(date, _, _) => JsonValue::String(date.to_string()),
            Self::DateTime(dt, _, _) => JsonValue::String(dt.to_string()),
            Self::Time(time, _, _) => JsonValue::String(time.to_string()),
            Self::Quantity {
                value,
                unit,
                code,
                system,
                ..
            } => {
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
                if let Some(code) = code {
                    map.insert("code".to_string(), JsonValue::String(code.clone()));
                }
                if let Some(system) = system {
                    map.insert("system".to_string(), JsonValue::String(system.clone()));
                }
                JsonValue::Object(map)
            }
            Self::Resource(json, _, _) => (**json).clone(),
            Self::Collection(collection) => collection.to_json_value(),
            Self::Empty => JsonValue::Null,
        }
    }

    /// Create a new value with updated type info
    pub fn with_type_info(&self, new_type_info: crate::core::model_provider::TypeInfo) -> Self {
        match self {
            Self::Boolean(b, _, primitive) => Self::Boolean(*b, new_type_info, primitive.clone()),
            Self::Integer(i, _, primitive) => Self::Integer(*i, new_type_info, primitive.clone()),
            Self::Decimal(d, _, primitive) => Self::Decimal(*d, new_type_info, primitive.clone()),
            Self::String(s, _, primitive) => {
                Self::String(s.clone(), new_type_info, primitive.clone())
            }
            Self::Date(date, _, primitive) => {
                Self::Date(date.clone(), new_type_info, primitive.clone())
            }
            Self::DateTime(dt, _, primitive) => {
                Self::DateTime(dt.clone(), new_type_info, primitive.clone())
            }
            Self::Time(time, _, primitive) => {
                Self::Time(time.clone(), new_type_info, primitive.clone())
            }
            Self::Quantity {
                value,
                unit,
                code,
                system,
                ucum_unit,
                calendar_unit,
                ..
            } => {
                // Ensure system field has default value for UCUM units when not already set
                let resolved_system = if system.is_none() && ucum_unit.is_some() {
                    Some("http://unitsofmeasure.org".to_string())
                } else {
                    system.clone()
                };

                Self::Quantity {
                    value: *value,
                    unit: unit.clone(),
                    code: code.clone(),
                    system: resolved_system,
                    ucum_unit: ucum_unit.clone(),
                    calendar_unit: *calendar_unit,
                    type_info: new_type_info,
                    primitive_element: None,
                }
            }
            Self::Resource(json, _, primitive) => {
                Self::Resource(json.clone(), new_type_info, primitive.clone())
            }
            Self::Collection(collection) => Self::Collection(collection.clone()),
            Self::Empty => Self::Empty,
        }
    }

    /// Convert to EvaluationResult for efficient operations
    pub fn to_evaluation_result(&self) -> EvaluationResult {
        match self {
            Self::Boolean(b, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Boolean(*b, type_info_result)
            }
            Self::Integer(i, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Integer(*i, type_info_result)
            }
            Self::Decimal(d, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Decimal(*d, type_info_result)
            }
            Self::String(s, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::String(s.clone(), type_info_result)
            }
            Self::Date(d, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Date(d.to_string(), type_info_result)
            }
            Self::DateTime(dt, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::DateTime(dt.to_string(), type_info_result)
            }
            Self::Time(t, type_info, _) => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Time(t.to_string(), type_info_result)
            }
            Self::Quantity {
                value,
                unit,
                type_info,
                ..
            } => {
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Quantity(
                    *value,
                    unit.clone().unwrap_or_default(),
                    type_info_result,
                )
            }
            Self::Resource(json, type_info, _) => {
                // Convert to object representation
                let mut map = std::collections::HashMap::new();
                if let JsonValue::Object(obj) = json.as_ref() {
                    for (key, value) in obj {
                        map.insert(key.clone(), self.json_to_evaluation_result(value));
                    }
                }
                let type_info_result = type_info
                    .namespace
                    .as_ref()
                    .map(|ns| TypeInfoResult::new(ns, &type_info.type_name));
                EvaluationResult::Object {
                    map,
                    type_info: type_info_result,
                }
            }
            Self::Collection(collection) => {
                let items = collection
                    .iter()
                    .map(|v| v.to_evaluation_result())
                    .collect();
                EvaluationResult::Collection {
                    items,
                    has_undefined_order: !collection.is_ordered(),
                    type_info: None,
                }
            }
            Self::Empty => EvaluationResult::Empty,
        }
    }

    /// Helper method to convert JSON values to EvaluationResult
    #[allow(clippy::only_used_in_recursion)]
    fn json_to_evaluation_result(&self, value: &JsonValue) -> EvaluationResult {
        match value {
            JsonValue::Null => EvaluationResult::Empty,
            JsonValue::Bool(b) => EvaluationResult::boolean(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    EvaluationResult::integer(i)
                } else if let Some(f) = n.as_f64() {
                    if let Ok(d) = f.to_string().parse::<Decimal>() {
                        EvaluationResult::decimal(d)
                    } else {
                        EvaluationResult::Empty
                    }
                } else {
                    EvaluationResult::Empty
                }
            }
            JsonValue::String(s) => EvaluationResult::string(s.clone()),
            JsonValue::Array(arr) => {
                let items = arr
                    .iter()
                    .map(|v| self.json_to_evaluation_result(v))
                    .collect();
                EvaluationResult::collection(items)
            }
            JsonValue::Object(obj) => {
                let mut map = std::collections::HashMap::new();
                for (key, val) in obj {
                    map.insert(key.clone(), self.json_to_evaluation_result(val));
                }
                EvaluationResult::object(map)
            }
        }
    }

    /// Convert from EvaluationResult back to FhirPathValue
    pub fn from_evaluation_result(eval_result: &EvaluationResult) -> Self {
        match eval_result {
            EvaluationResult::Empty => Self::Empty,
            EvaluationResult::Boolean(b, type_info) => {
                let ti = type_info
                    .as_ref()
                    .map(|ti| TypeInfo {
                        type_name: ti.name.clone(),
                        singleton: Some(true),
                        namespace: Some(ti.namespace.clone()),
                        name: Some(ti.name.clone()),
                        is_empty: Some(false),
                    })
                    .unwrap_or_else(|| TypeInfo {
                        type_name: "Boolean".to_string(),
                        singleton: Some(true),
                        namespace: Some("System".to_string()),
                        name: Some("Boolean".to_string()),
                        is_empty: Some(false),
                    });
                Self::Boolean(*b, ti, None)
            }
            EvaluationResult::Integer(i, type_info) => {
                let ti = type_info
                    .as_ref()
                    .map(|ti| TypeInfo {
                        type_name: ti.name.clone(),
                        singleton: Some(true),
                        namespace: Some(ti.namespace.clone()),
                        name: Some(ti.name.clone()),
                        is_empty: Some(false),
                    })
                    .unwrap_or_else(|| TypeInfo {
                        type_name: "Integer".to_string(),
                        singleton: Some(true),
                        namespace: Some("System".to_string()),
                        name: Some("Integer".to_string()),
                        is_empty: Some(false),
                    });
                Self::Integer(*i, ti, None)
            }
            EvaluationResult::String(s, type_info) => {
                let ti = type_info
                    .as_ref()
                    .map(|ti| TypeInfo {
                        type_name: ti.name.clone(),
                        singleton: Some(true),
                        namespace: Some(ti.namespace.clone()),
                        name: Some(ti.name.clone()),
                        is_empty: Some(false),
                    })
                    .unwrap_or_else(|| TypeInfo {
                        type_name: "String".to_string(),
                        singleton: Some(true),
                        namespace: Some("System".to_string()),
                        name: Some("String".to_string()),
                        is_empty: Some(false),
                    });
                Self::String(s.clone(), ti, None)
            }
            EvaluationResult::Decimal(d, type_info) => {
                let ti = type_info
                    .as_ref()
                    .map(|ti| TypeInfo {
                        type_name: ti.name.clone(),
                        singleton: Some(true),
                        namespace: Some(ti.namespace.clone()),
                        name: Some(ti.name.clone()),
                        is_empty: Some(false),
                    })
                    .unwrap_or_else(|| TypeInfo {
                        type_name: "Decimal".to_string(),
                        singleton: Some(true),
                        namespace: Some("System".to_string()),
                        name: Some("Decimal".to_string()),
                        is_empty: Some(false),
                    });
                Self::Decimal(*d, ti, None)
            }
            EvaluationResult::Collection { items, .. } => {
                let values = items.iter().map(Self::from_evaluation_result).collect();
                Self::Collection(Collection::from_values(values))
            }
            // Add other variants as needed
            _ => Self::Empty, // Fallback for unhandled cases
        }
    }
}

impl fmt::Display for FhirPathValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(b, _, _) => write!(f, "{b}"),
            Self::Integer(i, _, _) => write!(f, "{i}"),
            Self::Decimal(d, _, _) => write!(f, "{d}"),
            Self::String(s, _, _) => write!(f, "'{s}'"),
            Self::Date(d, _, _) => write!(f, "@{d}"),
            Self::DateTime(dt, _, _) => write!(f, "@{dt}"),
            Self::Time(t, _, _) => write!(f, "@T{t}"),
            Self::Quantity { value, unit, .. } => {
                if let Some(unit) = unit {
                    write!(f, "{value} '{unit}'")
                } else {
                    write!(f, "{value}")
                }
            }
            Self::Resource(json, type_info, _) => {
                // Try to extract resource type for better display
                if let Some(resource_type) = json.get("resourceType").and_then(|rt| rt.as_str()) {
                    write!(f, "{resource_type}({json})")
                } else {
                    let display_name = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    write!(f, "{display_name}({json})")
                }
            }
            Self::Collection(collection) => {
                write!(f, "Collection[")?;
                for (i, val) in collection.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }
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
        let type_name = value.display_type_name();
        let is_fhir_type = matches!(value, FhirPathValue::Resource(json, type_info, _) if json.get("resourceType").is_some() || type_info.namespace.as_deref() == Some("FHIR"));

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
            (Self::Integer(a, _, _), Self::Integer(b, _, _)) => a.partial_cmp(b),
            (Self::Decimal(a, _, _), Self::Decimal(b, _, _)) => a.partial_cmp(b),
            (Self::Integer(a, _, _), Self::Decimal(b, _, _)) => Decimal::from(*a).partial_cmp(b),
            (Self::Decimal(a, _, _), Self::Integer(b, _, _)) => a.partial_cmp(&Decimal::from(*b)),

            // String comparisons
            (Self::String(a, _, _), Self::String(b, _, _)) => a.partial_cmp(b),

            // Boolean comparison
            (Self::Boolean(a, _, _), Self::Boolean(b, _, _)) => a.partial_cmp(b),

            // Temporal comparisons
            (Self::Date(a, _, _), Self::Date(b, _, _)) => a.partial_cmp(b),
            (Self::DateTime(a, _, _), Self::DateTime(b, _, _)) => a.partial_cmp(b),
            (Self::Time(a, _, _), Self::Time(b, _, _)) => a.partial_cmp(b),

            // Quantity comparisons (only if compatible units)
            (Self::Quantity { value: v1, .. }, Self::Quantity { value: v2, .. }) => {
                if self.is_quantity_compatible(other) {
                    v1.partial_cmp(v2)
                } else {
                    None // Incompatible units
                }
            }

            _ => None, // Different types are not comparable
        }
    }
}

/// Type details with constraints and cardinality information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDetails {
    /// Base type information
    pub base_info: TypeInfo,
    /// Cardinality constraints
    pub cardinality: Cardinality,
    /// Type-specific constraints
    pub constraints: Vec<TypeConstraint>,
    /// Whether this property is required
    pub is_required: bool,
    /// Default value if any
    pub default_value: Option<JsonValue>,
}

/// Cardinality specifications for properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Cardinality {
    /// 0..1 (zero or one)
    ZeroToOne,
    /// 0..* (zero or many)
    ZeroToMany,
    /// 1..1 (exactly one)
    OneToOne,
    /// 1..* (one or many)
    OneToMany,
    /// Exact count (e.g., exactly 3)
    Exact(usize),
    /// Range (min, max) where max=None means unlimited
    Range(usize, Option<usize>),
}

/// Type constraints for validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeConstraint {
    /// ValueSet constraint with ValueSet URL
    ValueSet(String),
    /// Regular expression pattern
    Pattern(String),
    /// Minimum string length
    MinLength(usize),
    /// Maximum string length
    MaxLength(usize),
    /// Minimum value (for numeric types)
    MinValue(JsonValue),
    /// Maximum value (for numeric types)
    MaxValue(JsonValue),
    /// Fixed value constraint
    FixedValue(JsonValue),
    /// Required property constraint
    RequiredProperty(String),
    /// Conditional constraint based on other properties
    ConditionalConstraint {
        condition: String,
        constraint: Box<TypeConstraint>,
    },
}

impl TypeDetails {
    /// Create new type details with basic information
    pub fn new(base_info: TypeInfo, cardinality: Cardinality) -> Self {
        Self {
            base_info,
            cardinality,
            constraints: Vec::new(),
            is_required: false,
            default_value: None,
        }
    }

    /// Create type details with constraints
    pub fn with_constraints(
        base_info: TypeInfo,
        cardinality: Cardinality,
        constraints: Vec<TypeConstraint>,
    ) -> Self {
        Self {
            base_info,
            cardinality,
            constraints,
            is_required: false,
            default_value: None,
        }
    }

    /// Mark this type as required
    pub fn required(mut self) -> Self {
        self.is_required = true;
        self
    }

    /// Set default value
    pub fn with_default(mut self, default_value: JsonValue) -> Self {
        self.default_value = Some(default_value);
        self
    }

    /// Add a constraint
    pub fn add_constraint(mut self, constraint: TypeConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Check if this type allows multiple values
    pub fn is_multiple(&self) -> bool {
        match self.cardinality {
            Cardinality::ZeroToMany | Cardinality::OneToMany => true,
            Cardinality::Exact(n) => n > 1,
            Cardinality::Range(_, Some(max)) => max > 1,
            Cardinality::Range(_, None) => true,
            _ => false,
        }
    }

    /// Check if this type is optional (allows zero values)
    pub fn is_optional(&self) -> bool {
        matches!(
            self.cardinality,
            Cardinality::ZeroToOne | Cardinality::ZeroToMany | Cardinality::Range(0, _)
        )
    }

    /// Get minimum cardinality
    pub fn min_cardinality(&self) -> usize {
        match self.cardinality {
            Cardinality::ZeroToOne | Cardinality::ZeroToMany => 0,
            Cardinality::OneToOne | Cardinality::OneToMany => 1,
            Cardinality::Exact(n) => n,
            Cardinality::Range(min, _) => min,
        }
    }

    /// Get maximum cardinality (None means unlimited)
    pub fn max_cardinality(&self) -> Option<usize> {
        match self.cardinality {
            Cardinality::ZeroToOne | Cardinality::OneToOne => Some(1),
            Cardinality::ZeroToMany | Cardinality::OneToMany => None,
            Cardinality::Exact(n) => Some(n),
            Cardinality::Range(_, max) => max,
        }
    }
}

impl Cardinality {
    /// Parse cardinality from FHIR-style string (e.g., "0..1", "1..*")
    pub fn from_fhir_string(s: &str) -> Result<Self> {
        match s {
            "0..1" => Ok(Self::ZeroToOne),
            "0..*" => Ok(Self::ZeroToMany),
            "1..1" => Ok(Self::OneToOne),
            "1..*" => Ok(Self::OneToMany),
            _ => {
                if let Ok(n) = s.parse::<usize>() {
                    Ok(Self::Exact(n))
                } else if let Some((min_str, max_str)) = s.split_once("..") {
                    let min = min_str.parse::<usize>().map_err(|_| {
                        FhirPathError::evaluation_error(
                            FP0051,
                            format!("Invalid cardinality format: {s}"),
                        )
                    })?;
                    let max = if max_str == "*" {
                        None
                    } else {
                        Some(max_str.parse::<usize>().map_err(|_| {
                            FhirPathError::evaluation_error(
                                FP0051,
                                format!("Invalid cardinality format: {s}"),
                            )
                        })?)
                    };
                    Ok(Self::Range(min, max))
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0051,
                        format!("Invalid cardinality format: {s}"),
                    ))
                }
            }
        }
    }

    /// Convert to FHIR-style string representation
    pub fn to_fhir_string(&self) -> String {
        match self {
            Self::ZeroToOne => "0..1".to_string(),
            Self::ZeroToMany => "0..*".to_string(),
            Self::OneToOne => "1..1".to_string(),
            Self::OneToMany => "1..*".to_string(),
            Self::Exact(n) => n.to_string(),
            Self::Range(min, Some(max)) => format!("{min}..{max}"),
            Self::Range(min, None) => format!("{min}..*"),
        }
    }

    /// Check if a count satisfies this cardinality
    pub fn allows_count(&self, count: usize) -> bool {
        let min = match self {
            Self::ZeroToOne | Self::ZeroToMany => 0,
            Self::OneToOne | Self::OneToMany => 1,
            Self::Exact(n) => *n,
            Self::Range(min, _) => *min,
        };

        let max = match self {
            Self::ZeroToOne | Self::OneToOne => Some(1),
            Self::ZeroToMany | Self::OneToMany => None,
            Self::Exact(n) => Some(*n),
            Self::Range(_, max) => *max,
        };

        count >= min && max.is_none_or(|m| count <= m)
    }
}

impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_fhir_string())
    }
}

impl TypeConstraint {
    /// Check if a value satisfies this constraint
    pub fn validate(&self, value: &FhirPathValue) -> Result<bool> {
        match self {
            Self::ValueSet(_url) => {
                // ValueSet validation would require terminology service
                // For now, always return true as this is infrastructure
                Ok(true)
            }
            Self::Pattern(pattern) => {
                if let Ok(string_val) = value.to_string() {
                    // Simple pattern matching - in real implementation would use regex
                    Ok(string_val.contains(pattern))
                } else {
                    Ok(false)
                }
            }
            Self::MinLength(min_len) => {
                if let Ok(string_val) = value.to_string() {
                    Ok(string_val.len() >= *min_len)
                } else {
                    Ok(false)
                }
            }
            Self::MaxLength(max_len) => {
                if let Ok(string_val) = value.to_string() {
                    Ok(string_val.len() <= *max_len)
                } else {
                    Ok(false)
                }
            }
            Self::MinValue(min_val) => {
                // Simplified validation - real implementation would handle all value types
                match (value, min_val) {
                    (FhirPathValue::Integer(val, _, _), JsonValue::Number(min)) => {
                        if let Some(min_i64) = min.as_i64() {
                            Ok(*val >= min_i64)
                        } else {
                            Ok(false)
                        }
                    }
                    (FhirPathValue::Decimal(val, _, _), JsonValue::Number(min)) => {
                        if let Some(min_f64) = min.as_f64() {
                            if let Ok(val_f64) = val.to_string().parse::<f64>() {
                                Ok(val_f64 >= min_f64)
                            } else {
                                Ok(false)
                            }
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            }
            Self::MaxValue(max_val) => {
                // Similar to MinValue but with opposite comparison
                match (value, max_val) {
                    (FhirPathValue::Integer(val, _, _), JsonValue::Number(max)) => {
                        if let Some(max_i64) = max.as_i64() {
                            Ok(*val <= max_i64)
                        } else {
                            Ok(false)
                        }
                    }
                    (FhirPathValue::Decimal(val, _, _), JsonValue::Number(max)) => {
                        if let Some(max_f64) = max.as_f64() {
                            if let Ok(val_f64) = val.to_string().parse::<f64>() {
                                Ok(val_f64 <= max_f64)
                            } else {
                                Ok(false)
                            }
                        } else {
                            Ok(false)
                        }
                    }
                    _ => Ok(false),
                }
            }
            Self::FixedValue(fixed_val) => {
                let value_json = value.to_json_value();
                Ok(value_json == *fixed_val)
            }
            Self::RequiredProperty(_prop_name) => {
                // Property requirement validation would be done at the object level
                Ok(true)
            }
            Self::ConditionalConstraint { constraint, .. } => {
                // Conditional constraints require evaluation context
                // For now, validate the inner constraint unconditionally
                constraint.validate(value)
            }
        }
    }

    /// Get human-readable description of this constraint
    pub fn description(&self) -> String {
        match self {
            Self::ValueSet(url) => format!("Must be a value from ValueSet: {url}"),
            Self::Pattern(pattern) => format!("Must match pattern: {pattern}"),
            Self::MinLength(min) => format!("Minimum length: {min}"),
            Self::MaxLength(max) => format!("Maximum length: {max}"),
            Self::MinValue(min) => format!("Minimum value: {min}"),
            Self::MaxValue(max) => format!("Maximum value: {max}"),
            Self::FixedValue(val) => format!("Fixed value: {val}"),
            Self::RequiredProperty(prop) => format!("Required property: {prop}"),
            Self::ConditionalConstraint {
                condition,
                constraint,
            } => {
                format!("When {}: {}", condition, constraint.description())
            }
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::core::model_provider::{ElementInfo, ModelProvider, TypeInfo};
    use octofhir_fhir_model::error::Result as ModelResult;

    /// Create a test model provider for unit tests
    pub fn create_test_model_provider() -> impl ModelProvider {
        TestModelProvider
    }

    #[derive(Debug)]
    struct TestModelProvider;

    #[async_trait::async_trait]
    impl ModelProvider for TestModelProvider {
        async fn get_type(&self, _type_name: &str) -> ModelResult<Option<TypeInfo>> {
            Ok(Some(TypeInfo {
                type_name: "TestType".to_string(),
                singleton: Some(true),
                namespace: Some("Test".to_string()),
                name: Some("TestType".to_string()),
                is_empty: Some(false),
            }))
        }

        async fn get_element_type(
            &self,
            _parent_type: &TypeInfo,
            _property_name: &str,
        ) -> ModelResult<Option<TypeInfo>> {
            Ok(Some(TypeInfo {
                type_name: "String".to_string(),
                singleton: Some(true),
                namespace: Some("System".to_string()),
                name: Some("String".to_string()),
                is_empty: Some(false),
            }))
        }

        fn of_type(&self, _type_info: &TypeInfo, _target_type: &str) -> Option<TypeInfo> {
            None
        }

        fn get_element_names(&self, _parent_type: &TypeInfo) -> Vec<String> {
            vec![]
        }

        async fn get_children_type(
            &self,
            _parent_type: &TypeInfo,
        ) -> ModelResult<Option<TypeInfo>> {
            Ok(None)
        }

        async fn get_elements(&self, _type_name: &str) -> ModelResult<Vec<ElementInfo>> {
            Ok(vec![])
        }

        async fn get_resource_types(&self) -> ModelResult<Vec<String>> {
            Ok(vec!["TestResource".to_string()])
        }

        async fn get_complex_types(&self) -> ModelResult<Vec<String>> {
            Ok(vec!["TestType".to_string()])
        }

        async fn get_primitive_types(&self) -> ModelResult<Vec<String>> {
            Ok(vec![
                "string".to_string(),
                "integer".to_string(),
                "boolean".to_string(),
            ])
        }
    }
}
