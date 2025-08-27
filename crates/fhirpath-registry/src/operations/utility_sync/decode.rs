//! Decode function implementation - sync version

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Decode function - decodes strings from various formats
#[derive(Debug, Clone)]
pub struct DecodeFunction;

impl DecodeFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for DecodeFunction {
    fn name(&self) -> &'static str {
        "decode"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "decode",
                parameters: vec![ParameterType::String],
                return_type: ValueType::String,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
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
                function_name: "decode".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let encoding_type = validation::validate_single_string_arg(args, "decode")?;
        let input_string = validation::validate_string_input(context, "decode")?;

        let decoded = match encoding_type.as_str() {
            "base64" => base64_decode(&input_string)?,
            "url" => url_decode(&input_string)?,
            "html" => html_decode(&input_string),
            "hex" => hex_decode(&input_string)?,
            "urlbase64" => urlbase64_decode(&input_string)?,
            _ => {
                return Err(FhirPathError::evaluation_error(format!(
                    "Unsupported encoding: {encoding_type}"
                )));
            }
        };

        Ok(FhirPathValue::String(decoded.into()))
    }
}

impl Default for DecodeFunction {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_decode(input: &str) -> Result<String> {
    use base64::{Engine, engine::general_purpose};
    let bytes = general_purpose::STANDARD
        .decode(input)
        .map_err(|_| FhirPathError::evaluation_error("Invalid base64 encoding"))?;
    String::from_utf8(bytes)
        .map_err(|_| FhirPathError::evaluation_error("Invalid UTF-8 in decoded base64"))
}

fn url_decode(input: &str) -> Result<String> {
    use percent_encoding::percent_decode;
    percent_decode(input.as_bytes())
        .decode_utf8()
        .map(|s| s.to_string())
        .map_err(|_| FhirPathError::evaluation_error("Invalid URL encoding"))
}

fn html_decode(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
}

fn hex_decode(input: &str) -> Result<String> {
    // Hex string should have even length
    if input.len() % 2 != 0 {
        return Err(FhirPathError::evaluation_error(
            "Invalid hex encoding: odd length",
        ));
    }

    let mut bytes = Vec::new();
    for chunk in input.as_bytes().chunks(2) {
        let hex_str = std::str::from_utf8(chunk).map_err(|_| {
            FhirPathError::evaluation_error("Invalid hex encoding: non-UTF8 characters")
        })?;
        let byte_val = u8::from_str_radix(hex_str, 16).map_err(|_| {
            FhirPathError::evaluation_error(format!(
                "Invalid hex encoding: invalid hex digits '{hex_str}'"
            ))
        })?;
        bytes.push(byte_val);
    }

    String::from_utf8(bytes)
        .map_err(|_| FhirPathError::evaluation_error("Invalid UTF-8 in decoded hex"))
}

fn urlbase64_decode(input: &str) -> Result<String> {
    use base64::{Engine, engine::general_purpose};
    let bytes = general_purpose::URL_SAFE
        .decode(input)
        .map_err(|_| FhirPathError::evaluation_error("Invalid urlbase64 encoding"))?;
    String::from_utf8(bytes)
        .map_err(|_| FhirPathError::evaluation_error("Invalid UTF-8 in decoded urlbase64"))
}
