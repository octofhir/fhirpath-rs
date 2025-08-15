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

//! ToChars function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// ToChars function: returns the list of characters in the input string
pub struct ToCharsFunction;

impl ToCharsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toChars", OperationType::Function)
            .description("Returns the list of characters in the input string")
            .example("'abc'.toChars()")
            .example("Patient.name.family.toChars()")
            .returns(TypeConstraint::Collection(Box::new(TypeConstraint::Specific(FhirPathType::String))))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ToCharsFunction {
    fn identifier(&self) -> &str {
        "toChars"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ToCharsFunction::create_metadata()
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

        self.evaluate_to_chars(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_to_chars(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ToCharsFunction {
    fn evaluate_to_chars(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                message: "toChars() takes no arguments".to_string(),
            });
        }

        // Get input string from context - handle both single strings and collections with single strings
        match &context.input {
            FhirPathValue::String(s) => {
                // Convert each character to a FhirPathValue::String and collect into a collection
                let chars: Vec<FhirPathValue> = s
                    .as_ref()
                    .chars()
                    .map(|c| FhirPathValue::String(c.to_string().into()))
                    .collect();
                
                Ok(FhirPathValue::Collection(chars.into()))
            },
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.iter().next().unwrap() {
                    FhirPathValue::String(s) => {
                        // Convert each character to a FhirPathValue::String and collect into a collection
                        let chars: Vec<FhirPathValue> = s
                            .as_ref()
                            .chars()
                            .map(|c| FhirPathValue::String(c.to_string().into()))
                            .collect();
                        
                        Ok(FhirPathValue::Collection(chars.into()))
                    },
                    _ => Err(FhirPathError::EvaluationError {
                        message: "toChars() requires input to be a string".to_string(),
                    }),
                }
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "toChars() requires input to be a string".to_string(),
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
    async fn test_to_chars_function() {
        let to_chars_fn = ToCharsFunction::new();

        // Test basic case
        let string = FhirPathValue::String("abc".into());
        let context = create_test_context(string);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        assert_eq!(result, expected);

        // Test empty string
        let string = FhirPathValue::String("".into());
        let context = create_test_context(string);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::new()));

        // Test single character
        let string = FhirPathValue::String("x".into());
        let context = create_test_context(string);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("x".into()),
        ]);
        assert_eq!(result, expected);

        // Test with spaces and special characters
        let string = FhirPathValue::String("a b!".into());
        let context = create_test_context(string);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String(" ".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("!".into()),
        ]);
        assert_eq!(result, expected);

        // Test with unicode characters
        let string = FhirPathValue::String("hé世".into());
        let context = create_test_context(string);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("h".into()),
            FhirPathValue::String("é".into()),
            FhirPathValue::String("世".into()),
        ]);
        assert_eq!(result, expected);

        // Test with emojis (multi-byte characters)
        let string = FhirPathValue::String("a🎉b".into());
        let context = create_test_context(string);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("🎉".into()),
            FhirPathValue::String("b".into()),
        ]);
        assert_eq!(result, expected);

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = to_chars_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let to_chars_fn = ToCharsFunction::new();
        let string = FhirPathValue::String("xyz".into());
        let context = create_test_context(string);

        let sync_result = to_chars_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("x".into()),
            FhirPathValue::String("y".into()),
            FhirPathValue::String("z".into()),
        ]);
        assert_eq!(sync_result, expected);
        assert!(to_chars_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let to_chars_fn = ToCharsFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let result = to_chars_fn.try_evaluate_sync(&[], &context).unwrap();
        assert!(result.is_err());

        // Test with arguments (should be none)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("invalid".into())];
        let result = to_chars_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}