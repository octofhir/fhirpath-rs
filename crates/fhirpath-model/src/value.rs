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

//! Core value types for FHIRPath expressions

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use super::json_arc::ArcJsonValue;
use super::quantity::Quantity;
use super::resource::FhirResource;
use super::types::TypeInfo;

/// Core value type for FHIRPath expressions
///
/// This enum represents all possible values that can be produced by FHIRPath expressions.
/// All values in FHIRPath are conceptual collections, but single values are represented
/// directly for performance reasons.
#[derive(Clone, PartialEq)]
pub enum FhirPathValue {
    /// Boolean value
    Boolean(bool),

    /// Integer value (64-bit signed)
    Integer(i64),

    /// Decimal value with arbitrary precision
    Decimal(Decimal),

    /// String value
    String(Arc<str>),

    /// Date value (without time)
    Date(NaiveDate),

    /// DateTime value with timezone
    DateTime(DateTime<FixedOffset>),

    /// Time value (without date)
    Time(NaiveTime),

    /// Quantity value with optional unit
    Quantity(Arc<Quantity>),

    /// Collection of values (the fundamental FHIRPath concept)
    Collection(Collection),

    /// FHIR Resource or complex object
    Resource(Arc<FhirResource>),

    /// JSON value with copy-on-write semantics
    JsonValue(ArcJsonValue),

    /// Type information object with namespace and name properties
    TypeInfoObject {
        /// Type namespace
        namespace: Arc<str>,
        /// Type name
        name: Arc<str>,
    },

    /// Empty value (equivalent to an empty collection)
    Empty,
}

/// Custom deserialization for FhirPathValue to handle `Arc<str>`
impl<'de> Deserialize<'de> for FhirPathValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct FhirPathValueVisitor;

        impl<'de> Visitor<'de> for FhirPathValueVisitor {
            type Value = FhirPathValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a FhirPathValue")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FhirPathValue::Boolean(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FhirPathValue::Integer(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if let Ok(d) = Decimal::try_from(value) {
                    Ok(FhirPathValue::Decimal(d))
                } else {
                    Ok(FhirPathValue::String(Arc::from(value.to_string())))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FhirPathValue::String(Arc::from(value)))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FhirPathValue::String(Arc::from(value.as_str())))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }
                Ok(FhirPathValue::Collection(Collection::from_vec(vec)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut obj = serde_json::Map::new();
                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    obj.insert(key, value);
                }

                // Check for special object types
                if obj.contains_key("namespace") && obj.contains_key("name") {
                    if let (Some(namespace), Some(name)) = (
                        obj.get("namespace").and_then(|v| v.as_str()),
                        obj.get("name").and_then(|v| v.as_str()),
                    ) {
                        return Ok(FhirPathValue::TypeInfoObject {
                            namespace: Arc::from(namespace),
                            name: Arc::from(name),
                        });
                    }
                }

                // Otherwise treat as resource
                Ok(FhirPathValue::Resource(Arc::new(FhirResource::from_json(
                    serde_json::Value::Object(obj),
                ))))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FhirPathValue::Empty)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FhirPathValue::Empty)
            }
        }

        deserializer.deserialize_any(FhirPathValueVisitor)
    }
}

/// Collection type that wraps an Arc slice for zero-copy operations with CoW semantics
#[derive(Clone, PartialEq)]
pub struct Collection(Arc<[FhirPathValue]>);

