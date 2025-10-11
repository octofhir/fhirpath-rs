//! Diagnostic generation for FHIRPath documents

use crate::directives::DirectiveContent;
use crate::document::FhirPathDocument;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};
use octofhir_fhirpath::FhirPathEngine;

/// Generate diagnostics for a document
pub async fn generate_diagnostics(
    document: &FhirPathDocument,
    _engine: &FhirPathEngine,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check for directive errors (missing files)
    for directive in &document.directives {
        if let DirectiveContent::FilePath { path, resolved } = &directive.content
            && resolved.is_none()
        {
            diagnostics.push(Diagnostic {
                range: directive.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("file-not-found".to_string())),
                source: Some("fhirpath-lsp".to_string()),
                message: format!("File not found: {}", path),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }
    }

    // Check each expression for syntax errors
    for expr in &document.expressions {
        match octofhir_fhirpath::parse_expression(&expr.text) {
            Ok(_ast) => {
                // TODO: Run static analyzer for semantic validation
                // This will be implemented in future tasks
                // let analysis = analyzer.analyze(&ast, &context);
                // Convert warnings/errors to diagnostics
            }
            Err(parse_error) => {
                // Syntax error detected
                diagnostics.push(create_syntax_error_diagnostic(
                    &expr.text,
                    &parse_error.to_string(),
                    expr.range,
                ));
            }
        }
    }

    diagnostics
}

/// Create a syntax error diagnostic
fn create_syntax_error_diagnostic(
    _expression: &str,
    error_message: &str,
    range: Range,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("syntax-error".to_string())),
        source: Some("fhirpath-lsp".to_string()),
        message: format!("Syntax error: {}", error_message),
        related_information: None,
        tags: None,
        code_description: None,
        data: None,
    }
}

/// Create a semantic error diagnostic (placeholder for future use)
#[allow(dead_code)]
fn create_semantic_error_diagnostic(error_message: &str, range: Range) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("semantic-error".to_string())),
        source: Some("fhirpath-lsp".to_string()),
        message: error_message.to_string(),
        related_information: None,
        tags: None,
        code_description: None,
        data: None,
    }
}

/// Create a warning diagnostic (placeholder for future use)
#[allow(dead_code)]
fn create_warning_diagnostic(message: &str, range: Range) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::WARNING),
        code: None,
        source: Some("fhirpath-lsp".to_string()),
        message: message.to_string(),
        related_information: None,
        tags: None,
        code_description: None,
        data: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;
    use url::Url;

    #[tokio::test]
    async fn test_syntax_error_diagnostic() {
        let text = "Patient.name."; // Invalid syntax - trailing dot
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            text.to_string(),
            1,
        );

        // We can't test with actual engine yet, but we can test the structure
        assert_eq!(doc.expressions.len(), 1);
        assert_eq!(doc.expressions[0].text, "Patient.name.");
    }

    #[test]
    fn test_create_syntax_error_diagnostic() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diag =
            create_syntax_error_diagnostic("Patient.name.", "Unexpected end of input", range);

        assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diag.source, Some("fhirpath-lsp".to_string()));
        assert!(diag.message.contains("Syntax error"));
    }

    #[test]
    fn test_create_warning_diagnostic() {
        let range = Range::new(Position::new(0, 0), Position::new(0, 10));
        let diag = create_warning_diagnostic("Unused variable", range);

        assert_eq!(diag.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diag.source, Some("fhirpath-lsp".to_string()));
    }
}
