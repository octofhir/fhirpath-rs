//! Unescape function implementation - sync version

use crate::traits::{SyncOperation, EvaluationContext, validation};
use crate::signature::{FunctionSignature, ValueType, ParameterType};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

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
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "unescape",
                parameters: vec![ParameterType::String],
                return_type: ValueType::String,
                variadic: false,
            }
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
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
        _ => Err(FhirPathError::EvaluationError {
            message: format!("Unsupported unescape format: '{}'. Supported formats are 'html' and 'json'", format).into(),
            expression: None,
            location: None,
        }),
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
                            _ => return Err(FhirPathError::EvaluationError {
                                message: "Invalid Unicode escape sequence in JSON".into(),
                                expression: None,
                                location: None,
                            }),
                        }
                    }
                    if let Ok(code_point) = u32::from_str_radix(&unicode_chars, 16) {
                        if let Some(unicode_char) = char::from_u32(code_point) {
                            result.push(unicode_char);
                        } else {
                            return Err(FhirPathError::EvaluationError {
                                message: "Invalid Unicode code point".into(),
                                expression: None,
                                location: None,
                            });
                        }
                    } else {
                        return Err(FhirPathError::EvaluationError {
                            message: "Invalid Unicode escape sequence".into(),
                            expression: None,
                            location: None,
                        });
                    }
                }
                Some(other) => return Err(FhirPathError::EvaluationError {
                    message: format!("Invalid JSON escape sequence: \\{}", other).into(),
                    expression: None,
                    location: None,
                }),
                None => return Err(FhirPathError::EvaluationError {
                    message: "Incomplete escape sequence at end of string".into(),
                    expression: None,
                    location: None,
                }),
            }
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}