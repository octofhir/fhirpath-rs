//! Decode function implementation
//!
//! The decode function decodes strings according to specified format.
//! Syntax: string.decode(format)

use base64::Engine;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Decode function evaluator
pub struct DecodeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl DecodeFunctionEvaluator {
    /// Create a new decode function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
                        is_expression: true,
                        description: "Decoding format (base64, urlbase64, hex, url, html)"
                            .to_string(),
                        default_value: None,
                    }],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
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
impl FunctionEvaluator for DecodeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "decode function requires exactly one argument (format)".to_string(),
            ));
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

        // Evaluate format argument
        let format_result = evaluator.evaluate(&args[0], context).await?;
        let format_values: Vec<FhirPathValue> = format_result.value.iter().cloned().collect();

        if format_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "decode function format argument must evaluate to a single value".to_string(),
            ));
        }

        let format_str = match &format_values[0] {
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
                            format!("Invalid base64 string: {}", e),
                        )
                    })?;
                String::from_utf8(bytes).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0060,
                        format!("Base64 decoded bytes are not valid UTF-8: {}", e),
                    )
                })?
            }
            "urlbase64" => {
                let bytes = base64::engine::general_purpose::URL_SAFE
                    .decode(&input_str)
                    .map_err(|e| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0059,
                            format!("Invalid URL-safe base64 string: {}", e),
                        )
                    })?;
                String::from_utf8(bytes).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0060,
                        format!("URL-safe base64 decoded bytes are not valid UTF-8: {}", e),
                    )
                })?
            }
            "hex" => {
                let bytes = hex::decode(&input_str).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0059,
                        format!("Invalid hex string: {}", e),
                    )
                })?;
                String::from_utf8(bytes).map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0060,
                        format!("Hex decoded bytes are not valid UTF-8: {}", e),
                    )
                })?
            }
            "url" => urlencoding::decode(&input_str)
                .map_err(|e| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0059,
                        format!("Invalid URL encoded string: {}", e),
                    )
                })?
                .to_string(),
            "html" => html_escape::decode_html_entities(&input_str).to_string(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!(
                        "Unsupported decoding format: {}. Supported formats: base64, urlbase64, hex, url, html",
                        format_str
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
