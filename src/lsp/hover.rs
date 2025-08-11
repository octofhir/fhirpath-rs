//! LSP hover implementation
//!
//! Provides hover information including type details, documentation, and examples.

use crate::analyzer::FhirPathAnalyzer;
use crate::model::provider::ModelProvider;
use lsp_types::*;

/// Get hover information for a position in the document
pub async fn get_hover<P: ModelProvider>(
    _text: &str,
    _position: Position,
    _analyzer: &FhirPathAnalyzer<P>,
) -> Result<Option<Hover>, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement hover functionality
    // 1. Parse expression at position
    // 2. Get type information from analyzer
    // 3. Get documentation from ModelProvider
    // 4. Format as markdown hover content
    
    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**FHIRPath Expression**\n\nHover information coming soon!".to_string(),
        }),
        range: None,
    }))
}