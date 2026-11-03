//! Structurally-shared FHIR JSON node.
//!
//! [`FhirNode`] mirrors `serde_json::Value` but stores every container child
//! behind an `Arc`, so cloning a node — and therefore navigating into a child,
//! collecting `descendants()`/`children()`, or copying a value through the
//! evaluator — is an O(1) pointer bump instead of a deep copy of the JSON
//! subtree. `serde_json::Value` owns its children inline (`Vec<Value>`,
//! `Map<String, Value>`), so navigation over it must deep-clone; that clone was
//! the dominant FHIRPath allocation cost during constraint validation.
//!
//! A resource is converted from `serde_json::Value` to `FhirNode` exactly once
//! (at evaluation entry); all subsequent navigation shares Arcs.
//!
//! The public accessor surface deliberately mirrors the subset of the
//! `serde_json::Value` API used across the evaluator (`get`, `as_str`,
//! `as_array`, `as_object`, `is_object`, …) so call sites migrate with minimal
//! churn.

use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// An immutable, structurally-shared JSON node. Cloning is O(1).
#[derive(Debug, Clone)]
pub enum FhirNode {
    Null,
    Bool(bool),
    /// Numbers supplied through `serde_json::Value`, and source integer tokens.
    Number(serde_json::Number),
    /// A source decimal, retaining its exact digits and scale without enabling
    /// serde_json's globally unified `arbitrary_precision` feature.
    Decimal(rust_decimal::Decimal, Arc<str>),
    Str(Arc<str>),
    Array(Arc<[FhirNode]>),
    /// Object entries preserve insertion order for serialization. Equality is
    /// independent of property order. Lookups are linear, which is optimal for the small key
    /// counts of real FHIR elements and avoids per-object hashing overhead.
    Object(Arc<[(Arc<str>, FhirNode)]>),
}

impl FhirNode {
    /// Parse JSON without passing decimal tokens through a binary float.
    ///
    /// Use this at the input boundary: digits already discarded by parsing into
    /// `serde_json::Value` cannot be reconstructed by [`Self::from_json`].
    pub fn parse_json(input: &str) -> serde_json::Result<Self> {
        use serde_json::value::RawValue;
        fn read(raw: &RawValue, depth: usize) -> serde_json::Result<FhirNode> {
            use serde::de::Error;
            if depth > 128 {
                return Err(serde_json::Error::custom("JSON nesting exceeds 128"));
            }
            let text = raw.get();
            Ok(match text.as_bytes().first() {
                Some(b'{') => {
                    let fields: indexmap::IndexMap<String, &RawValue> = serde_json::from_str(text)?;
                    FhirNode::Object(
                        fields
                            .into_iter()
                            .map(|(k, v)| Ok((Arc::from(k), read(v, depth + 1)?)))
                            .collect::<serde_json::Result<_>>()?,
                    )
                }
                Some(b'[') => {
                    let values: Vec<&RawValue> = serde_json::from_str(text)?;
                    FhirNode::Array(
                        values
                            .into_iter()
                            .map(|v| read(v, depth + 1))
                            .collect::<serde_json::Result<_>>()?,
                    )
                }
                Some(b'-' | b'0'..=b'9') => {
                    if !text.contains(['.', 'e', 'E']) {
                        if let Ok(value) = text.parse::<i64>() {
                            return Ok(FhirNode::Number(value.into()));
                        }
                        if let Ok(value) = text.parse::<u64>() {
                            return Ok(FhirNode::Number(value.into()));
                        }
                    }
                    let mantissa = text.split(['e', 'E']).next().unwrap_or(text);
                    let decimal = rust_decimal::Decimal::from_str_exact(mantissa)
                        .ok()
                        .and_then(|value| {
                            if text.contains(['e', 'E']) {
                                rust_decimal::Decimal::from_scientific(text).ok()
                            } else {
                                Some(value)
                            }
                        })
                        .ok_or_else(|| {
                            serde_json::Error::custom("JSON decimal exceeds supported precision")
                        })?;
                    FhirNode::Decimal(decimal, Arc::from(text))
                }
                _ => FhirNode::from_json(&serde_json::from_str(text)?),
            })
        }
        let raw: &RawValue = serde_json::from_str(input)?;
        read(raw, 0)
    }

