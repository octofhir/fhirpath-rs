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

//! Substring function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Substring function: returns the part of the string starting at position start (zero-based). If length is given, will return at most length number of characters
pub struct SubstringFunction;

impl SubstringFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("substring", OperationType::Function)
            .description("Returns the part of the string starting at position start (zero-based). If length is given, will return at most length number of characters")
            .example("'hello world'.substring(6)")
            .example("'hello world'.substring(0, 5)")
            .parameter("start", TypeConstraint::Specific(FhirPathType::Integer), false)
            .parameter("length", TypeConstraint::Specific(FhirPathType::Integer), true)
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SubstringFunction {
    fn identifier(&self) -> &str {
        "substring"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SubstringFunction::create_metadata()
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

        self.evaluate_substring(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_substring(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SubstringFunction {
    fn evaluate_substring(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::EvaluationError {
                message: "substring() requires 1 or 2 arguments (start, optional length)".to_string(),
            });
        }

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Empty);
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                return self.process_single_value(value, args);
            }
            _ => {
                // Process as single value
                return self.process_single_value(input, args);
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, args: &[FhirPathValue]) -> Result<FhirPathValue> {
        // Get input string
        let input_str = match value {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FhirPathError::EvaluationError {
                message: "substring() requires input to be a string".to_string(),
            }),
        };

        // Get start position - handle both direct integers and collections containing integers
        let start = match &args[0] {
            FhirPathValue::Integer(i) => {
                if *i < 0 {
                    return Ok(FhirPathValue::Empty); // FHIRPath spec: negative start returns empty
                }
                *i as usize
            },
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.first().unwrap() {
                    FhirPathValue::Integer(i) => {
                        if *i < 0 {
                            return Ok(FhirPathValue::Empty); // FHIRPath spec: negative start returns empty
                        }
                        *i as usize
                    },
                    _ => return Ok(FhirPathValue::Empty), // Invalid start parameter returns empty
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty), // Empty start returns empty
            _ => return Ok(FhirPathValue::Empty), // Invalid start parameter returns empty
        };

        // Get optional length
        let length = if args.len() == 2 {
            match &args[1] {
                FhirPathValue::Integer(i) => {
                    if *i < 0 {
                        return Ok(FhirPathValue::Empty); // FHIRPath spec: negative length returns empty
                    }
                    Some(*i as usize)
                },
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.first().unwrap() {
                        FhirPathValue::Integer(i) => {
                            if *i < 0 {
                                return Ok(FhirPathValue::Empty); // FHIRPath spec: negative length returns empty
                            }
                            Some(*i as usize)
                        },
                        _ => return Ok(FhirPathValue::Empty), // Invalid length parameter returns empty
                    }
                },
                FhirPathValue::Empty => return Ok(FhirPathValue::Empty), // Empty length returns empty
                _ => return Ok(FhirPathValue::Empty), // Invalid length parameter returns empty
            }
        } else {
            None
        };

        // Convert string to char indices for proper Unicode handling
        let chars: Vec<char> = input_str.as_ref().chars().collect();

        // Handle out of bounds start - return empty per FHIRPath spec
        if start >= chars.len() {
            return Ok(FhirPathValue::Empty);
        }

        // Calculate substring using character indices
        let result = if let Some(len) = length {
            let end = std::cmp::min(start + len, chars.len());
            chars[start..end].iter().collect::<String>()
        } else {
            chars[start..].iter().collect::<String>()
        };

        Ok(FhirPathValue::String(result.into()))
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
    async fn test_substring_function() {
        let substring_fn = SubstringFunction::new();

        // Test basic case with length
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(6), FhirPathValue::Integer(5)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("world".into()));

        // Test without length (to end of string)
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(6)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("world".into()));

        // Test start at beginning
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(5)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));

        // Test out of bounds start
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(10)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));

        // Test length beyond string length
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(100)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));

        // Test with unicode characters
        let string = FhirPathValue::String("héllo 世界".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(6), FhirPathValue::Integer(2)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("世界".into()));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::Integer(0)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let substring_fn = SubstringFunction::new();
        let string = FhirPathValue::String("test string".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(6)];

        let sync_result = substring_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("string".into()));
        assert!(substring_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let substring_fn = SubstringFunction::new();
        
        // Test with negative start (should return empty per FHIRPath spec)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(-1)];
        let result = substring_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with negative length (should return empty per FHIRPath spec)
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(-1)];
        let result = substring_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::Integer(0)];
        let result = substring_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong argument count
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![];
        let result = substring_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}