impl Collection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self(Arc::from([]))
    }

    /// Create a collection from a vector
    pub fn from_vec(values: Vec<FhirPathValue>) -> Self {
        Self(values.into())
    }

    /// Create a collection from an iterator (more efficient than collect + from_vec)
    pub fn from_iter<I: IntoIterator<Item = FhirPathValue>>(iter: I) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>().into())
    }

    /// Create a collection by reserving capacity (prevents reallocations)
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity).into())
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

    /// Get a mutable vector for bulk operations - this clones the data (CoW)
    pub fn to_mut_vec(&mut self) -> Vec<FhirPathValue> {
        self.0.to_vec()
    }

    /// Replace the collection contents with a new vector
    pub fn replace_with_vec(&mut self, vec: Vec<FhirPathValue>) {
        self.0 = vec.into();
    }

    /// Push a value to the collection using CoW semantics
    pub fn push(&mut self, value: FhirPathValue) {
        self.push_impl(value);
    }

    /// Extend the collection with another using CoW semantics
    pub fn extend(&mut self, other: Collection) {
        // Optimize for common cases
        if self.is_empty() {
            // If self is empty, just replace with other
            self.0 = other.0;
            return;
        }
        if other.is_empty() {
            // If other is empty, nothing to do
            return;
        }

        let mut vec: Vec<FhirPathValue> = self.0.to_vec();
        vec.extend(other.0.iter().cloned());
        self.0 = vec.into();
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
        self.0.to_vec()
    }

    /// Check if the collection contains a value
    pub fn contains(&self, value: &FhirPathValue) -> bool {
        self.0.contains(value)
    }

    /// Get an element by index
    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        self.0.get(index)
    }

    /// Create a new collection from a slice without cloning (zero-copy)
    pub fn from_slice(slice: &[FhirPathValue]) -> Self
    where
        FhirPathValue: Clone,
    {
        Self(slice.to_vec().into())
    }

    /// Get a reference to the underlying Arc slice
    pub fn as_arc(&self) -> &Arc<[FhirPathValue]> {
        &self.0
    }

    /// Create a collection that shares data with this one (zero-copy clone)
    pub fn share(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    /// Check if we need to clone for mutation (CoW helper)
    #[allow(dead_code)]
    fn ensure_unique(&mut self) {
        if Arc::strong_count(&self.0) > 1 {
            // Multiple references exist - need to clone
            let vec: Vec<FhirPathValue> = self.0.to_vec();
            self.0 = vec.into();
        }
    }

    /// Push a value to the collection, handling CoW by creating new Arc if needed
    fn push_impl(&mut self, value: FhirPathValue) {
        let mut vec: Vec<FhirPathValue> = self.0.to_vec();
        vec.push(value);
        self.0 = vec.into();
    }

    /// Check if this collection has unique ownership (no other references)
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.0) == 1
    }

    /// Check if mutation is possible without cloning
    pub fn can_mutate_inplace(&self) -> bool {
        self.is_unique()
    }

    /// Clone the inner data if needed for mutation
    pub fn clone_for_mutation(&self) -> Vec<FhirPathValue> {
        self.0.to_vec()
    }

    /// Append a value, creating a new collection (preserves immutability)
    pub fn append(&self, value: FhirPathValue) -> Self {
        let mut vec = self.0.to_vec();
        vec.push(value);
        Self(vec.into())
    }

    /// Concatenate two collections efficiently
    pub fn concat(&self, other: &Collection) -> Self {
        if self.is_empty() {
            return other.share();
        }
        if other.is_empty() {
            return self.share();
        }
        let mut vec = self.0.to_vec();
        vec.extend(other.0.iter().cloned());
        Self(vec.into())
    }

    /// Filter the collection, creating a new one
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&FhirPathValue) -> bool,
    {
        let filtered: Vec<FhirPathValue> =
            self.0.iter().filter(|v| predicate(v)).cloned().collect();
        Self(filtered.into())
    }

    /// Map over the collection, creating a new one
    pub fn map<F>(&self, mapper: F) -> Self
    where
        F: Fn(&FhirPathValue) -> FhirPathValue,
    {
        let mapped: Vec<FhirPathValue> = self.0.iter().map(mapper).collect();
        Self(mapped.into())
    }

    /// Flatten a collection of collections
    pub fn flatten(&self) -> Self {
        let mut result = Vec::new();
        for value in self.0.iter() {
            match value {
                FhirPathValue::Collection(inner) => {
                    result.extend(inner.0.iter().cloned());
                }
                FhirPathValue::Empty => {}
                other => result.push(other.clone()),
            }
        }
        Self(result.into())
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<FhirPathValue>> for Collection {
    fn from(values: Vec<FhirPathValue>) -> Self {
        Self(values.into())
    }
}

impl IntoIterator for Collection {
    type Item = FhirPathValue;
    type IntoIter = std::vec::IntoIter<FhirPathValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned().collect::<Vec<_>>().into_iter()
    }
}

impl FhirPathValue {
    /// Create an empty collection
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Normalize collection result
    /// - Empty collections → `FhirPathValue::Empty`
    /// - Single-item collections → unwrapped single value
    /// - Multi-item collections → `FhirPathValue::Collection`
    pub fn normalize_collection_result(items: Vec<FhirPathValue>) -> FhirPathValue {
        if items.is_empty() {
            FhirPathValue::Empty
        } else if items.len() == 1 {
            items.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(Collection::from_vec(items))
        }
    }

    /// Create a collection from a vector of values
    pub fn collection(values: Vec<FhirPathValue>) -> Self {
        Self::Collection(Collection::from_vec(values))
    }

    /// Create a single-item collection
    pub fn singleton(value: FhirPathValue) -> Self {
        Self::Collection(Collection::from_vec(vec![value]))
    }

    /// Create a quantity value with optimization for common values
    pub fn quantity(value: Decimal, unit: Option<String>) -> Self {
        // Optimize common unitless values
        if unit.is_none() {
            use once_cell::sync::Lazy;
            use rust_decimal::Decimal;
            use std::collections::HashMap;
            use std::sync::Arc;

            static COMMON_QUANTITIES: Lazy<HashMap<String, Arc<Quantity>>> = Lazy::new(|| {
                let mut map = HashMap::new();
                map.insert("0".to_string(), Arc::new(Quantity::unitless(Decimal::ZERO)));
                map.insert("1".to_string(), Arc::new(Quantity::unitless(Decimal::ONE)));
                map.insert(
                    "-1".to_string(),
                    Arc::new(Quantity::unitless(-Decimal::ONE)),
                );
                map
            });

            let value_str = value.to_string();
            if let Some(cached) = COMMON_QUANTITIES.get(&value_str) {
                return Self::Quantity(Arc::clone(cached));
            }
        }

        Self::Quantity(Arc::new(Quantity::new(value, unit)))
    }

    /// Create an interned string value (more memory efficient for common strings)
    pub fn interned_string<S: AsRef<str>>(s: S) -> Self {
        use crate::string_intern::intern_string;
        Self::String(intern_string(s))
    }

    /// Create a resource value from JSON (Arc-wrapped for sharing)
    pub fn resource_from_json(data: Value) -> Self {
        Self::Resource(Arc::new(FhirResource::from_json(data)))
    }

    /// Create a resource value from an existing FhirResource (Arc-wrapped)
    pub fn resource(resource: FhirResource) -> Self {
        Self::Resource(Arc::new(resource))
    }

    /// Create a JSON value with CoW semantics
    pub fn json_value(value: Value) -> Self {
        Self::JsonValue(ArcJsonValue::new(value))
    }

    /// Create a JSON value from an Arc for zero-copy sharing
    pub fn json_value_from_arc(arc_json: ArcJsonValue) -> Self {
        Self::JsonValue(arc_json)
    }

