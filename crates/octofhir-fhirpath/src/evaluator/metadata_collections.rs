//! Metadata-aware collection evaluator for FHIRPath expressions
//!
//! This module provides collection evaluation capabilities that maintain rich metadata
//! throughout collection operations like union, filter, and collection creation.

use async_trait::async_trait;

use crate::{
    ast::ExpressionNode,
    core::{FhirPathValue, Result},
    evaluator::{EvaluationContext, traits::MetadataAwareCollectionEvaluator},
    path::CanonicalPath,
    typing::{TypeResolver, type_utils},
    wrapped::{WrappedCollection, WrappedValue, collection_utils},
};

/// Metadata-aware collection evaluator
#[derive(Debug, Clone)]
pub struct MetadataCollectionEvaluator;

impl MetadataCollectionEvaluator {
    /// Create a new metadata-aware collection evaluator
    pub fn new() -> Self {
        Self
    }

    /// Flatten nested wrapped collections into a single collection
    fn flatten_wrapped_collections(
        &self,
        collections: Vec<WrappedCollection>,
    ) -> WrappedCollection {
        let mut flattened = Vec::new();
        for collection in collections {
            flattened.extend(collection);
        }
        flattened
    }

    /// Determine the common type for a collection of wrapped values
    fn determine_collection_type(&self, values: &[WrappedValue]) -> String {
        if values.is_empty() {
            return "empty".to_string();
        }

        let types: Vec<String> = values.iter().map(|v| v.fhir_type().to_string()).collect();

        type_utils::get_common_type(&types)
    }

    /// Create a unified path for a collection result
    fn create_collection_path(&self, values: &[WrappedValue]) -> CanonicalPath {
        if values.is_empty() {
            return CanonicalPath::empty();
        }

        // If all values have the same base path, use that
        if let Some(first_path) = values.first().map(|v| v.path()) {
            let all_same_base = values
                .iter()
                .all(|v| v.path().parent().as_ref() == first_path.parent().as_ref());

            if all_same_base {
                if let Some(parent) = first_path.parent() {
                    return parent.append_wildcard();
                }
            }
        }

        // Default to empty path for mixed collections
        CanonicalPath::empty()
    }

    /// Update indices for collection elements
    fn update_collection_indices(&self, mut collection: WrappedCollection) -> WrappedCollection {
        let collection_len = collection.len();
        for (i, wrapped) in collection.iter_mut().enumerate() {
            if collection_len > 1 {
                // Update the path with correct index
                let base_path = wrapped
                    .path()
                    .parent()
                    .map(|p| p.clone())
                    .unwrap_or_else(|| CanonicalPath::empty());
                let new_path = base_path.append_index(i);

                wrapped.metadata.path = new_path;
                wrapped.metadata.index = Some(i);
            }
        }
        collection
    }

    /// Filter collection elements based on a boolean condition
    async fn apply_filter_condition(
        &mut self,
        collection: &WrappedCollection,
        condition: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        let mut filtered_results = Vec::new();

        for wrapped_element in collection {
            // Create a new context with this element as the focus
            let element_context = self.create_element_context(context, wrapped_element);

            // Evaluate the condition with this element
            // For now, we'll use a simplified approach - real implementation would
            // need access to the full evaluator to evaluate the condition expression
            let should_include = self
                .evaluate_simple_condition(condition, wrapped_element, &element_context, resolver)
                .await?;

            if should_include {
                filtered_results.push(wrapped_element.clone());
            }
        }

        Ok(filtered_results)
    }

    /// Create an evaluation context focused on a specific element
    fn create_element_context(
        &self,
        base_context: &EvaluationContext,
        element: &WrappedValue,
    ) -> EvaluationContext {
        // Create a new context with the element as the focus
        let element_collection = collection_utils::single(element.clone());
        let plain_collection = crate::core::Collection::from_values(
            element_collection
                .iter()
                .map(|w| w.as_plain().clone())
                .collect(),
        );

        // Create new context with element as start context
        let mut new_context = EvaluationContext::new(plain_collection);

        // Copy variables from original context
        for (name, value) in &base_context.variables {
            new_context.set_variable(name.clone(), value.clone());
        }

        new_context
    }

    /// Simplified condition evaluation (placeholder for full implementation)
    async fn evaluate_simple_condition(
        &self,
        _condition: &ExpressionNode,
        _element: &WrappedValue,
        _context: &EvaluationContext,
        _resolver: &TypeResolver,
    ) -> Result<bool> {
        // This is a placeholder implementation
        // Real implementation would need access to the full evaluator
        // to evaluate the condition expression properly
        Ok(true) // For now, include all elements
    }

