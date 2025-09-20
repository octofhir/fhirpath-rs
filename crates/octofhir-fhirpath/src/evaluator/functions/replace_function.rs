//! Replace function implementation
//!
//! The replace function returns a new string with occurrences of a pattern
//! replaced by a substitution value. It handles empty pattern and substitution
//! arguments according to the FHIRPath specification.

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Replace function evaluator
pub struct ReplaceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ReplaceFunctionEvaluator {
    /// Create a new replace function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
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
impl PureFunctionEvaluator for ReplaceFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("replace function expects 2 arguments, got {}", args.len()),
            ));
        }

        if args[0].is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if args[0].len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replace function pattern parameter must be a single value".to_string(),
            ));
        }

        let pattern = args[0][0].as_string().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                "replace function pattern parameter must be a string".to_string(),
            )
        })?;

        if args[1].is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if args[1].len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "replace function substitution parameter must be a single value".to_string(),
            ));
        }

        let substitution = args[1][0].as_string().ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                "replace function substitution parameter must be a string".to_string(),
            )
        })?;

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let mut results = Vec::new();
        for value in input {
            if let FhirPathValue::String(s, type_info, primitive) = &value {
                let replaced = Self::apply_replace(s, pattern, substitution);
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
