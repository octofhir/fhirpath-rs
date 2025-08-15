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

//! IndexOf function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// IndexOf function: returns the 0-based index of the first position substring is found in the input string, or -1 if not found
pub struct IndexOfFunction;

impl IndexOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("indexOf", OperationType::Function)
            .description("Returns the 0-based index of the first position substring is found in the input string, or -1 if it is not found")
            .example("'hello world'.indexOf('world')")
            .example("'abcdef'.indexOf('cd')")
            .parameter("substring", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for IndexOfFunction {
    fn identifier(&self) -> &str {
        "indexOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            IndexOfFunction::create_metadata()
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

        self.evaluate_index_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_index_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IndexOfFunction {
    fn evaluate_index_of(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "indexOf() requires exactly one argument (substring)".to_string(),
            });
        }

        // Get substring parameter first - handle both direct strings and single-element collections
        let substring = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                match items.len() {
                    0 => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
                    1 => {
                        match items.first().unwrap() {
                            FhirPathValue::String(s) => s,
                            _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))), 
                        }
                    },
                    _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))), // Multiple values not allowed
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))), // Non-string parameters result in empty
        };

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
                return self.process_single_value(value, substring);
            }
            _ => {
                // Process as single value
                return self.process_single_value(input, substring);
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, substring: &str) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                // Handle empty substring (returns 0 per spec)
                if substring.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![
                        FhirPathValue::Integer(0)
                    ])));
                }

                // Find index
                let index = match s.find(substring) {
                    Some(idx) => idx as i64,
                    None => -1,
                };

                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Integer(index)
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Ok(FhirPathValue::Collection(Collection::from(vec![]))), // Non-string values result in empty
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
    async fn test_index_of_function() {
        let index_of_fn = IndexOfFunction::new();

        // Test basic case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("world".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(6)])));

        // Test substring at beginning
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(0)])));

        // Test substring not found
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("xyz".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(-1)])));

        // Test empty substring
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(0)])));

        // Test with unicode
        let string = FhirPathValue::String("héllo 世界".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("世界".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(7)])));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String("test".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test with non-string input (should return empty)
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::String("test".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test with non-string parameter (should return empty)
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(42)];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(Collection::from(vec![]));
        let context = create_test_context(empty_collection);
        let args = vec![FhirPathValue::String("test".into())];
        let result = index_of_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let index_of_fn = IndexOfFunction::new();
        let string = FhirPathValue::String("test string".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("string".into())];

        let sync_result = index_of_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(5)])));
        assert!(index_of_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let index_of_fn = IndexOfFunction::new();
        
        // Test with non-string input (should return empty, not error)
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::String("test".into())];
        let result = index_of_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test with wrong number of arguments (should error)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![];
        let result = index_of_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with too many arguments (should error)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("a".into()), FhirPathValue::String("b".into())];
        let result = index_of_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}