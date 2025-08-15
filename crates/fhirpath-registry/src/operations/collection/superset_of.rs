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

//! SupersetOf function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use std::collections::HashSet;

/// SupersetOf function: returns true if the input collection is a superset of the other collection
pub struct SupersetOfFunction;

impl SupersetOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("supersetOf", OperationType::Function)
            .description("Returns true if the input collection is a superset of the other collection (all items in the other collection are also in the input collection). Any collection is a superset of an empty collection.")
            .example("Patient.name.given.supersetOf(Patient.name.family)")
            .example("Bundle.entry.supersetOf(Bundle.contained)")
            .parameter("other", TypeConstraint::Specific(FhirPathType::Collection), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SupersetOfFunction {
    fn identifier(&self) -> &str {
        "supersetOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SupersetOfFunction::create_metadata()
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

        // Fallback to async evaluation (though supersetOf is always sync)
        self.evaluate_superset_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_superset_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SupersetOfFunction {
    fn evaluate_superset_of(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate exactly one argument (the subset collection)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments { message: 
                "supersetOf() requires exactly one collection argument".to_string()
            });
        }

        let other = &args[0];

        // Convert both inputs to collections
        let left_items = self.to_collection_items(&context.input);
        let right_items = self.to_collection_items(other);

        // Any collection is a superset of empty collection
        if right_items.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Build a set of keys from the left collection for efficient lookup
        let mut left_keys = HashSet::new();
        for item in &left_items {
            let key = self.value_to_comparable_key(item)?;
            left_keys.insert(key);
        }

        // Check if all items from right collection are in left collection
        for item in &right_items {
            let key = self.value_to_comparable_key(item)?;
            if !left_keys.contains(&key) {
                // Found an item in right that's not in left - not a superset
                return Ok(FhirPathValue::Boolean(false));
            }
        }

        // All items in right are in left - it's a superset
        Ok(FhirPathValue::Boolean(true))
    }

    /// Convert a FhirPathValue to a vector of items (flattening if it's a collection)
    fn to_collection_items(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.clone_for_mutation(),
            FhirPathValue::Empty => vec![],
            _ => vec![value.clone()],
        }
    }

    /// Convert a FhirPathValue to a comparable key for superset detection
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

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_superset_of_empty_collections() {
        let superset_of_fn = SupersetOfFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection.clone());
        
        let result = superset_of_fn.evaluate(&[empty_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Empty is superset of empty
    }

    #[tokio::test]
    async fn test_superset_of_non_empty_with_empty() {
        let superset_of_fn = SupersetOfFunction::new();
        let non_empty_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
        ]);
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(non_empty_collection);
        
        let result = superset_of_fn.evaluate(&[empty_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Any collection is superset of empty
    }

    #[tokio::test]
    async fn test_superset_of_empty_with_non_empty() {
        let superset_of_fn = SupersetOfFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let non_empty_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
        ]);
        let context = create_test_context(empty_collection);
        
        let result = superset_of_fn.evaluate(&[non_empty_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // Empty is not superset of non-empty
    }

    #[tokio::test]
    async fn test_superset_of_identical_collections() {
        let superset_of_fn = SupersetOfFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(collection.clone());
        
        let result = superset_of_fn.evaluate(&[collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Collection is superset of itself
    }

    #[tokio::test]
    async fn test_superset_of_true_superset() {
        let superset_of_fn = SupersetOfFunction::new();
        let superset_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let subset_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(superset_collection);
        
        let result = superset_of_fn.evaluate(&[subset_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_superset_of_false_superset() {
        let superset_of_fn = SupersetOfFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("d".into()), // Not in left collection
        ]);
        let context = create_test_context(left_collection);
        
        let result = superset_of_fn.evaluate(&[right_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_superset_of_disjoint_collections() {
        let superset_of_fn = SupersetOfFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("x".into()),
            FhirPathValue::String("y".into()),
        ]);
        let context = create_test_context(left_collection);
        
        let result = superset_of_fn.evaluate(&[right_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_superset_of_with_duplicates() {
        let superset_of_fn = SupersetOfFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("a".into()), // duplicate
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
        ]);
        let context = create_test_context(left_collection);
        
        let result = superset_of_fn.evaluate(&[right_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Duplicates don't affect superset relationship
    }

    #[tokio::test]
    async fn test_superset_of_single_items() {
        let superset_of_fn = SupersetOfFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let single_item = FhirPathValue::String("a".into());
        let context = create_test_context(collection);
        
        let result = superset_of_fn.evaluate(&[single_item], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_superset_of_single_item_not_in_collection() {
        let superset_of_fn = SupersetOfFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let single_item = FhirPathValue::String("x".into());
        let context = create_test_context(collection);
        
        let result = superset_of_fn.evaluate(&[single_item], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_superset_of_mixed_types() {
        let superset_of_fn = SupersetOfFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::Boolean(true),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
        ]);
        let context = create_test_context(left_collection);
        
        let result = superset_of_fn.evaluate(&[right_collection], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_superset_of_no_arguments_error() {
        let superset_of_fn = SupersetOfFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);
        
        let result = superset_of_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_superset_of_too_many_arguments_error() {
        let superset_of_fn = SupersetOfFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);
        
        let result = superset_of_fn.evaluate(&[
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into())
        ], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let superset_of_fn = SupersetOfFunction::new();
        let left_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let right_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
        ]);
        let context = create_test_context(left_collection);

        let sync_result = superset_of_fn.try_evaluate_sync(&[right_collection], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(superset_of_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let superset_of_fn = SupersetOfFunction::new();
        let metadata = superset_of_fn.metadata();

        assert_eq!(metadata.basic.name, "supersetOf");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
        assert_eq!(metadata.parameters.len(), 1);
    }
}