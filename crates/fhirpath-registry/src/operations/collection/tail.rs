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

//! Tail function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Tail function: returns all but the first item in the input collection
pub struct TailFunction;

impl TailFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("tail", OperationType::Function)
            .description("Returns all but the first item in the input collection. If the input collection is empty or has only one item, returns an empty collection.")
            .example("Patient.name.tail()")
            .example("Bundle.entry.tail()")
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TailFunction {
    fn identifier(&self) -> &str {
        "tail"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            TailFunction::create_metadata()
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

        // Fallback to async evaluation (though tail is always sync)
        self.evaluate_tail(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_tail(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TailFunction {
    fn evaluate_tail(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments { message: 
                "tail() takes no arguments".to_string()
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() <= 1 {
                    Ok(FhirPathValue::collection(vec![]))
                } else {
                    Ok(FhirPathValue::collection(items.as_arc().as_ref()[1..].to_vec()))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            _ => {
                // Single item - return empty collection (tail of single item is empty)
                Ok(FhirPathValue::collection(vec![]))
            }
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
    async fn test_tail_empty_collection() {
        let tail_fn = TailFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);
        
        let result = tail_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_tail_single_item() {
        let tail_fn = TailFunction::new();
        let single_item = FhirPathValue::String("test".into());
        let context = create_test_context(single_item);
        
        let result = tail_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_tail_single_item_collection() {
        let tail_fn = TailFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("only".into())]);
        let context = create_test_context(collection);
        
        let result = tail_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_tail_multiple_items() {
        let tail_fn = TailFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        let context = create_test_context(collection);
        
        let result = tail_fn.evaluate(&[], &context).await.unwrap();
        let expected = FhirPathValue::collection(vec![
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_tail_with_arguments_error() {
        let tail_fn = TailFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);
        
        let result = tail_fn.evaluate(&[FhirPathValue::Integer(1)], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let tail_fn = TailFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = tail_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        let expected = FhirPathValue::collection(vec![
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        assert_eq!(sync_result, expected);
        assert!(tail_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let tail_fn = TailFunction::new();
        let metadata = tail_fn.metadata();

        assert_eq!(metadata.basic.name, "tail");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}