//! Substring function implementation
//!
//! The substring function returns a portion of a string.
//! Syntax: string.substring(start [, length])

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Substring function evaluator
pub struct SubstringFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SubstringFunctionEvaluator {
    /// Create a new substring function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "substring".to_string(),
                description: "Returns a portion of a string".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "start".to_string(),
                            parameter_type: vec!["Integer".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Starting position (0-based)".to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "length".to_string(),
                            parameter_type: vec!["Integer".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Length of substring".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "String".to_string(),
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
impl PureFunctionEvaluator for SubstringFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "substring function requires 1 or 2 arguments (start [, length])".to_string(),
            ));
        }

        // Get start argument from pre-evaluated args
        let start_values = &args[0];

        if start_values.is_empty() {
            // If start parameter is empty, return empty collection
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if start_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "substring function start parameter must evaluate to a single value".to_string(),
            ));
        }

        let start = match &start_values[0] {
            FhirPathValue::Integer(i, _, _) => *i,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "substring function start parameter must be an integer".to_string(),
                ));
            }
        };

        // Get optional length argument from pre-evaluated args
        let length = if args.len() > 1 {
            let length_values = &args[1];

            if length_values.is_empty() {
                // If length parameter is empty, return empty collection
                return Ok(EvaluationResult {
                    value: crate::core::Collection::empty(),
                });
            }

            if length_values.len() != 1 {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    "substring function length parameter must evaluate to a single value"
                        .to_string(),
                ));
            }

            match &length_values[0] {
                FhirPathValue::Integer(i, _, _) => Some(*i),
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        "substring function length parameter must be an integer".to_string(),
                    ));
                }
            }
        } else {
            None
        };

        let mut results = Vec::new();

        for value in input {
            match &value {
                FhirPathValue::String(s, _, _) => {
                    let chars: Vec<char> = s.chars().collect();
                    let str_len = chars.len() as i64;

                    // According to FHIRPath spec, negative start or start >= string length should return empty
                    if start < 0 || start >= str_len {
                        // Return empty collection - don't add anything to results
                        continue;
                    }

                    let start_idx = start as usize;

                    let substring = if let Some(len) = length {
                        if len <= 0 {
                            String::new()
                        } else {
                            let end_idx = (start_idx + len as usize).min(chars.len());
                            chars[start_idx..end_idx].iter().collect()
                        }
                    } else {
                        chars[start_idx..].iter().collect()
                    };

                    results.push(FhirPathValue::string(substring));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0055,
                        "substring function can only be called on strings".to_string(),
                    ));
                }
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
