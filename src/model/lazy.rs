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

//! Lazy evaluation for collection operations
//!
//! This module provides lazy evaluation capabilities for FHIRPath collections,
//! allowing operations to be chained without materializing intermediate results.
//! This significantly improves performance for complex collection operations.

use crate::model::FhirPathValue;
use std::fmt;
use std::sync::Arc;

/// Lazy collection that defers evaluation until materialization
#[derive(Clone)]
pub enum LazyCollection {
    /// Materialized collection with concrete values
    Materialized(Arc<[FhirPathValue]>),

    /// Filtered collection that applies a predicate
    Filtered {
        /// Base collection to filter
        base: Box<LazyCollection>,
        /// Predicate function for filtering
        predicate: Arc<dyn Fn(&FhirPathValue) -> bool + Send + Sync>,
    },

    /// Mapped collection that transforms values
    Mapped {
        /// Base collection to transform
        base: Box<LazyCollection>,
        /// Transform function for mapping
        transform: Arc<dyn Fn(&FhirPathValue) -> FhirPathValue + Send + Sync>,
    },

    /// Flattened collection that expands nested collections
    Flattened {
        /// Base collection containing collections to flatten
        base: Box<LazyCollection>,
    },

    /// Collection that takes the first N elements
    Take {
        /// Base collection to take from
        base: Box<LazyCollection>,
        /// Number of elements to take
        count: usize,
    },

    /// Collection that skips the first N elements
    Skip {
        /// Base collection to skip from
        base: Box<LazyCollection>,
        /// Number of elements to skip
        count: usize,
    },

    /// Concatenation of two collections
    Concat {
        /// First collection
        first: Box<LazyCollection>,
        /// Second collection
        second: Box<LazyCollection>,
    },

    /// Distinct collection that removes duplicates
    Distinct {
        /// Base collection to deduplicate
        base: Box<LazyCollection>,
    },

    /// Empty collection
    Empty,
}

impl LazyCollection {
    /// Create a new lazy collection from materialized values
    pub fn from_vec(values: Vec<FhirPathValue>) -> Self {
        Self::Materialized(values.into())
    }

    /// Create a new lazy collection from an Arc slice
    pub fn from_arc(values: Arc<[FhirPathValue]>) -> Self {
        Self::Materialized(values)
    }

    /// Create an empty lazy collection
    pub fn empty() -> Self {
        Self::Empty
    }

