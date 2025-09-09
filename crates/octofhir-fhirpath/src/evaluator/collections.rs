//! Collection evaluation implementation for FHIRPath collection operations
//!
//! This module implements minimal CollectionEvaluator functionality for:
//! - Collection literals
//! - Union operations
//! - Basic filtering (placeholder for now)
//! - Collection utility operations

use async_trait::async_trait;
use std::collections::HashSet;

use crate::{
    ast::ExpressionNode,
    core::{FhirPathValue, Result},
    core::types::Collection,
    evaluator::metadata_collections::MetadataCollectionEvaluator,
    evaluator::{
        EvaluationContext,
        traits::{CollectionEvaluator, MetadataAwareCollectionEvaluator},
    },
    typing::TypeResolver,
    wrapped::{WrappedCollection, WrappedValue, collection_utils},
};

/// Implementation of CollectionEvaluator
pub struct CollectionEvaluatorImpl;

impl CollectionEvaluatorImpl {
    /// Create a new collection evaluator
    pub fn new() -> Self {
        Self
    }

    /// Helper to flatten nested collections
    fn flatten_collection(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => {
                let mut result = Vec::new();
                for item in items.iter() {
                    result.extend(self.flatten_collection(item));
                }
                result
            }
            FhirPathValue::Empty => Vec::new(),
            single_value => vec![single_value.clone()],
        }
    }

    /// Helper to remove duplicates from a collection
    fn remove_duplicates(&self, items: Vec<FhirPathValue>) -> Vec<FhirPathValue> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for item in items {
            // For simplicity, use string representation for deduplication
            // In a full implementation, this would need proper value comparison
            let key = format!("{:?}", item);
            if seen.insert(key) {
                result.push(item);
            }
        }

        result
    }
}

impl Default for CollectionEvaluatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CollectionEvaluator for CollectionEvaluatorImpl {
    fn create_collection(&self, elements: Vec<FhirPathValue>) -> FhirPathValue {
        // Filter out empty values
        let filtered_elements: Vec<FhirPathValue> = elements
            .into_iter()
            .filter(|item| !matches!(item, FhirPathValue::Empty))
            .collect();

        match filtered_elements.len() {
            0 => FhirPathValue::Empty,
            1 => filtered_elements.into_iter().next().unwrap(),
            _ => FhirPathValue::Collection(Collection::from_values(filtered_elements)),
        }
    }

    fn union_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> FhirPathValue {
        let mut left_items = self.flatten_collection(left);
        let right_items = self.flatten_collection(right);

        // Add right items to left, maintaining order
        left_items.extend(right_items);

        // Remove duplicates to follow FHIRPath union semantics
        let unique_items = self.remove_duplicates(left_items);

        self.create_collection(unique_items)
    }

    async fn filter_collection(
        &mut self,
        collection: &FhirPathValue,
        _condition: &ExpressionNode,
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // For now, return the collection unchanged
        // Full implementation would evaluate the condition for each item
        Ok(collection.clone())
    }

    fn value_length(&self, value: &FhirPathValue) -> usize {
        match value {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        }
    }

    fn contains_value(&self, collection: &FhirPathValue, value: &FhirPathValue) -> bool {
        match collection {
            FhirPathValue::Collection(items) => {
                items.iter().any(|item| {
                    // Simple comparison - in full implementation would use proper equality
                    format!("{:?}", item) == format!("{:?}", value)
                })
            }
            FhirPathValue::Empty => false,
            single_value => format!("{:?}", single_value) == format!("{:?}", value),
        }
    }
}

impl CollectionEvaluatorImpl {
    /// Bridge method to create collection with metadata awareness
    pub async fn create_collection_with_metadata_bridge(
        &self,
        elements: Vec<WrappedCollection>,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let metadata_evaluator = MetadataCollectionEvaluator::new();
        metadata_evaluator
            .create_collection_with_metadata(elements, resolver)
            .await
    }

    /// Bridge method to union collections with metadata
    pub async fn union_collections_with_metadata_bridge(
        &self,
        left: &WrappedCollection,
        right: &WrappedCollection,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let metadata_evaluator = MetadataCollectionEvaluator::new();
        metadata_evaluator
            .union_collections_with_metadata(left, right, resolver)
            .await
    }

