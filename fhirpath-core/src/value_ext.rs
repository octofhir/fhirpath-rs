//! FHIRPath value types and data model
//!
//! This module defines the core value types used in FHIRPath expressions.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use octofhir_ucum_core::{self, OwnedUnitExpr};

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

    /// Date value (without time)
    Date(NaiveDate),

    /// DateTime value with timezone
    DateTime(DateTime<Utc>),

    /// Time value (without date)
    Time(NaiveTime),

    /// Quantity value with optional unit
    Quantity {
        value: Decimal,
        unit: Option<String>,
        /// Cached parsed UCUM unit expression for performance
        #[serde(skip)]
        #[serde(default)]
        ucum_expr: Option<Arc<OwnedUnitExpr>>,
    },

    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Vec<FhirPathValue>),

    /// FHIR Resource or complex object
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

    /// Get the JSON representation
    pub fn to_json(&self) -> &Value {
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
                    Value::Array(arr) => {
                        // For arrays, collect results from each element
                        // This is handled by the caller in evaluate_path_navigation
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
}

impl FhirPathValue {
    /// Create an empty collection
    pub fn empty() -> Self {
        Self::Collection(Vec::new())
    }

    /// Create a collection from a vector of values
    pub fn collection(values: Vec<FhirPathValue>) -> Self {
        Self::Collection(values)
    }

    /// Create a quantity with a value and optional unit
    pub fn quantity(value: Decimal, unit: Option<String>) -> Self {
        let ucum_expr = unit.as_ref().and_then(|u| Self::parse_ucum_unit(u));
        Self::Quantity {
            value,
            unit,
            ucum_expr,
        }
    }

    /// Parse a UCUM unit string into a OwnedUnitExpr with defensive programming and caching
    pub fn parse_ucum_unit(unit_str: &str) -> Option<Arc<OwnedUnitExpr>> {
        use std::collections::HashMap;
        use std::sync::{Mutex, OnceLock};

        // Static cache for parsed UCUM units
        static UCUM_CACHE: OnceLock<Mutex<HashMap<String, Option<Arc<OwnedUnitExpr>>>>> = OnceLock::new();

        // Early validation to prevent potential issues
        if unit_str.is_empty() {
            log::debug!("Empty unit string provided");
            return None;
        }

        // Limit unit string length to prevent potential DoS
        if unit_str.len() > 256 {
            log::debug!("Unit string too long: {}", unit_str.len());
            return None;
        }

        // Check for obviously invalid characters that could cause issues
        if unit_str.contains('\0') || unit_str.contains('\n') || unit_str.contains('\r') {
            log::debug!("Invalid characters in unit string: {}", unit_str);
            return None;
        }

        // Check cache first
        let cache = UCUM_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Ok(cache_guard) = cache.lock() {
            if let Some(cached_result) = cache_guard.get(unit_str) {
                log::debug!("Using cached UCUM unit: {}", unit_str);
                return cached_result.clone();
            }
        }

        // Use a timeout mechanism to prevent hanging
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let (tx, rx) = mpsc::channel();
        let unit_str_owned = unit_str.to_string();

        // Spawn a thread with timeout for UCUM parsing
        thread::spawn(move || {
            let result = octofhir_ucum_core::parse_expression(&unit_str_owned);
            let _ = tx.send(result);
        });

        // Wait for result with timeout
        let result = match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(expr)) => {
                log::debug!("Successfully parsed UCUM unit: {}", unit_str);
                Some(Arc::new(expr))
            },
            Ok(Err(e)) => {
                log::debug!("Failed to parse UCUM unit '{}': {}", unit_str, e);
                None
            }
            Err(_) => {
                log::warn!("UCUM parsing timed out for unit '{}'", unit_str);
                None
            }
        };

        // Cache the result (both success and failure)
        if let Ok(mut cache_guard) = cache.lock() {
            // Limit cache size to prevent memory issues
            if cache_guard.len() >= 1000 {
                cache_guard.clear();
                log::debug!("Cleared UCUM cache due to size limit");
            }
            cache_guard.insert(unit_str.to_string(), result.clone());
        }

        result
    }

    /// Check if two quantities have compatible dimensions
    pub fn has_compatible_dimensions(&self, other: &FhirPathValue) -> bool {
        match (self, other) {
            (Self::Quantity { unit: Some(unit1), .. }, Self::Quantity { unit: Some(unit2), .. }) => {
                // Use the UCUM library to check if units are comparable
                octofhir_ucum_core::is_comparable(unit1, unit2).unwrap_or(false)
            },
            (Self::Quantity { unit: None, .. }, Self::Quantity { unit: None, .. }) => {
                // Unitless quantities are comparable
                true
            },
            _ => false,
        }
    }

    /// Convert a quantity to a different unit
    pub fn convert_to_unit(&self, target_unit: &str) -> Result<Self, crate::error::FhirPathError> {
        match self {
            Self::Quantity { value, unit: Some(unit), ucum_expr: _ } => {
                // Validate the target unit
                match octofhir_ucum_core::validate(target_unit) {
                    Ok(_) => {
                        // Check if units are comparable
                        match octofhir_ucum_core::is_comparable(unit, target_unit) {
                            Ok(true) => {
                                // If units are the same, no conversion needed
                                if unit == target_unit {
                                    return Ok(Self::quantity(value.clone(), Some(target_unit.to_string())));
                                }

                                // For different units, we'll return an error for now to avoid freezing
                                // This is a temporary solution until we can implement proper UCUM conversion
                                Err(crate::error::FhirPathError::conversion_error(
                                    format!("Unit conversion not yet implemented"),
                                    format!("from {} to {}", unit, target_unit)
                                ))
                            },
                            Ok(false) => Err(crate::error::FhirPathError::conversion_error(
                                format!("Units are not comparable"),
                                format!("{} and {}", unit, target_unit)
                            )),
                            Err(e) => Err(crate::error::FhirPathError::conversion_error(
                                format!("Error checking unit compatibility"),
                                format!("{}: {}", unit, e)
                            )),
                        }
                    },
                    Err(e) => Err(crate::error::FhirPathError::conversion_error(
                        format!("Invalid target unit"),
                        format!("{}: {}", target_unit, e)
                    )),
                }
            },
            Self::Quantity { value, unit: None, .. } => {
                // Unitless quantity - can only convert to another unitless quantity
                if target_unit.is_empty() {
                    Ok(Self::quantity(value.clone(), None))
                } else {
                    Err(crate::error::FhirPathError::conversion_error(
                        format!("Cannot convert unitless quantity"),
                        format!("to {}", target_unit)
                    ))
                }
            },
            _ => Err(crate::error::FhirPathError::conversion_error(
                format!("Cannot convert non-quantity value"),
                format!("to unit {}", target_unit)
            )),
        }
    }

    /// Get the most granular unit between two quantities
    pub fn most_granular_unit(&self, other: &FhirPathValue) -> Option<String> {
        match (self, other) {
            (Self::Quantity { unit: Some(unit1), .. }, Self::Quantity { unit: Some(unit2), .. }) => {
                // For now, we'll use a simple heuristic: longer unit strings are likely more granular
                // In a more sophisticated implementation, we would use UCUM to determine granularity
                if unit1.len() >= unit2.len() {
                    Some(unit1.clone())
                } else {
                    Some(unit2.clone())
                }
            },
            (Self::Quantity { unit: Some(unit), .. }, _) | (_, Self::Quantity { unit: Some(unit), .. }) => {
                Some(unit.clone())
            },
            _ => None,
        }
    }

    /// Check if the value is empty (empty collection or Empty variant)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Collection(items) => items.is_empty(),
            _ => false,
        }
    }

    /// Check if the value is a single item (not a collection)
    pub fn is_single(&self) -> bool {
        match self {
            Self::Collection(items) => items.len() == 1,
            Self::Empty => false,
            _ => true,
        }
    }

    /// Get the length of a collection, or 1 for single values, 0 for empty
    pub fn len(&self) -> usize {
        match self {
            Self::Collection(items) => items.len(),
            Self::Empty => 0,
            _ => 1,
        }
    }

    /// Convert to a collection (wrapping single values)
    pub fn to_collection(self) -> Vec<FhirPathValue> {
        match self {
            Self::Collection(items) => items,
            Self::Empty => Vec::new(),
            single => vec![single],
        }
    }

    /// Get the first item from a collection, or the value itself if single
    pub fn first(&self) -> Option<&FhirPathValue> {
        match self {
            Self::Collection(items) => items.first(),
            Self::Empty => None,
            single => Some(single),
        }
    }

    /// Convert to boolean following FHIRPath rules
    pub fn to_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            Self::Integer(i) => Some(*i != 0),
            Self::Decimal(d) => Some(!d.is_zero()),
            Self::String(s) => Some(!s.is_empty()),
            Self::Collection(items) => Some(!items.is_empty()),
            Self::Empty => Some(false),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string_value(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            Self::Boolean(b) => Some(b.to_string()),
            Self::Integer(i) => Some(i.to_string()),
            Self::Decimal(d) => Some(d.to_string()),
            Self::Date(d) => Some(d.format("%Y-%m-%d").to_string()),
            Self::DateTime(dt) => Some(dt.to_rfc3339()),
            Self::Time(t) => Some(t.format("%H:%M:%S").to_string()),
            Self::Quantity { value, unit, .. } => {
                if let Some(unit) = unit {
                    Some(format!("{} {}", value, unit))
                } else {
                    Some(value.to_string())
                }
            }
            _ => None,
        }
    }

    /// Get the type name for this value
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
            Self::Collection(_) => "Collection",
            Self::Resource(_) => "Resource",
            Self::Empty => "Empty",
        }
    }
}

