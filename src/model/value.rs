//! Core value types for FHIRPath expressions

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

use super::quantity::Quantity;
use super::resource::FhirResource;
use super::types::TypeInfo;

/// Core value type for FHIRPath expressions
///
/// This enum represents all possible values that can be produced by FHIRPath expressions.
/// All values in FHIRPath are conceptual collections, but single values are represented
/// directly for performance reasons.
#[derive(Clone, PartialEq, Deserialize)]
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
    DateTime(DateTime<FixedOffset>),

    /// Time value (without date)
    Time(NaiveTime),

    /// Quantity value with optional unit
    Quantity(Quantity),

    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Collection),

    /// FHIR Resource or complex object
    Resource(FhirResource),

    /// Type information object with namespace and name properties
    TypeInfoObject {
        /// Type namespace
        namespace: String,
        /// Type name
        name: String,
    },

    /// Empty value (equivalent to an empty collection)
    Empty,
}

/// Collection type that wraps a vector of values
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Collection(Vec<FhirPathValue>);

impl Collection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create a collection from a vector
    pub fn from_vec(values: Vec<FhirPathValue>) -> Self {
        Self(values)
    }

    /// Get the length of the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get an iterator over the values
    pub fn iter(&self) -> std::slice::Iter<FhirPathValue> {
        self.0.iter()
    }

    /// Get a mutable iterator over the values
    pub fn iter_mut(&mut self) -> std::slice::IterMut<FhirPathValue> {
        self.0.iter_mut()
    }

    /// Push a value to the collection
    pub fn push(&mut self, value: FhirPathValue) {
        self.0.push(value);
    }

    /// Extend the collection with another
    pub fn extend(&mut self, other: Collection) {
        self.0.extend(other.0);
    }

    /// Get the first value
    pub fn first(&self) -> Option<&FhirPathValue> {
        self.0.first()
    }

    /// Get the last value
    pub fn last(&self) -> Option<&FhirPathValue> {
        self.0.last()
    }

    /// Take ownership of the inner vector
    pub fn into_vec(self) -> Vec<FhirPathValue> {
        self.0
    }

    /// Check if the collection contains a value
    pub fn contains(&self, value: &FhirPathValue) -> bool {
        self.0.contains(value)
    }

    /// Get an element by index
    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        self.0.get(index)
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<FhirPathValue>> for Collection {
    fn from(values: Vec<FhirPathValue>) -> Self {
        Self(values)
    }
}