    /// Navigate a JSON Pointer without materializing subtrees.
    pub fn pointer(&self, pointer: &str) -> Option<&Self> {
        if pointer.is_empty() {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let mut current = self;
        for part in pointer[1..].split('/') {
            let key = if part.contains('~') {
                std::borrow::Cow::Owned(part.replace("~1", "/").replace("~0", "~"))
            } else {
                std::borrow::Cow::Borrowed(part)
            };
            current = match current {
                Self::Array(items) => {
                    if key.starts_with('+') || (key.len() > 1 && key.starts_with('0')) {
                        return None;
                    }
                    items.get(key.parse::<usize>().ok()?)?
                }
                _ => current.get(&key)?,
            };
        }
        Some(current)
    }

    /// Convert a `serde_json::Value` into a shared node tree. This is the only
    /// deep walk; it happens once per resource at evaluation entry.
    pub fn from_json(value: &JsonValue) -> Self {
        match value {
            JsonValue::Null => FhirNode::Null,
            JsonValue::Bool(b) => FhirNode::Bool(*b),
            JsonValue::Number(n) => FhirNode::Number(n.clone()),
            JsonValue::String(s) => FhirNode::Str(Arc::from(s.as_str())),
            JsonValue::Array(arr) => FhirNode::Array(arr.iter().map(FhirNode::from_json).collect()),
            JsonValue::Object(map) => FhirNode::Object(
                map.iter()
                    .map(|(k, v)| (Arc::from(k.as_str()), FhirNode::from_json(v)))
                    .collect(),
            ),
        }
    }

    /// Materialize back into a `serde_json::Value` (deep). Used only where an
    /// owned serde value is genuinely required (output serialization fallbacks,
    /// interop); the hot navigation paths never call this.
    pub fn to_json(&self) -> JsonValue {
        match self {
            FhirNode::Null => JsonValue::Null,
            FhirNode::Bool(b) => JsonValue::Bool(*b),
            FhirNode::Number(n) => JsonValue::Number(n.clone()),
            FhirNode::Decimal(n, _) => {
                serde_json::from_str(&n.to_string()).expect("valid decimal JSON")
            }
            FhirNode::Str(s) => JsonValue::String(s.to_string()),
            FhirNode::Array(arr) => JsonValue::Array(arr.iter().map(FhirNode::to_json).collect()),
            FhirNode::Object(entries) => {
                let mut map = serde_json::Map::with_capacity(entries.len());
                for (k, v) in entries.iter() {
                    map.insert(k.to_string(), v.to_json());
                }
                JsonValue::Object(map)
            }
        }
    }

    /// Address of the shared allocation backing this node, for use as a
    /// memoization key on subtree traversals.
    ///
    /// Only container nodes have one — scalars are inline and indistinguishable
    /// by address. Two nodes with the same identity are the same subtree, *provided
    /// the allocation is still alive*, so any cache keyed on this MUST also keep a
    /// clone of the node alive: a dropped `Arc` can have its address reused by a
    /// later allocation, which would otherwise alias two unrelated subtrees.
    #[inline]
    pub fn identity(&self) -> Option<usize> {
        match self {
            FhirNode::Object(entries) => Some(Arc::as_ptr(entries) as *const u8 as usize),
            FhirNode::Array(items) => Some(Arc::as_ptr(items) as *const u8 as usize),
            _ => None,
        }
    }

    // --- serde_json::Value-compatible accessors ---

    /// Object field lookup by key (linear over ordered entries). O(1) on the
    /// returned node — no clone.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&FhirNode> {
        match self {
            FhirNode::Object(entries) => entries.iter().find(|(k, _)| &**k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FhirNode::Str(s) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FhirNode::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            FhirNode::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            FhirNode::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            FhirNode::Number(n) => n.as_f64(),
            FhirNode::Decimal(n, _) => rust_decimal::prelude::ToPrimitive::to_f64(n),
            _ => None,
        }
    }

    /// The decimal a number (or a numeric string) was written as, with its
    /// digits kept: `1.0` keeps its decimal place, `0.1` stays `0.1`.
    pub fn as_decimal(&self) -> Option<rust_decimal::Decimal> {
        match self {
            FhirNode::Number(n) => crate::core::value::utils::decimal_from_json_number(n),
            FhirNode::Decimal(n, _) => Some(*n),
            FhirNode::Str(s) => crate::core::value::utils::decimal_from_text(s),
            _ => None,
        }
    }

    /// Returns the object entries as an ordered slice. Replaces
    /// `serde_json::Value::as_object` — iterate as `(&str, &FhirNode)`.
    #[inline]
    pub fn as_object(&self) -> Option<&[(Arc<str>, FhirNode)]> {
        match self {
            FhirNode::Object(entries) => Some(entries),
            _ => None,
        }
    }

    #[inline]
    pub fn as_array(&self) -> Option<&[FhirNode]> {
        match self {
            FhirNode::Array(arr) => Some(arr),
            _ => None,
        }
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, FhirNode::Object(_))
    }

    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, FhirNode::Array(_))
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, FhirNode::Str(_))
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, FhirNode::Null)
    }

    /// Iterate object entries as `(&str, &FhirNode)`.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &FhirNode)> {
        let slice: &[(Arc<str>, FhirNode)] = match self {
            FhirNode::Object(entries) => entries,
            _ => &[],
        };
        slice.iter().map(|(k, v)| (&**k, v))
    }
}

