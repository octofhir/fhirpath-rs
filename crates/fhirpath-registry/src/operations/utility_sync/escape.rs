//! Escape function implementation - sync version

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Escape function - escapes special characters in strings
#[derive(Debug, Clone)]
pub struct EscapeFunction;

impl EscapeFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for EscapeFunction {
    fn name(&self) -> &'static str {
        "escape"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "escape",
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
        validation::validate_arg_count(args.len(), 1, "escape")?;

        let input_string = validation::validate_string_input(context, "escape")?;

        let format = validation::extract_string_arg(args, 0, "escape", "format")?;

        match escape_with_format(&input_string, &format) {
            Ok(escaped) => Ok(FhirPathValue::String(escaped.into())),
            Err(e) => Err(e),
        }
    }
}

impl Default for EscapeFunction {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_with_format(input: &str, format: &str) -> Result<String> {
    match format {
        "html" => Ok(escape_html(input)),
        "json" => Ok(escape_json(input)),
        _ => Err(FhirPathError::evaluation_error(format!(
                "Unsupported escape format: '{format}'. Supported formats are 'html' and 'json'"
            ))),
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\u{08}', "\\b")
        .replace('\u{0C}', "\\f")
}