    /// Remove duplicate values from collection while preserving metadata
    fn remove_duplicates(&self, collection: WrappedCollection) -> WrappedCollection {
        let mut unique_values = Vec::new();
        let mut seen_values = std::collections::HashSet::new();

        for wrapped in collection {
            // Create a key for deduplication based on the actual value
            let value_key = format!("{:?}", wrapped.as_plain());

            if !seen_values.contains(&value_key) {
                seen_values.insert(value_key);
                unique_values.push(wrapped);
            }
        }

        unique_values
    }
}

#[async_trait]
impl MetadataAwareCollectionEvaluator for MetadataCollectionEvaluator {
    async fn create_collection_with_metadata(
        &self,
        elements: Vec<WrappedCollection>,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Flatten all element collections into a single collection
        let flattened = self.flatten_wrapped_collections(elements);

        // Update indices for proper collection semantics
        let indexed = self.update_collection_indices(flattened);

        Ok(indexed)
    }

    async fn union_collections_with_metadata(
        &self,
        left: &WrappedCollection,
        right: &WrappedCollection,
        _resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Combine both collections
        let mut combined = left.clone();
        combined.extend_from_slice(right);

        // Remove duplicates according to FHIRPath semantics
        let deduplicated = self.remove_duplicates(combined);

        // Update indices
        let indexed = self.update_collection_indices(deduplicated);

        Ok(indexed)
    }

    async fn filter_collection_with_metadata(
        &mut self,
        collection: &WrappedCollection,
        condition: &ExpressionNode,
        context: &EvaluationContext,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        self.apply_filter_condition(collection, condition, context, resolver)
            .await
    }

    fn contains_wrapped_value(&self, collection: &WrappedCollection, value: &WrappedValue) -> bool {
        collection.iter().any(|wrapped| {
            // Compare the actual values (not metadata)
            wrapped.as_plain() == value.as_plain()
        })
    }
}

impl Default for MetadataCollectionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Collection operation utilities for metadata-aware operations
pub mod collection_ops {
    use super::*;

    /// Check if a wrapped collection is empty
    pub fn is_empty(collection: &WrappedCollection) -> bool {
        collection.is_empty()
    }

    /// Get the length of a wrapped collection
    pub fn len(collection: &WrappedCollection) -> usize {
        collection.len()
    }

    /// Get the first wrapped value from a collection
    pub fn first(collection: &WrappedCollection) -> Option<&WrappedValue> {
        collection.first()
    }

    /// Get the last wrapped value from a collection
    pub fn last(collection: &WrappedCollection) -> Option<&WrappedValue> {
        collection.last()
    }

    /// Get a wrapped value at a specific index
    pub fn get(collection: &WrappedCollection, index: usize) -> Option<&WrappedValue> {
        collection.get(index)
    }

    /// Convert a wrapped collection to a plain collection
    pub fn to_plain_collection(collection: WrappedCollection) -> crate::core::Collection {
        let values: Vec<FhirPathValue> = collection.into_iter().map(|w| w.into_plain()).collect();
        crate::core::Collection::from_values(values)
    }

    /// Create a wrapped collection from a single wrapped value
    pub fn single_wrapped(value: WrappedValue) -> WrappedCollection {
        vec![value]
    }

    /// Concatenate multiple wrapped collections
    pub fn concat(collections: Vec<WrappedCollection>) -> WrappedCollection {
        let mut result = Vec::new();
        for collection in collections {
            result.extend(collection);
        }
        result
    }

    /// Sort a wrapped collection by a key function (preserving metadata)
    pub fn sort_by<F, K>(mut collection: WrappedCollection, key_fn: F) -> WrappedCollection
    where
        F: Fn(&WrappedValue) -> K,
        K: Ord,
    {
        collection.sort_by(|a, b| key_fn(a).cmp(&key_fn(b)));
        collection
    }

    /// Map over a wrapped collection while preserving metadata structure
    pub fn map<F>(collection: WrappedCollection, transform_fn: F) -> WrappedCollection
    where
        F: FnMut(WrappedValue) -> WrappedValue,
    {
        collection.into_iter().map(transform_fn).collect()
    }

    /// Filter a wrapped collection while preserving metadata
    pub fn filter<F>(collection: WrappedCollection, predicate: F) -> WrappedCollection
    where
        F: Fn(&WrappedValue) -> bool,
    {
        collection.into_iter().filter(predicate).collect()
    }

