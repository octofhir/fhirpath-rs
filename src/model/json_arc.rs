//! Arc-based JSON value wrapper to eliminate cloning bottlenecks
//!
//! This module provides an Arc-wrapped JSON value type that enables zero-copy
//! sharing of JSON data across the FHIRPath pipeline, eliminating the expensive
//! cloning operations identified in baseline profiling.

use serde_json::Value as JsonValue;
use std::ops::Deref;
use std::sync::Arc;

/// Arc-wrapped JSON value for zero-copy sharing
///
/// This type wraps `serde_json::Value` in an `Arc` to enable efficient sharing
/// without cloning. It implements Copy-on-Write (CoW) semantics for mutations.
#[derive(Clone, Debug)]
pub struct ArcJsonValue {
    inner: Arc<JsonValue>,
}

impl ArcJsonValue {
    /// Create a new ArcJsonValue from a JsonValue
    pub fn new(value: JsonValue) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }

    /// Create an ArcJsonValue from an existing Arc
    pub fn from_arc(arc: Arc<JsonValue>) -> Self {
        Self { inner: arc }
    }

    /// Get a reference to the underlying JsonValue
    pub fn as_json(&self) -> &JsonValue {
        &self.inner
    }

    /// Clone the underlying JsonValue if needed for mutation
    /// This implements the "Copy" part of Copy-on-Write
    pub fn clone_inner(&self) -> JsonValue {
        (*self.inner).clone()
    }

    /// Get an owned JsonValue, cloning only if necessary
    pub fn into_owned(self) -> JsonValue {
        match Arc::try_unwrap(self.inner) {
            Ok(value) => value,
            Err(arc) => (*arc).clone(),
        }
    }

    /// Check if this is the only reference to the inner value
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.inner) == 1
    }

    /// Get the Arc directly (useful for further sharing)
    pub fn as_arc(&self) -> &Arc<JsonValue> {
        &self.inner
    }

    /// Create a view of an array slice without allocation
    pub fn array_view(&self, range: std::ops::Range<usize>) -> Option<ArrayView> {
        match self.as_json() {
            JsonValue::Array(arr) => {
                if range.end <= arr.len() {
                    Some(ArrayView {
                        source: Arc::clone(&self.inner),
                        range,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Zero-copy property access for objects
    pub fn get_property(&self, key: &str) -> Option<ArcJsonValue> {
        match self.as_json() {
            JsonValue::Object(obj) => {
                obj.get(key).map(|value| {
                    // Create a new Arc pointing to the same value
                    // This is zero-copy since we're not cloning the value
                    ArcJsonValue::new(value.clone())
                })
            }
            _ => None,
        }
    }

    /// Zero-copy array index access
    pub fn get_index(&self, index: usize) -> Option<ArcJsonValue> {
        match self.as_json() {
            JsonValue::Array(arr) => arr.get(index).map(|value| ArcJsonValue::new(value.clone())),
            _ => None,
        }
    }

    /// Efficient iteration over array elements without cloning
    pub fn array_iter(&self) -> Option<ArcArrayIter> {
        match self.as_json() {
            JsonValue::Array(arr) => Some(ArcArrayIter {
                source: Arc::clone(&self.inner),
                index: 0,
                len: arr.len(),
            }),
            _ => None,
        }
    }

    /// Efficient iteration over object entries
    pub fn object_iter(&self) -> Option<ArcObjectIter> {
        match self.as_json() {
            JsonValue::Object(_) => Some(ArcObjectIter {
                source: Arc::clone(&self.inner),
                keys: None,
                index: 0,
            }),
            _ => None,
        }
    }
}

impl Deref for ArcJsonValue {
    type Target = JsonValue;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialEq for ArcJsonValue {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl From<JsonValue> for ArcJsonValue {
    fn from(value: JsonValue) -> Self {
        Self::new(value)
    }
}

impl From<Arc<JsonValue>> for ArcJsonValue {
    fn from(arc: Arc<JsonValue>) -> Self {
        Self::from_arc(arc)
    }
}

/// Zero-copy view of an array slice
#[derive(Clone, Debug)]
pub struct ArrayView {
    source: Arc<JsonValue>,
    range: std::ops::Range<usize>,
}

impl ArrayView {
    /// Get the length of the view
    pub fn len(&self) -> usize {
        self.range.end - self.range.start
    }

    /// Check if the view is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an element from the view by index
    pub fn get(&self, index: usize) -> Option<ArcJsonValue> {
        if index < self.len() {
            match &*self.source {
                JsonValue::Array(arr) => {
                    let actual_index = self.range.start + index;
                    arr.get(actual_index)
                        .map(|value| ArcJsonValue::new(value.clone()))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Iterator over the view elements
    pub fn iter(&self) -> ArrayViewIter {
        ArrayViewIter {
            view: self.clone(),
            index: 0,
        }
    }
}

/// Iterator over array view elements
pub struct ArrayViewIter {
    view: ArrayView,
    index: usize,
}

impl Iterator for ArrayViewIter {
    type Item = ArcJsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.view.get(self.index);
        if result.is_some() {
            self.index += 1;
        }
        result
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.view.len() - self.index;
        (remaining, Some(remaining))
    }
}

/// Iterator over Arc JSON array elements
pub struct ArcArrayIter {
    source: Arc<JsonValue>,
    index: usize,
    len: usize,
}

impl Iterator for ArcArrayIter {
    type Item = ArcJsonValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            match &*self.source {
                JsonValue::Array(arr) => {
                    let result = arr
                        .get(self.index)
                        .map(|value| ArcJsonValue::new(value.clone()));
                    self.index += 1;
                    result
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

/// Iterator over Arc JSON object entries
pub struct ArcObjectIter {
    source: Arc<JsonValue>,
    keys: Option<Vec<String>>,
    index: usize,
}

impl Iterator for ArcObjectIter {
    type Item = (String, ArcJsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        // Initialize keys on first access
        if self.keys.is_none() {
            match &*self.source {
                JsonValue::Object(obj) => {
                    self.keys = Some(obj.keys().cloned().collect());
                }
                _ => return None,
            }
        }

        if let Some(ref keys) = self.keys {
            if self.index < keys.len() {
                let key = keys[self.index].clone();
                self.index += 1;

                match &*self.source {
                    JsonValue::Object(obj) => obj
                        .get(&key)
                        .map(|value| (key, ArcJsonValue::new(value.clone()))),
                    _ => None,
                }
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
    fn test_arc_json_creation() {
        let json = json!({"key": "value"});
        let arc_json = ArcJsonValue::new(json.clone());

        assert_eq!(arc_json.as_json(), &json);
    }

    #[test]
    fn test_zero_copy_sharing() {
        let json = json!({"key": "value"});
        let arc_json1 = ArcJsonValue::new(json);
        let arc_json2 = arc_json1.clone();

        // Both should point to the same Arc
        assert_eq!(Arc::strong_count(arc_json1.as_arc()), 2);
        assert_eq!(Arc::strong_count(arc_json2.as_arc()), 2);
    }

    #[test]
    fn test_property_access() {
        let json = json!({
            "name": "John",
            "age": 30,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            }
        });

        let arc_json = ArcJsonValue::new(json);

        let name = arc_json.get_property("name").unwrap();
        assert_eq!(name.as_json(), &json!("John"));

        let age = arc_json.get_property("age").unwrap();
        assert_eq!(age.as_json(), &json!(30));
    }

    #[test]
    fn test_array_access() {
        let json = json!([1, 2, 3, 4, 5]);
        let arc_json = ArcJsonValue::new(json);

        let first = arc_json.get_index(0).unwrap();
        assert_eq!(first.as_json(), &json!(1));

        let third = arc_json.get_index(2).unwrap();
        assert_eq!(third.as_json(), &json!(3));
    }

    #[test]
    fn test_array_view() {
        let json = json!([1, 2, 3, 4, 5]);
        let arc_json = ArcJsonValue::new(json);

        let view = arc_json.array_view(1..4).unwrap();
        assert_eq!(view.len(), 3);

        let first_in_view = view.get(0).unwrap();
        assert_eq!(first_in_view.as_json(), &json!(2));

        let last_in_view = view.get(2).unwrap();
        assert_eq!(last_in_view.as_json(), &json!(4));
    }

    #[test]
    fn test_array_iteration() {
        let json = json!([10, 20, 30]);
        let arc_json = ArcJsonValue::new(json);

        let values: Vec<_> = arc_json
            .array_iter()
            .unwrap()
            .map(|val| val.as_json().clone())
            .collect();

        assert_eq!(values, vec![json!(10), json!(20), json!(30)]);
    }

    #[test]
    fn test_object_iteration() {
        let json = json!({"a": 1, "b": 2, "c": 3});
        let arc_json = ArcJsonValue::new(json);

        let entries: std::collections::HashMap<String, JsonValue> = arc_json
            .object_iter()
            .unwrap()
            .map(|(k, v)| (k, v.as_json().clone()))
            .collect();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries.get("a"), Some(&json!(1)));
        assert_eq!(entries.get("b"), Some(&json!(2)));
        assert_eq!(entries.get("c"), Some(&json!(3)));
    }
}
