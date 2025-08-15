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

//! Intersect function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use std::collections::HashSet;

/// Intersect function: returns the intersection of two collections
pub struct IntersectFunction;

impl IntersectFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("intersect", OperationType::Function)
            .description("Returns the intersection of the input collection and the other collection (items that appear in both collections). Duplicates are removed from the result.")
            .example("Patient.name.given.intersect(Patient.name.family)")
            .example("Bundle.entry.intersect(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for IntersectFunction {
    fn identifier(&self) -> &str {
        "intersect"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            IntersectFunction::create_metadata()
        });
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

        // Fallback to async evaluation (though intersect is always sync)
        self.evaluate_intersect(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_intersect(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IntersectFunction {
    fn evaluate_intersect(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate exactly one argument (the other collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments { message: 
                "intersect() requires exactly one collection argument".to_string()
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Build a set of keys from the right collection for efficient lookup
        let mut right_keys = HashSet::new();
        for item in &right_items {
            let key = self.value_to_comparable_key(item)?;
            right_keys.insert(key);
        }

        // Find items from left collection that are also in right collection
        let mut seen = HashSet::new();
        let mut result_items = Vec::new();

        for item in &left_items {
            let key = self.value_to_comparable_key(item)?;
            // Item must be in right collection and not already added to result
            if right_keys.contains(&key) && seen.insert(key) {
                result_items.push(item.clone());
            }
        }

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

    /// Convert a FhirPathValue to a comparable key for intersection detection
    fn value_to_comparable_key(&self, value: &FhirPathValue) -> Result<String> {
        match value {
            FhirPathValue::String(s) => Ok(format!("string:{}", s.as_ref())),
            FhirPathValue::Integer(i) => Ok(format!("integer:{}", i)),
            FhirPathValue::Decimal(d) => Ok(format!("decimal:{}", d)),
            FhirPathValue::Boolean(b) => Ok(format!("boolean:{}", b)),
            FhirPathValue::Date(d) => Ok(format!("date:{}", d)),
            FhirPathValue::DateTime(dt) => Ok(format!("datetime:{}", dt)),
            FhirPathValue::Time(t) => Ok(format!("time:{}", t)),
            FhirPathValue::JsonValue(json) => Ok(format!("json:{}", json.to_string())),
            FhirPathValue::Collection(_) => {
                // Collections are compared structurally - convert to JSON representation
                Ok(format!("collection:{}", serde_json::to_string(value).map_err(|_| {
                    FhirPathError::InvalidArguments { message: "Cannot serialize collection for comparison".to_string() }
                })?))
            }
            FhirPathValue::Empty => Ok("empty".to_string()),
            FhirPathValue::Quantity(q) => Ok(format!("quantity:{}", q.to_string())),
            FhirPathValue::Resource(r) => {
                let id = r.as_json().get("id").and_then(|v| v.as_str()).unwrap_or("");
                Ok(format!("resource:{}", id))
            },
            FhirPathValue::TypeInfoObject { name, .. } => Ok(format!("typeinfo:{}", name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use std::sync::Arc;
    use serde_json::json;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_intersect_empty_collections() {
        let intersect_fn = IntersectFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection.clone());
        
        let result = intersect_fn.evaluate(&[empty_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_intersect_with_empty() {
        let intersect_fn = IntersectFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(collection);
        
        let result = intersect_fn.evaluate(&[empty_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_intersect_disjoint_collections() {
        let intersect_fn = IntersectFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("c".into()),
            FhirPathValue::String("d".into()),
        ]);
        let context = create_test_context(left_collection);
        
        let result = intersect_fn.evaluate(&[right_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_intersect_overlapping_collections() {
        let intersect_fn = IntersectFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
            FhirPathValue::String("d".into()),
        ]);
        let context = create_test_context(left_collection);
        
        let result = intersect_fn.evaluate(&[right_collection], &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2); // b, c
            assert!(items.contains(&FhirPathValue::String("b".into())));
            assert!(items.contains(&FhirPathValue::String("c".into())));
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_intersect_identical_collections() {
        let intersect_fn = IntersectFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection.clone());
        
        let result = intersect_fn.evaluate(&[collection.clone()], &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3);
            assert!(items.contains(&FhirPathValue::String("a".into())));
            assert!(items.contains(&FhirPathValue::String("b".into())));
            assert!(items.contains(&FhirPathValue::String("c".into())));
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_intersect_with_duplicates_removes_duplicates() {
        let intersect_fn = IntersectFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("b".into()), // duplicate in left
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("b".into()),
            FhirPathValue::String("b".into()), // duplicate in right
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(left_collection);
        
        let result = intersect_fn.evaluate(&[right_collection], &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1); // Only one "b" (duplicates removed)
            assert_eq!(items.get(0).unwrap(), FhirPathValue::String("b".into()));
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_intersect_single_items() {
        let intersect_fn = IntersectFunction::new();
        let single_item = FhirPathValue::String("a".into());
        let another_item = FhirPathValue::String("a".into()); // same item
        let context = create_test_context(single_item.clone());
        
        let result = intersect_fn.evaluate(&[another_item], &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0).unwrap(), single_item);
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_intersect_different_single_items() {
        let intersect_fn = IntersectFunction::new();
        let single_item = FhirPathValue::String("a".into());
        let different_item = FhirPathValue::String("b".into());
        let context = create_test_context(single_item);
        
        let result = intersect_fn.evaluate(&[different_item], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_intersect_mixed_types() {
        let intersect_fn = IntersectFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::Boolean(true),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(42), // common
            FhirPathValue::Boolean(false),
        ]);
        let context = create_test_context(left_collection);
        
        let result = intersect_fn.evaluate(&[right_collection], &context).await.unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0).unwrap(), FhirPathValue::Integer(42));
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_intersect_no_arguments_error() {
        let intersect_fn = IntersectFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);
        
        let result = intersect_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_intersect_too_many_arguments_error() {
        let intersect_fn = IntersectFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);
        
        let result = intersect_fn.evaluate(&[
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into())
        ], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let intersect_fn = IntersectFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(left_collection);

        let sync_result = intersect_fn.try_evaluate_sync(&[right_collection], &context).unwrap().unwrap();
        
        if let FhirPathValue::Collection(items) = sync_result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0).unwrap(), FhirPathValue::String("b".into()));
        } else {
            panic!("Expected collection result");
        }
        assert!(intersect_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let intersect_fn = IntersectFunction::new();
        let metadata = intersect_fn.metadata();

        assert_eq!(metadata.basic.name, "intersect");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
        assert_eq!(metadata.parameters.len(), 1);
    }
}