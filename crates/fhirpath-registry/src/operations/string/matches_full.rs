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

//! MatchesFull function implementation for FHIRPath

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

/// MatchesFull function: returns true when the entire value matches the given regular expression
pub struct MatchesFullFunction;

impl MatchesFullFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("matchesFull", OperationType::Function)
            .description("Returns true if the entire input string matches the provided regular expression pattern")
            .example("'123'.matchesFull('\\\\d+')")
            .example("'hello123world'.matchesFull('\\\\d+') // returns false")
            .parameter("regex", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MatchesFullFunction {
    fn identifier(&self) -> &str {
        "matchesFull"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            MatchesFullFunction::create_metadata()
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

        self.evaluate_matches_full(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_matches_full(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MatchesFullFunction {
    fn evaluate_matches_full(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "matchesFull() requires exactly one argument (regex)".to_string(),
            });
        }

        // Get regex pattern parameter - handle both direct strings and collections containing strings
        let pattern = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.first().unwrap() {
                    FhirPathValue::String(s) => s,
                    _ => return Err(FhirPathError::EvaluationError {
                        message: "matchesFull() regex parameter must be a string".to_string(),
                    }),
                }
            },
            _ => return Err(FhirPathError::EvaluationError {
                message: "matchesFull() regex parameter must be a string".to_string(),
            }),
        };

        // If pattern is empty string, return empty per spec
        if pattern.as_ref().is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Handle different input types according to FHIRPath spec
        match &context.input {
            // Single string - normal case
            FhirPathValue::String(s) => {
                // Ensure the pattern matches the entire string by anchoring it
                let anchored_pattern = if pattern.as_ref().starts_with('^') && pattern.as_ref().ends_with('$') {
                    pattern.as_ref().to_string()
                } else if pattern.as_ref().starts_with('^') {
                    format!("{}$", pattern.as_ref())
                } else if pattern.as_ref().ends_with('$') {
                    format!("^{}", pattern.as_ref())
                } else {
                    format!("^{}$", pattern.as_ref())
                };

                let regex = Regex::new(&anchored_pattern).map_err(|e| {
                    FhirPathError::EvaluationError {
                        message: format!("Invalid regex pattern '{}': {}", pattern.as_ref(), e),
                    }
                })?;

                let matches = regex.is_match(s.as_ref());
                Ok(FhirPathValue::Boolean(matches))
            }
            // Empty input collection - return empty per spec
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            // Collection with items - check spec requirements
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    // Empty collection - return empty per spec
                    Ok(FhirPathValue::Empty)
                } else if collection.len() > 1 {
                    // Multiple items - signal error per spec
                    return Err(FhirPathError::EvaluationError {
                        message: "matchesFull() evaluation ended - input collection contains multiple items".to_string(),
                    });
                } else {
                    // Single item in collection - evaluate it
                    match collection.first().unwrap() {
                        FhirPathValue::String(s) => {
                            let anchored_pattern = if pattern.as_ref().starts_with('^') && pattern.as_ref().ends_with('$') {
                                pattern.as_ref().to_string()
                            } else if pattern.as_ref().starts_with('^') {
                                format!("{}$", pattern.as_ref())
                            } else if pattern.as_ref().ends_with('$') {
                                format!("^{}", pattern.as_ref())
                            } else {
                                format!("^{}$", pattern.as_ref())
                            };

                            let regex = Regex::new(&anchored_pattern).map_err(|e| {
                                FhirPathError::EvaluationError {
                                    message: format!("Invalid regex pattern '{}': {}", pattern.as_ref(), e),
                                }
                            })?;

                            let matches = regex.is_match(s.as_ref());
                            Ok(FhirPathValue::Boolean(matches))
                        }
                        FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                        _ => Err(FhirPathError::EvaluationError {
                            message: "matchesFull() can only be applied to strings".to_string(),
                        }),
                    }
                }
            }
            _ => Err(FhirPathError::EvaluationError {
                message: "matchesFull() can only be applied to strings or collections containing strings".to_string(),
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
    async fn test_matches_full_function() {
        let matches_full_fn = MatchesFullFunction::new();

        // Test full match - should pass
        let string = FhirPathValue::String("123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test partial match - should fail (this is the key difference from matches())
        let string = FhirPathValue::String("hello123world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test cases from the FHIRPath spec test suite
        // testMatchesFullWithinUrl1: 'library' should not match full URL
        let string = FhirPathValue::String("http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("library".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // testMatchesFullWithinUrl3: 'Library' should not match full URL
        let string = FhirPathValue::String("http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("Library".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // testMatchesFullWithinUrl4: '^Library$' should not match full URL
        let string = FhirPathValue::String("http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("^Library$".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // testMatchesFullWithinUrl1a: '.*Library.*' should match full URL
        let string = FhirPathValue::String("http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(".*Library.*".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // testMatchesFullWithinUrl2: 'Measure' should not match full URL
        let string = FhirPathValue::String("http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("Measure".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test pattern that already has anchors
        let string = FhirPathValue::String("123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("^\\d+$".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test spec examples
        // 'N8000123123'.matchesFull('N[0-9]{8}') should return false (10 digits, not 8)
        let string = FhirPathValue::String("N8000123123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("N[0-9]{8}".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // 'N8000123123'.matchesFull('N[0-9]{10}') should return true
        let string = FhirPathValue::String("N8000123123".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("N[0-9]{10}".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_sync_evaluation() {
        let matches_full_fn = MatchesFullFunction::new();
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("test".into())];

        assert!(matches_full_fn.supports_sync());
        let sync_result = matches_full_fn.try_evaluate_sync(&args, &context);
        assert!(sync_result.is_some());
        assert_eq!(sync_result.unwrap().unwrap(), FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_error_conditions() {
        let matches_full_fn = MatchesFullFunction::new();
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);

        // Test wrong number of arguments
        let args = vec![];
        let result = matches_full_fn.evaluate(&args, &context).await;
        assert!(result.is_err());

        let args = vec![
            FhirPathValue::String("pattern".into()),
            FhirPathValue::String("extra".into()),
        ];
        let result = matches_full_fn.evaluate(&args, &context).await;
        assert!(result.is_err());

        // Test non-string pattern
        let args = vec![FhirPathValue::Integer(42)];
        let result = matches_full_fn.evaluate(&args, &context).await;
        assert!(result.is_err());

        // Test invalid regex pattern
        let args = vec![FhirPathValue::String("[".into())]; // Invalid regex
        let result = matches_full_fn.evaluate(&args, &context).await;
        assert!(result.is_err());

        // Test non-string input
        let context = create_test_context(FhirPathValue::Integer(42));
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_full_fn.evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_collection_handling() {
        let matches_full_fn = MatchesFullFunction::new();

        // Test empty regex parameter returns empty
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Empty];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test empty string pattern returns empty
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test empty collection input returns empty
        use octofhir_fhirpath_model::Collection;
        let empty_collection = FhirPathValue::Collection(Collection::new());
        let context = create_test_context(empty_collection);
        let args = vec![FhirPathValue::String("test".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_multiple_items_error() {
        let matches_full_fn = MatchesFullFunction::new();

        // Test collection with multiple items should cause error
        use octofhir_fhirpath_model::Collection;
        let multiple_items = vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ];
        let collection = FhirPathValue::Collection(Collection::from_vec(multiple_items));
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("test".into())];
        let result = matches_full_fn.evaluate(&args, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("multiple items"));
    }

    #[tokio::test]
    async fn test_single_item_collection() {
        let matches_full_fn = MatchesFullFunction::new();

        // Test collection with single item should work normally
        use octofhir_fhirpath_model::Collection;
        let single_item = vec![FhirPathValue::String("123".into())];
        let collection = FhirPathValue::Collection(Collection::from_vec(single_item));
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("\\d+".into())];
        let result = matches_full_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
