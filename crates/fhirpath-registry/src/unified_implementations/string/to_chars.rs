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

//! Unified toChars() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use std::sync::Arc;

/// Unified toChars() function implementation
///
/// Returns the list of characters in the input string
pub struct UnifiedToCharsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToCharsFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toChars", FunctionCategory::StringOperations)
            .display_name("To Characters")
            .description("Returns the list of characters in the input string")
            .example("'hello'.toChars()")
            .example("'ab'.toChars()")
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Exact(TypeInfo::String))))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("toChars()")
            .keywords(vec!["toChars", "characters", "string", "split", "chars"])
            .usage_pattern(
                "Split string into characters",
                "string.toChars()",
                "String manipulation and character-based operations"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToCharsFunction {
    fn name(&self) -> &str {
        "toChars"
    }

    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }

    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }

    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate no arguments - this is a member function
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }

        // Get the input collection from context
        let input = &context.input;

        match input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "toChars() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                let item = items.first().unwrap();
                match item {
                    FhirPathValue::String(s) => {
                        let chars: Vec<FhirPathValue> = s
                            .chars()
                            .map(|c| FhirPathValue::String(Arc::from(c.to_string())))
                            .collect();
                        Ok(FhirPathValue::Collection(chars.into()))
                    },
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            _ => {
                // Single item case
                match input {
                    FhirPathValue::String(s) => {
                        let chars: Vec<FhirPathValue> = s
                            .chars()
                            .map(|c| FhirPathValue::String(Arc::from(c.to_string())))
                            .collect();
                        Ok(FhirPathValue::Collection(chars.into()))
                    },
                    _ => Ok(FhirPathValue::Empty),
                }
            }
        }
    }

    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_to_chars_function() {
        let to_chars_func = UnifiedToCharsFunction::new();

        // Test basic string
        let context = EvaluationContext::new(FhirPathValue::String("abc".to_string()));
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], FhirPathValue::String(Arc::from("a")));
                assert_eq!(items[1], FhirPathValue::String(Arc::from("b")));
                assert_eq!(items[2], FhirPathValue::String(Arc::from("c")));
            },
            _ => panic!("Expected Collection result"),
        }

        // Test from the specification test case
        let context = EvaluationContext::new(FhirPathValue::String("t2".to_string()));
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], FhirPathValue::String(Arc::from("t")));
                assert_eq!(items[1], FhirPathValue::String(Arc::from("2")));
            },
            _ => panic!("Expected Collection result"),
        }

        // Test empty string
        let context = EvaluationContext::new(FhirPathValue::String("".to_string()));
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 0);
            },
            _ => panic!("Expected Collection result"),
        }

        // Test single character
        let context = EvaluationContext::new(FhirPathValue::String("x".to_string()));
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0], FhirPathValue::String(Arc::from("x")));
            },
            _ => panic!("Expected Collection result"),
        }

        // Test unicode characters
        let context = EvaluationContext::new(FhirPathValue::String("🔥👍".to_string()));
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], FhirPathValue::String(Arc::from("🔥")));
                assert_eq!(items[1], FhirPathValue::String(Arc::from("👍")));
            },
            _ => panic!("Expected Collection result"),
        }

        // Test non-string input (should return empty)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = to_chars_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("test".to_string()));
        let result = to_chars_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(to_chars_func.name(), "toChars");
        assert_eq!(to_chars_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(to_chars_func.metadata().basic.display_name, "To Characters");
        assert!(to_chars_func.metadata().basic.is_pure);
    }
}