    /// Check if the value is empty (empty collection or Empty variant)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Collection(items) => items.is_empty(),
            Self::JsonValue(json) => json.is_null(),
            _ => false,
        }
    }

    /// Check if the value is a single item (not a collection)
    pub fn is_single(&self) -> bool {
        match self {
            Self::Collection(items) => items.len() == 1,
            Self::Empty => false,
            Self::JsonValue(json) => !json.is_null(),
            _ => true,
        }
    }

    /// Get the length of a collection, or 1 for single values, 0 for empty
    pub fn len(&self) -> usize {
        match self {
            Self::Collection(items) => items.len(),
            Self::Empty => 0,
            Self::JsonValue(json) => {
                if json.is_null() {
                    0
                } else {
                    1
                }
            }
            _ => 1,
        }
    }

    /// Convert to a collection (wrapping single values)
    pub fn to_collection(self) -> Collection {
        match self {
            Self::Collection(items) => items,
            Self::Empty => Collection::new(),
            Self::JsonValue(json) if json.is_null() => Collection::new(),
            single => Collection::from_vec(vec![single]),
        }
    }

    /// Get the first item from a collection, or the value itself if single
    pub fn first(&self) -> Option<&FhirPathValue> {
        match self {
            Self::Collection(items) => items.first(),
            Self::Empty => None,
            Self::JsonValue(json) if json.is_null() => None,
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
            Self::JsonValue(json) => match json.as_json() {
                Value::Bool(b) => Some(*b),
                Value::Null => Some(false),
                Value::String(s) => Some(!s.is_empty()),
                Value::Number(n) => Some(n.as_f64().is_some_and(|f| f != 0.0)),
                Value::Array(arr) => Some(!arr.is_empty()),
                Value::Object(obj) => Some(!obj.is_empty()),
            },
            Self::Empty => Some(false),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string_value(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.as_ref().to_string()),
            Self::Boolean(b) => Some(b.to_string()),
            Self::Integer(i) => Some(i.to_string()),
            Self::Decimal(d) => Some(d.to_string()),
            Self::Date(d) => Some(d.format("%Y-%m-%d").to_string()),
            Self::DateTime(dt) => Some(dt.to_rfc3339()),
            Self::Time(t) => Some(t.format("%H:%M:%S").to_string()),
            Self::Quantity(q) => Some(q.to_string()),
            Self::JsonValue(json) => match json.as_json() {
                Value::String(s) => Some(s.clone()),
                Value::Bool(b) => Some(b.to_string()),
                Value::Number(n) => Some(n.to_string()),
                Value::Null => Some("".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Convert to quantity following FHIRPath rules
    pub fn to_quantity_value(&self) -> Option<Arc<Quantity>> {
        match self {
            // Already a quantity
            Self::Quantity(q) => Some(Arc::clone(q)),
            // Integer to quantity with unit '1' (dimensionless)
            Self::Integer(i) => Some(Arc::new(Quantity::new(
                Decimal::from(*i),
                Some("1".to_string()),
            ))),
            // Decimal to quantity with unit '1' (dimensionless)
            Self::Decimal(d) => Some(Arc::new(Quantity::new(*d, Some("1".to_string())))),
            // String parsing for quantities with units
            Self::String(s) => {
                // Try to parse as quantity with unit (e.g., "5 kg", "1.5 'm'")
                let s = s.trim();
                if s.is_empty() {
                    return None;
                }

                // Look for space to separate value from unit
                if let Some(space_pos) = s.find(' ') {
                    let (value_str, unit_str) = s.split_at(space_pos);
                    let unit_str = unit_str.trim();

                    // Parse the numeric value
                    if let Ok(decimal_val) = value_str.parse::<Decimal>() {
                        // Handle quoted units like 'wk', 'mo', 'a' and standard units
                        let unit = if unit_str.starts_with('\'') && unit_str.ends_with('\'') {
                            let unquoted = &unit_str[1..unit_str.len() - 1];
                            // Keep UCUM units as-is for proper comparison behavior
                            Some(unquoted.to_string())
                        } else if !unit_str.is_empty() {
                            // Only accept standard unquoted units, not UCUM abbreviations
                            match unit_str {
                                "day" | "week" | "month" | "year" | "kg" | "g" | "mg" | "m"
                                | "cm" | "mm" => Some(unit_str.to_string()),
                                // Reject unquoted UCUM abbreviations like "wk", "mo", "a"
                                "wk" | "mo" | "a" | "d" => return None,
                                _ => Some(unit_str.to_string()),
                            }
                        } else {
                            None
                        };

                        return Some(Arc::new(Quantity::new(decimal_val, unit)));
                    }
                } else {
                    // Try parsing as a simple number (quantity with unit '1')
                    if let Ok(decimal_val) = s.parse::<Decimal>() {
                        return Some(Arc::new(Quantity::new(decimal_val, Some("1".to_string()))));
                    }
                }
                None
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
            Self::Quantity(_) => "Quantity",
            Self::Collection(_) => "Collection",
            Self::Resource(_) => "Resource",
            Self::JsonValue(_) => "JsonValue",
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
            Self::JsonValue(_) => TypeInfo::Any,          // JsonValue can be any type
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
            Self::String(s) => Some(s.as_ref()),
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
                | (JsonValue(_), JsonValue(_))
                | (TypeInfoObject { .. }, TypeInfoObject { .. })
        )
    }

    /// Get a shared reference to JSON data with CoW semantics
    pub fn as_json_cow(&self) -> Option<&ArcJsonValue> {
        match self {
            Self::JsonValue(json) => Some(json),
            _ => None,
        }
    }

    /// Clone JSON data for mutation (CoW operation)
    pub fn clone_json_for_mutation(&self) -> Option<Value> {
        match self {
            Self::JsonValue(json) => Some(json.clone_inner()),
            _ => None,
        }
    }

    /// Try to get JSON property with zero-copy access
    pub fn get_json_property(&self, key: &str) -> Option<FhirPathValue> {
        match self {
            Self::JsonValue(json) => json.get_property(key).map(Self::JsonValue),
            _ => None,
        }
    }

    /// Try to get JSON array element with zero-copy access
    pub fn get_json_index(&self, index: usize) -> Option<FhirPathValue> {
        match self {
            Self::JsonValue(json) => json.get_index(index).map(Self::JsonValue),
            _ => None,
        }
    }

    /// Check if this value shares memory with another (useful for CoW optimization)
    pub fn shares_memory_with(&self, other: &FhirPathValue) -> bool {
        match (self, other) {
            (Self::Collection(c1), Self::Collection(c2)) => Arc::ptr_eq(c1.as_arc(), c2.as_arc()),
            (Self::JsonValue(j1), Self::JsonValue(j2)) => Arc::ptr_eq(j1.as_arc(), j2.as_arc()),
            (Self::Resource(r1), Self::Resource(r2)) => Arc::ptr_eq(r1, r2),
            (Self::Quantity(q1), Self::Quantity(q2)) => Arc::ptr_eq(q1, q2),
            _ => false,
        }
    }

    /// FHIRPath-specific equality checking (separate from PartialEq)
    ///
    /// This method implements FHIRPath equality rules that include:
    /// - Numeric type coercion (Integer vs Decimal)
    /// - Collection comparison with element-wise equality
    /// - Proper handling of Empty values
    /// - Mixed single value vs single-item collection comparison
    pub fn fhirpath_equals(&self, other: &FhirPathValue) -> bool {
        use rust_decimal::Decimal;

        match (self, other) {
            // Boolean equality
            (Self::Boolean(a), Self::Boolean(b)) => a == b,

            // String equality
            (Self::String(a), Self::String(b)) => a == b,

            // Numeric equality with type coercion
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Decimal(a), Self::Decimal(b)) => a == b,
            (Self::Integer(a), Self::Decimal(b)) => Decimal::from(*a) == *b,
            (Self::Decimal(a), Self::Integer(b)) => *a == Decimal::from(*b),

            // Date/Time equality
            (Self::Date(a), Self::Date(b)) => a == b,
            (Self::DateTime(a), Self::DateTime(b)) => a == b,
            (Self::Time(a), Self::Time(b)) => a == b,

            // Quantity equality (with unit compatibility)
            (Self::Quantity(a), Self::Quantity(b)) => a.value == b.value && a.unit == b.unit,

            // Collection equality (element-wise)
            (Self::Collection(a), Self::Collection(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.fhirpath_equals(y))
            }

            // Empty equality
            (Self::Empty, Self::Empty) => true,

            // Mixed single value vs single-item collection
            (Self::Collection(coll), single) => {
                coll.len() == 1
                    && coll
                        .first()
                        .is_some_and(|item| item.fhirpath_equals(single))
            }
            (single, Self::Collection(coll)) => {
                coll.len() == 1
                    && coll
                        .first()
                        .is_some_and(|item| single.fhirpath_equals(item))
            }

            // JsonValue equality
            (Self::JsonValue(a), Self::JsonValue(b)) => a.as_json() == b.as_json(),

            // Resource equality (compare JSON representations)
            (Self::Resource(a), Self::Resource(b)) => a.to_json() == b.to_json(),

            // TypeInfo equality
            (
                Self::TypeInfoObject {
                    namespace: ns1,
                    name: n1,
                },
                Self::TypeInfoObject {
                    namespace: ns2,
                    name: n2,
                },
            ) => ns1 == ns2 && n1 == n2,

            // All other combinations are not equal
            _ => false,
        }
    }

    /// Static version for convenience
    pub fn equals_static(left: &FhirPathValue, right: &FhirPathValue) -> bool {
        left.fhirpath_equals(right)
    }
}

/// Convert from serde_json::Value to FhirPathValue with CoW optimization
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
                        Self::String(n.to_string().into())
                    }
                } else {
                    Self::String(n.to_string().into())
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
                    Self::String(Arc::from(s.as_str()))
                }
            }
            Value::Array(arr) => {
                let items: Vec<FhirPathValue> = arr.into_iter().map(FhirPathValue::from).collect();
                // Always return Collection for arrays to preserve semantics
                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    FhirPathValue::Collection(Collection::from_vec(items))
                }
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
                                return Self::Quantity(Arc::new(Quantity::new(
                                    decimal_value,
                                    unit,
                                )));
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
                            namespace: Arc::from(namespace),
                            name: Arc::from(name),
                        };
                    }
                }

                // If this looks like a FHIR Resource (has resourceType), wrap as Resource
                if obj.get("resourceType").and_then(|v| v.as_str()).is_some() {
                    return Self::Resource(Arc::new(FhirResource::from_json(value)));
                }

                // For other complex JSON objects, use JsonValue with CoW semantics for sharing
                Self::JsonValue(ArcJsonValue::new(value))
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
            FhirPathValue::String(s) => Value::String(s.as_ref().to_string()),
            FhirPathValue::Date(d) => Value::String(format!("@{}", d.format("%Y-%m-%d"))),
            FhirPathValue::DateTime(dt) => {
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
                Value::String(format!("@{formatted}"))
            }
            FhirPathValue::Time(t) => Value::String(format!("@T{}", t.format("%H:%M:%S"))),
            FhirPathValue::Quantity(q) => q.to_json(),
            FhirPathValue::Collection(items) => {
                let json_items: Vec<Value> = items.into_iter().map(Value::from).collect();
                Value::Array(json_items)
            }
            FhirPathValue::Resource(resource) => resource.to_json(),
            FhirPathValue::JsonValue(arc_json) => arc_json.into_owned(),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "namespace".to_string(),
                    Value::String(namespace.as_ref().to_string()),
                );
                obj.insert("name".to_string(), Value::String(name.as_ref().to_string()));
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
            Self::String(s) => write!(f, "{}", s.as_ref()),
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
            Self::JsonValue(json) => write!(f, "{}", json.as_json()),
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
            Self::String(s) => write!(f, "String({})", s.as_ref()),
            Self::Boolean(b) => write!(f, "Boolean({b})"),
            Self::Integer(i) => write!(f, "Integer({i})"),
            Self::Decimal(d) => write!(f, "Decimal({d})"),
            Self::Date(d) => write!(f, "Date({})", d.format("%Y-%m-%d")),
            Self::DateTime(dt) => write!(f, "DateTime({})", dt.to_rfc3339()),
            Self::Time(t) => write!(f, "Time({})", t.format("%H:%M:%S")),
            Self::Quantity(q) => {
                // Use the same format as toString() for consistency
                let formatted_value = q.value.to_string();
                if let Some(unit) = &q.unit {
                    // Only quote UCUM units, leave standard units unquoted
                    match unit.as_str() {
                        "wk" | "mo" | "a" | "d" => write!(f, "{formatted_value} '{unit}'"),
                        _ => write!(f, "{formatted_value} {unit}"),
                    }
                } else {
                    write!(f, "{formatted_value}")
                }
            }
            Self::Collection(items) => {
                // Show the collection contents without nested Collection wrapper
                let item_strings: Vec<String> =
                    items.iter().map(|item| format!("{item:?}")).collect();
                write!(f, "Collection([{}])", item_strings.join(", "))
            }
            Self::Resource(resource) => write!(f, "Resource({})", resource.to_json()),
            Self::JsonValue(json) => write!(f, "JsonValue({:?})", json.as_json()),
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

