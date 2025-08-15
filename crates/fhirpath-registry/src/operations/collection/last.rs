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

//! Last function implementation for FHIRPath

use crate::metadata::{MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Last function: returns a collection containing only the last item in the input collection
pub struct LastFunction;

impl LastFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("last", OperationType::Function)
            .description("Returns a collection containing only the last item in the input collection. If the input collection is empty, returns an empty collection.")
            .example("Patient.name.last()")
            .example("Bundle.entry.last().resource")
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LastFunction {
    fn identifier(&self) -> &str {
        "last"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| LastFunction::create_metadata());
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

        // Fallback to async evaluation (though last is always sync)
        self.evaluate_last(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_last(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LastFunction {
    fn evaluate_last(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments {
                message: "last() takes no arguments".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                if let Some(last_item) = items.last() {
                    Ok(FhirPathValue::collection(vec![last_item.clone()]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            _ => {
                // Single item - return as singleton collection
                Ok(FhirPathValue::collection(vec![context.input.clone()]))
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
    async fn test_last_empty_collection() {
        let last_fn = LastFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);

        let result = last_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![]));
    }

    #[tokio::test]
    async fn test_last_single_item() {
        let last_fn = LastFunction::new();
        let single_item = FhirPathValue::String("test".into());
        let context = create_test_context(single_item.clone());

        let result = last_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![single_item]));
    }

    #[tokio::test]
    async fn test_last_multiple_items() {
        let last_fn = LastFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        let context = create_test_context(collection);

        let result = last_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(
            result,
            FhirPathValue::collection(vec![FhirPathValue::String("third".into())])
        );
    }

    #[tokio::test]
    async fn test_last_with_arguments_error() {
        let last_fn = LastFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::String("test".into())]);
        let context = create_test_context(collection);

        let result = last_fn
            .evaluate(&[FhirPathValue::Integer(1)], &context)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let last_fn = LastFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = last_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(
            sync_result,
            FhirPathValue::collection(vec![FhirPathValue::String("second".into())])
        );
        assert!(last_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let last_fn = LastFunction::new();
        let metadata = last_fn.metadata();

        assert_eq!(metadata.basic.name, "last");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}
