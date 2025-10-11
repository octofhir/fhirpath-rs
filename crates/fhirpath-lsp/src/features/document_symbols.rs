//! Document symbol provider for outline view

use crate::directives::{DirectiveContent, DirectiveKind};
use crate::document::FhirPathDocument;
use lsp_types::{DocumentSymbol, DocumentSymbolResponse, SymbolKind};

/// Generate document symbols for outline view
pub fn generate_document_symbols(document: &FhirPathDocument) -> DocumentSymbolResponse {
    let mut symbols = Vec::new();

    // Add directive symbols
    for directive in &document.directives {
        let (name, detail) = match (&directive.kind, &directive.content) {
            (DirectiveKind::Input, DirectiveContent::InlineResource(resource)) => {
                let resource_type = resource
                    .get("resourceType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let id = resource
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed");
                (
                    format!("@input: {} ({})", resource_type, id),
                    Some(format!("Inline {} resource", resource_type)),
                )
            }
            (DirectiveKind::InputFile, DirectiveContent::FilePath { path, resolved }) => {
                let status = if resolved.is_some() { "✓" } else { "✗" };
                (
                    format!("@input-file: {} {}", status, path),
                    Some(format!("External resource file: {}", path)),
                )
            }
            _ => ("@directive".to_string(), None),
        };

        symbols.push(DocumentSymbol {
            name,
            detail,
            kind: SymbolKind::MODULE,
            range: directive.range,
            selection_range: directive.range,
            children: None,
            #[allow(deprecated)]
            deprecated: None,
            tags: None,
        });
    }

    // Add expression symbols
    for (i, expr) in document.expressions.iter().enumerate() {
        let name = if expr.text.len() > 50 {
            format!("Expression {}: {}...", i + 1, &expr.text[..47])
        } else {
            format!("Expression {}: {}", i + 1, expr.text)
        };

        symbols.push(DocumentSymbol {
            name,
            detail: Some(expr.text.clone()),
            kind: SymbolKind::FUNCTION,
            range: expr.range,
            selection_range: expr.range,
            children: None,
            #[allow(deprecated)]
            deprecated: None,
            tags: None,
        });
    }

    DocumentSymbolResponse::Nested(symbols)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Url;

    #[test]
    fn test_generate_document_symbols_empty() {
        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, String::new(), 1);

        let symbols = generate_document_symbols(&doc);

        match symbols {
            DocumentSymbolResponse::Nested(symbols) => {
                assert_eq!(symbols.len(), 0);
            }
            _ => panic!("Expected nested symbols"),
        }
    }

    #[test]
    fn test_generate_document_symbols_with_expressions() {
        let text = "Patient.name.family; Patient.birthDate";
        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, text.to_string(), 1);

        let symbols = generate_document_symbols(&doc);

        match symbols {
            DocumentSymbolResponse::Nested(symbols) => {
                assert_eq!(symbols.len(), 2);
                assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
                assert_eq!(symbols[1].kind, SymbolKind::FUNCTION);
                assert!(symbols[0].name.contains("Patient.name.family"));
                assert!(symbols[1].name.contains("Patient.birthDate"));
            }
            _ => panic!("Expected nested symbols"),
        }
    }

    #[test]
    fn test_generate_document_symbols_with_directive() {
        let text = r#"/**
 * @input-file ./examples/patient.json
 */

Patient.name.family"#;

        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, text.to_string(), 1);

        let symbols = generate_document_symbols(&doc);

        match symbols {
            DocumentSymbolResponse::Nested(symbols) => {
                // Should have 1 directive + 1 expression
                assert_eq!(symbols.len(), 2);
                assert_eq!(symbols[0].kind, SymbolKind::MODULE);
                assert!(symbols[0].name.contains("@input-file"));
                assert_eq!(symbols[1].kind, SymbolKind::FUNCTION);
            }
            _ => panic!("Expected nested symbols"),
        }
    }

    #[test]
    fn test_generate_document_symbols_with_inline_input() {
        let text = r#"/**
 * @input {
 *   "resourceType": "Patient",
 *   "id": "example"
 * }
 */

Patient.name.family"#;

        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, text.to_string(), 1);

        let symbols = generate_document_symbols(&doc);

        match symbols {
            DocumentSymbolResponse::Nested(symbols) => {
                // Should have 1 directive + 1 expression
                assert_eq!(symbols.len(), 2);
                assert_eq!(symbols[0].kind, SymbolKind::MODULE);
                assert!(symbols[0].name.contains("@input"));
                assert!(symbols[0].name.contains("Patient"));
                assert_eq!(symbols[1].kind, SymbolKind::FUNCTION);
            }
            _ => panic!("Expected nested symbols"),
        }
    }

    #[test]
    fn test_generate_document_symbols_long_expression() {
        let text = "Patient.name.where(use = 'official').family.where(length() > 10)";
        let uri = Url::parse("file:///test.fhirpath").unwrap();
        let doc = FhirPathDocument::new(uri, text.to_string(), 1);

        let symbols = generate_document_symbols(&doc);

        match symbols {
            DocumentSymbolResponse::Nested(symbols) => {
                assert_eq!(symbols.len(), 1);
                // Long expressions should be truncated in the name
                assert!(symbols[0].name.len() < 100);
                // But full text should be in detail
                assert_eq!(symbols[0].detail.as_ref().unwrap(), text);
            }
            _ => panic!("Expected nested symbols"),
        }
    }
}
