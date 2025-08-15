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

//! Length function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// Length function: returns the length of a string
pub struct LengthFunction;

impl LengthFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("length", OperationType::Function)
            .description("Returns the number of characters in a string")
            .example("Patient.name.given.first().length()")
            .example("'hello world'.length()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LengthFunction {
    fn identifier(&self) -> &str {
        "length"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            LengthFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        self.evaluate_length(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_length(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LengthFunction {
    fn evaluate_length(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                message: "length() takes no arguments".to_string(),
            });
        }

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                return self.process_single_value(value);
            }
            _ => {
                // Process as single value
                return self.process_single_value(input);
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                let length = s.chars().count() as i64;
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Integer(length)
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "length() can only be called on string values".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_length_function() {
        let length_fn = LengthFunction::new();

        // Test with string
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(11));

        // Test with empty string
        let empty_string = FhirPathValue::String("".into());
        let context = create_test_context(empty_string);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test with unicode characters
        let unicode_string = FhirPathValue::String("héllo 世界".into());
        let context = create_test_context(unicode_string);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(8));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with non-string should error
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let result = length_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let length_fn = LengthFunction::new();
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);

        let sync_result = length_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Integer(4));
        assert!(length_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let length_fn = LengthFunction::new();
        let metadata = length_fn.metadata();

        assert_eq!(metadata.basic.name, "length");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}