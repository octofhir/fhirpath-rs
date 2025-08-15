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

//! All function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// All function: returns true if criteria is true for all items in the collection
pub struct AllFunction;

impl AllFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("all", OperationType::Function)
            .description("Returns true if the criteria evaluates to true for all items in the collection. Returns true for an empty collection.")
            .example("Patient.name.all(use = 'official')")
            .example("Bundle.entry.all(resource.resourceType = 'Patient')")
            .parameter("criteria", TypeConstraint::Specific(FhirPathType::Boolean), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn to_boolean(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Empty => Ok(false),
            FhirPathValue::Boolean(b) => Ok(*b),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(false)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    Ok(true) // Non-empty collection is truthy
                }
            }
            _ => Ok(true), // Non-empty, non-boolean values are truthy
        }
    }
}

#[async_trait]
impl FhirPathOperation for AllFunction {
    fn identifier(&self) -> &str {
        "all"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| AllFunction::create_metadata());
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

        // Fallback to async evaluation (though all is always sync)
        self.evaluate_all(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_all(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AllFunction {
    fn evaluate_all(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (the criteria)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "all() requires exactly one criteria argument".to_string(),
            });
        }

        let criteria = &args[0];

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Empty collection - all() returns true for empty collections
                if items.is_empty() {
                    return Ok(FhirPathValue::Boolean(true));
                }

                // Check criteria for each item
                for item in items.iter() {
                    // Create new context for each item
                    let item_context = EvaluationContext::new(item.clone(), context.registry.clone(), context.model_provider.clone());

                    // For mock implementation, evaluate criteria as simple boolean check
                    // In real implementation, this would evaluate the lambda expression
                    let criteria_result = match criteria {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::String(s) if s.as_ref() == "true" => true,
                        FhirPathValue::String(s) if s.as_ref() == "false" => false,
                        _ => {
                            // Mock: if criteria is a string that matches a field in the item, check if that field exists
                            if let (
                                FhirPathValue::String(field_name),
                                FhirPathValue::JsonValue(obj),
                            ) = (criteria, item)
                            {
                                obj.as_object()
                                    .map(|o| o.contains_key(field_name.as_ref()))
                                    .unwrap_or(false)
                            } else {
                                Self::to_boolean(criteria)?
                            }
                        }
                    };

                    // If any item doesn't satisfy criteria, return false
                    if !criteria_result {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                // All items satisfied the criteria
                Ok(FhirPathValue::Boolean(true))
            }
            FhirPathValue::Empty => {
                // Empty collection - all() returns true
                Ok(FhirPathValue::Boolean(true))
            }
            single_item => {
                // Single item - check criteria against it
                let criteria_result = Self::to_boolean(criteria)?;
                Ok(FhirPathValue::Boolean(criteria_result))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use serde_json::json;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_all_empty_collection() {
        let all_fn = AllFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);

        let result = all_fn
            .evaluate(&[FhirPathValue::Boolean(true)], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = all_fn
            .evaluate(&[FhirPathValue::Boolean(false)], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_all_single_item() {
        let all_fn = AllFunction::new();
        let single_item = FhirPathValue::String("test".into());
        let context = create_test_context(single_item);

        // True criteria should return true
        let result = all_fn
            .evaluate(&[FhirPathValue::Boolean(true)], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // False criteria should return false
        let result = all_fn
            .evaluate(&[FhirPathValue::Boolean(false)], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_all_multiple_items_boolean_criteria() {
        let all_fn = AllFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("item1".into()),
            FhirPathValue::String("item2".into()),
            FhirPathValue::String("item3".into()),
        ]);
        let context = create_test_context(collection.clone());

        // All items should satisfy true criteria
        let result = all_fn
            .evaluate(&[FhirPathValue::Boolean(true)], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // No items should satisfy false criteria
        let result = all_fn
            .evaluate(&[FhirPathValue::Boolean(false)], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_all_with_objects() {
        let all_fn = AllFunction::new();

        // Collection where all objects have the specified field
        let all_have_field = FhirPathValue::collection(vec![
            FhirPathValue::JsonValue(json!({"name": "John", "active": true})),
            FhirPathValue::JsonValue(json!({"name": "Jane", "active": false})),
            FhirPathValue::JsonValue(json!({"name": "Bob", "active": true})),
        ]);
        let context = create_test_context(all_have_field);

        // Check if all objects have "active" field
        let result = all_fn
            .evaluate(&[FhirPathValue::String("active".into())], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Check if all objects have "missing" field
        let result = all_fn
            .evaluate(&[FhirPathValue::String("missing".into())], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_all_mixed_objects() {
        let all_fn = AllFunction::new();

        // Collection where some objects have the field, some don't
        let mixed_collection = FhirPathValue::collection(vec![
            FhirPathValue::JsonValue(json!({"name": "John", "active": true})),
            FhirPathValue::JsonValue(json!({"name": "Jane"})), // missing "active"
            FhirPathValue::JsonValue(json!({"name": "Bob", "active": true})),
        ]);
        let context = create_test_context(mixed_collection);

        // Not all objects have "active" field
        let result = all_fn
            .evaluate(&[FhirPathValue::String("active".into())], &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_all_no_arguments_error() {
        let all_fn = AllFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = all_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_all_too_many_arguments_error() {
        let all_fn = AllFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = all_fn
            .evaluate(
                &[FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)],
                &context,
            )
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let all_fn = AllFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("item1".into()),
            FhirPathValue::String("item2".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = all_fn
            .try_evaluate_sync(&[FhirPathValue::Boolean(true)], &context)
            .unwrap()
            .unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(all_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let all_fn = AllFunction::new();
        let metadata = all_fn.metadata();

        assert_eq!(metadata.basic.name, "all");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
        assert_eq!(metadata.parameters.len(), 1);
    }
}
