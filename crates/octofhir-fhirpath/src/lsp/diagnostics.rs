//! Diagnostic conversion from AriadneDiagnostic to LSP Diagnostic

use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity as LspSeverity, Location,
    NumberOrString, Position, Range, Url,
};

/// Provider for converting FHIRPath diagnostics to LSP format
pub struct DiagnosticProvider;

impl DiagnosticProvider {
    /// Convert an AriadneDiagnostic to an LSP Diagnostic
    pub fn convert(diagnostic: &AriadneDiagnostic, source_text: &str) -> Diagnostic {
        let range = Self::span_to_range(&diagnostic.span, source_text);
        let severity = Self::convert_severity(&diagnostic.severity);

        let mut message = diagnostic.message.clone();

        // Append help text if available
        if let Some(ref help) = diagnostic.help {
            message.push_str("\n\nHelp: ");
            message.push_str(help);
        }

        // Append note if available
        if let Some(ref note) = diagnostic.note {
            message.push_str("\n\nNote: ");
            message.push_str(note);
        }

        Diagnostic {
            range,
            severity: Some(severity),
            code: Some(NumberOrString::String(diagnostic.error_code.to_string())),
            code_description: None,
            source: Some("fhirpath".to_string()),
            message,
            related_information: Self::convert_related(&diagnostic.related, source_text),
            tags: None,
            data: None,
        }
    }

    /// Convert an AriadneDiagnostic to an LSP Diagnostic with document URI for related info
    pub fn convert_with_uri(
        diagnostic: &AriadneDiagnostic,
        source_text: &str,
        document_uri: &Url,
    ) -> Diagnostic {
        let range = Self::span_to_range(&diagnostic.span, source_text);
        let severity = Self::convert_severity(&diagnostic.severity);

        let mut message = diagnostic.message.clone();

        // Append help text if available
        if let Some(ref help) = diagnostic.help {
            message.push_str("\n\nHelp: ");
            message.push_str(help);
        }

        // Append note if available
        if let Some(ref note) = diagnostic.note {
            message.push_str("\n\nNote: ");
            message.push_str(note);
        }

        Diagnostic {
            range,
            severity: Some(severity),
            code: Some(NumberOrString::String(diagnostic.error_code.to_string())),
            code_description: None,
            source: Some("fhirpath".to_string()),
            message,
            related_information: Self::convert_related_with_uri(
                &diagnostic.related,
                source_text,
                document_uri,
            ),
            tags: None,
            data: None,
        }
    }

    /// Convert a span (byte range) to an LSP Range
    fn span_to_range(span: &std::ops::Range<usize>, text: &str) -> Range {
        let start = Self::offset_to_position(span.start, text);
        let end = Self::offset_to_position(span.end, text);
        Range { start, end }
    }

    /// Convert a byte offset to an LSP Position (line, character)
    fn offset_to_position(offset: usize, text: &str) -> Position {
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

        Position {
            line,
            character: col,
        }
    }

    /// Convert DiagnosticSeverity to LSP severity
    fn convert_severity(severity: &DiagnosticSeverity) -> LspSeverity {
        match severity {
            DiagnosticSeverity::Error => LspSeverity::ERROR,
            DiagnosticSeverity::Warning => LspSeverity::WARNING,
            DiagnosticSeverity::Info => LspSeverity::INFORMATION,
            DiagnosticSeverity::Hint => LspSeverity::HINT,
        }
    }

    /// Convert related diagnostics (without URI - uses placeholder)
    fn convert_related(
        related: &[crate::diagnostics::RelatedDiagnostic],
        source_text: &str,
    ) -> Option<Vec<DiagnosticRelatedInformation>> {
        if related.is_empty() {
            return None;
        }

        // Use a placeholder URI for same-file references
        let placeholder_uri = Url::parse("file:///unknown").unwrap_or_else(|_| {
            // Fallback in case parsing fails
            Url::parse("file:///").unwrap()
        });

        let infos: Vec<_> = related
            .iter()
            .map(|r| DiagnosticRelatedInformation {
                location: Location {
                    uri: placeholder_uri.clone(),
                    range: Self::span_to_range(&r.span, source_text),
                },
                message: r.message.clone(),
            })
            .collect();

        Some(infos)
    }

    /// Convert related diagnostics with proper document URI
    fn convert_related_with_uri(
        related: &[crate::diagnostics::RelatedDiagnostic],
        source_text: &str,
        document_uri: &Url,
    ) -> Option<Vec<DiagnosticRelatedInformation>> {
        if related.is_empty() {
            return None;
        }

        let infos: Vec<_> = related
            .iter()
            .map(|r| DiagnosticRelatedInformation {
                location: Location {
                    uri: document_uri.clone(),
                    range: Self::span_to_range(&r.span, source_text),
                },
                message: r.message.clone(),
            })
            .collect();

        Some(infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error_code::FP0052;

    #[test]
    fn test_offset_to_position_single_line() {
        let text = "Patient.name";
        let pos = DiagnosticProvider::offset_to_position(8, text);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 8);
    }

    #[test]
    fn test_offset_to_position_multi_line() {
        let text = "Patient\n.name";
        let pos = DiagnosticProvider::offset_to_position(9, text);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 1);
    }

    #[test]
    fn test_convert_severity() {
        assert_eq!(
            DiagnosticProvider::convert_severity(&DiagnosticSeverity::Error),
            LspSeverity::ERROR
        );
        assert_eq!(
            DiagnosticProvider::convert_severity(&DiagnosticSeverity::Warning),
            LspSeverity::WARNING
        );
    }

    #[test]
    fn test_convert_diagnostic() {
        let ariadne_diag = AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: FP0052,
            message: "Unknown property 'nme'".to_string(),
            span: 8..11,
            help: Some("Did you mean 'name'?".to_string()),
            note: None,
            related: vec![],
        };

        let source = "Patient.nme";
        let lsp_diag = DiagnosticProvider::convert(&ariadne_diag, source);

        assert_eq!(lsp_diag.severity, Some(LspSeverity::ERROR));
        assert_eq!(lsp_diag.range.start.line, 0);
        assert_eq!(lsp_diag.range.start.character, 8);
        assert!(lsp_diag.message.contains("Unknown property"));
        assert!(lsp_diag.message.contains("Did you mean"));
    }
}
