//! Hover provider for FHIRPath expressions

use crate::core::ModelProvider;
use crate::evaluator::FunctionRegistry;
use crate::lsp::LspContext;
use crate::lsp::completion::CompletionContext;

use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use std::sync::Arc;

/// Provider for hover information on FHIRPath expressions
pub struct HoverProvider {
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    function_registry: Arc<FunctionRegistry>,
}

impl HoverProvider {
    /// Create a new hover provider
    pub fn new(
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        function_registry: Arc<FunctionRegistry>,
    ) -> Self {
        Self {
            model_provider,
            function_registry,
        }
    }

    /// Provide hover information at the given cursor offset
    pub async fn provide(
        &self,
        expression: &str,
        cursor_offset: usize,
        lsp_context: &LspContext,
    ) -> Option<Hover> {
        // First try to find the token at cursor
        let token = Self::extract_token_at(expression, cursor_offset)?;

        // Check for keywords
        if let Some(hover) = Self::keyword_hover(&token) {
            return Some(hover);
        }

        // Check for functions
        if let Some(hover) = self.function_hover(&token) {
            return Some(hover);
        }

        // Check for properties — resolve the type chain to determine context
        if let Some(hover) = self
            .property_hover(expression, cursor_offset, &token, lsp_context)
            .await
        {
            return Some(hover);
        }

        // Check for constants
        if token.starts_with('%') {
            return Self::constant_hover(&token, lsp_context);
        }

        None
    }

    /// Extract the token (word) at the given cursor offset
    fn extract_token_at(expression: &str, cursor_offset: usize) -> Option<String> {
        let offset = cursor_offset.min(expression.len());

        // Handle % prefix for constants
        if offset > 0 && expression.as_bytes().get(offset.wrapping_sub(1)) == Some(&b'%') {
            let end = expression[offset..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| offset + i)
                .unwrap_or(expression.len());
            let start = offset - 1;
            let token = &expression[start..end];
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }

        // Find word boundaries around cursor ($ is part of keyword tokens)
        let is_token_char = |c: char| c.is_alphanumeric() || c == '_' || c == '$';

        let before = &expression[..offset];
        let token_start = before
            .rfind(|c: char| !is_token_char(c))
            .map(|i| i + 1)
            .unwrap_or(0);

        let after = &expression[offset..];
        let token_end = after
            .find(|c: char| !is_token_char(c))
            .map(|i| offset + i)
            .unwrap_or(expression.len());

        let token = &expression[token_start..token_end];
        if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        }
    }

    /// Get hover for FHIRPath keywords
    fn keyword_hover(token: &str) -> Option<Hover> {
        let (detail, docs) = match token {
            "$this" => (
                "keyword `$this`",
                "References the current item in iteration contexts like `where()`, `select()`, or `repeat()`.",
            ),
            "$index" => (
                "keyword `$index`",
                "The zero-based index of the current item during iteration.",
            ),
            "$total" => (
                "keyword `$total`",
                "The running accumulator value in the `aggregate()` function.",
            ),
            "true" => ("literal `Boolean`", "Boolean true value."),
            "false" => ("literal `Boolean`", "Boolean false value."),
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\n{}", detail, docs),
            }),
            range: None,
        })
    }

    /// Get hover for FHIRPath functions
    fn function_hover(&self, token: &str) -> Option<Hover> {
        let meta = self.function_registry.get_metadata(token)?;

        let mut value = format!("**{}**", token);

        // Build signature
        let params: Vec<String> = meta
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

        value.push_str(&format!(
            "\n\n```\n{}({}) → {}\n```",
            token,
            params.join(", "),
            meta.signature.return_type
        ));

        if !meta.description.is_empty() {
            value.push_str(&format!("\n\n{}", meta.description));
        }

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        })
    }

    /// Get hover for property access — resolve type chain
    async fn property_hover(
        &self,
        expression: &str,
        cursor_offset: usize,
        token: &str,
        lsp_context: &LspContext,
    ) -> Option<Hover> {
        // Check if cursor is at a position preceded by '.' (property access)
        let before = &expression[..cursor_offset.min(expression.len())];
        let token_start = before
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        // Determine parent type by resolving the chain up to this property
        let (base_type, properties) =
            CompletionContext::extract_property_chain(expression, token_start);

        let start_type = lsp_context.resource_type.as_ref().cloned().or(base_type)?;

        // Traverse the property chain to find the parent type
        let mut current_type = start_type.clone();
        for prop in &properties {
            let type_info = self.model_provider.get_type(&current_type).await.ok()??;
            let element_type = self
                .model_provider
                .get_element_type(&type_info, prop)
                .await
                .ok()??;
            current_type = element_type.type_name;
        }

        // Now look up the token on the current type
        let type_info = self.model_provider.get_type(&current_type).await.ok()??;
        let element_type = self
            .model_provider
            .get_element_type(&type_info, token)
            .await
            .ok()??;

        // Get element details for documentation
        let elements = self.model_provider.get_elements(&current_type).await.ok()?;
        let element_info = elements.iter().find(|e| e.name == token);

        let mut value = format!("**{}.{}**", current_type, token);
        value.push_str(&format!("\n\nType: `{}`", element_type.type_name));

        if let Some(singleton) = element_type.singleton {
            if singleton {
                value.push_str(" (0..1)");
            } else {
                value.push_str(" (0..*)");
            }
        }

        if let Some(info) = element_info
            && let Some(ref doc) = info.documentation
        {
            value.push_str(&format!("\n\n{}", doc));
        }

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        })
    }

    /// Get hover for constants
    fn constant_hover(token: &str, lsp_context: &LspContext) -> Option<Hover> {
        let (type_name, description) = match token {
            "%resource" => ("Resource", "The root resource being evaluated"),
            "%context" => ("Element", "The evaluation context element"),
            "%ucum" => ("string", "UCUM unit system URL (http://unitsofmeasure.org)"),
            "%sct" => ("string", "SNOMED CT system URL (http://snomed.info/sct)"),
            "%loinc" => ("string", "LOINC system URL (http://loinc.org)"),
            _ => {
                // Check external constants
                let name = token.strip_prefix('%')?;
                if let Some(info) = lsp_context.constants.get(name) {
                    let desc = info.description.as_deref().unwrap_or("External constant");
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "**{}**\n\nType: `{}`\n\n{}",
                                token, info.type_name, desc
                            ),
                        }),
                        range: None,
                    });
                }
                return None;
            }
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\nType: `{}`\n\n{}", token, type_name, description),
            }),
            range: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_at_simple() {
        assert_eq!(
            HoverProvider::extract_token_at("Patient.name", 3),
            Some("Patient".to_string())
        );
    }

    #[test]
    fn test_extract_token_at_after_dot() {
        assert_eq!(
            HoverProvider::extract_token_at("Patient.name", 10),
            Some("name".to_string())
        );
    }

    #[test]
    fn test_extract_token_at_keyword() {
        assert_eq!(
            HoverProvider::extract_token_at("$this", 3),
            Some("$this".to_string())
        );
    }

    #[test]
    fn test_keyword_hover() {
        assert!(HoverProvider::keyword_hover("$this").is_some());
        assert!(HoverProvider::keyword_hover("$index").is_some());
        assert!(HoverProvider::keyword_hover("$total").is_some());
        assert!(HoverProvider::keyword_hover("something").is_none());
    }
}
