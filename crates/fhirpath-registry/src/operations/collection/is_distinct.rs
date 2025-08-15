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

//! IsDistinct function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use std::collections::HashSet;

/// IsDistinct function: returns true if all items in the collection are distinct
pub struct IsDistinctFunction;

impl IsDistinctFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("isDistinct", OperationType::Function)
            .description("Returns true if all items in the collection are distinct (no duplicates). Returns true for empty collections and single-item collections.")
            .example("Patient.name.given.isDistinct()")
            .example("Bundle.entry.resource.id.isDistinct()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linearithmic, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for IsDistinctFunction {
    fn identifier(&self) -> &str {
        "isDistinct"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            IsDistinctFunction::create_metadata()
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

        // Fallback to async evaluation (though isDistinct is always sync)
        self.evaluate_is_distinct(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_is_distinct(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IsDistinctFunction {
    fn evaluate_is_distinct(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments { message: 
                "isDistinct() takes no arguments".to_string()
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Empty or single item collections are always distinct
                if items.len() <= 1 {
                    return Ok(FhirPathValue::Boolean(true));
                }

                // Use HashSet to check for duplicates
                let mut seen = HashSet::new();
                for item in items.iter() {
                    // Convert item to a comparable representation
                    let key = self.value_to_comparable_key(item)?;
                    if !seen.insert(key) {
                        // Duplicate found
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                // All items are distinct
                Ok(FhirPathValue::Boolean(true))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => {
                // Single item is always distinct
                Ok(FhirPathValue::Boolean(true))
            }
        }
    }

    /// Convert a FhirPathValue to a comparable key for duplicate detection
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
    async fn test_is_distinct_empty_collection() {
        let is_distinct_fn = IsDistinctFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);
        
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_is_distinct_single_item() {
        let is_distinct_fn = IsDistinctFunction::new();
        let single_item = FhirPathValue::String("test".into());
        let context = create_test_context(single_item);
        
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_is_distinct_all_distinct() {
        let is_distinct_fn = IsDistinctFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection);
        
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_is_distinct_with_duplicates() {
        let is_distinct_fn = IsDistinctFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("a".into()), // duplicate
        ]);
        let context = create_test_context(collection);
        
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_is_distinct_mixed_types_distinct() {
        let is_distinct_fn = IsDistinctFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_is_distinct_numbers() {
        let is_distinct_fn = IsDistinctFunction::new();
        
        // All distinct numbers
        let distinct_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let context = create_test_context(distinct_collection);
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // With duplicate numbers
        let duplicate_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1), // duplicate
        ]);
        let context = create_test_context(duplicate_collection);
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_is_distinct_with_objects() {
        let is_distinct_fn = IsDistinctFunction::new();
        
        // Distinct objects
        let distinct_objects = FhirPathValue::collection(vec![
            FhirPathValue::JsonValue(json!({"name": "John"})),
            FhirPathValue::JsonValue(json!({"name": "Jane"})),
        ]);
        let context = create_test_context(distinct_objects);
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Duplicate objects
        let duplicate_objects = FhirPathValue::collection(vec![
            FhirPathValue::JsonValue(json!({"name": "John"})),
            FhirPathValue::JsonValue(json!({"name": "Jane"})),
            FhirPathValue::JsonValue(json!({"name": "John"})), // duplicate
        ]);
        let context = create_test_context(duplicate_objects);
        let result = is_distinct_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_is_distinct_with_arguments_error() {
        let is_distinct_fn = IsDistinctFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);
        
        let result = is_distinct_fn.evaluate(&[FhirPathValue::Boolean(true)], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let is_distinct_fn = IsDistinctFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = is_distinct_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(is_distinct_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let is_distinct_fn = IsDistinctFunction::new();
        let metadata = is_distinct_fn.metadata();

        assert_eq!(metadata.basic.name, "isDistinct");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}