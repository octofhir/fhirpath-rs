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

//! Count function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// Count function: returns the number of items in a collection
pub struct CountFunction;

impl CountFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("count", OperationType::Function)
            .description("Returns the number of items in a collection")
            .example("Patient.name.count()")
            .example("Bundle.entry.count()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for CountFunction {
    fn identifier(&self) -> &str {
        "count"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| CountFunction::create_metadata());
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

        // Fallback to async evaluation (though count is always sync)
        match &context.input {
            FhirPathValue::Collection(collection) => {
                Ok(FhirPathValue::Integer(collection.len() as i64))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match &context.input {
            FhirPathValue::Collection(collection) => {
                Ok(FhirPathValue::Integer(collection.len() as i64))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
    async fn test_count_function() {
        let count_fn = CountFunction::new();

        // Test with collection
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection);
        let result = count_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test with empty collection
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = count_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test with single value
        let single = FhirPathValue::String("test".into());
        let context = create_test_context(single);
        let result = count_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
    }

    #[test]
    fn test_sync_evaluation() {
        let count_fn = CountFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(collection);

        let sync_result = count_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Integer(2));
        assert!(count_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let count_fn = CountFunction::new();
        let metadata = count_fn.metadata();

        assert_eq!(metadata.basic.name, "count");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}
