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

//! Trim function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// Trim function: removes leading and trailing whitespace from a string
pub struct TrimFunction;

impl TrimFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("trim", OperationType::Function)
            .description("Returns the string with leading and trailing whitespace removed")
            .example("'  hello world  '.trim()")
            .example("Patient.name.family.trim()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TrimFunction {
    fn identifier(&self) -> &str {
        "trim"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            TrimFunction::create_metadata()
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

        self.evaluate_trim(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_trim(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TrimFunction {
    fn evaluate_trim(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                message: "trim() takes no arguments".to_string(),
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
                let trimmed_str = s.as_ref().trim().to_string();
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::String(trimmed_str.into())
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "trim() requires input to be a string".to_string(),
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
    async fn test_trim_function() {
        let trim_fn = TrimFunction::new();

        // Test basic case with leading and trailing spaces
        let string = FhirPathValue::String("  hello world  ".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with leading spaces only
        let string = FhirPathValue::String("  hello world".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with trailing spaces only
        let string = FhirPathValue::String("hello world  ".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with no whitespace to trim
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with only whitespace
        let string = FhirPathValue::String("   ".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));

        // Test empty string
        let string = FhirPathValue::String("".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));

        // Test with different types of whitespace (tabs, newlines)
        let string = FhirPathValue::String("\t\n hello world \r\n\t".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with internal whitespace (should be preserved)
        let string = FhirPathValue::String("  hello   world  ".into());
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello   world".into()));

        // Test with unicode whitespace
        let string = FhirPathValue::String("\u{00A0}hello\u{00A0}".into()); // Non-breaking spaces
        let context = create_test_context(string);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = trim_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let trim_fn = TrimFunction::new();
        let string = FhirPathValue::String("  test string  ".into());
        let context = create_test_context(string);

        let sync_result = trim_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("test string".into()));
        assert!(trim_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let trim_fn = TrimFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let result = trim_fn.try_evaluate_sync(&[], &context).unwrap();
        assert!(result.is_err());

        // Test with arguments (should be none)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("invalid".into())];
        let result = trim_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}