    /// Create a filtered lazy collection
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&FhirPathValue) -> bool + Send + Sync + 'static,
    {
        Self::Filtered {
            base: Box::new(self),
            predicate: Arc::new(predicate),
        }
    }

    /// Create a mapped lazy collection
    pub fn map<F>(self, transform: F) -> Self
    where
        F: Fn(&FhirPathValue) -> FhirPathValue + Send + Sync + 'static,
    {
        Self::Mapped {
            base: Box::new(self),
            transform: Arc::new(transform),
        }
    }

    /// Create a flattened lazy collection
    pub fn flatten(self) -> Self {
        Self::Flattened {
            base: Box::new(self),
        }
    }

    /// Create a lazy collection that takes the first N elements
    pub fn take(self, count: usize) -> Self {
        if count == 0 {
            return Self::Empty;
        }
        Self::Take {
            base: Box::new(self),
            count,
        }
    }

    /// Create a lazy collection that skips the first N elements
    pub fn skip(self, count: usize) -> Self {
        if count == 0 {
            return self;
        }
        Self::Skip {
            base: Box::new(self),
            count,
        }
    }

    /// Concatenate two lazy collections
    pub fn concat(self, other: LazyCollection) -> Self {
        match (&self, &other) {
            (Self::Empty, _) => other,
            (_, Self::Empty) => self,
            _ => Self::Concat {
                first: Box::new(self),
                second: Box::new(other),
            },
        }
    }

    /// Create a lazy collection with distinct values
    pub fn distinct(self) -> Self {
        Self::Distinct {
            base: Box::new(self),
        }
    }

    /// Check if the collection is empty (this may require partial evaluation)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Materialized(values) => values.is_empty(),
            // For lazy operations, we can't determine emptiness without evaluation
            // We use a heuristic approach here
            _ => self.into_iter().next().is_none(),
        }
    }

    /// Get the length of the collection (this requires full materialization)
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Materialized(values) => values.len(),
            _ => self.into_iter().count(),
        }
    }

    /// Get the first element without full materialization
    pub fn first(&self) -> Option<FhirPathValue> {
        self.into_iter().next()
    }

    /// Get the last element (requires full evaluation for most operations)
    pub fn last(&self) -> Option<FhirPathValue> {
        match self {
            Self::Empty => None,
            Self::Materialized(values) => values.last().cloned(),
            _ => {
                let mut last = None;
                for item in self.into_iter() {
                    last = Some(item);
                }
                last
            }
        }
    }

    /// Materialize the lazy collection into a concrete vector
    pub fn materialize(self) -> Vec<FhirPathValue> {
        self.into_iter().collect()
    }

    /// Convert to an iterator for lazy evaluation
    pub fn into_iter(&self) -> LazyIterator {
        LazyIterator::new(self.clone())
    }

    /// Check if this collection contains a specific value
    pub fn contains(&self, value: &FhirPathValue) -> bool {
        self.into_iter().any(|v| &v == value)
    }

    /// Optimize the lazy collection by collapsing redundant operations
    pub fn optimize(self) -> Self {
        match self {
            // Collapse nested filters
            Self::Filtered {
                base,
                predicate: p1,
            } => {
                if let Self::Filtered {
                    base: inner_base,
                    predicate: p2,
                } = *base
                {
                    let combined_predicate =
                        Arc::new(move |value: &FhirPathValue| p2(value) && p1(value));
                    Self::Filtered {
                        base: inner_base,
                        predicate: combined_predicate,
                    }
                } else {
                    Self::Filtered {
                        base,
                        predicate: p1,
                    }
                }
            }

            // Collapse consecutive take operations
            Self::Take { base, count: c1 } => {
                if let Self::Take {
                    base: inner_base,
                    count: c2,
                } = *base
                {
                    Self::Take {
                        base: inner_base,
                        count: c1.min(c2),
                    }
                } else {
                    Self::Take { base, count: c1 }
                }
            }

            // Collapse consecutive skip operations
            Self::Skip { base, count: c1 } => {
                if let Self::Skip {
                    base: inner_base,
                    count: c2,
                } = *base
                {
                    Self::Skip {
                        base: inner_base,
                        count: c1 + c2,
                    }
                } else {
                    Self::Skip { base, count: c1 }
                }
            }

            // Return other operations as-is
            other => other,
        }
    }
}

/// Iterator for lazy evaluation of collections
///
/// This iterator implements a stack-based evaluation system that processes
/// lazy collection operations on-demand. Each operation type is represented
/// as a state on the evaluation stack.
#[derive(Clone)]
pub struct LazyIterator {
    /// Stack of iterator states representing chained operations
    stack: Vec<IteratorState>,
}

