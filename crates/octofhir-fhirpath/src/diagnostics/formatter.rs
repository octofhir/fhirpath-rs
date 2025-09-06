//! Diagnostic formatter for different output contexts and formats

use super::{AriadneDiagnostic, DiagnosticEngine, DiagnosticSeverity};
use serde_json::{Value, json};
use std::io::Write;

/// Formatter for diagnostic output in different contexts
pub struct DiagnosticFormatter;

impl DiagnosticFormatter {
    /// Format diagnostic for CLI pretty output (pure Ariadne output like Rust compiler)
    pub fn format_pretty(
        engine: &DiagnosticEngine,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Return pure Ariadne output - no extra headers or emojis
        engine.format_diagnostic(diagnostic, source_id)
    }

    /// Format diagnostic for CLI raw output (no colors, just text)
    pub fn format_raw(diagnostic: &AriadneDiagnostic) -> String {
        format!(
            "[{}] {}: {}\n  at span {}..{}\n  help: {}\n  docs: {}",
            diagnostic.error_code.code_str(),
            match diagnostic.severity {
                DiagnosticSeverity::Error => "error",
                DiagnosticSeverity::Warning => "warning",
                DiagnosticSeverity::Info => "info",
                DiagnosticSeverity::Hint => "hint",
            },
            diagnostic.message,
            diagnostic.span.start,
            diagnostic.span.end,
            diagnostic.help.as_deref().unwrap_or("(none)"),
            diagnostic.error_code.docs_url()
        )
    }

    /// Format diagnostic for JSON output (structured data)
    pub fn format_json(diagnostic: &AriadneDiagnostic) -> Value {
        json!({
            "error_code": diagnostic.error_code.code_str(),
            "severity": match diagnostic.severity {
                DiagnosticSeverity::Error => "error",
                DiagnosticSeverity::Warning => "warning",
                DiagnosticSeverity::Info => "info",
                DiagnosticSeverity::Hint => "hint",
            },
            "message": diagnostic.message,
            "span": {
                "start": diagnostic.span.start,
                "end": diagnostic.span.end
            },
            "help": diagnostic.help,
            "note": diagnostic.note,
            "docs_url": diagnostic.error_code.docs_url(),
            "related": diagnostic.related.iter().map(|r| json!({
                "message": r.message,
                "span": {
                    "start": r.span.start,
                    "end": r.span.end
                },
                "severity": match r.severity {
                    DiagnosticSeverity::Error => "error",
                    DiagnosticSeverity::Warning => "warning",
                    DiagnosticSeverity::Info => "info",
                    DiagnosticSeverity::Hint => "hint",
                }
            })).collect::<Vec<_>>()
        })
    }

    /// Format multiple diagnostics as batch report (pure Ariadne output)
    pub fn format_batch_pretty(
        engine: &DiagnosticEngine,
        diagnostics: &[AriadneDiagnostic],
        source_id: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut output = String::new();

        // Output each diagnostic with pure Ariadne formatting
        for diagnostic in diagnostics {
            let ariadne_output = engine.format_diagnostic(diagnostic, source_id)?;
            output.push_str(&ariadne_output);
            // Add a blank line between diagnostics for readability
            output.push('\n');
        }

        Ok(output)
    }