/// Convert from serde_json::Value to FhirPathValue
impl From<Value> for FhirPathValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => Self::Boolean(b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Self::Decimal(Decimal::try_from(f).unwrap_or_default())
                } else {
                    Self::String(n.to_string())
                }
            }
            Value::String(s) => {
                // Try to parse as date/datetime/time first
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                    Self::Date(date)
                } else if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&s) {
                    Self::DateTime(datetime.with_timezone(&chrono::Utc))
                } else if let Ok(time) = chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S") {
                    Self::Time(time)
                } else if let Ok(time) = chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S%.f") {
                    Self::Time(time)
                } else {
                    Self::String(s)
                }
            }
            Value::Array(arr) => {
                let items: Vec<FhirPathValue> = arr.into_iter().map(FhirPathValue::from).collect();
                Self::Collection(items)
            }
            Value::Object(ref obj) => {
                // Check if this looks like a Quantity
                if obj.contains_key("value") && (obj.contains_key("unit") || obj.contains_key("code")) {
                    if let Some(value_json) = obj.get("value") {
                        if let Some(value_num) = value_json.as_f64() {
                            let unit = obj.get("unit")
                                .or_else(|| obj.get("code"))
                                .and_then(|u| u.as_str())
                                .map(|s| s.to_string());

                            if let Ok(decimal_value) = Decimal::try_from(value_num) {
                                return Self::Quantity {
                                    value: decimal_value,
                                    unit: unit.clone(),
                                    ucum_expr: unit.as_ref().and_then(|u| Self::parse_ucum_unit(u)),
                                };
                            }
                        }
                    }
                }

                // Check if this is a FHIR resource (has resourceType)
                if obj.contains_key("resourceType") {
                    Self::Resource(FhirResource::new(value))
                } else {
                    // For other objects, wrap as Resource but without resourceType
                    Self::Resource(FhirResource::new(value))
                }
            }
            Value::Null => Self::Empty,
        }
    }
}

