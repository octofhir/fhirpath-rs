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

//! Collection operations evaluator

use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use std::collections::HashSet;

/// Specialized evaluator for collection operations
pub struct CollectionEvaluator;

impl CollectionEvaluator {
    /// Helper to extract collections from values
    fn to_collection(value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.iter().cloned().collect(),
            FhirPathValue::Empty => vec![],
            other => vec![other.clone()],
        }
    }

    /// Helper to check if two values are equal using FHIRPath equality rules
    fn values_equal(left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use crate::evaluators::comparison::ComparisonEvaluator;

        match ComparisonEvaluator::compare_equal_with_collections(left, right) {
            Some(result) => result,
            None => false, // Empty comparisons are treated as false for collection operations
        }
    }

    /// Evaluate union operation (combines two collections, preserving duplicates)
    pub async fn evaluate_union(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let mut result_items = Vec::new();

        // Add all items from left collection
        result_items.extend(Self::to_collection(left));

        // Add all items from right collection
        result_items.extend(Self::to_collection(right));

        Ok(FhirPathValue::Collection(Collection::from(result_items)))
    }

    /// Evaluate contains operation (checks if collection contains an item)
    pub async fn evaluate_contains(
        collection: &FhirPathValue,
        item: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let collection_items = Self::to_collection(collection);
        let search_items = Self::to_collection(item);

        // According to FHIRPath spec: if search item is empty, return empty collection
        if search_items.is_empty() {
            return Ok(FhirPathValue::Collection(Default::default()));
        }

        // Contains returns true if ALL items from the search are found in the collection
        let all_found = search_items.iter().all(|search_item| {
            collection_items
                .iter()
                .any(|collection_item| Self::values_equal(collection_item, search_item))
        });

        Ok(FhirPathValue::Boolean(all_found))
    }

    /// Evaluate in operation (checks if item is in collection)
    pub async fn evaluate_in(
        item: &FhirPathValue,
        collection: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // "in" is essentially the reverse of "contains"
        Self::evaluate_contains(collection, item).await
    }

    /// Evaluate distinct operation (removes duplicates from collection)
    pub async fn evaluate_distinct(collection: &FhirPathValue) -> EvaluationResult<FhirPathValue> {
        let items = Self::to_collection(collection);
        let mut seen = HashSet::new();
        let mut distinct_items = Vec::new();

        for item in items {
            // Create a simple string representation for deduplication
            let key = Self::value_to_comparable_key(&item);
            if seen.insert(key) {
                distinct_items.push(item);
            }
        }

        Ok(FhirPathValue::Collection(Collection::from(distinct_items)))
    }

    /// Evaluate intersect operation (returns items that appear in both collections)
    pub async fn evaluate_intersect(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let left_items = Self::to_collection(left);
        let right_items = Self::to_collection(right);
        let mut intersection_items = Vec::new();

        for left_item in &left_items {
            // Check if this left item exists in right collection
            let found_in_right = right_items
                .iter()
                .any(|right_item| Self::values_equal(left_item, right_item));

            if found_in_right {
                // Only add if not already in intersection (avoid duplicates)
                let already_in_intersection = intersection_items
                    .iter()
                    .any(|existing| Self::values_equal(existing, left_item));

                if !already_in_intersection {
                    intersection_items.push(left_item.clone());
                }
            }
        }

        Ok(FhirPathValue::Collection(Collection::from(
            intersection_items,
        )))
    }

    // Helper method to create a comparable key for deduplication
    fn value_to_comparable_key(value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::Boolean(b) => format!("bool:{b}"),
            FhirPathValue::Integer(i) => format!("int:{i}"),
            FhirPathValue::Decimal(d) => format!("decimal:{d}"),
            FhirPathValue::String(s) => format!("string:{s}"),
            FhirPathValue::Date(d) => format!("date:{d}"),
            FhirPathValue::DateTime(dt) => format!("datetime:{dt}"),
            FhirPathValue::Time(t) => format!("time:{t}"),
            FhirPathValue::Quantity(q) => format!(
                "quantity:{}:{}",
                q.value,
                q.unit.as_ref().unwrap_or(&"".to_string())
            ),
            FhirPathValue::Empty => "empty".to_string(),
            FhirPathValue::Collection(items) => {
                let item_keys: Vec<String> =
                    items.iter().map(Self::value_to_comparable_key).collect();
                format!("collection:[{}]", item_keys.join(","))
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                format!("type:{namespace}.{name}")
            }
            _ => format!("other:{value:?}"),
        }
    }
}
