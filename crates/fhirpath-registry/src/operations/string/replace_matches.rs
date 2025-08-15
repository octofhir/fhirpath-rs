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

//! ReplaceMatches function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use regex::Regex;

/// ReplaceMatches function: replaces all instances matching a regex pattern with a substitution string
pub struct ReplaceMatchesFunction;

impl ReplaceMatchesFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("replaceMatches", OperationType::Function)
            .description("Replaces all instances matching a regex pattern with a substitution string, supporting capture groups")
            .example("'hello 123 world 456'.replaceMatches('\\\\d+', 'X')")
            .example("'John Doe'.replaceMatches('(\\\\w+) (\\\\w+)', '$2, $1')")
            .parameter("regex", TypeConstraint::Specific(FhirPathType::String), false)
            .parameter("substitution", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ReplaceMatchesFunction {
    fn identifier(&self) -> &str {
        "replaceMatches"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ReplaceMatchesFunction::create_metadata()
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

        self.evaluate_replace_matches(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_replace_matches(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ReplaceMatchesFunction {
    fn evaluate_replace_matches(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 2 {
            return Err(FhirPathError::EvaluationError {
                message: "replaceMatches() requires exactly two arguments (regex, substitution)".to_string(),
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

        // Special case: empty pattern should return the original string unchanged for replaceMatches
        if pattern.is_empty() {
            return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::String(input_str.into())
            ])));
        }

        // Compile regex
        let regex = Regex::new(&pattern).map_err(|e| {
            FhirPathError::EvaluationError {
                message: format!("Invalid regex pattern '{}': {}", pattern, e),
            }
        })?;

        // Perform regex replacement
        let result = regex.replace_all(&input_str, &substitution);
        Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
            FhirPathValue::String(result.to_string().into())
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
    async fn test_replace_matches_function() {
        let replace_matches_fn = ReplaceMatchesFunction::new();

        // Test basic replacement of digits
        let string = FhirPathValue::String("hello 123 world 456".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello X world X".into()));

        // Test replacement with capture groups
        let string = FhirPathValue::String("John Doe".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("(\\w+) (\\w+)".into()),
            FhirPathValue::String("$2, $1".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Doe, John".into()));

        // Test no matches
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Test replace with empty string (removal)
        let string = FhirPathValue::String("hello123world456".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("helloworld".into()));

        // Test multiple capture groups
        let string = FhirPathValue::String("2023-12-25".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("(\\d{4})-(\\d{2})-(\\d{2})".into()),
            FhirPathValue::String("$3/$2/$1".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("25/12/2023".into()));

        // Test case insensitive replacement
        let string = FhirPathValue::String("Hello WORLD test".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("(?i)hello|world".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("X X test".into()));

        // Test replacing word boundaries
        let string = FhirPathValue::String("cat catch cats".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("\\bcat\\b".into()),
            FhirPathValue::String("dog".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("dog catch dogs".into()));

        // Test with unicode characters
        let string = FhirPathValue::String("héllo wörld 世界".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("世界".into()),
            FhirPathValue::String("universe".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("héllo wörld universe".into()));

        // Test replace all occurrences
        let string = FhirPathValue::String("aaa bbb aaa ccc aaa".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("aaa".into()),
            FhirPathValue::String("XXX".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("XXX bbb XXX ccc XXX".into()));

        // Test with phone number formatting
        let string = FhirPathValue::String("1234567890".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("(\\d{3})(\\d{3})(\\d{4})".into()),
            FhirPathValue::String("($1) $2-$3".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("(123) 456-7890".into()));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let replace_matches_fn = ReplaceMatchesFunction::new();
        let string = FhirPathValue::String("test123demo".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("-".into()),
        ];

        let sync_result = replace_matches_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("test-demo".into()));
        assert!(replace_matches_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let replace_matches_fn = ReplaceMatchesFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string regex argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::Integer(42),
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string substitution argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::Integer(42),
        ];
        let result = replace_matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with invalid regex pattern
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![
            FhirPathValue::String("[".into()), // Invalid regex
            FhirPathValue::String("X".into()),
        ];
        let result = replace_matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = replace_matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}