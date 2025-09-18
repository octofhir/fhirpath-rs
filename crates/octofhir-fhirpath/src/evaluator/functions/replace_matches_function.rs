//! replaceMatches function implementation
//!
//! The replaceMatches function returns the input string with all matches of a
//! regular expression pattern replaced by the provided substitution.

use std::sync::Arc;

use regex::Regex;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// replaceMatches function evaluator
pub struct ReplaceMatchesFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ReplaceMatchesFunctionEvaluator {
    /// Create a new replaceMatches function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
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
                            is_expression: true,
                            description: "Regular expression pattern to match".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "substitution".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Replacement text for each match".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 2,
                    max_params: Some(2),
                },
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
impl FunctionEvaluator for ReplaceMatchesFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!(
                    "replaceMatches function expects 2 arguments, got {}",
                    args.len()
                ),
            ));
        }

        // Evaluate the pattern argument
        let pattern_result = evaluator.evaluate(&args[0], context).await?;
        if pattern_result.value.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if pattern_result.value.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replaceMatches function pattern parameter must evaluate to a single value"
                    .to_string(),
            ));
        }

        let pattern = pattern_result
            .value
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "replaceMatches function pattern parameter must be a string".to_string(),
                )
            })?
            .to_string();

        // Evaluate the substitution argument
        let substitution_result = evaluator.evaluate(&args[1], context).await?;
        if substitution_result.value.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if substitution_result.value.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replaceMatches function substitution parameter must evaluate to a single value"
                    .to_string(),
            ));
        }

        let substitution = substitution_result
            .value
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
            Some(Regex::new(&pattern).map_err(|err| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!("Invalid regular expression pattern '{}': {}", pattern, err),
                )
            })?)
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
