//! Escape function implementation
//!
//! The escape function escapes special characters in strings for specific contexts.
//! Syntax: string.escape(format)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Escape function evaluator
pub struct EscapeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl EscapeFunctionEvaluator {
    /// Create a new escape function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "escape".to_string(),
                description: "Escapes special characters in a string for the specified format"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "String".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "format".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: true,
                        description: "Escape format (html, json, regex, sql)".to_string(),
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

    /// Escape string for JSON
    fn escape_json(input: &str) -> String {
        let mut result = String::with_capacity(input.len() + 10);
        for ch in input.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\u{08}' => result.push_str("\\b"), // backspace
                '\u{0C}' => result.push_str("\\f"), // form feed
                c if c.is_control() => {
                    result.push_str(&format!("\\u{:04x}", c as u32));
                }
                c => result.push(c),
            }
        }
        result
    }

    /// Escape string for regex
    fn escape_regex(input: &str) -> String {
        let mut result = String::with_capacity(input.len() * 2);
        for ch in input.chars() {
            match ch {
                '.' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
                | '\\' => {
                    result.push('\\');
                    result.push(ch);
                }
                c => result.push(c),
            }
        }
        result
    }

    /// Escape string for SQL
    fn escape_sql(input: &str) -> String {
        input.replace('\'', "''")
    }

    /// Escape string for HTML
    fn escape_html(input: &str) -> String {
        // Simple HTML escaping following the reference implementation
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for EscapeFunctionEvaluator {
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
                "escape function requires exactly one argument (format)".to_string(),
            ));
        }

        if input.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "escape function can only be called on a single string value".to_string(),
            ));
        }

        // Get the input string
        let input_str = match &input[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0055,
                    "escape function can only be called on string values".to_string(),
                ));
            }
        };

        // Evaluate format argument
        let format_result = evaluator.evaluate(&args[0], context).await?;
        let format_values: Vec<FhirPathValue> = format_result.value.iter().cloned().collect();

        if format_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "escape function format argument must evaluate to a single value".to_string(),
            ));
        }

        let format_str = match &format_values[0] {
            FhirPathValue::String(s, _, _) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0057,
                    "escape function format argument must be a string".to_string(),
                ));
            }
        };

        // Perform escaping based on format
        let escaped = match format_str.to_lowercase().as_str() {
            "html" => Self::escape_html(&input_str),
            "json" => Self::escape_json(&input_str),
            "regex" => Self::escape_regex(&input_str),
            "sql" => Self::escape_sql(&input_str),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0058,
                    format!(
                        "Unsupported escape format: {}. Supported formats: html, json, regex, sql",
                        format_str
                    ),
                ));
            }
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::string(escaped)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
