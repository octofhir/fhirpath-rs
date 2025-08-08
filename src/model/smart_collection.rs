use crate::model::{FhirPathValue, value::Collection};
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SmartCollection {
    Shared(Arc<[FhirPathValue]>),
    Owned(Vec<FhirPathValue>),
    View {
        base: Arc<[FhirPathValue]>,
        range: Range<usize>,
    },
}

impl SmartCollection {
    pub fn empty() -> Self {
        Self::Owned(Vec::new())
    }

    pub fn from_vec(values: Vec<FhirPathValue>) -> Self {
        Self::Owned(values)
    }

    pub fn from_shared(values: Arc<[FhirPathValue]>) -> Self {
        Self::Shared(values)
    }

    pub fn from_single(value: FhirPathValue) -> Self {
        Self::Owned(vec![value])
    }

    pub fn view(base: Arc<[FhirPathValue]>, range: Range<usize>) -> Self {
        if range.start >= base.len() || range.end > base.len() || range.start >= range.end {
            Self::Owned(Vec::new())
        } else {
            Self::View { base, range }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Shared(arr) => arr.len(),
            Self::Owned(vec) => vec.len(),
            Self::View { range, .. } => range.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

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

    pub fn to_vec(&self) -> Vec<FhirPathValue> {
        match self {
            Self::Shared(arr) => arr.to_vec(),
            Self::Owned(vec) => vec.clone(),
            Self::View { base, range } => base[range.clone()].to_vec(),
        }
    }

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

    pub fn push(&mut self, value: FhirPathValue) {
        self.make_owned();
        if let Self::Owned(vec) = self {
            vec.push(value);
        }
    }

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

    pub fn concat(&self, other: &SmartCollection) -> SmartCollection {
        let total_len = self.len() + other.len();
        let mut result = Vec::with_capacity(total_len);

        result.extend(self.iter().cloned());
        result.extend(other.iter().cloned());

        Self::Owned(result)
    }

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

    pub fn first(&self) -> Option<&FhirPathValue> {
        self.get(0)
    }

    pub fn last(&self) -> Option<&FhirPathValue> {
        if self.is_empty() {
            None
        } else {
            self.get(self.len() - 1)
        }
    }

    pub fn take(&self, n: usize) -> SmartCollection {
        if n >= self.len() {
            self.clone()
        } else {
            self.slice(0..n)
        }
    }

    pub fn skip(&self, n: usize) -> SmartCollection {
        if n >= self.len() {
            Self::empty()
        } else {
            self.slice(n..self.len())
        }
    }

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

    pub fn map<F>(&self, mapper: F) -> SmartCollection
    where
        F: Fn(&FhirPathValue) -> FhirPathValue,
    {
        let mapped: Vec<FhirPathValue> = self.iter().map(mapper).collect();

        Self::from_vec(mapped)
    }

    pub fn contains(&self, value: &FhirPathValue) -> bool {
        self.iter().any(|v| v == value)
    }

    pub fn is_shared(&self) -> bool {
        matches!(self, Self::Shared(_))
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub fn is_view(&self) -> bool {
        matches!(self, Self::View { .. })
    }

    pub fn sharing_benefit(&self) -> f64 {
        match self {
            Self::Shared(_) => 1.0,   // Already shared
            Self::Owned(_) => 0.0,    // No sharing benefit until promoted
            Self::View { .. } => 0.8, // High benefit if promoted to shared
        }
    }
}

pub enum SmartCollectionIter<'a> {
    Shared {
        slice: &'a [FhirPathValue],
        index: usize,
    },
    Owned {
        slice: &'a [FhirPathValue],
        index: usize,
    },
    View {
        slice: &'a [FhirPathValue],
        range: Range<usize>,
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

pub struct SmartCollectionBuilder {
    collection: SmartCollection,
    promotion_threshold: usize,
}

impl SmartCollectionBuilder {
    pub fn new() -> Self {
        Self {
            collection: SmartCollection::empty(),
            promotion_threshold: 10,
        }
    }

    pub fn with_promotion_threshold(mut self, threshold: usize) -> Self {
        self.promotion_threshold = threshold;
        self
    }

    pub fn push(mut self, value: FhirPathValue) -> Self {
        self.collection.push(value);
        self.maybe_promote();
        self
    }

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
    use crate::model::FhirPathValue;

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
