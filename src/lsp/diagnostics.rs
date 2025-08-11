//! LSP diagnostics implementation
//!
//! Provides diagnostic publishing and error reporting functionality.

use crate::analyzer::FhirPathAnalyzer;
use crate::model::provider::ModelProvider;
use lsp_types::*;

/// Diagnostic publisher for the LSP server
pub struct DiagnosticPublisher<P: ModelProvider> {
    analyzer: std::sync::Arc<FhirPathAnalyzer<P>>,
}

impl<P: ModelProvider> DiagnosticPublisher<P> {
    /// Create a new diagnostic publisher
    pub fn new(analyzer: std::sync::Arc<FhirPathAnalyzer<P>>) -> Self {
        Self { analyzer }
    }

    /// Analyze document and generate diagnostics
    pub async fn analyze_document(
        &self,
        uri: &Url,
        text: &str,
    ) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error + Send + Sync>> {
        // Parse the expression
        let expression = match crate::parser::parse(text) {
            Ok(expr) => expr,
            Err(parse_error) => {
                // Return parse error as diagnostic
                return Ok(vec![Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, text.len() as u32)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("parse-error".to_string())),
                    code_description: None,
                    source: Some("fhirpath".to_string()),
                    message: format!("Parse error: {}", parse_error),
                    related_information: None,
                    tags: None,
                    data: None,
                }]);
            }
        };

        // Analyze the expression
        let analysis_result = self
            .analyzer
            .analyze(&expression, Some("Resource"))
            .await
            .map_err(|e| format!("Analysis error: {}", e))?;

        // Convert analyzer diagnostics to LSP diagnostics
        let mut diagnostics = Vec::new();
        for analyzer_diagnostic in analysis_result.diagnostics {
            diagnostics.push(convert_diagnostic(analyzer_diagnostic, text));
        }

        Ok(diagnostics)
    }
}

/// Convert analyzer diagnostic to LSP diagnostic
fn convert_diagnostic(
    diagnostic: crate::analyzer::diagnostics::Diagnostic,
    text: &str,
) -> Diagnostic {
    let range = span_to_range(&diagnostic.span, text);

    Diagnostic {
        range,
        severity: Some(match diagnostic.severity {
            crate::analyzer::diagnostics::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
            crate::analyzer::diagnostics::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
            crate::analyzer::diagnostics::DiagnosticSeverity::Information => {
                DiagnosticSeverity::INFORMATION
            }
            crate::analyzer::diagnostics::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
        }),
        code: diagnostic.code.map(NumberOrString::String),
        code_description: None,
        source: Some("fhirpath".to_string()),
        message: diagnostic.message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert span to LSP range
fn span_to_range(span: &Option<crate::parser::span::Span>, text: &str) -> Range {
    if let Some(span) = span {
        let start_pos = offset_to_position(span.start, text);
        let end_pos = offset_to_position(span.end, text);
        Range::new(start_pos, end_pos)
    } else {
        // Default to start of document if no span
        Range::new(Position::new(0, 0), Position::new(0, 0))
    }
}

/// Convert byte offset to LSP position
fn offset_to_position(offset: usize, text: &str) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;

    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }

    Position::new(line, character)
}