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

use crate::{FhirPathValue, value::Collection};
use std::ops::Range;
use std::sync::Arc;

/// Smart collection that can efficiently share data or own it depending on usage patterns
#[derive(Debug, Clone)]
pub enum SmartCollection {
    /// Shared reference-counted array of values
    Shared(Arc<[FhirPathValue]>),
    /// Owned vector of values that can be modified
    Owned(Vec<FhirPathValue>),
    /// View into a shared array with a specific range
    View {
        /// The shared base array being viewed
        base: Arc<[FhirPathValue]>,
        /// The range of indices to view from the base array
        range: Range<usize>,
    },
}

impl SmartCollection {
    /// Creates an empty smart collection
    pub fn empty() -> Self {
        Self::Owned(Vec::new())
    }

    /// Creates a smart collection from an owned vector
    pub fn from_vec(values: Vec<FhirPathValue>) -> Self {
        Self::Owned(values)
    }

    /// Creates a smart collection from a shared array
    pub fn from_shared(values: Arc<[FhirPathValue]>) -> Self {
        Self::Shared(values)
    }

    /// Creates a smart collection containing a single value
    pub fn from_single(value: FhirPathValue) -> Self {
        Self::Owned(vec![value])
    }

    /// Creates a view into a shared array with the given range
    pub fn view(base: Arc<[FhirPathValue]>, range: Range<usize>) -> Self {
        if range.start >= base.len() || range.end > base.len() || range.start >= range.end {
            Self::Owned(Vec::new())
        } else {
            Self::View { base, range }
        }
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        match self {
            Self::Shared(arr) => arr.len(),
            Self::Owned(vec) => vec.len(),
            Self::View { range, .. } => range.len(),
        }
    }

    /// Returns true if the collection contains no elements
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets a reference to the element at the given index
    pub fn get(&self, index: usize) -> Option<&FhirPathValue> {
        match self {
            Self::Shared(arr) => arr.get(index),
            Self::Owned(vec) => vec.get(index),
            Self::View { base, range } => {
                if index < range.len() {
                    base.get(range.start + index)
                } else {
                    None
                }
            }
        }
    }

    /// Returns an iterator over the elements of the collection
    pub fn iter(&self) -> SmartCollectionIter {
        match self {
            Self::Shared(arr) => SmartCollectionIter::Shared {
                slice: arr,
                index: 0,
            },
            Self::Owned(vec) => SmartCollectionIter::Owned {
                slice: vec.as_slice(),
                index: 0,
            },
            Self::View { base, range } => SmartCollectionIter::View {
                slice: base,
                range: range.clone(),
                index: 0,
            },
        }
    }

    /// Converts the collection to an owned vector
    pub fn to_vec(&self) -> Vec<FhirPathValue> {
        match self {
            Self::Shared(arr) => arr.to_vec(),
            Self::Owned(vec) => vec.clone(),
            Self::View { base, range } => base[range.clone()].to_vec(),
        }
    }

    /// Promotes the collection to a shared format for efficient memory usage
    pub fn promote_to_shared(&mut self) {
        match self {
            Self::Owned(vec) => {
                let shared = Arc::from(vec.as_slice());
                *self = Self::Shared(shared);
            }
            Self::View { base, range } => {
                let shared = Arc::from(&base[range.clone()]);
                *self = Self::Shared(shared);
            }
            Self::Shared(_) => {}
        }
    }

    /// Converts the collection to an owned format allowing mutations
    pub fn make_owned(&mut self) {
        match self {
            Self::Shared(arr) => {
                let owned = arr.to_vec();
                *self = Self::Owned(owned);
            }
            Self::View { base, range } => {
                let owned = base[range.clone()].to_vec();
                *self = Self::Owned(owned);
            }
            Self::Owned(_) => {}
        }
    }

    /// Adds a value to the end of the collection
    pub fn push(&mut self, value: FhirPathValue) {
        self.make_owned();
        if let Self::Owned(vec) = self {
            vec.push(value);
        }
    }

