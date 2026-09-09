//! Stable set operations with the same equality relation as the '=' operator.
//! Hashes only narrow candidates; collisions always receive a full comparison.
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::LazyLock;

use super::operations::EqualsOperatorEvaluator;
use crate::core::{FhirPathValue, node::FhirNode};
use smallvec::SmallVec;

static EQUALITY: LazyLock<EqualsOperatorEvaluator> = LazyLock::new(EqualsOperatorEvaluator::new);

#[derive(Default)]
pub struct ValueSet {
    values: Vec<FhirPathValue>,
    buckets: HashMap<u64, SmallVec<[usize; 1]>>,
    fallback: Vec<usize>,
}

impl ValueSet {
    pub(crate) fn insert(&mut self, value: FhirPathValue) -> bool {
        let mut hasher = self.buckets.hasher().build_hasher();
        let hashable = hash_value(&value, &mut hasher);
        let key = hasher.finish();
        let equal =
            |index: &usize| EQUALITY.compare_values(&self.values[*index], &value) == Some(true);
        let duplicate = if hashable {
            self.buckets
                .get(&key)
                .is_some_and(|indices| indices.iter().any(&equal))
                || self.fallback.iter().any(&equal)
        } else {
            self.values
                .iter()
                .any(|existing| EQUALITY.compare_values(existing, &value) == Some(true))
        };
        if duplicate {
            return false;
        }
        let index = self.values.len();
        self.values.push(value);
        if hashable {
            self.buckets.entry(key).or_default().push(index);
        } else {
            self.fallback.push(index);
        }
        true
    }

    pub(crate) fn into_values(self) -> Vec<FhirPathValue> {
        self.values
    }
}

pub fn distinct(values: impl IntoIterator<Item = FhirPathValue>) -> Vec<FhirPathValue> {
    let mut set = ValueSet::default();
    for value in values {
        set.insert(value);
    }
    set.into_values()
}

fn hash_value(value: &FhirPathValue, h: &mut impl Hasher) -> bool {
    match value {
        FhirPathValue::Integer(value, _, _) => {
            0u8.hash(h);
            rust_decimal::Decimal::from(*value).normalize().hash(h);
        }
        FhirPathValue::Decimal(value, _, _) => {
            0u8.hash(h);
            value.normalize().hash(h);
        }
        FhirPathValue::Boolean(value, _, _) => {
            1u8.hash(h);
            value.hash(h);
        }
        FhirPathValue::String(value, _, _) => {
            2u8.hash(h);
            value.hash(h);
        }
        FhirPathValue::Resource(node, info, _) if info.type_name != "Quantity" => {
            3u8.hash(h);
            hash_node(node, h);
        }
        // Temporal cross-type/precision comparisons and UCUM conversions do
        // not have a safe cheap canonical hash. Preserve '=' via fallback.
        _ => return false,
    }
    true
}

fn hash_node(node: &FhirNode, h: &mut impl Hasher) {
    std::mem::discriminant(node).hash(h);
    match node {
        FhirNode::Null => {}
        FhirNode::Bool(v) => v.hash(h),
        FhirNode::Number(v) => v.hash(h),
        FhirNode::Str(v) => v.hash(h),
        FhirNode::Array(values) => {
            values.len().hash(h);
            for value in values.iter() {
                hash_node(value, h);
            }
        }
        FhirNode::Object(values) => {
            values.len().hash(h);
            for (key, value) in values.iter() {
                key.hash(h);
                hash_node(value, h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{test_context, test_engine};

    #[tokio::test]
    async fn set_functions_use_equality_not_debug_or_string_coercion() {
        let engine = test_engine().await;
        let context = test_context();
        for (expression, expected) in [
            ("(1 | 1.0).count()", serde_json::json!(1)),
            ("(1 | '1').count()", serde_json::json!(2)),
            ("(1.combine(1.0)).distinct().count()", serde_json::json!(1)),
            ("(1.combine('1')).isDistinct()", serde_json::json!(true)),
            (
                "(1.00000000001 | 1.00000000002).count()",
                serde_json::json!(2),
            ),
            ("(1 'm' | 100 'cm').count()", serde_json::json!(1)),
        ] {
            assert_eq!(
                engine
                    .evaluate(expression, &context)
                    .await
                    .unwrap()
                    .value
                    .to_json_value(),
                expected,
                "{expression}"
            );
        }
    }

    #[test]
    fn deduplicates_complex_values_and_keeps_first_order() {
        let a = FhirPathValue::resource(serde_json::json!({"resourceType":"Patient","id":"a"}));
        let b = FhirPathValue::resource(serde_json::json!({"resourceType":"Patient","id":"b"}));
        assert_eq!(distinct([a.clone(), b.clone(), a.clone()]), vec![a, b]);
    }
}
