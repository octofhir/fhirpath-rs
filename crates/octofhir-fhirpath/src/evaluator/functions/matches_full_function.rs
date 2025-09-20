//! MatchesFull function implementation
//!
//! The matchesFull function tests whether a string fully matches a regular expression pattern.
//! Unlike matches() which does partial matching, matchesFull requires the entire string to match.
//! Syntax: string.matchesFull(pattern)

use regex::Regex;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// MatchesFull function evaluator
pub struct MatchesFullFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MatchesFullFunctionEvaluator {
    /// Create a new matchesFull function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "matchesFull".to_string(),
                description: "Tests whether a string fully matches a regular expression pattern. The entire string must match the pattern.".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "pattern".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Regular expression pattern to match against the entire string".to_string(),
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
impl PureFunctionEvaluator for MatchesFullFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "matchesFull function requires exactly one argument (pattern)".to_string(),
            ));
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "matchesFull function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "matchesFull function can only be called on string values".to_string(),
                ));
            }
        };

        // Get pattern argument (pre-evaluated)
        let pattern_values = &args[0];

        if pattern_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "matchesFull function pattern argument must evaluate to a single value".to_string(),
            ));
        }

        let pattern_str = match &pattern_values[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "matchesFull function pattern argument must be a string".to_string(),
                ));
            }
        };

        // Create full-match pattern by anchoring if not already anchored
        let full_pattern = if pattern_str.starts_with('^') && pattern_str.ends_with('$') {
            // Pattern already anchored
            pattern_str
        } else if pattern_str.starts_with('^') {
            // Only start anchored, add end anchor
            format!("{pattern_str}$")
        } else if pattern_str.ends_with('$') {
            // Only end anchored, add start anchor
            format!("^{pattern_str}")
        } else {
            // Not anchored, add both anchors
            format!("^{pattern_str}$")
        };

        // Compile the regex pattern
        let regex = match Regex::new(&full_pattern) {
            Ok(r) => r,
            Err(e) => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!("Invalid regular expression pattern '{full_pattern}': {e}"),
                ));
            }
        };

        // Test if the string matches the full pattern
        let matches = regex.is_match(&input_str);

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::boolean(matches)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
