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

//! Take function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Take function: returns a collection containing only the first num items
pub struct TakeFunction;

impl TakeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("take", OperationType::Function)
            .description("Returns a collection containing only the first num items in the input collection. If num is negative or zero, returns an empty collection. If num is greater than the collection length, returns the entire collection.")
            .example("Patient.name.take(2)")
            .example("Bundle.entry.take(10)")
            .parameter("num", TypeConstraint::Specific(FhirPathType::Integer), false)
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TakeFunction {
    fn identifier(&self) -> &str {
        "take"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            TakeFunction::create_metadata()
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

        // Fallback to async evaluation (though take is always sync)
        self.evaluate_take(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_take(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TakeFunction {
    fn evaluate_take(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {

        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments { message:
                "take() requires exactly one integer argument".to_string()
            });
        }

        // Extract the take count - handle both single values and collections with single values
        let take_count = match &args[0] {
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
                "take() argument must be an integer".to_string()
            }),
        };

        // Handle negative or zero take count
        if take_count <= 0 {
            return Ok(FhirPathValue::collection(vec![]));
        }

        let take_count = take_count as usize;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if take_count >= items.len() {
                    Ok(context.input.clone())
                } else {
                    Ok(FhirPathValue::collection(items.as_arc().as_ref()[..take_count].to_vec()))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            _ => {
                // Single item - take it if take_count > 0, otherwise return empty
                if take_count > 0 {
                    Ok(FhirPathValue::collection(vec![context.input.clone()]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
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
    async fn test_take_empty_collection() {
        let take_fn = TakeFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);

        let result = take_fn.evaluate(&[FhirPathValue::Integer(2)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_take_single_item() {
        let take_fn = TakeFunction::new();
        let single_item = FhirPathValue::String("test".into());
        let context = create_test_context(single_item.clone());

        // Take 0 should return empty
        let result = take_fn.evaluate(&[FhirPathValue::Integer(0)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));

        // Take 1 should return the item as collection
        let result = take_fn.evaluate(&[FhirPathValue::Integer(1)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![single_item]));
    }

    #[tokio::test]
    async fn test_take_multiple_items() {
        let take_fn = TakeFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
            FhirPathValue::String("fourth".into()),
        ]);
        let context = create_test_context(collection.clone());

        // Take 0 should return empty collection
        let result = take_fn.evaluate(&[FhirPathValue::Integer(0)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));

        // Take 2 should return first two items
        let result = take_fn.evaluate(&[FhirPathValue::Integer(2)], &context).await.unwrap();
        let expected = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ]);
        assert_eq!(result, expected);

        // Take more than collection size should return entire collection
        let result = take_fn.evaluate(&[FhirPathValue::Integer(10)], &context).await.unwrap();
        assert_eq!(result, collection);
    }

    #[tokio::test]
    async fn test_take_negative_count() {
        let take_fn = TakeFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ]);
        let context = create_test_context(collection);

        // Negative take should return empty collection
        let result = take_fn.evaluate(&[FhirPathValue::Integer(-5)], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_take_no_arguments_error() {
        let take_fn = TakeFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = take_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_take_wrong_argument_type_error() {
        let take_fn = TakeFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = take_fn.evaluate(&[FhirPathValue::String("not_a_number".into())], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let take_fn = TakeFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = take_fn.try_evaluate_sync(&[FhirPathValue::Integer(2)], &context).unwrap().unwrap();
        let expected = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ]);
        assert_eq!(sync_result, expected);
        assert!(take_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let take_fn = TakeFunction::new();
        let metadata = take_fn.metadata();

        assert_eq!(metadata.basic.name, "take");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}