    /// Take the first N elements from a wrapped collection
    pub fn take(collection: WrappedCollection, n: usize) -> WrappedCollection {
        collection.into_iter().take(n).collect()
    }

    /// Skip the first N elements from a wrapped collection
    pub fn skip(collection: WrappedCollection, n: usize) -> WrappedCollection {
        collection.into_iter().skip(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{Collection, FhirPathValue},
        evaluator::EvaluationContext,
        path::CanonicalPath,
        typing::TypeResolver,
        wrapped::{ValueMetadata, WrappedValue},
    };
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    fn create_test_resolver() -> TypeResolver {
        let provider = Arc::new(EmptyModelProvider);
        TypeResolver::new(provider)
    }

    #[tokio::test]
    async fn test_create_collection_with_metadata() {
        let evaluator = MetadataCollectionEvaluator::new();
        let resolver = create_test_resolver();

        // Create test elements
        let element1 = vec![WrappedValue::new(
            FhirPathValue::String("John".to_string()),
            ValueMetadata::primitive(
                "string".to_string(),
                CanonicalPath::parse("Patient.name.given").unwrap(),
            ),
        )];

        let element2 = vec![WrappedValue::new(
            FhirPathValue::String("Jane".to_string()),
            ValueMetadata::primitive(
                "string".to_string(),
                CanonicalPath::parse("Patient.name.given").unwrap(),
            ),
        )];

        let elements = vec![element1, element2];

        let result = evaluator
            .create_collection_with_metadata(elements, &resolver)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path().to_string(), "Patient.name.given[0]");
        assert_eq!(result[1].path().to_string(), "Patient.name.given[1]");
    }

    #[tokio::test]
    async fn test_union_collections_with_metadata() {
        let evaluator = MetadataCollectionEvaluator::new();
        let resolver = create_test_resolver();

        // Create test collections
        let left = vec![WrappedValue::new(
            FhirPathValue::String("John".to_string()),
            ValueMetadata::primitive(
                "string".to_string(),
                CanonicalPath::parse("Patient.name.given").unwrap(),
            ),
        )];

        let right = vec![WrappedValue::new(
            FhirPathValue::String("Jane".to_string()),
            ValueMetadata::primitive(
                "string".to_string(),
                CanonicalPath::parse("Patient.name.given").unwrap(),
            ),
        )];

        let result = evaluator
            .union_collections_with_metadata(&left, &right, &resolver)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].fhir_type(), "string");
        assert_eq!(result[1].fhir_type(), "string");
    }

    #[test]
    fn test_contains_wrapped_value() {
        let evaluator = MetadataCollectionEvaluator::new();

        let target_value = WrappedValue::new(
            FhirPathValue::String("John".to_string()),
            ValueMetadata::primitive(
                "string".to_string(),
                CanonicalPath::parse("test.path").unwrap(),
            ),
        );

        let collection = vec![
            WrappedValue::new(
                FhirPathValue::String("John".to_string()),
                ValueMetadata::primitive(
                    "string".to_string(),
                    CanonicalPath::parse("different.path").unwrap(),
                ),
            ),
            WrappedValue::new(
                FhirPathValue::String("Jane".to_string()),
                ValueMetadata::primitive(
                    "string".to_string(),
                    CanonicalPath::parse("test.path").unwrap(),
                ),
            ),
        ];

        // Should find the value based on actual value, not metadata
        assert!(evaluator.contains_wrapped_value(&collection, &target_value));
    }

    #[test]
    fn test_collection_operations() {
        use super::collection_ops;

        let collection = vec![
            WrappedValue::new(
                FhirPathValue::String("First".to_string()),
                ValueMetadata::primitive(
                    "string".to_string(),
                    CanonicalPath::parse("test[0]").unwrap(),
                ),
            ),
            WrappedValue::new(
                FhirPathValue::String("Second".to_string()),
                ValueMetadata::primitive(
                    "string".to_string(),
                    CanonicalPath::parse("test[1]").unwrap(),
                ),
            ),
        ];

        assert_eq!(collection_ops::len(&collection), 2);
        assert!(!collection_ops::is_empty(&collection));

        let first = collection_ops::first(&collection).unwrap();
        assert_eq!(first.path().to_string(), "test[0]");

        let last = collection_ops::last(&collection).unwrap();
        assert_eq!(last.path().to_string(), "test[1]");
    }
}
