//! Matches function implementation
//!
//! The matches function tests whether a string contains a substring that matches a regular expression pattern.
//! Unlike matchesFull() which requires the entire string to match, matches() does partial matching.
//! Syntax: string.matches(pattern)

use regex::Regex;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::EvaluationResult;

/// Matches function evaluator
pub struct MatchesFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MatchesFunctionEvaluator {
    /// Create a new matches function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "matches".to_string(),
                description: "Tests whether a string contains a substring that matches a regular expression pattern.".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "pattern".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Regular expression pattern to search for within the string".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::StringManipulation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for MatchesFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "matches function requires exactly one argument (pattern)".to_string(),
            ));
        }

        // Handle empty input - return empty per FHIRPath specification
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "matches function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "matches function can only be called on string values".to_string(),
                ));
            }
        };

        // Get pattern argument from pre-evaluated args
        let pattern_values = &args[0];

        // Handle empty pattern - return empty per FHIRPath specification
        if pattern_values.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if pattern_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "matches function pattern argument must evaluate to a single value".to_string(),
            ));
        }

        let pattern_str = match &pattern_values[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "matches function pattern argument must be a string".to_string(),
                ));
            }
        };

        // Compile the regex pattern (no anchoring for partial matching)
        let regex = match Regex::new(&pattern_str) {
            Ok(r) => r,
            Err(e) => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!(
                        "Invalid regular expression pattern '{}': {}",
                        pattern_str, e
                    ),
                ));
            }
        };

        // Test if the string contains a match for the pattern
        let matches = regex.is_match(&input_str);

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::boolean(matches)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
