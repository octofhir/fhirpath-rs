//! Signature help provider for FHIRPath function calls

use crate::evaluator::FunctionRegistry;

use lsp_types::{
    Documentation, MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureInformation,
};
use std::sync::Arc;

/// Provider for signature help on FHIRPath function calls
pub struct SignatureHelpProvider {
    function_registry: Arc<FunctionRegistry>,
}

impl SignatureHelpProvider {
    /// Create a new signature help provider
    pub fn new(function_registry: Arc<FunctionRegistry>) -> Self {
        Self { function_registry }
    }

    /// Provide signature help at the given cursor offset
    pub fn provide(&self, expression: &str, cursor_offset: usize) -> Option<SignatureHelp> {
        let (func_name, active_param) = Self::detect_function_context(expression, cursor_offset)?;
        let meta = self.function_registry.get_metadata(&func_name)?;

        let parameters: Vec<ParameterInformation> = meta
            .signature
            .parameters
            .iter()
            .map(|p| {
                let types = p.parameter_type.join(" | ");
                let label = if p.optional {
                    format!("{}?: {}", p.name, types)
                } else {
                    format!("{}: {}", p.name, types)
                };

                ParameterInformation {
                    label: ParameterLabel::Simple(label),
                    documentation: if p.description.is_empty() {
                        None
                    } else {
                        Some(Documentation::String(p.description.clone()))
                    },
                }
            })
            .collect();

        let params_str: Vec<String> = meta
            .signature
            .parameters
            .iter()
            .map(|p| {
                let types = p.parameter_type.join(" | ");
                if p.optional {
                    format!("{}?: {}", p.name, types)
                } else {
                    format!("{}: {}", p.name, types)
                }
            })
            .collect();

        let label = format!(
            "{}({}) → {}",
            func_name,
            params_str.join(", "),
            meta.signature.return_type
        );

        let signature = SignatureInformation {
            label,
            documentation: if meta.description.is_empty() {
                None
            } else {
                Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: meta.description.clone(),
                }))
            },
            parameters: Some(parameters),
            active_parameter: Some(active_param),
        };

        Some(SignatureHelp {
            signatures: vec![signature],
            active_signature: Some(0),
            active_parameter: Some(active_param),
        })
    }

    /// Detect the function name and active parameter index at cursor position
    ///
    /// Walks backwards from cursor to find the enclosing function call,
    /// counting commas to determine the active parameter.
    fn detect_function_context(expression: &str, cursor_offset: usize) -> Option<(String, u32)> {
        let before = &expression[..cursor_offset.min(expression.len())];

        let mut paren_depth = 0i32;
        let mut comma_count = 0u32;

        for (i, c) in before.char_indices().rev() {
            match c {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    } else {
                        // Found the opening paren of our function call
                        // Extract function name before this paren
                        let before_paren = &before[..i];
                        let func_start = before_paren
                            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                            .map(|j| j + 1)
                            .unwrap_or(0);

                        let func_name = &before_paren[func_start..];
                        if func_name.is_empty() {
                            return None;
                        }

                        return Some((func_name.to_string(), comma_count));
                    }
                }
                ',' if paren_depth == 0 => {
                    comma_count += 1;
                }
                _ => {}
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_function_simple() {
        let (name, param) = SignatureHelpProvider::detect_function_context("where(", 6).unwrap();
        assert_eq!(name, "where");
        assert_eq!(param, 0);
    }

    #[test]
    fn test_detect_function_second_param() {
        let (name, param) =
            SignatureHelpProvider::detect_function_context("substring(1, ", 13).unwrap();
        assert_eq!(name, "substring");
        assert_eq!(param, 1);
    }

    #[test]
    fn test_detect_function_nested() {
        let (name, param) =
            SignatureHelpProvider::detect_function_context("iif(a, where(", 13).unwrap();
        assert_eq!(name, "where");
        assert_eq!(param, 0);
    }

    #[test]
    fn test_detect_function_after_method() {
        let (name, param) =
            SignatureHelpProvider::detect_function_context("Patient.name.where(", 19).unwrap();
        assert_eq!(name, "where");
        assert_eq!(param, 0);
    }

    #[test]
    fn test_no_function_context() {
        assert!(SignatureHelpProvider::detect_function_context("Patient.name", 12).is_none());
    }
}