    /// Convert plain collection result to wrapped collection
    pub async fn wrap_collection_result(
        &self,
        result: FhirPathValue,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        use crate::path::CanonicalPath;
        use crate::typing::type_utils;
        use crate::wrapped::ValueMetadata;

        match result {
            FhirPathValue::Empty => Ok(collection_utils::empty()),
            FhirPathValue::Collection(values) => {
                let wrapped_values: Vec<WrappedValue> = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let fhir_type = type_utils::fhirpath_value_to_fhir_type(&value);
                        let path = CanonicalPath::parse(&format!("[{}]", i)).unwrap();
                        let metadata = ValueMetadata {
                            fhir_type,
                            resource_type: None,
                            path,
                            index: Some(i),
                        };
                        WrappedValue::new(value, metadata)
                    })
                    .collect();
                Ok(wrapped_values)
            }
            single_value => {
                let fhir_type = type_utils::fhirpath_value_to_fhir_type(&single_value);
                let metadata = ValueMetadata {
                    fhir_type,
                    resource_type: None,
                    path: CanonicalPath::empty(),
                    index: None,
                };
                Ok(collection_utils::single(WrappedValue::new(
                    single_value,
                    metadata,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_create_collection() {
        let evaluator = CollectionEvaluatorImpl::new();

        // Empty collection
        let result = evaluator.create_collection(vec![]);
        assert_eq!(result, FhirPathValue::Empty);

        // Single item collection
        let result = evaluator.create_collection(vec![FhirPathValue::Integer(42)]);
        assert_eq!(result, FhirPathValue::Integer(42));

        // Multiple item collection
        let result = evaluator.create_collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], FhirPathValue::Integer(1));
                assert_eq!(items[1], FhirPathValue::Integer(2));
                assert_eq!(items[2], FhirPathValue::Integer(3));
            }
            _ => panic!("Expected Collection"),
        }

        // Collection with empty values (should be filtered out)
        let result = evaluator.create_collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Empty,
            FhirPathValue::Integer(2),
        ]);
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], FhirPathValue::Integer(1));
                assert_eq!(items[1], FhirPathValue::Integer(2));
            }
            _ => panic!("Expected Collection"),
        }
    }

    #[tokio::test]
    async fn test_union_values() {
        let evaluator = CollectionEvaluatorImpl::new();

        // Union of single values
        let left = FhirPathValue::Integer(1);
        let right = FhirPathValue::Integer(2);
        let result = evaluator.union_values(&left, &right);
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], FhirPathValue::Integer(1));
                assert_eq!(items[1], FhirPathValue::Integer(2));
            }
            _ => panic!("Expected Collection"),
        }

        // Union with collections
        let left =
            FhirPathValue::Collection(vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)]);
        let right = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(2), // Duplicate - should be removed
            FhirPathValue::Integer(3),
        ]);
        let result = evaluator.union_values(&left, &right);
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3); // Duplicates removed
                assert!(items.contains(&FhirPathValue::Integer(1)));
                assert!(items.contains(&FhirPathValue::Integer(2)));
                assert!(items.contains(&FhirPathValue::Integer(3)));
            }
            _ => panic!("Expected Collection"),
        }

        // Union with empty
        let left = FhirPathValue::Integer(1);
        let right = FhirPathValue::Empty;
        let result = evaluator.union_values(&left, &right);
        assert_eq!(result, FhirPathValue::Integer(1));
    }

    #[test]
    fn test_value_length() {
        let evaluator = CollectionEvaluatorImpl::new();

        // Single value
        assert_eq!(evaluator.value_length(&FhirPathValue::Integer(42)), 1);

        // Empty value
        assert_eq!(evaluator.value_length(&FhirPathValue::Empty), 0);

        // Collection
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        assert_eq!(evaluator.value_length(&collection), 3);
    }

    #[test]
    fn test_contains_value() {
        let evaluator = CollectionEvaluatorImpl::new();

        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::String("test".to_string()),
            FhirPathValue::Boolean(true),
        ]);

        // Value is in collection
        assert!(evaluator.contains_value(&collection, &FhirPathValue::Integer(1)));
        assert!(evaluator.contains_value(&collection, &FhirPathValue::String("test".to_string())));

        // Value is not in collection
        assert!(!evaluator.contains_value(&collection, &FhirPathValue::Integer(99)));
        assert!(
            !evaluator.contains_value(&collection, &FhirPathValue::String("missing".to_string()))
        );

        // Single value contains itself
        let single = FhirPathValue::Integer(42);
        assert!(evaluator.contains_value(&single, &FhirPathValue::Integer(42)));
        assert!(!evaluator.contains_value(&single, &FhirPathValue::Integer(1)));

        // Empty contains nothing
        assert!(!evaluator.contains_value(&FhirPathValue::Empty, &FhirPathValue::Integer(1)));
    }
}
