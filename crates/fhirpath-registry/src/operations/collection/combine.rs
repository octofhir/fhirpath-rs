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

//! Combine function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Combine function: combines two collections without removing duplicates
pub struct CombineFunction;

impl CombineFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("combine", OperationType::Function)
            .description("Combines the input collection with the other collection, preserving all duplicates. Unlike union(), this function does not remove duplicates.")
            .example("Patient.name.given.combine(Patient.name.family)")
            .example("Bundle.entry.combine(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for CombineFunction {
    fn identifier(&self) -> &str {
        "combine"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| CombineFunction::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync path first for performance
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        // Fallback to async evaluation (though combine is always sync)
        self.evaluate_combine(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_combine(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CombineFunction {
    fn evaluate_combine(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (the other collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "combine() requires exactly one collection argument".to_string(),
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Simply concatenate both collections (preserving duplicates)
        let mut result_items = Vec::new();
        result_items.extend(left_items);
        result_items.extend(right_items);

        if result_items.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(result_items))
        }
    }

    /// Convert a FhirPathValue to a vector of items (flattening if it's a collection)
    fn to_collection_items(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.clone_for_mutation(),
            FhirPathValue::Empty => vec![],
            _ => vec![value.clone()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_combine_empty_collections() {
        let combine_fn = CombineFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection.clone());

        let result = combine_fn
            .evaluate(&[empty_collection], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_combine_with_empty() {
        let combine_fn = CombineFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(collection.clone());

        let result = combine_fn
            .evaluate(&[empty_collection], &context)
            .await
            .unwrap();
        assert_eq!(result, collection);
    }

    #[tokio::test]
    async fn test_combine_disjoint_collections() {
        let combine_fn = CombineFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("c".into()),
            FhirPathValue::String("d".into()),
        ]);
        let context = create_test_context(left_collection);

        let result = combine_fn
            .evaluate(&[right_collection], &context)
            .await
            .unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items.get(0).unwrap(), FhirPathValue::String("a".into()));
            assert_eq!(items[1], FhirPathValue::String("b".into()));
            assert_eq!(items[2], FhirPathValue::String("c".into()));
            assert_eq!(items[3], FhirPathValue::String("d".into()));
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_combine_overlapping_collections_preserves_duplicates() {
        let combine_fn = CombineFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("b".into()), // duplicate (should be preserved)
            FhirPathValue::String("c".into()), // duplicate (should be preserved)
            FhirPathValue::String("d".into()),
        ]);
        let context = create_test_context(left_collection);

        let result = combine_fn
            .evaluate(&[right_collection], &context)
            .await
            .unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 6); // a, b, c, b, c, d (duplicates preserved)
            assert_eq!(items.get(0).unwrap(), FhirPathValue::String("a".into()));
            assert_eq!(items[1], FhirPathValue::String("b".into()));
            assert_eq!(items[2], FhirPathValue::String("c".into()));
            assert_eq!(items[3], FhirPathValue::String("b".into())); // duplicate preserved
            assert_eq!(items[4], FhirPathValue::String("c".into())); // duplicate preserved
            assert_eq!(items[5], FhirPathValue::String("d".into()));
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_combine_single_items() {
        let combine_fn = CombineFunction::new();
        let single_item = FhirPathValue::String("a".into());
        let another_item = FhirPathValue::String("b".into());
        let context = create_test_context(single_item.clone());

        let result = combine_fn
            .evaluate(&[another_item.clone()], &context)
            .await
            .unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.get(0).unwrap(), single_item);
            assert_eq!(items[1], another_item);
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_combine_duplicate_single_items_preserves_duplicates() {
        let combine_fn = CombineFunction::new();
        let item = FhirPathValue::String("a".into());
        let context = create_test_context(item.clone());

        let result = combine_fn
            .evaluate(&[item.clone()], &context)
            .await
            .unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2); // Duplicates preserved
            assert_eq!(items.get(0).unwrap(), item);
            assert_eq!(items[1], item);
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_combine_mixed_types() {
        let combine_fn = CombineFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Integer(42), // duplicate (should be preserved)
        ]);
        let context = create_test_context(left_collection);

        let result = combine_fn
            .evaluate(&[right_collection], &context)
            .await
            .unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 4); // test, 42, true, 42 (duplicate preserved)
            assert_eq!(items.get(0).unwrap(), FhirPathValue::String("test".into()));
            assert_eq!(items[1], FhirPathValue::Integer(42));
            assert_eq!(items[2], FhirPathValue::Boolean(true));
            assert_eq!(items[3], FhirPathValue::Integer(42)); // duplicate preserved
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_combine_with_empty_input() {
        let combine_fn = CombineFunction::new();
        let empty_input = FhirPathValue::Empty;
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(empty_input);

        let result = combine_fn
            .evaluate(&[collection.clone()], &context)
            .await
            .unwrap();
        assert_eq!(result, collection);
    }

    #[tokio::test]
    async fn test_combine_no_arguments_error() {
        let combine_fn = CombineFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = combine_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_combine_too_many_arguments_error() {
        let combine_fn = CombineFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = combine_fn
            .evaluate(
                &[
                    FhirPathValue::String("a".into()),
                    FhirPathValue::String("b".into()),
                ],
                &context,
            )
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let combine_fn = CombineFunction::new();
        let left_collection = FhirPathValue::collection(vec![FhirPathValue::String("a".into())]);
        let right_collection = FhirPathValue::collection(vec![FhirPathValue::String("b".into())]);
        let context = create_test_context(left_collection);

        let sync_result = combine_fn
            .try_evaluate_sync(&[right_collection], &context)
            .unwrap()
            .unwrap();

        if let FhirPathValue::Collection(items) = sync_result {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected collection result");
        }
        assert!(combine_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let combine_fn = CombineFunction::new();
        let metadata = combine_fn.metadata();

        assert_eq!(metadata.basic.name, "combine");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
        assert_eq!(metadata.parameters.len(), 1);
    }
}
