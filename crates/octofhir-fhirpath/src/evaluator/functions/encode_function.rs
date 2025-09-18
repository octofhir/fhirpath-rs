//! Encode function implementation
//!
//! The encode function encodes strings according to specified format.
//! Syntax: string.encode(format)

use base64::Engine;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Encode function evaluator
pub struct EncodeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl EncodeFunctionEvaluator {
    /// Create a new encode function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "encode".to_string(),
                description: "Encodes a string using the specified encoding format".to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "format".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: true,
                        description: "Encoding format (base64, urlbase64, hex, url, html)"
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
impl FunctionEvaluator for EncodeFunctionEvaluator {
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
                "encode function requires exactly one argument (format)".to_string(),
            ));
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "encode function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "encode function can only be called on string values".to_string(),
                ));
            }
        };

        // Evaluate format argument
        let format_result = evaluator.evaluate(&args[0], context).await?;
        let format_values: Vec<FhirPathValue> = format_result.value.iter().cloned().collect();

        if format_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "encode function format argument must evaluate to a single value".to_string(),
            ));
        }

        let format_str = match &format_values[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "encode function format argument must be a string".to_string(),
                ));
            }
        };

        // Perform encoding based on format
        let encoded = match format_str.to_lowercase().as_str() {
            "base64" => base64::engine::general_purpose::STANDARD.encode(input_str.as_bytes()),
            "urlbase64" => base64::engine::general_purpose::URL_SAFE.encode(input_str.as_bytes()),
            "hex" => hex::encode(input_str.as_bytes()),
            "url" => urlencoding::encode(&input_str).to_string(),
            "html" => html_escape::encode_text(&input_str).to_string(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!(
                        "Unsupported encoding format: {}. Supported formats: base64, urlbase64, hex, url, html",
                        format_str
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::string(encoded)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
