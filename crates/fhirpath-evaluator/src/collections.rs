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

//! Collection operation optimizations for FHIRPath evaluation
//!
//! This module provides optimized collection operations that reduce memory allocations
//! and improve performance during FHIRPath expression evaluation.

use octofhir_fhirpath_model::value::{Collection, FhirPathValue};
use octofhir_fhirpath_model::value_pool::{get_pooled_collection_vec, return_pooled_collection_vec};

/// Trait for providing size hints to optimize collection operations
pub trait SizeHint {
    /// Estimate the size of the resulting collection
    fn size_hint(&self) -> (usize, Option<usize>);

    /// Get an upper bound estimate for collection sizing
    fn upper_bound_hint(&self) -> usize {
        match self.size_hint() {
            (_, Some(upper)) => upper,
            (lower, None) => lower.max(16), // Reasonable default
        }
    }
}

impl SizeHint for Collection {
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl SizeHint for &[FhirPathValue] {
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T: SizeHint> SizeHint for &T {
    fn size_hint(&self) -> (usize, Option<usize>) {
        (*self).size_hint()
    }
}

/// Optimized collection builder that pre-allocates based on hints
pub struct OptimizedCollectionBuilder {
    inner: Vec<FhirPathValue>,
    use_pooling: bool,
}

impl OptimizedCollectionBuilder {
    /// Create a new builder with capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            use_pooling: false,
        }
    }

    /// Create a new builder using pooled memory
    pub fn with_pooled_capacity(capacity: usize) -> Self {
        let mut vec = get_pooled_collection_vec();
        vec.reserve(capacity);
        Self {
            inner: vec,
            use_pooling: true,
        }
    }

    /// Create a builder from a size hint
    pub fn from_hint<H: SizeHint>(hint: &H) -> Self {
        let capacity = hint.upper_bound_hint();
        if capacity <= 1024 {
            // Use pooling for reasonable sizes
            Self::with_pooled_capacity(capacity)
        } else {
            Self::with_capacity(capacity)
        }
    }

    /// Push a value to the collection
    pub fn push(&mut self, value: FhirPathValue) {
        self.inner.push(value);
    }

    /// Extend with another collection
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = FhirPathValue>,
    {
        self.inner.extend(iter);
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Get current length
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Build the final collection
    pub fn build(self) -> Collection {
        if self.use_pooling {
            // Return to pool after use - Collection will own the data

            Collection::from_vec(self.inner)
        } else {
            Collection::from_vec(self.inner)
        }
    }

    /// Build and explicitly return vector to pool if pooled
    pub fn build_with_pool_return(mut self) -> Collection {
        let collection = Collection::from_vec(self.inner.clone());
        if self.use_pooling {
            self.inner.clear();
            return_pooled_collection_vec(self.inner);
        }
        collection
    }
}

/// Optimized filtering operations
pub struct FilterOps;

impl FilterOps {
    /// Filter a collection in-place when possible, otherwise create optimized new collection
    pub fn filter_optimized<F>(collection: &Collection, predicate: F) -> Collection
    where
        F: Fn(&FhirPathValue) -> bool,
    {
        // Estimate result size for pre-allocation
        let estimated_size = (collection.len() / 2).max(1); // Conservative estimate
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(estimated_size);

        for value in collection.iter() {
            if predicate(value) {
                builder.push(value.clone());
            }
        }

        builder.build()
    }

    /// Filter and map in one pass for efficiency
    pub fn filter_map_optimized<F, T>(collection: &Collection, f: F) -> Collection
    where
        F: Fn(&FhirPathValue) -> Option<T>,
        T: Into<FhirPathValue>,
    {
        let estimated_size = (collection.len() / 2).max(1);
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(estimated_size);

        for value in collection.iter() {
            if let Some(mapped) = f(value) {
                builder.push(mapped.into());
            }
        }

        builder.build()
    }

    /// Select (map) operation with pre-allocation
    pub fn select_optimized<F>(collection: &Collection, mapper: F) -> Collection
    where
        F: Fn(&FhirPathValue) -> FhirPathValue,
    {
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(collection.len());

        for value in collection.iter() {
            builder.push(mapper(value));
        }

        builder.build()
    }

