//! Go to definition provider for FHIRPath expressions
//!
//! Resolves property accesses to FHIR element definitions.

use crate::core::ModelProvider;
use crate::lsp::LspContext;
use crate::lsp::completion::CompletionContext;

use lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};
use std::sync::Arc;

/// Provider for go-to-definition on FHIRPath expressions
pub struct GotoDefinitionProvider {
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
}

impl GotoDefinitionProvider {
    /// Create a new goto definition provider
    pub fn new(model_provider: Arc<dyn ModelProvider + Send + Sync>) -> Self {
        Self { model_provider }
    }

    /// Provide definition location for the token at the given cursor offset
    pub async fn provide(
        &self,
        expression: &str,
        cursor_offset: usize,
        lsp_context: &LspContext,
    ) -> Option<GotoDefinitionResponse> {
        // Extract token at cursor
        let token = Self::extract_token_at(expression, cursor_offset)?;

        // Determine if this is a property access
        let before = &expression[..cursor_offset.min(expression.len())];
        let token_start = before
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        // Extract the property chain to resolve the parent type
        let (base_type, properties) =
            CompletionContext::extract_property_chain(expression, token_start);

        let start_type = lsp_context.resource_type.as_ref().cloned().or(base_type)?;

        // Traverse the property chain to find the parent type
        let mut current_type = start_type;
        for prop in &properties {
            let type_info = self.model_provider.get_type(&current_type).await.ok()??;
            let element_type = self
                .model_provider
                .get_element_type(&type_info, prop)
                .await
                .ok()??;
            current_type = element_type.type_name;
        }

        // Look up the token on the current type to get element info
        let elements = self.model_provider.get_elements(&current_type).await.ok()?;
        // Verify the element exists on this type
        elements.iter().find(|e| e.name == token)?;

        // Build a FHIR specification URL for the element
        let fhir_url = format!(
            "http://hl7.org/fhir/StructureDefinition/{}/#{}.{}",
            current_type, current_type, token
        );

        // Return as a link (URI-based location)
        // LSP clients that support it will open the URL; others will show it
        let url = Url::parse(&fhir_url).ok()?;
        let zero_range = Range {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        };

        Some(GotoDefinitionResponse::Scalar(Location {
            uri: url,
            range: zero_range,
        }))
    }

    /// Extract the token at the given cursor position
    fn extract_token_at(expression: &str, cursor_offset: usize) -> Option<String> {
        let offset = cursor_offset.min(expression.len());

        let is_token_char = |c: char| c.is_alphanumeric() || c == '_';

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token() {
        assert_eq!(
            GotoDefinitionProvider::extract_token_at("Patient.name", 10),
            Some("name".to_string())
        );
        assert_eq!(
            GotoDefinitionProvider::extract_token_at("Patient.name", 3),
            Some("Patient".to_string())
        );
    }
}