    /// Extends the collection with elements from another collection
    pub fn extend(&mut self, other: SmartCollection) {
        match (&mut *self, other) {
            (Self::Owned(vec), Self::Owned(other_vec)) => {
                vec.extend(other_vec);
            }
            (Self::Owned(vec), other) => {
                vec.extend(other.iter().cloned());
            }
            (this, other) => {
                this.make_owned();
                if let Self::Owned(vec) = this {
                    vec.extend(other.iter().cloned());
                }
            }
        }
    }

    /// Creates a new collection by concatenating this collection with another
    pub fn concat(&self, other: &SmartCollection) -> SmartCollection {
        let total_len = self.len() + other.len();
        let mut result = Vec::with_capacity(total_len);

        result.extend(self.iter().cloned());
        result.extend(other.iter().cloned());

        Self::Owned(result)
    }

    /// Creates a new collection containing elements from the given range
    pub fn slice(&self, range: Range<usize>) -> SmartCollection {
        let len = self.len();
        let start = range.start.min(len);
        let end = range.end.min(len);

        if start >= end {
            return Self::empty();
        }

        match self {
            Self::Shared(arr) => Self::view(arr.clone(), start..end),
            Self::Owned(vec) => Self::Owned(vec[start..end].to_vec()),
            Self::View {
                base,
                range: view_range,
            } => {
                let new_start = view_range.start + start;
                let new_end = view_range.start + end;
                Self::view(base.clone(), new_start..new_end)
            }
        }
    }

    /// Returns a reference to the first element, or None if the collection is empty
    pub fn first(&self) -> Option<&FhirPathValue> {
        self.get(0)
    }

    /// Returns a reference to the last element, or None if the collection is empty
    pub fn last(&self) -> Option<&FhirPathValue> {
        if self.is_empty() {
            None
        } else {
            self.get(self.len() - 1)
        }
    }

    /// Takes the first n elements from the collection
    pub fn take(&self, n: usize) -> SmartCollection {
        if n >= self.len() {
            self.clone()
        } else {
            self.slice(0..n)
        }
    }

    /// Skips the first n elements and returns the rest
    pub fn skip(&self, n: usize) -> SmartCollection {
        if n >= self.len() {
            Self::empty()
        } else {
            self.slice(n..self.len())
        }
    }

    /// Filters the collection using a predicate function
    pub fn filter<F>(&self, predicate: F) -> SmartCollection
    where
        F: Fn(&FhirPathValue) -> bool,
    {
        let filtered: Vec<FhirPathValue> = self
            .iter()
            .filter(|&value| predicate(value))
            .cloned()
            .collect();

        Self::from_vec(filtered)
    }

    /// Maps each element in the collection using a transformation function
    pub fn map<F>(&self, mapper: F) -> SmartCollection
    where
        F: Fn(&FhirPathValue) -> FhirPathValue,
    {
        let mapped: Vec<FhirPathValue> = self.iter().map(mapper).collect();

        Self::from_vec(mapped)
    }

    /// Returns true if the collection contains the specified value
    pub fn contains(&self, value: &FhirPathValue) -> bool {
        self.iter().any(|v| v == value)
    }

    /// Returns true if this collection is in shared format
    pub fn is_shared(&self) -> bool {
        matches!(self, Self::Shared(_))
    }

    /// Returns true if this collection is in owned format
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Returns true if this collection is a view into another collection
    pub fn is_view(&self) -> bool {
        matches!(self, Self::View { .. })
    }

    /// Returns a score (0.0-1.0) indicating the benefit of promoting this collection to shared format
    pub fn sharing_benefit(&self) -> f64 {
        match self {
            Self::Shared(_) => 1.0,   // Already shared
            Self::Owned(_) => 0.0,    // No sharing benefit until promoted
            Self::View { .. } => 0.8, // High benefit if promoted to shared
        }
    }
}