    /// Flatten collections efficiently
    pub fn flatten_optimized(collection: &Collection) -> Collection {
        // Estimate size by counting nested collections
        let estimated_size = Self::estimate_flattened_size(collection);
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(estimated_size);

        for value in collection.iter() {
            match value {
                FhirPathValue::Collection(inner) => {
                    builder.extend(inner.iter().cloned());
                }
                FhirPathValue::Empty => {
                    // Skip empty values
                }
                other => {
                    builder.push(other.clone());
                }
            }
        }

        builder.build()
    }

    /// Estimate the size after flattening
    fn estimate_flattened_size(collection: &Collection) -> usize {
        let mut size = 0;
        for value in collection.iter() {
            match value {
                FhirPathValue::Collection(inner) => {
                    size += inner.len();
                }
                FhirPathValue::Empty => {
                    // No contribution to size
                }
                _ => {
                    size += 1;
                }
            }
        }
        size.max(1)
    }
}

/// Specialized iterator for Bundle.entry traversal optimization
pub struct BundleEntryIterator<'a> {
    bundle: &'a FhirPathValue,
    index: usize,
}

impl<'a> BundleEntryIterator<'a> {
    /// Create a new Bundle.entry iterator
    pub fn new(bundle: &'a FhirPathValue) -> Option<Self> {
        // Check if this looks like a Bundle resource
        match bundle {
            FhirPathValue::Resource(resource) => {
                if resource.resource_type() == Some("Bundle") {
                    Some(Self { bundle, index: 0 })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Fast-path check if bundle has entries
    pub fn has_entries(&self) -> bool {
        match self.bundle {
            FhirPathValue::Resource(resource) => resource.has_property("entry"),
            _ => false,
        }
    }

    /// Get estimated entry count for pre-allocation
    pub fn estimated_entry_count(&self) -> usize {
        match self.bundle {
            FhirPathValue::Resource(resource) => {
                // Try to get the entry array and count it
                if let Some(entry_value) = resource.get_property("entry") {
                    match entry_value {
                        serde_json::Value::Array(entries) => entries.len(),
                        serde_json::Value::Null => 0,
                        _ => 1, // Single entry
                    }
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

impl<'a> Iterator for BundleEntryIterator<'a> {
    type Item = FhirPathValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.bundle {
            FhirPathValue::Resource(resource) => {
                if let Some(entry_value) = resource.get_property("entry") {
                    match entry_value {
                        serde_json::Value::Array(entries) => {
                            if self.index < entries.len() {
                                let result = entries
                                    .get(self.index)
                                    .map(|v| FhirPathValue::from(v.clone()));
                                self.index += 1;
                                result
                            } else {
                                None
                            }
                        }
                        serde_json::Value::Null => None,
                        single if self.index == 0 => {
                            self.index += 1;
                            Some(FhirPathValue::from(single.clone()))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.estimated_entry_count().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> SizeHint for BundleEntryIterator<'a> {
    fn size_hint(&self) -> (usize, Option<usize>) {
        Iterator::size_hint(self)
    }
}

/// Collection operation utilities
pub struct CollectionUtils;

impl CollectionUtils {
    /// Combine multiple collections efficiently
    pub fn combine_optimized(collections: &[Collection]) -> Collection {
        if collections.is_empty() {
            return Collection::new();
        }

        if collections.len() == 1 {
            return collections[0].share();
        }

        // Calculate total size for pre-allocation
        let total_size: usize = collections.iter().map(|c| c.len()).sum();
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(total_size);

        for collection in collections {
            builder.extend(collection.iter().cloned());
        }

        builder.build()
    }

    /// Union operation with deduplication
    pub fn union_optimized(left: &Collection, right: &Collection) -> Collection {
        let estimated_size = left.len() + right.len();
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(estimated_size);
        let mut seen = std::collections::HashSet::new();

        // Add items from left collection
        for value in left.iter() {
            let hash = Self::value_hash(value);
            if seen.insert(hash) {
                builder.push(value.clone());
            }
        }

        // Add unique items from right collection
        for value in right.iter() {
            let hash = Self::value_hash(value);
            if seen.insert(hash) {
                builder.push(value.clone());
            }
        }

        builder.build()
    }

    /// Create a simple hash for FhirPathValue (for deduplication)
    fn value_hash(value: &FhirPathValue) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash based on type and key properties
        match value {
            FhirPathValue::Boolean(b) => {
                "bool".hash(&mut hasher);
                b.hash(&mut hasher);
            }
            FhirPathValue::Integer(i) => {
                "int".hash(&mut hasher);
                i.hash(&mut hasher);
            }
            FhirPathValue::String(s) => {
                "str".hash(&mut hasher);
                s.as_ref().hash(&mut hasher);
            }
            FhirPathValue::Empty => {
                "empty".hash(&mut hasher);
            }
            _ => {
                // For complex types, use display representation
                "complex".hash(&mut hasher);
                format!("{value}").hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    /// Skip operation with optimized allocation
    pub fn skip_optimized(collection: &Collection, count: usize) -> Collection {
        if count >= collection.len() {
            return Collection::new();
        }

        let remaining = collection.len() - count;
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(remaining);

        for (index, value) in collection.iter().enumerate() {
            if index >= count {
                builder.push(value.clone());
            }
        }

        builder.build()
    }

    /// Take operation with optimized allocation
    pub fn take_optimized(collection: &Collection, count: usize) -> Collection {
        let take_count = count.min(collection.len());
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(take_count);

        for (index, value) in collection.iter().enumerate() {
            if index >= take_count {
                break;
            }
            builder.push(value.clone());
        }

        builder.build()
    }

    /// Distinct operation with optimized deduplication
    pub fn distinct_optimized(collection: &Collection) -> Collection {
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(collection.len());
        let mut seen = std::collections::HashSet::new();

        for value in collection.iter() {
            let hash = Self::value_hash(value);
            if seen.insert(hash) {
                builder.push(value.clone());
            }
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::value::FhirPathValue;

    #[test]
    fn test_optimized_collection_builder() {
        let mut builder = OptimizedCollectionBuilder::with_capacity(3);
        builder.push(FhirPathValue::Integer(1));
        builder.push(FhirPathValue::Integer(2));
        builder.push(FhirPathValue::Integer(3));

        let collection = builder.build();
        assert_eq!(collection.len(), 3);
    }

    #[test]
    fn test_pooled_collection_builder() {
        let mut builder = OptimizedCollectionBuilder::with_pooled_capacity(2);
        builder.push(FhirPathValue::Boolean(true));
        builder.push(FhirPathValue::Boolean(false));

        let collection = builder.build();
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn test_filter_optimized() {
        let items = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ];
        let collection = Collection::from_vec(items);

        let filtered = FilterOps::filter_optimized(
            &collection,
            |v| matches!(v, FhirPathValue::Integer(i) if *i % 2 == 0),
        );

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_flatten_optimized() {
        let nested = vec![
            FhirPathValue::Collection(Collection::from_vec(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
            ])),
            FhirPathValue::Integer(3),
            FhirPathValue::Collection(Collection::from_vec(vec![FhirPathValue::Integer(4)])),
        ];
        let collection = Collection::from_vec(nested);

        let flattened = FilterOps::flatten_optimized(&collection);
        assert_eq!(flattened.len(), 4);
    }

    #[test]
    fn test_combine_optimized() {
        let col1 = Collection::from_vec(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);
        let col2 = Collection::from_vec(vec![FhirPathValue::Integer(3), FhirPathValue::Integer(4)]);
        let collections = vec![col1, col2];

        let combined = CollectionUtils::combine_optimized(&collections);
        assert_eq!(combined.len(), 4);
    }

    #[test]
    fn test_union_optimized() {
        let col1 = Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1), // Duplicate
        ]);
        let col2 = Collection::from_vec(vec![
            FhirPathValue::Integer(2), // Duplicate
            FhirPathValue::Integer(3),
        ]);

        let union = CollectionUtils::union_optimized(&col1, &col2);
        assert_eq!(union.len(), 3); // 1, 2, 3
    }

    #[test]
    fn test_skip_take_optimized() {
        let items = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
            FhirPathValue::Integer(5),
        ];
        let collection = Collection::from_vec(items);

        let skipped = CollectionUtils::skip_optimized(&collection, 2);
        assert_eq!(skipped.len(), 3);

        let taken = CollectionUtils::take_optimized(&collection, 3);
        assert_eq!(taken.len(), 3);
    }

    #[test]
    fn test_distinct_optimized() {
        let items = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1), // Duplicate
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2), // Duplicate
        ];
        let collection = Collection::from_vec(items);

        let distinct = CollectionUtils::distinct_optimized(&collection);
        assert_eq!(distinct.len(), 3); // 1, 2, 3
    }

    #[test]
    fn test_size_hint_trait() {
        let collection =
            Collection::from_vec(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);

        let hint = collection.size_hint();
        assert_eq!(hint, (2, Some(2)));
        assert_eq!(collection.upper_bound_hint(), 2);
    }
}