/// Custom serialization for Collection to handle Arc
impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Custom deserialization for Collection to handle Arc
impl<'de> Deserialize<'de> for Collection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<FhirPathValue>::deserialize(deserializer)?;
        Ok(Self(vec.into()))
    }
}

/// A reference wrapper for FhirPathValue that enables zero-copy operations
///
/// ValueRef uses Cow (Clone-on-Write) semantics to avoid unnecessary cloning
/// when working with FhirPathValue instances. It can hold either a borrowed
/// reference or an owned value, converting to owned only when necessary.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueRef<'a> {
    value: Cow<'a, FhirPathValue>,
}

impl<'a> ValueRef<'a> {
    /// Create a ValueRef from a borrowed FhirPathValue
    pub fn borrowed(value: &'a FhirPathValue) -> Self {
        Self {
            value: Cow::Borrowed(value),
        }
    }

    /// Create a ValueRef from an owned FhirPathValue
    pub fn owned(value: FhirPathValue) -> Self {
        Self {
            value: Cow::Owned(value),
        }
    }

    /// Get a reference to the inner value
    pub fn as_ref(&self) -> &FhirPathValue {
        &self.value
    }

    /// Convert to an owned FhirPathValue
    pub fn into_owned(self) -> FhirPathValue {
        self.value.into_owned()
    }

