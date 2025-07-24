//! LSP (Language Server Protocol) integration

use crate::diagnostic::{Diagnostic, DiagnosticCode, Severity};
use lsp_types;

/// Convert a diagnostic to LSP diagnostic
pub fn to_lsp_diagnostic(diagnostic: &Diagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: to_lsp_range(&diagnostic.location),
        severity: Some(to_lsp_severity(diagnostic.severity)),
        code: Some(lsp_types::NumberOrString::String(diagnostic.code_string())),
        code_description: None,
        source: Some("fhirpath".to_string()),
        message: diagnostic.message.clone(),
        related_information: if diagnostic.related.is_empty() {
            None
        } else {
            Some(
                diagnostic
                    .related
                    .iter()
                    .map(|r| lsp_types::DiagnosticRelatedInformation {
                        location: lsp_types::Location {
                            uri: lsp_types::Url::parse("file:///").unwrap(), // TODO: proper URI
                            range: to_lsp_range(&r.location),
                        },
                        message: r.message.clone(),
                    })
                    .collect(),
            )
        },
        tags: None,
        data: None,
    }
}

/// Convert severity to LSP severity
fn to_lsp_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
        Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
        Severity::Info => lsp_types::DiagnosticSeverity::INFORMATION,
        Severity::Hint => lsp_types::DiagnosticSeverity::HINT,
    }
}

/// Convert source location to LSP range
fn to_lsp_range(location: &crate::location::SourceLocation) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: location.span.start.line as u32,
            character: location.span.start.column as u32,
        },
        end: lsp_types::Position {
            line: location.span.end.line as u32,
            character: location.span.end.column as u32,
        },
    }
}

/// Convert suggestions to LSP code actions
pub fn to_lsp_code_actions(
    diagnostic: &Diagnostic,
    uri: &lsp_types::Url,
) -> Vec<lsp_types::CodeAction> {
    diagnostic
        .suggestions
        .iter()
        .filter_map(|suggestion| {
            suggestion.replacement.as_ref().map(|replacement| {
                let mut changes = std::collections::HashMap::new();
                changes.insert(
                    uri.clone(),
                    vec![lsp_types::TextEdit {
                        range: to_lsp_range(&suggestion.location),
                        new_text: replacement.clone(),
                    }],
                );

                lsp_types::CodeAction {
                    title: suggestion.message.clone(),
                    kind: Some(lsp_types::CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![to_lsp_diagnostic(diagnostic)]),
                    edit: Some(lsp_types::WorkspaceEdit {
                        changes: Some(changes),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(true),
                    disabled: None,
                    data: None,
                }
            })
        })
        .collect()
}

/// Create an LSP diagnostic with quick fixes
pub struct LspDiagnostic {
    /// The diagnostic
    pub diagnostic: lsp_types::Diagnostic,
    /// Associated code actions
    pub code_actions: Vec<lsp_types::CodeAction>,
}

impl From<&Diagnostic> for LspDiagnostic {
    fn from(diagnostic: &Diagnostic) -> Self {
        let uri = lsp_types::Url::parse("file:///").unwrap(); // TODO: proper URI handling
        Self {
            diagnostic: to_lsp_diagnostic(diagnostic),
            code_actions: to_lsp_code_actions(diagnostic, &uri),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::DiagnosticBuilder;
    use crate::location::{Position, Span};

    #[test]
    fn test_lsp_conversion() {
        let diagnostic = DiagnosticBuilder::unknown_function("foo")
            .with_span(Span::new(Position::new(5, 10), Position::new(5, 13)))
            .suggest("Did you mean 'for'?", Some("for".to_string()))
            .build();

        let lsp_diag = to_lsp_diagnostic(&diagnostic);
        
        assert_eq!(lsp_diag.severity, Some(lsp_types::DiagnosticSeverity::ERROR));
        assert_eq!(lsp_diag.message, "Unknown function 'foo'");
        assert_eq!(lsp_diag.range.start.line, 5);
        assert_eq!(lsp_diag.range.start.character, 10);
        assert_eq!(lsp_diag.range.end.line, 5);
        assert_eq!(lsp_diag.range.end.character, 13);
    }

    #[test]
    fn test_code_actions() {
        let diagnostic = DiagnosticBuilder::unknown_function("foo")
            .with_span(Span::new(Position::new(0, 0), Position::new(0, 3)))
            .suggest("Did you mean 'for'?", Some("for".to_string()))
            .suggest("Did you mean 'first'?", Some("first".to_string()))
            .build();

        let uri = lsp_types::Url::parse("file:///test.fhirpath").unwrap();
        let actions = to_lsp_code_actions(&diagnostic, &uri);

        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].title, "Did you mean 'for'?");
        assert_eq!(actions[1].title, "Did you mean 'first'?");
    }
}