/// Display implementation for FhirPathValue
impl std::fmt::Display for FhirPathValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Integer(i) => write!(f, "{}", i),
            Self::Decimal(d) => write!(f, "{}", d),
            Self::Date(d) => write!(f, "{}", d.format("%Y-%m-%d")),
            Self::DateTime(dt) => write!(f, "{}", dt.to_rfc3339()),
            Self::Time(t) => write!(f, "{}", t.format("%H:%M:%S")),
            Self::Quantity { value, unit, .. } => {
                if let Some(unit) = unit {
                    write!(f, "{} {}", value, unit)
                } else {
                    write!(f, "{}", value)
                }
            }
            Self::Collection(items) => {
                let item_strings: Vec<String> = items.iter()
                    .map(|item| item.to_string())
                    .collect();
                write!(f, "[{}]", item_strings.join(", "))
            }
            Self::Resource(resource) => write!(f, "{}", resource.data),
            Self::Empty => write!(f, ""),
        }
    }
}

/// Convert from FhirPathValue to serde_json::Value
impl From<FhirPathValue> for Value {
    fn from(fhir_value: FhirPathValue) -> Self {
        match fhir_value {
            FhirPathValue::Boolean(b) => Value::Bool(b),
            FhirPathValue::Integer(i) => Value::Number(i.into()),
            FhirPathValue::Decimal(d) => {
                // Convert decimal to JSON number - may lose precision
                if let Ok(f) = d.try_into() {
                    if let Some(num) = serde_json::Number::from_f64(f) {
                        Value::Number(num)
                    } else {
                        Value::String(d.to_string())
                    }
                } else {
                    Value::String(d.to_string())
                }
            }
            FhirPathValue::String(s) => Value::String(s),
            FhirPathValue::Date(d) => Value::String(d.format("%Y-%m-%d").to_string()),
            FhirPathValue::DateTime(dt) => Value::String(dt.to_rfc3339()),
            FhirPathValue::Time(t) => Value::String(t.format("%H:%M:%S").to_string()),
            FhirPathValue::Quantity { value, unit, .. } => {
                let mut obj = serde_json::Map::new();
                // Convert decimal to JSON value using the same logic as above
                let value_json = if let Ok(f) = value.try_into() {
                    if let Some(num) = serde_json::Number::from_f64(f) {
                        Value::Number(num)
                    } else {
                        Value::String(value.to_string())
                    }
                } else {
                    Value::String(value.to_string())
                };
                obj.insert("value".to_string(), value_json);
                if let Some(unit) = unit {
                    obj.insert("unit".to_string(), Value::String(unit));
                }
                Value::Object(obj)
            }
            FhirPathValue::Collection(items) => {
                let json_items: Vec<Value> = items.into_iter().map(Value::from).collect();
                Value::Array(json_items)
            }
            FhirPathValue::Resource(resource) => resource.data,
            FhirPathValue::Empty => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fhirpath_value_creation() {
        let bool_val = FhirPathValue::Boolean(true);
        assert_eq!(bool_val.type_name(), "Boolean");
        assert!(!bool_val.is_empty());
        assert!(bool_val.is_single());

        let empty_val = FhirPathValue::empty();
        assert!(empty_val.is_empty());
        assert!(!empty_val.is_single());
    }

    #[test]
    fn test_json_conversion() {
        let json_val = serde_json::json!({"name": "test", "value": 42});
        let fhir_val = FhirPathValue::from(json_val.clone());

        match fhir_val {
            FhirPathValue::Resource(resource) => {
                assert_eq!(resource.to_json(), &json_val);
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    #[test]
    fn test_collection_operations() {
        let items = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let collection = FhirPathValue::collection(items);

        assert_eq!(collection.len(), 3);
        assert!(!collection.is_empty());
        assert!(!collection.is_single());

        if let Some(first) = collection.first() {
            assert_eq!(*first, FhirPathValue::Integer(1));
        }
    }
}