    /// Check if this ValueRef owns its value
    pub fn is_owned(&self) -> bool {
        matches!(self.value, Cow::Owned(_))
    }

    /// Check if this ValueRef borrows its value
    pub fn is_borrowed(&self) -> bool {
        matches!(self.value, Cow::Borrowed(_))
    }

    /// Convert to owned if borrowed, no-op if already owned
    pub fn to_mut(&mut self) -> &mut FhirPathValue {
        self.value.to_mut()
    }

    /// Map the value, creating a new owned ValueRef
    pub fn map<F>(self, f: F) -> ValueRef<'a>
    where
        F: FnOnce(FhirPathValue) -> FhirPathValue,
    {
        ValueRef::owned(f(self.into_owned()))
    }

    /// Try to get a borrowed string value
    pub fn as_string(&self) -> Option<&str> {
        self.value.as_string()
    }

    /// Try to get an integer value
    pub fn as_integer(&self) -> Option<i64> {
        self.value.as_integer()
    }

    /// Try to get a boolean value
    pub fn as_boolean(&self) -> Option<bool> {
        self.value.as_boolean()
    }

    /// Check if the value is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Get the type name
    pub fn type_name(&self) -> &'static str {
        self.value.type_name()
    }
}

impl<'a> From<&'a FhirPathValue> for ValueRef<'a> {
    fn from(value: &'a FhirPathValue) -> Self {
        Self::borrowed(value)
    }
}

impl<'a> From<FhirPathValue> for ValueRef<'a> {
    fn from(value: FhirPathValue) -> Self {
        Self::owned(value)
    }
}

impl<'a> fmt::Display for ValueRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

// Convenience From implementations for string types
impl From<String> for FhirPathValue {
    fn from(s: String) -> Self {
        Self::String(Arc::from(s.as_str()))
    }
}

impl From<&str> for FhirPathValue {
    fn from(s: &str) -> Self {
        Self::String(Arc::from(s))
    }
}

