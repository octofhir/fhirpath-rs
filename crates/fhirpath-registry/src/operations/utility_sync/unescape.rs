//! Unescape function implementation - sync version

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// Unescape function - unescapes special characters in strings
#[derive(Debug, Clone)]
pub struct UnescapeFunction;

impl UnescapeFunction {
    pub fn new() -> Self {
        Self
    }
}

impl SyncOperation for UnescapeFunction {
    fn name(&self) -> &'static str {
        "unescape"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "unescape",
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
        validation::validate_arg_count(args.len(), 1, "unescape")?;

        let input_string = validation::validate_string_input(context, "unescape")?;

        let format = validation::extract_string_arg(args, 0, "unescape", "format")?;

        match unescape_with_format(&input_string, &format) {
            Ok(unescaped) => Ok(FhirPathValue::String(unescaped.into())),
            Err(e) => Err(e),
        }
    }
}

impl Default for UnescapeFunction {
    fn default() -> Self {
        Self::new()
    }
}

fn unescape_with_format(input: &str, format: &str) -> Result<String> {
    match format {
        "html" => Ok(unescape_html(input)),
        "json" => unescape_json(input),
        _ => Err(FhirPathError::evaluation_error(format!(
            "Unsupported unescape format: '{format}'. Supported formats are 'html' and 'json'"
        ))),
    }
}

fn unescape_html(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&") // This must be last to avoid double-unescaping
}

fn unescape_json(input: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\u{08}'),
                Some('f') => result.push('\u{0C}'),
                Some('u') => {
                    // Handle Unicode escape sequences \uXXXX
                    let mut unicode_chars = String::new();
                    for _ in 0..4 {
                        match chars.next() {
                            Some(c) if c.is_ascii_hexdigit() => unicode_chars.push(c),
                            _ => {
                                return Err(FhirPathError::evaluation_error(
                                    "Invalid Unicode escape sequence in JSON",
                                ));
                            }
                        }
                    }
                    if let Ok(code_point) = u32::from_str_radix(&unicode_chars, 16) {
                        if let Some(unicode_char) = char::from_u32(code_point) {
                            result.push(unicode_char);
                        } else {
                            return Err(FhirPathError::evaluation_error(
                                "Invalid Unicode code point",
                            ));
                        }
                    } else {
                        return Err(FhirPathError::evaluation_error(
                            "Invalid Unicode escape sequence",
                        ));
                    }
                }
                Some(other) => {
                    return Err(FhirPathError::evaluation_error(format!(
                        "Invalid JSON escape sequence: \\{other}"
                    )));
                }
                None => {
                    return Err(FhirPathError::evaluation_error(
                        "Incomplete escape sequence at end of string",
                    ));
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}