/// Shared singleton returned for missing keys / out-of-range indexes, mirroring
/// `serde_json::Value`'s `Index` behaviour (returns Null rather than panicking).
static NULL_NODE: FhirNode = FhirNode::Null;

impl std::ops::Index<&str> for FhirNode {
    type Output = FhirNode;
    fn index(&self, key: &str) -> &FhirNode {
        self.get(key).unwrap_or(&NULL_NODE)
    }
}

impl std::ops::Index<usize> for FhirNode {
    type Output = FhirNode;
    fn index(&self, idx: usize) -> &FhirNode {
        match self {
            FhirNode::Array(arr) => arr.get(idx).unwrap_or(&NULL_NODE),
            _ => &NULL_NODE,
        }
    }
}

impl std::fmt::Display for FhirNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(s) => f.write_str(&s),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

impl From<&JsonValue> for FhirNode {
    fn from(v: &JsonValue) -> Self {
        FhirNode::from_json(v)
    }
}

impl From<JsonValue> for FhirNode {
    fn from(v: JsonValue) -> Self {
        FhirNode::from_json(&v)
    }
}

impl PartialEq for FhirNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FhirNode::Null, FhirNode::Null) => true,
            (FhirNode::Bool(a), FhirNode::Bool(b)) => a == b,
            (FhirNode::Number(a), FhirNode::Number(b)) => a == b,
            (FhirNode::Decimal(a, _), FhirNode::Decimal(b, _)) => a == b,
            (FhirNode::Number(a), FhirNode::Decimal(b, _))
            | (FhirNode::Decimal(b, _), FhirNode::Number(a)) => {
                a.is_f64()
                    && crate::core::value::utils::decimal_from_json_number(a)
                        .is_some_and(|a| a == *b)
            }
            (FhirNode::Str(a), FhirNode::Str(b)) => a == b,
            (FhirNode::Array(a), FhirNode::Array(b)) => {
                // Fast path: same Arc allocation.
                Arc::ptr_eq(a, b) || a == b
            }
            (FhirNode::Object(a), FhirNode::Object(b)) => {
                Arc::ptr_eq(a, b)
                    || (a.len() == b.len()
                        && a.iter().all(|(key, value)| {
                            b.iter()
                                .any(|(other_key, other)| key == other_key && value == other)
                        }))
            }
            _ => false,
        }
    }
}

impl Serialize for FhirNode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            FhirNode::Null => serializer.serialize_unit(),
            FhirNode::Bool(b) => serializer.serialize_bool(*b),
            FhirNode::Number(n) => n.serialize(serializer),
            FhirNode::Decimal(_, source) => {
                serde_json::value::RawValue::from_string(source.to_string())
                    .map_err(serde::ser::Error::custom)?
                    .serialize(serializer)
            }
            FhirNode::Str(s) => serializer.serialize_str(s),
            FhirNode::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr.iter() {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            FhirNode::Object(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (k, v) in entries.iter() {
                    map.serialize_entry(&**k, v)?;
                }
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip_and_share() {
        let v = json!({
            "resourceType": "Patient",
            "name": [{"family": "Doe", "given": ["John", "Q"]}],
            "active": true,
            "count": 3
        });
        let node = FhirNode::from_json(&v);
        // accessors
        assert_eq!(
            node.get("resourceType").and_then(FhirNode::as_str),
            Some("Patient")
        );
        assert_eq!(node.get("active").and_then(FhirNode::as_bool), Some(true));
        assert_eq!(node.get("count").and_then(FhirNode::as_i64), Some(3));
        let name = node.get("name").and_then(FhirNode::as_array).unwrap();
        assert_eq!(name.len(), 1);
        // O(1) clone shares the Arc
        let first = name[0].clone();
        if let (FhirNode::Object(a), FhirNode::Object(b)) = (&name[0], &first) {
            assert!(Arc::ptr_eq(a, b));
        } else {
            panic!("expected object");
        }
        // round-trip preserves structure + key order
        assert_eq!(node.to_json(), v);
    }
}
