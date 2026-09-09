//! replaceMatches function implementation
//!
//! The replaceMatches function returns the input string with all matches of a
//! regular expression pattern replaced by the provided substitution.

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// replaceMatches function evaluator
#[derive(Clone)]
pub struct ReplaceMatchesFunctionEvaluator {
    compiled: Option<Arc<regex::Regex>>,
    metadata: FunctionMetadata,
}

impl ReplaceMatchesFunctionEvaluator {
    /// Create a new replaceMatches function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            compiled: None,
            metadata: FunctionMetadata {
                name: "replaceMatches".to_string(),
                description: "Replaces all matches of a regular expression pattern in a string with the substitution".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "pattern".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Regular expression pattern to match".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "substitution".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "Replacement text for each match".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 2,
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
impl PureFunctionEvaluator for ReplaceMatchesFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn prepare(&self, args: &[Collection]) -> Option<Arc<dyn PureFunctionEvaluator>> {
        let pattern = args.first()?.first()?.as_string()?;
        // Keep compiled-plan retention bounded even for adversarial regexes.
        // Oversized/expensive patterns keep the normal runtime compilation path.
        if pattern.len() > 4096 {
            return None;
        }

        let compiled = regex::RegexBuilder::new(pattern)
            .size_limit(256 * 1024)
            .build()
            .ok()?;
        let mut evaluator = self.clone();
        evaluator.compiled = Some(Arc::new(compiled));
        Some(Arc::new(evaluator))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if args.len() != 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!(
                    "replaceMatches function expects 2 arguments, got {}",
                    args.len()
                ),
            ));
        }

        // Get the pattern argument (pre-evaluated)
        let pattern_arg = &args[0];
        if pattern_arg.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if pattern_arg.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replaceMatches function pattern parameter must evaluate to a single value"
                    .to_string(),
            ));
        }

        let pattern = pattern_arg
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "replaceMatches function pattern parameter must be a string".to_string(),
                )
            })?
            .to_string();

        // Get the substitution argument (pre-evaluated)
        let substitution_arg = &args[1];
        if substitution_arg.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if substitution_arg.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replaceMatches function substitution parameter must evaluate to a single value"
                    .to_string(),
            ));
        }

        let substitution = substitution_arg
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "replaceMatches function substitution parameter must be a string".to_string(),
                )
            })?
            .to_string();

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let regex = if pattern.is_empty() {
            None
        } else {
            Some(
                self.compiled
                    .as_ref()
                    .filter(|regex| regex.as_str() == pattern.as_str())
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(|| super::regex_cache::compile(&pattern))
                    .map_err(|err| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            format!("Invalid regular expression pattern '{pattern}': {err}"),
                        )
                    })?,
            )
        };

        let mut results = Vec::with_capacity(input.len());
        for value in input {
            if let FhirPathValue::String(content, type_info, primitive) = &value {
                let replaced = if let Some(regex) = &regex {
                    regex
                        .replace_all(content, substitution.as_str())
                        .into_owned()
                } else {
                    content.clone()
                };

                results.push(FhirPathValue::String(
                    replaced,
                    type_info.clone(),
                    primitive.clone(),
                ));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "replaceMatches function can only be applied to strings, got {}",
                        value.type_name()
                    ),
                ));
            }
        }

        Ok(EvaluationResult {
            value: Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
