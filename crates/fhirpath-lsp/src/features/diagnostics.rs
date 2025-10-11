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

    // Check for directive errors (missing files and invalid resources)
    for directive in &document.directives {
        match &directive.content {
            DirectiveContent::FilePath { path, resolved } => {
                if resolved.is_none() {
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
            DirectiveContent::InlineResource(resource) => {
                // Validate FHIR resource structure
                diagnostics.extend(validate_fhir_resource(resource, directive.range));
            }
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

/// Validate a FHIR resource structure
fn validate_fhir_resource(resource: &serde_json::Value, range: Range) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Check if it's a JSON object
    if !resource.is_object() {
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("invalid-fhir-resource".to_string())),
            source: Some("fhirpath-lsp".to_string()),
            message: "FHIR resource must be a JSON object".to_string(),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        });
        return diagnostics;
    }

    let obj = resource.as_object().unwrap();

    // Check for required resourceType field
    if !obj.contains_key("resourceType") {
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("missing-resource-type".to_string())),
            source: Some("fhirpath-lsp".to_string()),
            message: "FHIR resource must have a 'resourceType' field".to_string(),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        });
    } else if let Some(resource_type) = obj.get("resourceType") {
        // Validate resourceType is a string
        if !resource_type.is_string() {
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("invalid-resource-type".to_string())),
                source: Some("fhirpath-lsp".to_string()),
                message: "'resourceType' must be a string".to_string(),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        } else {
            let rt = resource_type.as_str().unwrap();
            // Check if it's a valid FHIR resource type (basic validation)
            if rt.is_empty() || !rt.chars().next().unwrap().is_uppercase() {
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("unusual-resource-type".to_string())),
                    source: Some("fhirpath-lsp".to_string()),
                    message: format!(
                        "Resource type '{}' may not be a valid FHIR resource type",
                        rt
                    ),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }
        }
    }

    // Check for common structural issues
    if let Some(id) = obj.get("id")
        && !id.is_string()
    {
        diagnostics.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: Some(NumberOrString::String("invalid-id-type".to_string())),
            source: Some("fhirpath-lsp".to_string()),
            message: "'id' field should be a string".to_string(),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        });
    }

    // TODO: Use octofhir-fhirschema for comprehensive validation
    // This is a basic validation - full schema validation should be added
    // using the FhirSchemaModelProvider integration

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
