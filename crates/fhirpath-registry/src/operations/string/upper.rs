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

//! Upper function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// Upper function: converts string to uppercase
pub struct UpperFunction;

impl UpperFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("upper", OperationType::Function)
            .description("Returns the string with all characters converted to uppercase")
            .example("'hello world'.upper()")
            .example("Patient.name.family.upper()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for UpperFunction {
    fn identifier(&self) -> &str {
        "upper"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            UpperFunction::create_metadata()
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

        self.evaluate_upper(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_upper(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UpperFunction {
    fn evaluate_upper(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                message: "upper() takes no arguments".to_string(),
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
                let upper_str = s.as_ref().to_uppercase();
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::String(upper_str.into())
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "upper() requires input to be a string".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::MockModelProvider;
        use crate::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_upper_function() {
        let upper_fn = UpperFunction::new();

        // Test basic case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HELLO WORLD".into())
        ])));

        // Test mixed case
        let string = FhirPathValue::String("Hello World".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HELLO WORLD".into())
        ])));

        // Test already uppercase
        let string = FhirPathValue::String("HELLO WORLD".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HELLO WORLD".into())
        ])));

        // Test with numbers and symbols
        let string = FhirPathValue::String("hello123!@#".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HELLO123!@#".into())
        ])));

        // Test empty string
        let string = FhirPathValue::String("".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("".into())
        ])));

        // Test with unicode characters
        let string = FhirPathValue::String("héllo wörld".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HÉLLO WÖRLD".into())
        ])));

        // Test with non-Latin characters
        let string = FhirPathValue::String("hello 世界".into());
        let context = create_test_context(string);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HELLO 世界".into())
        ])));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let upper_fn = UpperFunction::new();
        let string = FhirPathValue::String("test string".into());
        let context = create_test_context(string);

        let sync_result = upper_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("TEST STRING".into())
        ])));
        assert!(upper_fn.supports_sync());
    }

    #[tokio::test]
    async fn test_collection_handling() {
        let upper_fn = UpperFunction::new();

        // Test single-item collection
        let single_item_collection = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("hello".into())
        ]));
        let context = create_test_context(single_item_collection);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("HELLO".into())
        ])));

        // Test empty collection
        let empty_collection = FhirPathValue::Collection(Collection::from(vec![]));
        let context = create_test_context(empty_collection);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test multi-item collection (should return empty per FHIRPath spec)
        let multi_item_collection = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into())
        ]));
        let context = create_test_context(multi_item_collection);
        let result = upper_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_error_conditions() {
        let upper_fn = UpperFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let result = upper_fn.try_evaluate_sync(&[], &context).unwrap();
        assert!(result.is_err());

        // Test with arguments (should be none)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("invalid".into())];
        let result = upper_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}