impl IntoIterator for Collection {
    type Item = FhirPathValue;
    type IntoIter = std::vec::IntoIter<FhirPathValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FhirPathValue {
    /// Create an empty collection
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Create a collection from a vector of values
    pub fn collection(values: Vec<FhirPathValue>) -> Self {
        Self::Collection(Collection::from_vec(values))
    }

    /// Create a single-item collection
    pub fn singleton(value: FhirPathValue) -> Self {
        Self::Collection(Collection::from_vec(vec![value]))
    }

    /// Create a quantity value
    pub fn quantity(value: Decimal, unit: Option<String>) -> Self {
        Self::Quantity(Quantity::new(value, unit))
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
    pub fn to_collection(self) -> Collection {
        match self {
            Self::Collection(items) => items,
            Self::Empty => Collection::new(),
            single => Collection::from_vec(vec![single]),
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
            Self::Quantity(q) => Some(q.to_string()),
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
            Self::Quantity(_) => "Quantity",
            Self::Collection(_) => "Collection",
            Self::Resource(_) => "Resource",
            Self::TypeInfoObject { .. } => "TypeInfo",
            Self::Empty => "Empty",
        }
    }

    /// Get the TypeInfo for this value
    pub fn to_type_info(&self) -> TypeInfo {
        match self {
            Self::Boolean(_) => TypeInfo::Boolean,
            Self::Integer(_) => TypeInfo::Integer,
            Self::Decimal(_) => TypeInfo::Decimal,
            Self::String(_) => TypeInfo::String,
            Self::Date(_) => TypeInfo::Date,
            Self::DateTime(_) => TypeInfo::DateTime,
            Self::Time(_) => TypeInfo::Time,
            Self::Quantity(_) => TypeInfo::Quantity,
            Self::Collection(items) => {
                // For collections, we try to determine the element type
                if items.is_empty() {
                    TypeInfo::Collection(Box::new(TypeInfo::Any))
                } else {
                    // Use the type of the first element
                    TypeInfo::Collection(Box::new(items.first().unwrap().to_type_info()))
                }
            }
            Self::Resource(resource) => {
                TypeInfo::Resource(resource.resource_type().unwrap_or("Unknown").to_string())
            }
            Self::TypeInfoObject { .. } => TypeInfo::Any, // TypeInfo objects don't have a type themselves
            Self::Empty => TypeInfo::Any,
        }
    }

    /// Try to convert to an integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to convert to a decimal
    pub fn as_decimal(&self) -> Option<&Decimal> {
        match self {
            Self::Decimal(d) => Some(d),
            _ => None,
        }
    }

    /// Try to convert to a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to convert to a boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if two values have compatible types for comparison
    pub fn is_comparable_to(&self, other: &FhirPathValue) -> bool {
        use FhirPathValue::*;
        matches!(
            (self, other),
            (Boolean(_), Boolean(_))
                | (Integer(_), Integer(_))
                | (Integer(_), Decimal(_))
                | (Decimal(_), Integer(_))
                | (Decimal(_), Decimal(_))
                | (String(_), String(_))
                | (Date(_), Date(_))
                | (DateTime(_), DateTime(_))
                | (Time(_), Time(_))
                | (Quantity(_), Quantity(_))
                | (TypeInfoObject { .. }, TypeInfoObject { .. })
        )
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
                    if let Ok(d) = Decimal::try_from(f) {
                        Self::Decimal(d)
                    } else {
                        Self::String(n.to_string())
                    }
                } else {
                    Self::String(n.to_string())
                }
            }
            Value::String(s) => {
                // Try to parse as date/datetime/time first
                if let Ok(date) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                    Self::Date(date)
                } else if let Ok(datetime) = DateTime::parse_from_rfc3339(&s) {
                    Self::DateTime(datetime.fixed_offset())
                } else if let Ok(time) = NaiveTime::parse_from_str(&s, "%H:%M:%S") {
                    Self::Time(time)
                } else if let Ok(time) = NaiveTime::parse_from_str(&s, "%H:%M:%S%.f") {
                    Self::Time(time)
                } else {
                    Self::String(s)
                }
            }
            Value::Array(arr) => {
                let items: Vec<FhirPathValue> = arr.into_iter().map(FhirPathValue::from).collect();
                Self::Collection(Collection::from_vec(items))
            }
            Value::Object(ref obj) => {
                // Check if this looks like a Quantity
                if obj.contains_key("value")
                    && (obj.contains_key("unit") || obj.contains_key("code"))
                {
                    if let Some(value_json) = obj.get("value") {
                        if let Some(value_num) = value_json.as_f64() {
                            let unit = obj
                                .get("code")
                                .or_else(|| obj.get("unit"))
                                .and_then(|u| u.as_str())
                                .map(|s| s.to_string());

                            if let Ok(decimal_value) = Decimal::try_from(value_num) {
                                return Self::Quantity(Quantity::new(decimal_value, unit));
                            }
                        }
                    }
                }

                // Check if this looks like a TypeInfo object
                if obj.contains_key("namespace") && obj.contains_key("name") {
                    if let (Some(namespace), Some(name)) = (
                        obj.get("namespace").and_then(|v| v.as_str()),
                        obj.get("name").and_then(|v| v.as_str()),
                    ) {
                        return Self::TypeInfoObject {
                            namespace: namespace.to_string(),
                            name: name.to_string(),
                        };
                    }
                }

                // Otherwise treat as a resource
                Self::Resource(FhirResource::from_json(value))
            }
            Value::Null => Self::Empty,
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
            FhirPathValue::Date(d) => Value::String(format!("@{}", d.format("%Y-%m-%d"))),
            FhirPathValue::DateTime(dt) => {
                Value::String(format!("@{}", dt.format("%Y-%m-%dT%H:%M:%S%.3f%z")))
            }
            FhirPathValue::Time(t) => Value::String(format!("@T{}", t.format("%H:%M:%S"))),
            FhirPathValue::Quantity(q) => q.to_json(),
            FhirPathValue::Collection(items) => {
                let json_items: Vec<Value> = items.into_iter().map(Value::from).collect();
                Value::Array(json_items)
            }
            FhirPathValue::Resource(resource) => resource.to_json(),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                let mut obj = serde_json::Map::new();
                obj.insert("namespace".to_string(), Value::String(namespace));
                obj.insert("name".to_string(), Value::String(name));
                Value::Object(obj)
            }
            FhirPathValue::Empty => Value::Null,
        }
    }
}

