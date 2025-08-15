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

//! Matches function implementation for FHIRPath

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

/// Matches function: returns true when the value matches the given regular expression
pub struct MatchesFunction;

impl MatchesFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("matches", OperationType::Function)
            .description("Returns true when the value matches the given regular expression")
            .example("'123'.matches('\\\\d+')")
            .example("Patient.name.family.matches('[A-Z][a-z]+')")
            .parameter("regex", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MatchesFunction {
    fn identifier(&self) -> &str {
        "matches"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            MatchesFunction::create_metadata()
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

        self.evaluate_matches(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_matches(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MatchesFunction {
    fn evaluate_matches(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "matches() requires exactly one argument (regex)".to_string(),
            });
        }

        // Get regex pattern parameter - handle both direct strings and collections containing strings
        let pattern = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.first().unwrap() {
                    FhirPathValue::String(s) => s,
                    _ => return Err(FhirPathError::EvaluationError {
                        message: "matches() regex parameter must be a string".to_string(),
                    }),
                }
            },
            FhirPathValue::Collection(items) if items.is_empty() => return Ok(FhirPathValue::Empty),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FhirPathError::EvaluationError {
                message: "matches() regex parameter must be a string".to_string(),
            }),
        };

        // If pattern is empty string, return empty per spec
        if pattern.as_ref().is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Compile regex with single-line mode (per FHIRPath spec - dot matches newlines)
        let pattern_with_flags = if pattern.as_ref().contains("(?") && pattern.as_ref().contains('s') {
            // Pattern already has single-line flag set
            pattern.as_ref().to_string()
        } else {
            // Add single-line flag to enable . to match newlines
            format!("(?s){}", pattern.as_ref())
        };

        let regex = Regex::new(&pattern_with_flags).map_err(|e| {
            FhirPathError::EvaluationError {
                message: format!("Invalid regex pattern '{}': {}", pattern.as_ref(), e),
            }
        })?;

        // Handle different input types
        match &context.input {
            FhirPathValue::String(s) => {
                let matches = regex.is_match(s.as_ref());
                Ok(FhirPathValue::Boolean(matches))
            }
            FhirPathValue::Collection(collection) => {
                let mut results = Vec::new();
                for value in collection.iter() {
                    match value {
                        FhirPathValue::String(s) => {
                            let matches = regex.is_match(s.as_ref());
                            results.push(FhirPathValue::Boolean(matches));
                        }
                        FhirPathValue::Empty => {
                            // Empty values are skipped in collections
                        }
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                                message: "matches() can only be applied to strings".to_string(),
                            });
                        }
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    use octofhir_fhirpath_model::Collection;
                    Ok(FhirPathValue::Collection(Collection::from_vec(results)))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "matches() can only be applied to strings or collections containing strings".to_string(),
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
    async fn test_matches_function() {
        let matches_fn = MatchesFunction::new();

        // Test basic digit pattern
        let string = FhirPathValue::String("123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test negative case - letters don't match digits
        let string = FhirPathValue::String("abc".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test partial match
        let string = FhirPathValue::String("hello123world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test exact match with anchors
        let string = FhirPathValue::String("123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("^\\d+$".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test exact match failure
        let string = FhirPathValue::String("123abc".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("^\\d+$".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test case insensitive pattern
        let string = FhirPathValue::String("Hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("(?i)hello".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test character class
        let string = FhirPathValue::String("Hello123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("[A-Za-z]+\\d+".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test word boundaries
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\bworld\\b".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test empty string with empty pattern
        let string = FhirPathValue::String("".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("^$".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test unicode support
        let string = FhirPathValue::String("héllo wörld 世界".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(".*世界.*".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test email pattern
        let string = FhirPathValue::String("test@example.com".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let matches_fn = MatchesFunction::new();
        let string = FhirPathValue::String("test123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("test\\d+".into())];

        let sync_result = matches_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(matches_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let matches_fn = MatchesFunction::new();

        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string regex argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(42)];
        let result = matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with invalid regex pattern
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("[".into())]; // Invalid regex
        let result = matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![];
        let result = matches_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}