/// Iterator over elements in a SmartCollection
pub enum SmartCollectionIter<'a> {
    /// Iterator over a shared array slice
    Shared {
        /// Reference to the slice being iterated
        slice: &'a [FhirPathValue],
        /// Current iteration index
        index: usize,
    },
    /// Iterator over an owned vector slice
    Owned {
        /// Reference to the slice being iterated
        slice: &'a [FhirPathValue],
        /// Current iteration index
        index: usize,
    },
    /// Iterator over a view into a shared array
    View {
        /// Reference to the full slice
        slice: &'a [FhirPathValue],
        /// Range of indices to iterate over
        range: Range<usize>,
        /// Current iteration index within the range
        index: usize,
    },
}

impl<'a> Iterator for SmartCollectionIter<'a> {
    type Item = &'a FhirPathValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SmartCollectionIter::Shared { slice, index } => {
                if *index < slice.len() {
                    let item = &slice[*index];
                    *index += 1;
                    Some(item)
                } else {
                    None
                }
            }
            SmartCollectionIter::Owned { slice, index } => {
                if *index < slice.len() {
                    let item = &slice[*index];
                    *index += 1;
                    Some(item)
                } else {
                    None
                }
            }
            SmartCollectionIter::View {
                slice,
                range,
                index,
            } => {
                let range_index = range.start + *index;
                if range_index < range.end && range_index < slice.len() {
                    let item = &slice[range_index];
                    *index += 1;
                    Some(item)
                } else {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = match self {
            SmartCollectionIter::Shared { slice, index } => slice.len().saturating_sub(*index),
            SmartCollectionIter::Owned { slice, index } => slice.len().saturating_sub(*index),
            SmartCollectionIter::View { range, index, .. } => range.len().saturating_sub(*index),
        };
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for SmartCollectionIter<'a> {}

impl Default for SmartCollection {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<Vec<FhirPathValue>> for SmartCollection {
    fn from(vec: Vec<FhirPathValue>) -> Self {
        Self::from_vec(vec)
    }
}

impl From<Arc<[FhirPathValue]>> for SmartCollection {
    fn from(arc: Arc<[FhirPathValue]>) -> Self {
        Self::from_shared(arc)
    }
}

impl From<FhirPathValue> for SmartCollection {
    fn from(value: FhirPathValue) -> Self {
        Self::from_single(value)
    }
}

impl From<Collection> for SmartCollection {
    fn from(collection: Collection) -> Self {
        // Convert to vec and then to SmartCollection
        let vec: Vec<FhirPathValue> = collection.into_iter().collect();
        Self::from_vec(vec)
    }
}

impl From<SmartCollection> for Collection {
    fn from(smart: SmartCollection) -> Self {
        match smart {
            SmartCollection::Shared(arc) => {
                // Convert Arc<[T]> to Vec and then to Collection
                Collection::from_vec(arc.to_vec())
            }
            SmartCollection::Owned(vec) => Collection::from_vec(vec),
            SmartCollection::View { base, range } => Collection::from_vec(base[range].to_vec()),
        }
    }
}

impl FromIterator<FhirPathValue> for SmartCollection {
    fn from_iter<T: IntoIterator<Item = FhirPathValue>>(iter: T) -> Self {
        Self::from_vec(iter.into_iter().collect())
    }
}

impl IntoIterator for SmartCollection {
    type Item = FhirPathValue;
    type IntoIter = std::vec::IntoIter<FhirPathValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a SmartCollection {
    type Item = &'a FhirPathValue;
    type IntoIter = SmartCollectionIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl PartialEq for SmartCollection {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl Eq for SmartCollection {}

/// Builder for constructing SmartCollections with automatic promotion optimization
pub struct SmartCollectionBuilder {
    /// The collection being built
    collection: SmartCollection,
    /// Threshold size for automatic promotion to shared format
    promotion_threshold: usize,
}

impl SmartCollectionBuilder {
    /// Creates a new builder with default settings
    pub fn new() -> Self {
        Self {
            collection: SmartCollection::empty(),
            promotion_threshold: 10,
        }
    }

    /// Sets the threshold size for automatic promotion to shared format
    pub fn with_promotion_threshold(mut self, threshold: usize) -> Self {
        self.promotion_threshold = threshold;
        self
    }

    /// Adds a value to the collection being built
    pub fn push(mut self, value: FhirPathValue) -> Self {
        self.collection.push(value);
        self.maybe_promote();
        self
    }

    /// Extends the collection with elements from another collection
    pub fn extend(mut self, other: SmartCollection) -> Self {
        self.collection.extend(other);
        self.maybe_promote();
        self
    }

    fn maybe_promote(&mut self) {
        if self.collection.len() >= self.promotion_threshold && !self.collection.is_shared() {
            self.collection.promote_to_shared();
        }
    }

    /// Builds and returns the final SmartCollection
    pub fn build(self) -> SmartCollection {
        self.collection
    }
}

impl Default for SmartCollectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FhirPathValue;

    #[test]
    fn test_smart_collection_creation() {
        let empty = SmartCollection::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let from_vec = SmartCollection::from_vec(vec![FhirPathValue::Boolean(true)]);
        assert_eq!(from_vec.len(), 1);
        assert!(!from_vec.is_empty());

        let single = SmartCollection::from_single(FhirPathValue::Integer(42));
        assert_eq!(single.len(), 1);
        assert_eq!(single.first(), Some(&FhirPathValue::Integer(42)));
    }

    #[test]
    fn test_promotion_to_shared() {
        let mut collection = SmartCollection::from_vec(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Integer(42),
        ]);

        assert!(collection.is_owned());
        collection.promote_to_shared();
        assert!(collection.is_shared());
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn test_view_operations() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ];
        let shared: Arc<[FhirPathValue]> = Arc::from(values.as_slice());
        let collection = SmartCollection::from_shared(shared.clone());

        let view = collection.slice(1..3);
        assert!(view.is_view());
        assert_eq!(view.len(), 2);
        assert_eq!(view.get(0), Some(&FhirPathValue::Integer(2)));
        assert_eq!(view.get(1), Some(&FhirPathValue::Integer(3)));
    }

    #[test]
    fn test_collection_operations() {
        let collection1 =
            SmartCollection::from_vec(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);

        let collection2 =
            SmartCollection::from_vec(vec![FhirPathValue::Integer(3), FhirPathValue::Integer(4)]);

        let concatenated = collection1.concat(&collection2);
        assert_eq!(concatenated.len(), 4);
        assert_eq!(concatenated.get(0), Some(&FhirPathValue::Integer(1)));
        assert_eq!(concatenated.get(3), Some(&FhirPathValue::Integer(4)));
    }

    #[test]
    fn test_smart_collection_builder() {
        let collection = SmartCollectionBuilder::new()
            .with_promotion_threshold(2)
            .push(FhirPathValue::Integer(1))
            .push(FhirPathValue::Integer(2))
            .build();

        assert!(collection.is_shared()); // Should be promoted at threshold
        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn test_filtering_and_mapping() {
        let collection = SmartCollection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]);

        let filtered = collection.filter(|v| matches!(v, FhirPathValue::Integer(i) if *i % 2 == 0));

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered.get(0), Some(&FhirPathValue::Integer(2)));
        assert_eq!(filtered.get(1), Some(&FhirPathValue::Integer(4)));
    }

    #[test]
    fn test_arc_sharing() {
        let values = vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)];
        let shared: Arc<[FhirPathValue]> = Arc::from(values.as_slice());

        let collection1 = SmartCollection::from_shared(shared.clone());
        let collection2 = SmartCollection::from_shared(shared);

        // Both collections should reference the same Arc
        match (&collection1, &collection2) {
            (SmartCollection::Shared(arc1), SmartCollection::Shared(arc2)) => {
                assert!(Arc::ptr_eq(arc1, arc2));
            }
            _ => panic!("Expected both collections to be shared"),
        }
    }
}