impl From<Arc<str>> for FhirPathValue {
    fn from(s: Arc<str>) -> Self {
        Self::String(s)
    }
}

impl From<FhirResource> for FhirPathValue {
    fn from(resource: FhirResource) -> Self {
        Self::Resource(Arc::new(resource))
    }
}

impl From<Arc<FhirResource>> for FhirPathValue {
    fn from(resource: Arc<FhirResource>) -> Self {
        Self::Resource(resource)
    }
}

impl From<Quantity> for FhirPathValue {
    fn from(quantity: Quantity) -> Self {
        Self::Quantity(Arc::new(quantity))
    }
}

impl From<Arc<Quantity>> for FhirPathValue {
    fn from(quantity: Arc<Quantity>) -> Self {
        Self::Quantity(quantity)
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
            FhirPathValue::JsonValue(_) => {
                // Expected
            }
            _ => panic!("Expected JsonValue variant"),
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

    #[test]
    fn test_value_ref_borrowed() {
        let value = FhirPathValue::Integer(42);
        let value_ref = ValueRef::borrowed(&value);

        assert!(value_ref.is_borrowed());
        assert!(!value_ref.is_owned());
        assert_eq!(value_ref.as_integer(), Some(42));
        assert_eq!(value_ref.type_name(), "Integer");
    }

    #[test]
    fn test_value_ref_owned() {
        let value = FhirPathValue::String(Arc::from("hello"));
        let value_ref = ValueRef::owned(value);

        assert!(!value_ref.is_borrowed());
        assert!(value_ref.is_owned());
        assert_eq!(value_ref.as_string(), Some("hello"));
    }

    #[test]
    fn test_value_ref_map() {
        let value = FhirPathValue::Integer(10);
        let value_ref = ValueRef::borrowed(&value);

        let mapped = value_ref.map(|v| {
            if let FhirPathValue::Integer(i) = v {
                FhirPathValue::Integer(i * 2)
            } else {
                v
            }
        });

        assert!(mapped.is_owned());
        assert_eq!(mapped.as_integer(), Some(20));
    }

    #[test]
    fn test_collection_zero_copy() {
        let items = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let collection1 = Collection::from_vec(items);
        let collection2 = collection1.share();

        // Both collections should point to the same Arc
        assert!(Arc::ptr_eq(collection1.as_arc(), collection2.as_arc()));
    }

    #[test]
    fn test_resource_sharing() {
        use serde_json::json;

        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123",
            "name": [{
                "given": ["John"],
                "family": "Doe"
            }]
        });

        // Test resource sharing with Arc
        let resource = FhirResource::from_json(patient_json.clone());
        let shared_arc = Arc::new(resource);

        let value1 = FhirPathValue::from(Arc::clone(&shared_arc));
        let value2 = FhirPathValue::from(Arc::clone(&shared_arc));
        let value3 = FhirPathValue::from(Arc::clone(&shared_arc));

        if let (
            FhirPathValue::Resource(arc1),
            FhirPathValue::Resource(arc2),
            FhirPathValue::Resource(arc3),
        ) = (&value1, &value2, &value3)
        {
            // All three should point to the same Arc
            assert!(Arc::ptr_eq(arc1, arc2));
            assert!(Arc::ptr_eq(arc2, arc3));
            assert_eq!(Arc::strong_count(arc1), 4); // shared_arc + 3 values
        }

        // Test convenience constructors
        let resource2 = FhirResource::from_json(patient_json);
        let value_from_resource = FhirPathValue::resource(resource2.clone());
        let value_from_into = FhirPathValue::from(resource2);

        // Both should be valid resource values
        assert!(matches!(value_from_resource, FhirPathValue::Resource(_)));
        assert!(matches!(value_from_into, FhirPathValue::Resource(_)));