    /// Format multiple diagnostics as raw text
    pub fn format_batch_raw(diagnostics: &[AriadneDiagnostic]) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "FHIRPath Analysis: {} issues found\n\n",
            diagnostics.len()
        ));

        for (i, diagnostic) in diagnostics.iter().enumerate() {
            output.push_str(&format!(
                "Issue {}: {}\n",
                i + 1,
                Self::format_raw(diagnostic)
            ));
            output.push('\n');
        }

        output
    }

    /// Format multiple diagnostics as JSON array
    pub fn format_batch_json(diagnostics: &[AriadneDiagnostic]) -> Value {
        json!({
            "diagnostics": diagnostics.iter().map(Self::format_json).collect::<Vec<_>>(),
            "summary": {
                "total_count": diagnostics.len(),
                "error_count": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Error)).count(),
                "warning_count": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Warning)).count(),
                "info_count": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Info)).count(),
                "hint_count": diagnostics.iter().filter(|d| matches!(d.severity, DiagnosticSeverity::Hint)).count(),
            }
        })
    }

    /// Format diagnostic table output (for structured display)
    pub fn format_table(diagnostics: &[AriadneDiagnostic]) -> String {
        if diagnostics.is_empty() {
            return "No diagnostics to display".to_string();
        }

        let mut output = String::new();

        // Header
        output.push_str("┌─────────┬─────────┬──────────┬─────────────────────────────────────────────────────────┐\n");
        output.push_str("│ Code    │ Level   │ Span     │ Message                                                 │\n");
        output.push_str("├─────────┼─────────┼──────────┼─────────────────────────────────────────────────────────┤\n");

        // Rows
        for diagnostic in diagnostics {
            let code = diagnostic.error_code.code_str();
            let level = match diagnostic.severity {
                DiagnosticSeverity::Error => "ERROR",
                DiagnosticSeverity::Warning => "WARN ",
                DiagnosticSeverity::Info => "INFO ",
                DiagnosticSeverity::Hint => "HINT ",
            };
            let span = format!("{}..{}", diagnostic.span.start, diagnostic.span.end);
            let message = if diagnostic.message.len() > 55 {
                format!("{}...", &diagnostic.message[..52])
            } else {
                diagnostic.message.clone()
            };

            output.push_str(&format!(
                "│ {:<7} │ {:<7} │ {:<8} │ {:<55} │\n",
                code, level, span, message
            ));
        }

        // Footer
        output.push_str("└─────────┴─────────┴──────────┴─────────────────────────────────────────────────────────┘\n");

        output
    }

    /// Write diagnostic to a writer (useful for streaming output)
    pub fn write_diagnostic<W: Write>(
        engine: &DiagnosticEngine,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        engine.emit_diagnostic(diagnostic, source_id, writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error_code::*;
    use crate::diagnostics::{DiagnosticEngine, DiagnosticSeverity};

    fn create_test_diagnostic() -> AriadneDiagnostic {
        AriadneDiagnostic {
            error_code: FP0055,
            severity: DiagnosticSeverity::Error,
            message: "Property 'invalid' not found".to_string(),
            span: 13..20,
            help: Some("Check available properties".to_string()),
            note: Some("Property must exist in FHIR schema".to_string()),
            related: vec![],
        }
    }

    #[test]
    fn test_format_raw() {
        let diagnostic = create_test_diagnostic();
        let raw = DiagnosticFormatter::format_raw(&diagnostic);

        assert!(raw.contains("[FP0055]"));
        assert!(raw.contains("error"));
        assert!(raw.contains("Property 'invalid' not found"));
        assert!(raw.contains("13..20"));
        assert!(raw.contains("Check available properties"));
    }

    #[test]
    fn test_format_json() {
        let diagnostic = create_test_diagnostic();
        let json = DiagnosticFormatter::format_json(&diagnostic);

        assert_eq!(json["error_code"], "FP0055");
        assert_eq!(json["severity"], "error");
        assert_eq!(json["message"], "Property 'invalid' not found");
        assert_eq!(json["span"]["start"], 13);
        assert_eq!(json["span"]["end"], 20);
        assert_eq!(json["help"], "Check available properties");
        assert_eq!(json["note"], "Property must exist in FHIR schema");
    }

    #[test]
    fn test_format_table() {
        let diagnostics = vec![
            create_test_diagnostic(),
            AriadneDiagnostic {
                error_code: FP0002,
                severity: DiagnosticSeverity::Warning,
                message: "Unused variable".to_string(),
                span: 5..10,
                help: None,
                note: None,
                related: vec![],
            },
        ];

        let table = DiagnosticFormatter::format_table(&diagnostics);

        assert!(table.contains("FP0055"));
        assert!(table.contains("FP0002"));
        assert!(table.contains("ERROR"));
        assert!(table.contains("WARN"));
        assert!(table.contains("Property 'invalid' not found"));
        assert!(table.contains("Unused variable"));
    }

    #[test]
    fn test_batch_formatting() {
        let diagnostics = vec![
            create_test_diagnostic(),
            AriadneDiagnostic {
                error_code: FP0001,
                severity: DiagnosticSeverity::Info,
                message: "Information message".to_string(),
                span: 0..5,
                help: None,
                note: None,
                related: vec![],
            },
        ];

        // Test raw batch formatting
        let raw_batch = DiagnosticFormatter::format_batch_raw(&diagnostics);
        assert!(raw_batch.contains("2 issues found"));
        assert!(raw_batch.contains("Issue 1"));
        assert!(raw_batch.contains("Issue 2"));

        // Test JSON batch formatting
        let json_batch = DiagnosticFormatter::format_batch_json(&diagnostics);
        assert_eq!(json_batch["summary"]["total_count"], 2);
        assert_eq!(json_batch["summary"]["error_count"], 1);
        assert_eq!(json_batch["summary"]["info_count"], 1);
        assert_eq!(json_batch["diagnostics"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_format_pretty() {
        let mut engine = DiagnosticEngine::new();
        let source_id = engine.add_source("test.fhirpath", "Patient.name.invalid");
        let diagnostic = create_test_diagnostic();

        let pretty = DiagnosticFormatter::format_pretty(&engine, &diagnostic, source_id);
        assert!(pretty.is_ok());

        let output = pretty.unwrap();
        assert!(output.contains("❌")); // Error emoji
        assert!(output.contains("FHIRPath Diagnostic Report"));
    }

    #[test]
    fn test_empty_diagnostics() {
        let empty_diagnostics: Vec<AriadneDiagnostic> = vec![];

        let table = DiagnosticFormatter::format_table(&empty_diagnostics);
        assert_eq!(table, "No diagnostics to display");

        let json_batch = DiagnosticFormatter::format_batch_json(&empty_diagnostics);
        assert_eq!(json_batch["summary"]["total_count"], 0);
    }
}
