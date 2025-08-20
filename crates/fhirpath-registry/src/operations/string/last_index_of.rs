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

//! LastIndexOf function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// LastIndexOf function - finds the last occurrence of a substring
#[derive(Debug, Clone)]
pub struct LastIndexOfFunction;

impl Default for LastIndexOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl LastIndexOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("lastIndexOf", OperationType::Function)
            .description("Find the last occurrence of a substring in a string")
            .example("'hello world'.lastIndexOf('l')")
            .example("Patient.name.family.lastIndexOf('-')")
            .parameter(
                "substring",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LastIndexOfFunction {
    fn identifier(&self) -> &str {
        "lastIndexOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(LastIndexOfFunction::create_metadata);
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

        self.evaluate_last_index_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_last_index_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LastIndexOfFunction {
    fn evaluate_last_index_of(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "lastIndexOf() requires exactly one argument (substring)".to_string(),
            });
        }

        // Get substring parameter
        let substring = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => match items.len() {
                0 => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
                1 => match items.first().unwrap() {
                    FhirPathValue::String(s) => s,
                    _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
                },
                _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "lastIndexOf() argument must be a string".to_string(),
                });
            }
        };

        // Handle collection inputs - process each element in collection
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }

                let mut all_results = Vec::new();
                for item in items.iter() {
                    match self.process_single_value(item, substring)? {
                        FhirPathValue::Collection(results) => {
                            all_results.append(&mut results.into_iter().collect());
                        }
                        single_result => all_results.push(single_result),
                    }
                }
                Ok(FhirPathValue::Collection(Collection::from(all_results)))
            }
            _ => {
                // Process as single value
                self.process_single_value(input, substring)
            }
        }
    }

    fn process_single_value(
        &self,
        input_value: &FhirPathValue,
        substring: &str,
    ) -> Result<FhirPathValue> {
        match input_value {
            FhirPathValue::String(text) => {
                let index = if substring.is_empty() {
                    // Empty substring returns length of string (FHIRPath spec behavior)
                    text.chars().count() as i64
                } else {
                    text.rfind(substring)
                        .map(|i| {
                            // Convert byte index to character index for Unicode support
                            text[..i].chars().count() as i64
                        })
                        .unwrap_or(-1)
                };
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Integer(index),
                ])))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => {
                // Non-string values result in error
                Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "lastIndexOf() can only be called on String values".to_string(),
                })
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
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_last_index_of_basic() {
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        let args = vec![FhirPathValue::String("l".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(9)); // last 'l' in "hello world"
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_substring() {
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        let args = vec![FhirPathValue::String("world".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(6)); // "world" starts at index 6
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_not_found() {
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        let args = vec![FhirPathValue::String("xyz".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(-1)); // not found
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_empty_substring() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::String("".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(5)); // length of string
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_empty_string() {
        let context = create_test_context(FhirPathValue::String("".into()));
        let args = vec![FhirPathValue::String("x".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(-1)); // not found
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_multiple_occurrences() {
        let context = create_test_context(FhirPathValue::String("abcdefabcdef".into()));
        let args = vec![FhirPathValue::String("abc".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(6)); // last occurrence
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_unicode() {
        let context = create_test_context(FhirPathValue::String("café résumé".into()));
        let args = vec![FhirPathValue::String("é".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(10)); // last é in "café résumé"
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_collection() {
        let collection = FhirPathValue::Collection(
            vec![
                FhirPathValue::String("hello world".into()),
                FhirPathValue::String("goodbye world".into()),
            ]
            .into(),
        );
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("o".into())];
        let result = LastIndexOfFunction::new()
            .evaluate(&args, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(7)); // "hello world"
                assert_eq!(values.get(1).unwrap(), &FhirPathValue::Integer(9)); // "goodbye world" - last 'o' in "world"
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_last_index_of_error_on_non_string_input() {
        let context = create_test_context(FhirPathValue::Integer(123));
        let args = vec![FhirPathValue::String("1".into())];
        let result = LastIndexOfFunction::new().evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_last_index_of_error_on_non_string_argument() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::Integer(123)];
        let result = LastIndexOfFunction::new().evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_last_index_of_error_wrong_argument_count() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = LastIndexOfFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());

        let args = vec![
            FhirPathValue::String("l".into()),
            FhirPathValue::String("o".into()),
        ];
        let result = LastIndexOfFunction::new().evaluate(&args, &context).await;
        assert!(result.is_err());
    }
}