        // Test access still works with Arc
        if let FhirPathValue::Resource(arc_resource) = &value1 {
            assert_eq!(arc_resource.resource_type(), Some("Patient"));
            assert!(arc_resource.has_property("name"));
        }
    }

    #[test]
    fn test_quantity_sharing() {
        use rust_decimal::Decimal;

        // Test common quantity optimization
        let zero1 = FhirPathValue::quantity(Decimal::ZERO, None);
        let zero2 = FhirPathValue::quantity(Decimal::ZERO, None);
        let one1 = FhirPathValue::quantity(Decimal::ONE, None);
        let one2 = FhirPathValue::quantity(Decimal::ONE, None);
        let neg_one1 = FhirPathValue::quantity(-Decimal::ONE, None);
        let neg_one2 = FhirPathValue::quantity(-Decimal::ONE, None);

        if let (FhirPathValue::Quantity(arc1), FhirPathValue::Quantity(arc2)) = (&zero1, &zero2) {
            // Common quantities should share the same Arc
            assert!(Arc::ptr_eq(arc1, arc2));
        }

        if let (FhirPathValue::Quantity(arc1), FhirPathValue::Quantity(arc2)) = (&one1, &one2) {
            assert!(Arc::ptr_eq(arc1, arc2));
        }

        if let (FhirPathValue::Quantity(arc1), FhirPathValue::Quantity(arc2)) =
            (&neg_one1, &neg_one2)
        {
            assert!(Arc::ptr_eq(arc1, arc2));
        }

        // Test quantities with units don't use shared optimization
        let meter1 = FhirPathValue::quantity(Decimal::ONE, Some("m".to_string()));
        let meter2 = FhirPathValue::quantity(Decimal::ONE, Some("m".to_string()));

        if let (FhirPathValue::Quantity(arc1), FhirPathValue::Quantity(arc2)) = (&meter1, &meter2) {
            // Different Arc instances for quantities with units
            assert!(!Arc::ptr_eq(arc1, arc2));
        }

        // Test From implementations work correctly
        let q = Quantity::unitless(Decimal::from(42));
        let value_from_quantity = FhirPathValue::from(q.clone());
        let value_from_arc = FhirPathValue::from(Arc::new(q));

        assert!(matches!(value_from_quantity, FhirPathValue::Quantity(_)));
        assert!(matches!(value_from_arc, FhirPathValue::Quantity(_)));
    }

    #[test]
    fn test_typeinfo_object_arc_usage() {
        // Test TypeInfoObject creation and usage with Arc<str>
        let type_info1 = FhirPathValue::TypeInfoObject {
            namespace: Arc::from("FHIR"),
            name: Arc::from("Patient"),
        };

        let type_info2 = FhirPathValue::TypeInfoObject {
            namespace: Arc::from("FHIR"),
            name: Arc::from("Patient"),
        };

        // Test pattern matching works correctly
        if let FhirPathValue::TypeInfoObject { namespace, name } = &type_info1 {
            assert_eq!(namespace.as_ref(), "FHIR");
            assert_eq!(name.as_ref(), "Patient");
        }

        // Test JSON serialization/deserialization works with Arc<str>
        if let (
            FhirPathValue::TypeInfoObject {
                namespace: ns1,
                name: n1,
            },
            FhirPathValue::TypeInfoObject {
                namespace: ns2,
                name: n2,
            },
        ) = (&type_info1, &type_info2)
        {
            // Arc<str> comparison works correctly
            assert_eq!(ns1.as_ref(), ns2.as_ref());
            assert_eq!(n1.as_ref(), n2.as_ref());
        }

        // Test that .into() works for creating TypeInfoObject fields
        let type_info_from_str = FhirPathValue::TypeInfoObject {
            namespace: "System".into(),
            name: "String".into(),
        };

        if let FhirPathValue::TypeInfoObject { namespace, name } = type_info_from_str {
            assert_eq!(namespace.as_ref(), "System");
            assert_eq!(name.as_ref(), "String");
        }
    }

    #[test]
    fn test_fhirpath_equality_basics() {
        // Boolean equality
        assert!(FhirPathValue::Boolean(true).fhirpath_equals(&FhirPathValue::Boolean(true)));
        assert!(!FhirPathValue::Boolean(true).fhirpath_equals(&FhirPathValue::Boolean(false)));

        // String equality
        assert!(
            FhirPathValue::String(Arc::from("hello"))
                .fhirpath_equals(&FhirPathValue::String(Arc::from("hello")))
        );
        assert!(
            !FhirPathValue::String(Arc::from("hello"))
                .fhirpath_equals(&FhirPathValue::String(Arc::from("world")))
        );

        // Empty equality
        assert!(FhirPathValue::Empty.fhirpath_equals(&FhirPathValue::Empty));
        assert!(!FhirPathValue::Empty.fhirpath_equals(&FhirPathValue::Boolean(true)));
    }

    #[test]
    fn test_fhirpath_equality_numeric_coercion() {
        use rust_decimal::Decimal;

        // Integer equality
        assert!(FhirPathValue::Integer(42).fhirpath_equals(&FhirPathValue::Integer(42)));
        assert!(!FhirPathValue::Integer(42).fhirpath_equals(&FhirPathValue::Integer(43)));

        // Decimal equality
        let dec1 = FhirPathValue::Decimal(Decimal::from(42));
        let dec2 = FhirPathValue::Decimal(Decimal::from(42));
        assert!(dec1.fhirpath_equals(&dec2));

        // Cross-type numeric equality (Integer vs Decimal)
        let int_val = FhirPathValue::Integer(42);
        let dec_val = FhirPathValue::Decimal(Decimal::from(42));
        assert!(int_val.fhirpath_equals(&dec_val));
        assert!(dec_val.fhirpath_equals(&int_val));

        // Cross-type numeric inequality
        let int_val = FhirPathValue::Integer(42);
        let dec_val = FhirPathValue::Decimal(Decimal::from(43));
        assert!(!int_val.fhirpath_equals(&dec_val));
        assert!(!dec_val.fhirpath_equals(&int_val));
    }

    #[test]
    fn test_fhirpath_equality_collections() {
        // Collection vs Collection
        let coll1 =
            FhirPathValue::collection(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);
        let coll2 =
            FhirPathValue::collection(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);
        let coll3 =
            FhirPathValue::collection(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(3)]);

        assert!(coll1.fhirpath_equals(&coll2));
        assert!(!coll1.fhirpath_equals(&coll3));

        // Different length collections
        let coll_short = FhirPathValue::collection(vec![FhirPathValue::Integer(1)]);
        assert!(!coll1.fhirpath_equals(&coll_short));

        // Empty collections
        let empty1 = FhirPathValue::Empty;
        let empty2 = FhirPathValue::Empty;
        assert!(empty1.fhirpath_equals(&empty2));
        assert!(empty2.fhirpath_equals(&empty1));
    }

    #[test]
    fn test_fhirpath_equality_single_vs_collection() {
        // Single item vs single-item collection
        let single = FhirPathValue::Integer(42);
        let single_coll = FhirPathValue::collection(vec![FhirPathValue::Integer(42)]);

        assert!(single.fhirpath_equals(&single_coll));
        assert!(single_coll.fhirpath_equals(&single));

        // Single item vs multi-item collection
        let multi_coll =
            FhirPathValue::collection(vec![FhirPathValue::Integer(42), FhirPathValue::Integer(43)]);
        assert!(!single.fhirpath_equals(&multi_coll));
        assert!(!multi_coll.fhirpath_equals(&single));

        // Different single values vs single-item collections
        let different_single = FhirPathValue::Integer(43);
        assert!(!different_single.fhirpath_equals(&single_coll));
        assert!(!single_coll.fhirpath_equals(&different_single));
    }

    #[test]
    fn test_fhirpath_equality_complex_collections() {
        use rust_decimal::Decimal;

        // Collections with mixed types that coerce
        let coll1 = FhirPathValue::collection(vec![
            FhirPathValue::Integer(42),
            FhirPathValue::String(Arc::from("hello")),
        ]);
        let coll2 = FhirPathValue::collection(vec![
            FhirPathValue::Decimal(Decimal::from(42)), // Should equal Integer(42)
            FhirPathValue::String(Arc::from("hello")),
        ]);

        assert!(coll1.fhirpath_equals(&coll2));

        // Nested collections
        let nested1 = FhirPathValue::collection(vec![
            FhirPathValue::collection(vec![FhirPathValue::Integer(1)]),
            FhirPathValue::Integer(2),
        ]);
        let nested2 = FhirPathValue::collection(vec![
            FhirPathValue::collection(vec![FhirPathValue::Integer(1)]),
            FhirPathValue::Integer(2),
        ]);

        assert!(nested1.fhirpath_equals(&nested2));
    }

    #[test]
    fn test_fhirpath_equality_date_time() {
        use chrono::{DateTime, NaiveDate, NaiveTime};

        // Date equality
        let date1 = FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        let date2 = FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        let date3 = FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap());

        assert!(date1.fhirpath_equals(&date2));
        assert!(!date1.fhirpath_equals(&date3));

        // Time equality
        let time1 = FhirPathValue::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        let time2 = FhirPathValue::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        let time3 = FhirPathValue::Time(NaiveTime::from_hms_opt(13, 0, 0).unwrap());

        assert!(time1.fhirpath_equals(&time2));
        assert!(!time1.fhirpath_equals(&time3));

        // DateTime equality
        let dt1 = FhirPathValue::DateTime(
            DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
                .unwrap()
                .fixed_offset(),
        );
        let dt2 = FhirPathValue::DateTime(
            DateTime::parse_from_rfc3339("2023-01-01T12:00:00Z")
                .unwrap()
                .fixed_offset(),
        );

        assert!(dt1.fhirpath_equals(&dt2));
    }

    #[test]
    fn test_fhirpath_equality_quantities() {
        use crate::quantity::Quantity;
        use rust_decimal::Decimal;

        // Same quantities
        let q1 = FhirPathValue::Quantity(Arc::new(Quantity::new(
            Decimal::from(5),
            Some("kg".to_string()),
        )));
        let q2 = FhirPathValue::Quantity(Arc::new(Quantity::new(
            Decimal::from(5),
            Some("kg".to_string()),
        )));

        assert!(q1.fhirpath_equals(&q2));

        // Different values
        let q3 = FhirPathValue::Quantity(Arc::new(Quantity::new(
            Decimal::from(6),
            Some("kg".to_string()),
        )));

        assert!(!q1.fhirpath_equals(&q3));

        // Different units
        let q4 = FhirPathValue::Quantity(Arc::new(Quantity::new(
            Decimal::from(5),
            Some("g".to_string()),
        )));

        assert!(!q1.fhirpath_equals(&q4));
    }

    #[test]
    fn test_fhirpath_equality_type_info() {
        // TypeInfo equality
        let type1 = FhirPathValue::TypeInfoObject {
            namespace: Arc::from("FHIR"),
            name: Arc::from("Patient"),
        };
        let type2 = FhirPathValue::TypeInfoObject {
            namespace: Arc::from("FHIR"),
            name: Arc::from("Patient"),
        };
        let type3 = FhirPathValue::TypeInfoObject {
            namespace: Arc::from("FHIR"),
            name: Arc::from("Observation"),
        };

        assert!(type1.fhirpath_equals(&type2));
        assert!(!type1.fhirpath_equals(&type3));
    }

    #[test]
    fn test_fhirpath_equality_json_values() {
        use serde_json::json;

        // JSON value equality
        let json1 = FhirPathValue::json_value(json!({"name": "test", "value": 42}));
        let json2 = FhirPathValue::json_value(json!({"name": "test", "value": 42}));
        let json3 = FhirPathValue::json_value(json!({"name": "test", "value": 43}));

        assert!(json1.fhirpath_equals(&json2));
        assert!(!json1.fhirpath_equals(&json3));
    }

    #[test]
    fn test_fhirpath_equality_cross_type_negative() {
        // Different types should not be equal (except numeric coercion)
        assert!(
            !FhirPathValue::Boolean(true)
                .fhirpath_equals(&FhirPathValue::String(Arc::from("true")))
        );
        assert!(
            !FhirPathValue::Integer(42).fhirpath_equals(&FhirPathValue::String(Arc::from("42")))
        );
        assert!(!FhirPathValue::Empty.fhirpath_equals(&FhirPathValue::Boolean(false)));
    }

    #[test]
    fn test_equals_static_method() {
        let val1 = FhirPathValue::Integer(42);
        let val2 = FhirPathValue::Integer(42);
        let val3 = FhirPathValue::Integer(43);

        assert!(FhirPathValue::equals_static(&val1, &val2));
        assert!(!FhirPathValue::equals_static(&val1, &val3));
    }
}