/// Internal state representation for lazy iterator operations
///
/// Each variant represents a different type of collection operation
/// that can be performed lazily. The iterator processes these states
/// in a stack-based manner to achieve efficient lazy evaluation.
#[derive(Clone)]
enum IteratorState {
    /// Materialized collection with concrete values
    Materialized {
        /// The actual values stored in Arc for zero-copy sharing
        values: Arc<[FhirPathValue]>,
        /// Current index position in the values array
        index: usize,
    },
    /// Filtered collection applying a predicate function
    Filtered {
        /// Base iterator to filter from
        base_iter: Box<LazyIterator>,
        /// Predicate function for filtering elements
        predicate: Arc<dyn Fn(&FhirPathValue) -> bool + Send + Sync>,
    },
    /// Mapped collection applying a transformation function
    Mapped {
        /// Base iterator to transform
        base_iter: Box<LazyIterator>,
        /// Transform function for mapping elements
        transform: Arc<dyn Fn(&FhirPathValue) -> FhirPathValue + Send + Sync>,
    },
    /// Flattened collection expanding nested collections
    Flattened {
        /// Base iterator containing collections to flatten
        base_iter: Box<LazyIterator>,
        /// Current inner iterator being processed
        current_inner: Option<Box<LazyIterator>>,
    },
    /// Take operation limiting to first N elements
    Take {
        /// Base iterator to take from
        base_iter: Box<LazyIterator>,
        /// Number of elements remaining to take
        remaining: usize,
    },
    /// Skip operation skipping first N elements  
    Skip {
        /// Base iterator to skip from
        base_iter: Box<LazyIterator>,
        /// Number of elements remaining to skip
        remaining: usize,
    },
    /// Concatenation of two iterators
    Concat {
        /// First iterator to process
        first_iter: Box<LazyIterator>,
        /// Second iterator to process after first is exhausted
        second_iter: Option<Box<LazyIterator>>,
    },
    /// Distinct operation removing duplicates
    Distinct {
        /// Base iterator to deduplicate
        base_iter: Box<LazyIterator>,
        /// Set of seen values (using string representation for hashing)
        seen: std::collections::HashSet<String>,
    },
    /// Empty iterator that yields no values
    Empty,
}

impl LazyIterator {
    fn new(collection: LazyCollection) -> Self {
        let state = match collection {
            LazyCollection::Materialized(values) => {
                IteratorState::Materialized { values, index: 0 }
            }
            LazyCollection::Filtered { base, predicate } => IteratorState::Filtered {
                base_iter: Box::new(LazyIterator::new(*base)),
                predicate,
            },
            LazyCollection::Mapped { base, transform } => IteratorState::Mapped {
                base_iter: Box::new(LazyIterator::new(*base)),
                transform,
            },
            LazyCollection::Flattened { base } => IteratorState::Flattened {
                base_iter: Box::new(LazyIterator::new(*base)),
                current_inner: None,
            },
            LazyCollection::Take { base, count } => IteratorState::Take {
                base_iter: Box::new(LazyIterator::new(*base)),
                remaining: count,
            },
            LazyCollection::Skip { base, count } => IteratorState::Skip {
                base_iter: Box::new(LazyIterator::new(*base)),
                remaining: count,
            },
            LazyCollection::Concat { first, second } => IteratorState::Concat {
                first_iter: Box::new(LazyIterator::new(*first)),
                second_iter: Some(Box::new(LazyIterator::new(*second))),
            },
            LazyCollection::Distinct { base } => IteratorState::Distinct {
                base_iter: Box::new(LazyIterator::new(*base)),
                seen: std::collections::HashSet::new(),
            },
            LazyCollection::Empty => IteratorState::Empty,
        };

        Self { stack: vec![state] }
    }
}

