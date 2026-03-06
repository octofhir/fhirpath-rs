//! Code action provider for FHIRPath expressions

use crate::diagnostics::AriadneDiagnostic;

use lsp_types::{CodeAction, CodeActionKind, TextEdit, Url, WorkspaceEdit};
use std::collections::HashMap;

/// Provider for code actions (quick fixes) based on diagnostics
pub struct CodeActionProvider;

impl CodeActionProvider {
    /// Generate code actions from diagnostics for the given range
    pub fn provide(
        diagnostics: &[AriadneDiagnostic],
        source_text: &str,
        document_uri: &Url,
        request_diagnostics: &[lsp_types::Diagnostic],
    ) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        for diag in diagnostics {
            // Extract "Did you mean 'X'?" suggestions
            if let Some(ref help) = diag.help
                && let Some(suggestion) = Self::extract_suggestion(help)
            {
                let range = Self::span_to_range(&diag.span, source_text);

                // Find matching LSP diagnostic
                let matching_diag = request_diagnostics
                    .iter()
                    .find(|d| d.range == range && d.message.contains(&diag.message));

                let action = CodeAction {
                    title: format!("Replace with '{}'", suggestion),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: matching_diag.cloned().map(|d| vec![d]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            document_uri.clone(),
                            vec![TextEdit {
                                range,
                                new_text: suggestion.to_string(),
                            }],
                        )])),
                        ..Default::default()
                    }),
                    is_preferred: Some(true),
                    ..Default::default()
                };

                actions.push(action);
            }

            // Extract replacement suggestions from help/note
            if let Some(ref note) = diag.note
                && let Some(suggestion) = Self::extract_suggestion(note)
            {
                let range = Self::span_to_range(&diag.span, source_text);

                let matching_diag = request_diagnostics.iter().find(|d| d.range == range);

                actions.push(CodeAction {
                    title: format!("Replace with '{}'", suggestion),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: matching_diag.cloned().map(|d| vec![d]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            document_uri.clone(),
                            vec![TextEdit {
                                range,
                                new_text: suggestion.to_string(),
                            }],
                        )])),
                        ..Default::default()
                    }),
                    is_preferred: Some(false),
                    ..Default::default()
                });
            }
        }

        actions
    }

    /// Extract a suggested replacement from help text like "Did you mean 'name'?"
    fn extract_suggestion(text: &str) -> Option<&str> {
        // Pattern: "Did you mean 'X'?"
        if let Some(start) = text.find('\'') {
            let after = &text[start + 1..];
            if let Some(end) = after.find('\'') {
                return Some(&after[..end]);
            }
        }
        // Pattern: "Did you mean `X`?"
        if let Some(start) = text.find('`') {
            let after = &text[start + 1..];
            if let Some(end) = after.find('`') {
                return Some(&after[..end]);
            }
        }
        None
    }

    /// Convert a byte span to an LSP Range
    fn span_to_range(span: &std::ops::Range<usize>, text: &str) -> lsp_types::Range {
        let start = Self::offset_to_position(span.start, text);
        let end = Self::offset_to_position(span.end, text);
        lsp_types::Range { start, end }
    }

    /// Convert byte offset to LSP Position
    fn offset_to_position(offset: usize, text: &str) -> lsp_types::Position {
        let mut line = 0u32;
        let mut col = 0u32;
        for (i, ch) in text.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        lsp_types::Position {
            line,
            character: col,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_suggestion_single_quotes() {
        assert_eq!(
            CodeActionProvider::extract_suggestion("Did you mean 'name'?"),
            Some("name")
        );
    }

    #[test]
    fn test_extract_suggestion_backticks() {
        assert_eq!(
            CodeActionProvider::extract_suggestion("Did you mean `name`?"),
            Some("name")
        );
    }

    #[test]
    fn test_extract_suggestion_none() {
        assert_eq!(
            CodeActionProvider::extract_suggestion("No suggestion here"),
            None
        );
    }
}
