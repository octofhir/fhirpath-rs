//! Unescape function implementation
//!
//! The unescape function unescapes special characters in strings for specific contexts.
//! Syntax: string.unescape(format)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::EvaluationResult;

/// Unescape function evaluator
pub struct UnescapeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl UnescapeFunctionEvaluator {
    /// Create a new unescape function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "unescape".to_string(),
                description: "Unescapes special characters in a string for the specified format"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "format".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "Unescape format (html, json, sql)".to_string(),
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

    /// Unescape JSON string
    fn unescape_json(input: &str) -> Result<String> {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\u{08}'), // backspace
                    Some('f') => result.push('\u{0C}'), // form feed
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('u') => {
                        // Parse Unicode escape sequence \uXXXX
                        let hex: String = (0..4).filter_map(|_| chars.next()).collect();

                        if hex.len() != 4 {
                            return Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0059,
                                "Invalid Unicode escape sequence: incomplete hex digits"
                                    .to_string(),
                            ));
                        }

                        let code_point = u32::from_str_radix(&hex, 16).map_err(|_| {
                            FhirPathError::evaluation_error(
                                crate::core::error_code::FP0059,
                                format!(
                                    "Invalid Unicode escape sequence: invalid hex digits '{}'",
                                    hex
                                ),
                            )
                        })?;

                        if let Some(unicode_char) = char::from_u32(code_point) {
                            result.push(unicode_char);
                        } else {
                            return Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0059,
                                format!("Invalid Unicode code point: U+{:04X}", code_point),
                            ));
                        }
                    }
                    Some(other) => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0059,
                            format!("Invalid escape sequence: \\{}", other),
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0059,
                            "Incomplete escape sequence at end of string".to_string(),
                        ));
                    }
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Unescape SQL string (convert '' back to ')
    fn unescape_sql(input: &str) -> String {
        input.replace("''", "'")
    }

    /// Unescape HTML string
    fn unescape_html(input: &str) -> String {
        // Reference implementation from FHIRPath specification
        // Note: &amp; must be last to avoid double-unescaping
        input
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for UnescapeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "unescape function requires exactly one argument (format)".to_string(),
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
                "unescape function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "unescape function can only be called on string values".to_string(),
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
                "unescape function format argument must be a single value".to_string(),
            ));
        }

        let format_str = match &args[0][0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "unescape function format argument must be a string".to_string(),
                ));
            }
        };

        // Perform unescaping based on format
        let unescaped = match format_str.to_lowercase().as_str() {
            "html" => Self::unescape_html(&input_str),
            "json" => Self::unescape_json(&input_str)?,
            "sql" => Self::unescape_sql(&input_str),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!(
                        "Unsupported unescape format: {}. Supported formats: html, json, sql",
                        format_str
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::string(unescaped)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