impl Iterator for LazyIterator {
    type Item = FhirPathValue;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut state) = self.stack.pop() {
            match &mut state {
                IteratorState::Materialized { values, index } => {
                    if *index < values.len() {
                        let value = values[*index].clone();
                        *index += 1;
                        self.stack.push(state);
                        return Some(value);
                    }
                }

                IteratorState::Filtered {
                    base_iter,
                    predicate,
                } => {
                    for value in base_iter.by_ref() {
                        if predicate(&value) {
                            self.stack.push(state);
                            return Some(value);
                        }
                    }
                }

                IteratorState::Mapped {
                    base_iter,
                    transform,
                } => {
                    if let Some(value) = base_iter.next() {
                        let transformed = transform(&value);
                        self.stack.push(state);
                        return Some(transformed);
                    }
                }

                IteratorState::Flattened {
                    base_iter,
                    current_inner,
                } => {
                    // Try to get next from current inner iterator
                    if let Some(inner) = current_inner {
                        if let Some(value) = inner.next() {
                            self.stack.push(state);
                            return Some(value);
                        }
                    }

                    // Get next collection from base iterator
                    for next_collection in base_iter.by_ref() {
                        match next_collection {
                            FhirPathValue::Collection(collection) => {
                                let lazy_collection =
                                    LazyCollection::from_arc(collection.as_arc().clone());
                                *current_inner = Some(Box::new(LazyIterator::new(lazy_collection)));

                                if let Some(inner) = current_inner {
                                    if let Some(value) = inner.next() {
                                        self.stack.push(state);
                                        return Some(value);
                                    }
                                }
                            }
                            FhirPathValue::Empty => {
                                // Skip empty values
                                continue;
                            }
                            other => {
                                // Single value
                                self.stack.push(state);
                                return Some(other);
                            }
                        }
                    }
                }

                IteratorState::Take {
                    base_iter,
                    remaining,
                } => {
                    if *remaining > 0 {
                        if let Some(value) = base_iter.next() {
                            *remaining -= 1;
                            self.stack.push(state);
                            return Some(value);
                        }
                    }
                }

                IteratorState::Skip {
                    base_iter,
                    remaining,
                } => {
                    // Skip remaining elements
                    while *remaining > 0 {
                        base_iter.next()?;
                        *remaining -= 1;
                    }

                    // Now return subsequent elements
                    if let Some(value) = base_iter.next() {
                        self.stack.push(state);
                        return Some(value);
                    }
                }

                IteratorState::Concat {
                    first_iter,
                    second_iter,
                } => {
                    if let Some(value) = first_iter.next() {
                        self.stack.push(state);
                        return Some(value);
                    }

                    // First iterator exhausted, try second
                    if let Some(mut second) = second_iter.take() {
                        if let Some(value) = second.next() {
                            *second_iter = Some(second);
                            self.stack.push(state);
                            return Some(value);
                        }
                    }
                }

                IteratorState::Distinct { base_iter, seen } => {
                    for value in base_iter.by_ref() {
                        let key = value.to_string();
                        if seen.insert(key) {
                            self.stack.push(state);
                            return Some(value);
                        }
                    }
                }

                IteratorState::Empty => {
                    // Empty iterator
                    return None;
                }
            }
        }

        None
    }
}

impl fmt::Debug for LazyCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Materialized(values) => write!(f, "Materialized({} items)", values.len()),
            Self::Filtered { base, .. } => write!(f, "Filtered({base:?})"),
            Self::Mapped { base, .. } => write!(f, "Mapped({base:?})"),
            Self::Flattened { base } => write!(f, "Flattened({base:?})"),
            Self::Take { base, count } => write!(f, "Take({base:?}, {count})"),
            Self::Skip { base, count } => write!(f, "Skip({base:?}, {count})"),
            Self::Concat { first, second } => write!(f, "Concat({first:?}, {second:?})"),
            Self::Distinct { base } => write!(f, "Distinct({base:?})"),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

impl fmt::Debug for LazyIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LazyIterator(stack_depth: {})", self.stack.len())
    }
}

/// Helper trait to convert collections to lazy collections
pub trait ToLazy {
    /// Convert to a lazy collection
    fn to_lazy(self) -> LazyCollection;
}

impl ToLazy for Vec<FhirPathValue> {
    fn to_lazy(self) -> LazyCollection {
        LazyCollection::from_vec(self)
    }
}

impl ToLazy for crate::model::value::Collection {
    fn to_lazy(self) -> LazyCollection {
        LazyCollection::from_arc(self.as_arc().clone())
    }
}

