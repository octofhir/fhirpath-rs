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

//! Skip function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Skip function: returns a collection skipping the first num items
pub struct SkipFunction;

impl SkipFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("skip", OperationType::Function)
            .description("Returns a collection containing all but the first num items in the input collection. If num is negative or zero, returns the entire collection. If num is greater than the collection length, returns an empty collection.")
            .example("Patient.name.skip(1)")
            .example("Bundle.entry.skip(5)")
            .parameter("num", TypeConstraint::Specific(FhirPathType::Integer), false)
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SkipFunction {
    fn identifier(&self) -> &str {
        "skip"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SkipFunction::create_metadata()
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

        // Fallback to async evaluation (though skip is always sync)
        self.evaluate_skip(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_skip(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SkipFunction {
    fn evaluate_skip(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments { message:
                "skip() requires exactly one integer argument".to_string()
            });
        }
        // Extract the skip count
        let skip_count = match &args[0] {
            FhirPathValue::Integer(n) => *n,
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.iter().next().unwrap() {
                    FhirPathValue::Integer(n) => *n,
                    _ => return Err(FhirPathError::InvalidArguments { message:
                    "take() argument must be an integer".to_string()
                    }),
                }
            },
            _ => return Err(FhirPathError::InvalidArguments { message:
                "skip() argument must be an integer".to_string()
            }),
        };

        // Handle negative or zero skip count
        if skip_count <= 0 {
            return Ok(context.input.clone());
        }

        let skip_count = skip_count as usize;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if skip_count >= items.len() {
                    Ok(FhirPathValue::collection(vec![]))
                } else {
                    Ok(FhirPathValue::collection(items.as_arc().as_ref()[skip_count..].to_vec()))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            _ => {
                // Single item - skip it if skip_count > 0, otherwise return it
                if skip_count > 0 {
                    Ok(FhirPathValue::collection(vec![]))
                } else {
                    Ok(context.input.clone())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_skip_empty_collection() {
        let skip_fn = SkipFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);

        let result = skip_fn.evaluate(&[FhirPathValue::Integer(2)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_skip_single_item() {
        let skip_fn = SkipFunction::new();
        let single_item = FhirPathValue::String("test".into());
        let context = create_test_context(single_item.clone());

        // Skip 0 should return the item
        let result = skip_fn.evaluate(&[FhirPathValue::Integer(0)], &context).await.unwrap();
        assert_eq!(result, single_item);

        // Skip 1 should return empty
        let result = skip_fn.evaluate(&[FhirPathValue::Integer(1)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_skip_multiple_items() {
        let skip_fn = SkipFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
            FhirPathValue::String("fourth".into()),
        ]);
        let context = create_test_context(collection.clone());

        // Skip 0 should return entire collection
        let result = skip_fn.evaluate(&[FhirPathValue::Integer(0)], &context).await.unwrap();
        assert_eq!(result, collection);

        // Skip 2 should return last two items
        let result = skip_fn.evaluate(&[FhirPathValue::Integer(2)], &context).await.unwrap();
        let expected = FhirPathValue::collection(vec![
            FhirPathValue::String("third".into()),
            FhirPathValue::String("fourth".into()),
        ]);
        assert_eq!(result, expected);

        // Skip more than collection size should return empty
        let result = skip_fn.evaluate(&[FhirPathValue::Integer(10)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_skip_negative_count() {
        let skip_fn = SkipFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ]);
        let context = create_test_context(collection.clone());

        // Negative skip should return entire collection
        let result = skip_fn.evaluate(&[FhirPathValue::Integer(-5)], &context).await.unwrap();
        assert_eq!(result, collection);
    }

    #[tokio::test]
    async fn test_skip_no_arguments_error() {
        let skip_fn = SkipFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = skip_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_skip_wrong_argument_type_error() {
        let skip_fn = SkipFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = skip_fn.evaluate(&[FhirPathValue::String("not_a_number".into())], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let skip_fn = SkipFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = skip_fn.try_evaluate_sync(&[FhirPathValue::Integer(1)], &context).unwrap().unwrap();
        let expected = FhirPathValue::collection(vec![
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        assert_eq!(sync_result, expected);
        assert!(skip_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let skip_fn = SkipFunction::new();
        let metadata = skip_fn.metadata();

        assert_eq!(metadata.basic.name, "skip");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
        assert_eq!(metadata.types.parameters.len(), 1);
    }
}
