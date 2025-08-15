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

//! Lower function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// Lower function: converts string to lowercase
pub struct LowerFunction;

impl LowerFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("lower", OperationType::Function)
            .description("Returns the string with all characters converted to lowercase")
            .example("'HELLO WORLD'.lower()")
            .example("Patient.name.family.lower()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LowerFunction {
    fn identifier(&self) -> &str {
        "lower"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            LowerFunction::create_metadata()
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

        self.evaluate_lower(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_lower(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LowerFunction {
    fn evaluate_lower(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                message: "lower() takes no arguments".to_string(),
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
                let lower_str = s.as_ref().to_lowercase();
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::String(lower_str.into())
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "lower() requires input to be a string".to_string(),
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
        use crate::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_lower_function() {
        let lower_fn = LowerFunction::new();

        // Test basic case
        let string = FhirPathValue::String("HELLO WORLD".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test mixed case
        let string = FhirPathValue::String("Hello World".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test already lowercase
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with numbers and symbols
        let string = FhirPathValue::String("HELLO123!@#".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello123!@#".into()));

        // Test empty string
        let string = FhirPathValue::String("".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));

        // Test with unicode characters
        let string = FhirPathValue::String("HÉLLO WÖRLD".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("héllo wörld".into()));

        // Test with non-Latin characters
        let string = FhirPathValue::String("HELLO 世界".into());
        let context = create_test_context(string);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello 世界".into()));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = lower_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let lower_fn = LowerFunction::new();
        let string = FhirPathValue::String("TEST STRING".into());
        let context = create_test_context(string);

        let sync_result = lower_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("test string".into()));
        assert!(lower_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let lower_fn = LowerFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let result = lower_fn.try_evaluate_sync(&[], &context).unwrap();
        assert!(result.is_err());

        // Test with arguments (should be none)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("invalid".into())];
        let result = lower_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}