impl ToLazy for FhirPathValue {
    fn to_lazy(self) -> LazyCollection {
        match self {
            FhirPathValue::Collection(collection) => collection.to_lazy(),
            FhirPathValue::Empty => LazyCollection::empty(),
            single => LazyCollection::from_vec(vec![single]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_materialized_collection() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let lazy = LazyCollection::from_vec(values.clone());

        let materialized = lazy.materialize();
        assert_eq!(materialized, values);
    }

    #[test]
    fn test_filter_operation() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ];
        let lazy = LazyCollection::from_vec(values).filter(|v| match v {
            FhirPathValue::Integer(i) => i % 2 == 0,
            _ => false,
        });

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![FhirPathValue::Integer(2), FhirPathValue::Integer(4)]
        );
    }

    #[test]
    fn test_map_operation() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let lazy = LazyCollection::from_vec(values).map(|v| match v {
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i * 2),
            other => other.clone(),
        });

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(4),
                FhirPathValue::Integer(6),
            ]
        );
    }

    #[test]
    fn test_chained_operations() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
            FhirPathValue::Integer(5),
        ];

        let lazy = LazyCollection::from_vec(values)
            .filter(|v| match v {
                FhirPathValue::Integer(i) => *i > 2,
                _ => false,
            })
            .map(|v| match v {
                FhirPathValue::Integer(i) => FhirPathValue::Integer(i * 10),
                other => other.clone(),
            })
            .take(2);

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![FhirPathValue::Integer(30), FhirPathValue::Integer(40)]
        );
    }

    #[test]
    fn test_take_operation() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ];
        let lazy = LazyCollection::from_vec(values).take(2);

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]
        );
    }

    #[test]
    fn test_skip_operation() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ];
        let lazy = LazyCollection::from_vec(values).skip(2);

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![FhirPathValue::Integer(3), FhirPathValue::Integer(4)]
        );
    }

    #[test]
    fn test_concat_operation() {
        let values1 = vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)];
        let values2 = vec![FhirPathValue::Integer(3), FhirPathValue::Integer(4)];

        let lazy1 = LazyCollection::from_vec(values1);
        let lazy2 = LazyCollection::from_vec(values2);
        let concatenated = lazy1.concat(lazy2);

        let result = concatenated.materialize();
        assert_eq!(
            result,
            vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(4),
            ]
        );
    }

    #[test]
    fn test_distinct_operation() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2),
        ];
        let lazy = LazyCollection::from_vec(values).distinct();

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
            ]
        );
    }

    #[test]
    fn test_empty_collection() {
        let lazy = LazyCollection::empty();
        assert!(lazy.is_empty());
        assert_eq!(lazy.len(), 0);
        assert_eq!(lazy.materialize(), Vec::<FhirPathValue>::new());
    }

    #[test]
    fn test_first_and_last() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];
        let lazy = LazyCollection::from_vec(values.clone());

        assert_eq!(lazy.first(), Some(FhirPathValue::Integer(1)));
        assert_eq!(lazy.last(), Some(FhirPathValue::Integer(3)));
    }

    #[test]
    fn test_lazy_evaluation_performance() {
        // Create a large collection
        let values: Vec<FhirPathValue> = (0..10000).map(FhirPathValue::Integer).collect();

        // Apply operations that should be lazy
        let lazy = LazyCollection::from_vec(values)
            .filter(|v| match v {
                FhirPathValue::Integer(i) => *i > 5000,
                _ => false,
            })
            .take(5); // Only take 5 elements

        // This should only process 5 elements from the filtered result,
        // not the entire 10000 element collection
        let result = lazy.materialize();
        assert_eq!(result.len(), 5);

        // Verify the results are correct
        let expected: Vec<FhirPathValue> = (5001..5006).map(FhirPathValue::Integer).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_optimization() {
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];

        // Create nested filters that should be optimized
        let lazy = LazyCollection::from_vec(values)
            .filter(|v| match v {
                FhirPathValue::Integer(i) => *i > 0,
                _ => false,
            })
            .filter(|v| match v {
                FhirPathValue::Integer(i) => *i < 3,
                _ => false,
            })
            .optimize();

        let result = lazy.materialize();
        assert_eq!(
            result,
            vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]
        );
    }
}
