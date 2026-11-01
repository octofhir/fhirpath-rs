//! MatchesFull function implementation
//!
//! The matchesFull function tests whether a string fully matches a regular expression pattern.
//! Unlike matches() which does partial matching, matchesFull requires the entire string to match.
//! Syntax: string.matchesFull(pattern)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// `pattern` anchored to the start and end of the whole input (not of a line, so it holds in
/// multi-line mode too).
fn anchor_whole(pattern: &str) -> String {
    format!(r"\A(?:{pattern})\z")
}

/// MatchesFull function evaluator
#[derive(Clone)]
pub struct MatchesFullFunctionEvaluator {
    compiled: Option<Arc<super::regex_cache::PreparedRegex>>,
    metadata: FunctionMetadata,
}

impl MatchesFullFunctionEvaluator {
    /// Create a new matchesFull function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            compiled: None,
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
                        },
                        FunctionParameter {
                            name: "flags".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: true,
                            is_expression: false,
                            description: "Regex flags: 'i' case-insensitive, 'm' multi-line".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(2),
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
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn prepare(&self, args: &[Collection]) -> Option<Arc<dyn PureFunctionEvaluator>> {
        let pattern = args.first()?.first()?.as_string()?;
        let flags = super::regex_cache::flags_arg(args, 1, "matchesFull").ok()?;
        let effective =
            super::regex_cache::effective_pattern(&anchor_whole(pattern), &flags).ok()?;
        let mut evaluator = self.clone();
        evaluator.compiled = Some(super::regex_cache::PreparedRegex::new(
            pattern, &flags, &effective,
        )?);
        Some(Arc::new(evaluator))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync_borrowed(input, &args)
    }

    fn evaluate_sync_borrowed(
        &self,
        input: Collection,
        args: &[Collection],
    ) -> Result<EvaluationResult> {
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "matchesFull function takes a pattern and optional flags".to_string(),
            ));
        }
        let flags = super::regex_cache::flags_arg(args, 1, "matchesFull")?;

        // An empty input or pattern gives an empty result.
        if input.is_empty() || args[0].is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
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

        // The whole string must match: anchor the pattern at both ends of the input.
        let regex =
            super::regex_cache::regex_for(self.compiled.as_ref(), &pattern_str, &flags, || {
                super::regex_cache::effective_pattern(&anchor_whole(&pattern_str), &flags)
            })?;

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
