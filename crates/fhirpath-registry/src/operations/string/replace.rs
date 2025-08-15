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

//! Replace function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Replace function: replaces all instances of a pattern with a substitution
pub struct ReplaceFunction;

impl ReplaceFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("replace", OperationType::Function)
            .description("Replaces all instances of pattern with substitution in the input string")
            .example("'hello world'.replace('world', 'universe')")
            .example("Patient.name.family.replace(' ', '')")
            .parameter("pattern", TypeConstraint::Specific(FhirPathType::String), false)
            .parameter("substitution", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ReplaceFunction {
    fn identifier(&self) -> &str {
        "replace"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ReplaceFunction::create_metadata()
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

        self.evaluate_replace(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_replace(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ReplaceFunction {
    fn evaluate_replace(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 2 {
            return Err(FhirPathError::EvaluationError {
                message: "replace() requires exactly two arguments (pattern, substitution)".to_string(),
            });
        }

        // Handle collection inputs
        let input = &context.input;
        
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![])));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![])));
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
        // Convert input to string (including numeric values)
        let input_str = match value {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))),
            _ => return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))), // Return empty for other non-convertible types
        };

        // Extract and convert pattern parameter to string (handle collections)
        let pattern = self.extract_string_from_value(&args[0])?;
        if pattern.is_none() {
            return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))); // Return empty for non-convertible types
        }
        let pattern = pattern.unwrap();

        // Extract and convert substitution parameter to string (handle collections)
        let substitution = self.extract_string_from_value(&args[1])?;
        if substitution.is_none() {
            return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))); // Return empty for non-convertible types
        }
        let substitution = substitution.unwrap();

        // Perform string replacement
        let result = if pattern.is_empty() {
            // Special case: empty pattern means insert substitution between every character
            let chars: Vec<char> = input_str.chars().collect();
            if chars.is_empty() {
                substitution.clone()
            } else {
                let mut result = String::with_capacity(input_str.len() + substitution.len() * (chars.len() + 1));
                result.push_str(&substitution);
                for ch in chars {
                    result.push(ch);
                    result.push_str(&substitution);
                }
                result
            }
        } else {
            input_str.replace(&pattern, &substitution)
        };
        
        Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
            FhirPathValue::String(result.into())
        ])))
    }

    /// Extract a string from a FhirPathValue, handling collections and type conversion
    fn extract_string_from_value(&self, value: &FhirPathValue) -> Result<Option<String>> {
        match value {
            FhirPathValue::String(s) => Ok(Some(s.as_ref().to_string())),
            FhirPathValue::Integer(i) => Ok(Some(i.to_string())),
            FhirPathValue::Decimal(d) => Ok(Some(d.to_string())),
            FhirPathValue::Boolean(b) => Ok(Some(b.to_string())),
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(None)
                } else if items.len() == 1 {
                    // Single element collection - recursively extract
                    self.extract_string_from_value(items.first().unwrap())
                } else {
                    // Multiple elements - can't convert
                    Ok(None)
                }
            }
            _ => Ok(None), // Other types can't be converted
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
    async fn test_replace_function() {
        let replace_fn = ReplaceFunction::new();

        // Test basic replacement
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("world".into()),
            FhirPathValue::String("universe".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello universe".into()));

        // Test multiple replacements
        let string = FhirPathValue::String("hello world world".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("world".into()),
            FhirPathValue::String("universe".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello universe universe".into()));

        // Test replacing with empty string (removal)
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String(" ".into()),
            FhirPathValue::String("".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("helloworld".into()));

        // Test no matches
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("xyz".into()),
            FhirPathValue::String("abc".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test empty pattern (should not change anything per typical behavior)
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test with unicode characters
        let string = FhirPathValue::String("héllo wörld".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("ö".into()),
            FhirPathValue::String("o".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("héllo world".into()));

        // Test overlapping patterns (should replace first occurrence)
        let string = FhirPathValue::String("aaaa".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("aa".into()),
            FhirPathValue::String("b".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("bb".into()));

        // Test case sensitivity
        let string = FhirPathValue::String("Hello World".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("hi".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Hello World".into())); // No match due to case

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::String("replacement".into()),
        ];
        let result = replace_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let replace_fn = ReplaceFunction::new();
        let string = FhirPathValue::String("test string".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::String("demo".into()),
        ];

        let sync_result = replace_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("demo string".into()));
        assert!(replace_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let replace_fn = ReplaceFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::String("replacement".into()),
        ];
        let result = replace_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string pattern argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::Integer(42),
            FhirPathValue::String("replacement".into()),
        ];
        let result = replace_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string substitution argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
        ];
        let result = replace_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("test".into())];
        let result = replace_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}