/// Display implementation for FhirPathValue
impl fmt::Display for FhirPathValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Decimal(d) => write!(f, "{d}"),
            Self::Date(d) => write!(f, "@{}", d.format("%Y-%m-%d")),
            Self::DateTime(dt) => {
                let formatted = dt.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string();
                // Convert timezone format from +0000 to +00:00
                let formatted = if formatted.len() >= 5 {
                    let (main, tz) = formatted.split_at(formatted.len() - 5);
                    if tz.len() == 5 && (tz.starts_with('+') || tz.starts_with('-')) {
                        format!("{}{}:{}", main, &tz[..3], &tz[3..])
                    } else {
                        formatted
                    }
                } else {
                    formatted
                };
                write!(f, "@{formatted}")
            }
            Self::Time(t) => write!(f, "@T{}", t.format("%H:%M:%S")),
            Self::Quantity(q) => write!(f, "{q}"),
            Self::Collection(items) => {
                let item_strings: Vec<String> = items.iter().map(|item| item.to_string()).collect();
                write!(f, "[{}]", item_strings.join(", "))
            }
            Self::Resource(resource) => write!(f, "{}", resource.to_json()),
            Self::TypeInfoObject { namespace, name } => {
                write!(f, "TypeInfo({namespace}.{name})")
            }
            Self::Empty => write!(f, ""),
        }
    }
}

/// Custom serialization for FhirPathValue that uses the proper FHIRPath format
impl Serialize for FhirPathValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Convert to JSON Value using the existing From implementation
        // which correctly formats dates with @ prefix
        let json_value: serde_json::Value = self.clone().into();
        json_value.serialize(serializer)
    }
}

/// Debug implementation for FhirPathValue - uses cleaner format than derived Debug
impl fmt::Debug for FhirPathValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "String({s})"),
            Self::Boolean(b) => write!(f, "Boolean({b})"),
            Self::Integer(i) => write!(f, "Integer({i})"),
            Self::Decimal(d) => write!(f, "Decimal({d})"),
            Self::Date(d) => write!(f, "Date({})", d.format("%Y-%m-%d")),
            Self::DateTime(dt) => write!(f, "DateTime({})", dt.to_rfc3339()),
            Self::Time(t) => write!(f, "Time({})", t.format("%H:%M:%S")),
            Self::Quantity(q) => write!(f, "Quantity({q})"),
            Self::Collection(items) => {
                // Show the collection contents without nested Collection wrapper
                let item_strings: Vec<String> =
                    items.iter().map(|item| format!("{item:?}")).collect();
                write!(f, "Collection([{}])", item_strings.join(", "))
            }
            Self::Resource(resource) => write!(f, "Resource({})", resource.to_json()),
            Self::TypeInfoObject { namespace, name } => {
                write!(f, "TypeInfo({namespace}.{name})")
            }
            Self::Empty => write!(f, "Empty"),
        }
    }
}

/// Debug implementation for Collection
impl fmt::Debug for Collection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let item_strings: Vec<String> = self.0.iter().map(|item| format!("{item:?}")).collect();
        write!(f, "[{}]", item_strings.join(", "))
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
            FhirPathValue::Resource(_) => {
                // Expected
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
