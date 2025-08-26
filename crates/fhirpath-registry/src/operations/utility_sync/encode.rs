//! Encode function implementation - sync version

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Encode function - encodes strings to various formats
#[derive(Debug, Clone)]
pub struct EncodeFunction;

impl EncodeFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for EncodeFunction {
    fn name(&self) -> &'static str {
        "encode"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "encode",
                parameters: vec![ParameterType::String],
                return_type: ValueType::String,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "encode".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let encoding_type = validation::validate_single_string_arg(args, "encode")?;
        let input_string = validation::validate_string_input(context, "encode")?;

        let encoded = match encoding_type.as_str() {
            "base64" => base64_encode(&input_string),
            "url" => url_encode(&input_string),
            "html" => html_encode(&input_string),
            "hex" => hex_encode(&input_string),
            "urlbase64" => urlbase64_encode(&input_string),
            _ => {
                return Err(FhirPathError::evaluation_error(format!("Unsupported encoding: {encoding_type}")));
            }
        };

        Ok(FhirPathValue::String(encoded.into()))
    }
}

impl Default for EncodeFunction {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_encode(input: &str) -> String {
    use base64::{Engine, engine::general_purpose};
    general_purpose::STANDARD.encode(input.as_bytes())
}

fn url_encode(input: &str) -> String {
    use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
    utf8_percent_encode(input, NON_ALPHANUMERIC).to_string()
}

fn html_encode(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn hex_encode(input: &str) -> String {
    input
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

fn urlbase64_encode(input: &str) -> String {
    use base64::{Engine, engine::general_purpose};
    // URL-safe base64 encoding (uses - and _ instead of + and /, with padding)
    general_purpose::URL_SAFE.encode(input.as_bytes())
}
