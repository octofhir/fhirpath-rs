//! Decode function implementation
//!
//! The decode function decodes strings according to specified format.
//! Syntax: string.decode(format)

use base64::Engine;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Decode function evaluator
pub struct DecodeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DecodeFunctionEvaluator {
    /// Create a new decode function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "decode".to_string(),
                description: "Decodes a string using the specified encoding format".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "format".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "Decoding format (base64, urlbase64, hex, url, html)"
                            .to_string(),
                        default_value: None,
                    }],
                    return_type: "String".to_string(),
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
impl PureFunctionEvaluator for DecodeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "decode function requires exactly one argument (format)".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "decode function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "decode function can only be called on string values".to_string(),
                ));
            }
        };

        // Handle empty format argument - propagate empty collections
        if args[0].is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if args[0].len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "decode function format argument must be a single value".to_string(),
            ));
        }

        let format_str = match &args[0][0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "decode function format argument must be a string".to_string(),
                ));
            }
        };

        // Perform decoding based on format
        let decoded = match format_str.to_lowercase().as_str() {
            "base64" => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(&input_str)
                    .map_err(|e| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0059,
                            format!("Invalid base64 string: {e}"),
                        )
                    })?;
                String::from_utf8(bytes).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0060,
                        format!("Base64 decoded bytes are not valid UTF-8: {e}"),
                    )
                })?
            }
            "urlbase64" => {
                let bytes = base64::engine::general_purpose::URL_SAFE
                    .decode(&input_str)
                    .map_err(|e| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0059,
                            format!("Invalid URL-safe base64 string: {e}"),
                        )
                    })?;
                String::from_utf8(bytes).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0060,
                        format!("URL-safe base64 decoded bytes are not valid UTF-8: {e}"),
                    )
                })?
            }
            "hex" => {
                let bytes = hex::decode(&input_str).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0059,
                        format!("Invalid hex string: {e}"),
                    )
                })?;
                String::from_utf8(bytes).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0060,
                        format!("Hex decoded bytes are not valid UTF-8: {e}"),
                    )
                })?
            }
            "url" => urlencoding::decode(&input_str)
                .map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0059,
                        format!("Invalid URL encoded string: {e}"),
                    )
                })?
                .to_string(),
            "html" => html_escape::decode_html_entities(&input_str).to_string(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!(
                        "Unsupported decoding format: {format_str}. Supported formats: base64, urlbase64, hex, url, html"
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::string(decoded)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
