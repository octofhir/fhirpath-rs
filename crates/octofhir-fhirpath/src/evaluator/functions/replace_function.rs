//! Replace function implementation
//!
//! The replace function returns a new string with occurrences of a pattern
//! replaced by a substitution value. It handles empty pattern and substitution
//! arguments according to the FHIRPath specification.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Replace function evaluator
pub struct ReplaceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ReplaceFunctionEvaluator {
    /// Create a new replace function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "replace".to_string(),
                description:
                    "Replaces all instances of a pattern in a string with the substitution"
                        .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "pattern".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "The substring to replace".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "substitution".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "The replacement value for each occurrence".to_string(),
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

    fn apply_replace(input: &str, pattern: &str, substitution: &str) -> String {
        if pattern.is_empty() {
            Self::surround_characters(input, substitution)
        } else {
            input.replace(pattern, substitution)
        }
    }

    fn surround_characters(input: &str, substitution: &str) -> String {
        if substitution.is_empty() {
            return input.to_string();
        }

        let char_count = input.chars().count();
        let mut result = String::with_capacity(input.len() + substitution.len() * (char_count + 1));
        result.push_str(substitution);

        for ch in input.chars() {
            result.push(ch);
            result.push_str(substitution);
        }

        result
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for ReplaceFunctionEvaluator {
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
                format!("replace function expects 2 arguments, got {}", args.len()),
            ));
        }

        let pattern_result = evaluator.evaluate(&args[0], context).await?;
        if pattern_result.value.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if pattern_result.value.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replace function pattern parameter must evaluate to a single value".to_string(),
            ));
        }

        let pattern = pattern_result
            .value
            .first()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "replace function pattern parameter must be a string".to_string(),
                )
            })?
            .to_string();

        let substitution_result = evaluator.evaluate(&args[1], context).await?;
        if substitution_result.value.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if substitution_result.value.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replace function substitution parameter must evaluate to a single value"
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
                    "replace function substitution parameter must be a string".to_string(),
                )
            })?
            .to_string();

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let mut results = Vec::new();
        for value in input {
            if let FhirPathValue::String(s, type_info, primitive) = &value {
                let replaced = Self::apply_replace(s, &pattern, &substitution);
                results.push(FhirPathValue::String(
                    replaced,
                    type_info.clone(),
                    primitive.clone(),
                ));
            } else {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    format!(
                        "replace function can only be applied to strings, got {}",
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
