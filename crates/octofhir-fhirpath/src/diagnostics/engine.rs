//! Core diagnostic engine for creating beautiful error reports with Ariadne

use super::{AriadneDiagnostic, DiagnosticSeverity, RelatedDiagnostic, SourceManager};
use crate::core::error_code::ErrorCode;
use ariadne::{Color, Label, Report, Source};
use std::io::IsTerminal;
use std::ops::Range;

/// Core diagnostic engine for creating beautiful error reports
pub struct DiagnosticEngine {
    /// Source text manager
    source_manager: SourceManager,
    /// Color scheme configuration
    color_scheme: ColorScheme,
    /// Whether to show colors in output
    show_colors: bool,
}

/// Color scheme configuration for diagnostics
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Error color
    pub error: Color,
    /// Warning color
    pub warning: Color,
    /// Suggestion/info color
    pub suggestion: Color,
    /// Note/hint color
    pub note: Color,
    /// Primary highlight color
    pub primary: Color,
    /// Secondary highlight color
    pub secondary: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            error: Color::Red,
            warning: Color::Yellow,
            suggestion: Color::Cyan,
            note: Color::Blue,
            primary: Color::Red,
            secondary: Color::Magenta,
        }
    }
}

impl DiagnosticEngine {
    /// Create a new diagnostic engine
    pub fn new() -> Self {
        Self {
            source_manager: SourceManager::new(),
            color_scheme: ColorScheme::default(),
            show_colors: Self::should_show_colors(),
        }
    }

    /// Create diagnostic engine with custom color scheme
    pub fn with_colors(color_scheme: ColorScheme) -> Self {
        Self {
            source_manager: SourceManager::new(),
            color_scheme,
            show_colors: Self::should_show_colors(),
        }
    }

    /// Check if colors should be displayed (respects NO_COLOR env vars and terminal detection)
    fn should_show_colors() -> bool {
        // Respect NO_COLOR environment variable
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Respect FHIRPATH_NO_COLOR environment variable
        if std::env::var("FHIRPATH_NO_COLOR").is_ok() {
            return false;
        }

        // Check if stderr is a terminal (using std::io::IsTerminal, stable since Rust 1.70)
        std::io::stderr().is_terminal()
    }

    /// Register source text for diagnostic reporting
    pub fn add_source<S: Into<String>>(&mut self, name: S, content: S) -> usize {
        self.source_manager.add_source(name.into(), content.into())
    }

    /// Create a simple diagnostic with error code
    pub fn create_diagnostic(
        &self,
        error_code: ErrorCode,
        severity: DiagnosticSeverity,
        span: Range<usize>,
        message: impl Into<String>,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity,
            error_code,
            message: message.into(),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        }
    }

    /// Build a unified report with multiple diagnostics
    pub fn build_unified_report(
        &self,
        diagnostics: &[AriadneDiagnostic],
        source_id: usize,
    ) -> Result<Report<'static, (usize, Range<usize>)>, Box<dyn std::error::Error>> {
        if diagnostics.is_empty() {
            return Err("No diagnostics to report".into());
        }

        let _source_info = self
            .source_manager
            .get_source(source_id)
            .ok_or("Source not found")?;

        // Use the first diagnostic as the primary report, but don't show the header
        let primary = &diagnostics[0];
        let mut report = Report::build(
            primary.severity.to_report_kind(),
            (source_id, primary.span.clone()),
        );
        // Don't add code or message header - the user wants to show only span information

        // Add labels for all diagnostics
        for diagnostic in diagnostics {
            let color = if self.show_colors {
                diagnostic.severity.color()
            } else {
                Color::Fixed(7) // Default terminal color when colors are disabled
            };

            let label = Label::new((source_id, diagnostic.span.clone()))
                .with_message(&diagnostic.message)
                .with_color(color);
            report = report.with_label(label);
        }

        // Add documentation links for all unique error codes (Rust-like format)
        let mut seen_codes = std::collections::HashSet::new();
        for diagnostic in diagnostics {
            if seen_codes.insert(diagnostic.error_code.code) {
                let docs_note = format!(
                    "For more information about this error, try: `octofhir-fhirpath docs {}`",
                    diagnostic.error_code.code_str()
                );
                report = report.with_note(&docs_note);
            }
        }

        Ok(report.finish())
    }

    /// Format and emit a unified report with multiple diagnostics
    pub fn emit_unified_report(
        &self,
        diagnostics: &[AriadneDiagnostic],
        source_id: usize,
        writer: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.build_unified_report(diagnostics, source_id)?;
        let source_info = self
            .source_manager
            .get_source(source_id)
            .ok_or("Source not found")?;

        let source = Source::from(&source_info.content);
        report.write((source_id, source), writer)?;
        Ok(())
    }

    /// Build a comprehensive diagnostic report using Ariadne
    pub fn build_report(
        &self,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
    ) -> Result<Report<'static, (usize, Range<usize>)>, Box<dyn std::error::Error>> {
        let _source_info = self
            .source_manager
            .get_source(source_id)
            .ok_or("Source not found")?;

        let mut report = Report::build(
            diagnostic.severity.to_report_kind(),
            (source_id, diagnostic.span.clone()),
        )
        .with_code(diagnostic.error_code.code_str())
        .with_message(&diagnostic.message);

        // Add primary label with error location
        let primary_color = if self.show_colors {
            diagnostic.severity.color()
        } else {
            Color::Fixed(7) // Default terminal color when colors are disabled
        };

        // Use the original diagnostic message as label
        let label_message = &diagnostic.message;

        let primary_label = Label::new((source_id, diagnostic.span.clone()))
            .with_message(label_message)
            .with_color(primary_color);
        report = report.with_label(primary_label);

        // Add help text if provided
        if let Some(help) = &diagnostic.help {
            report = report.with_help(help);
        }

        // Add note with documentation link (Rust-like format)
        let docs_note = format!(
            "For more information about this error, try: `octofhir-fhirpath docs {}`",
            diagnostic.error_code.code_str()
        );
        report = report.with_note(&docs_note);

        // Add custom note if provided
        if let Some(note) = &diagnostic.note {
            report = report.with_note(note);
        }

        // Add related diagnostics
        for related in &diagnostic.related {
            let related_color = if self.show_colors {
                related.severity.color()
            } else {
                Color::Fixed(7) // Default terminal color when colors are disabled
            };

            let related_label = Label::new((source_id, related.span.clone()))
                .with_message(&related.message)
                .with_color(related_color);
            report = report.with_label(related_label);
        }

        Ok(report.finish())
    }

    /// Format and emit a diagnostic report to a writer
    pub fn emit_diagnostic(
        &self,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
        writer: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let report = self.build_report(diagnostic, source_id)?;
        let source_info = self
            .source_manager
            .get_source(source_id)
            .ok_or("Source not found")?;

        let source = Source::from(&source_info.content);
        report.write((source_id, source), writer)?;
        Ok(())
    }

    /// Format diagnostic as string (useful for testing and JSON output)
    pub fn format_diagnostic(
        &self,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        self.emit_diagnostic(diagnostic, source_id, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Create a builder for complex diagnostics
    pub fn builder(&self) -> ReportBuilder {
        ReportBuilder::new()
    }

    /// Check if colors are enabled
    pub fn colors_enabled(&self) -> bool {
        self.show_colors
    }

    /// Get source manager reference
    pub fn source_manager(&self) -> &SourceManager {
        &self.source_manager
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating complex diagnostic reports
pub struct ReportBuilder {
    diagnostic: AriadneDiagnostic,
}

impl ReportBuilder {
    /// Create a new report builder
    pub fn new() -> Self {
        Self {
            diagnostic: AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: ErrorCode::new(1), // Default to FP0001
                message: String::new(),
                span: 0..0,
                help: None,
                note: None,
                related: Vec::new(),
            },
        }
    }

    /// Set the error code
    pub fn with_error_code(mut self, error_code: ErrorCode) -> Self {
        self.diagnostic.error_code = error_code;
        self
    }

    /// Set the severity level
    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.diagnostic.severity = severity;
        self
    }

    /// Set the primary message
    pub fn with_message<S: Into<String>>(mut self, message: S) -> Self {
        self.diagnostic.message = message.into();
        self
    }

    /// Set the source span
    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.diagnostic.span = span;
        self
    }

    /// Add help text
    pub fn with_help<S: Into<String>>(mut self, help: S) -> Self {
        self.diagnostic.help = Some(help.into());
        self
    }

    /// Add a note
    pub fn with_note<S: Into<String>>(mut self, note: S) -> Self {
        self.diagnostic.note = Some(note.into());
        self
    }

    /// Add a related diagnostic (like "defined here")
    pub fn with_related(
        mut self,
        message: String,
        span: Range<usize>,
        severity: DiagnosticSeverity,
    ) -> Self {
        self.diagnostic.related.push(RelatedDiagnostic {
            message,
            span,
            severity,
        });
        self
    }

    /// Build the final diagnostic
    pub fn build(self) -> AriadneDiagnostic {
        self.diagnostic
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error_code::*;

    #[test]
    fn test_basic_diagnostic_creation() {
        let engine = DiagnosticEngine::new();
        let diagnostic = engine.create_diagnostic(
            FP0001,
            DiagnosticSeverity::Error,
            5..10,
            "Test error message",
        );

        assert_eq!(diagnostic.error_code, FP0001);
        assert_eq!(diagnostic.message, "Test error message");
        assert_eq!(diagnostic.span, 5..10);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_source_management() {
        let mut engine = DiagnosticEngine::new();
        let source_id = engine.add_source("test.fhirpath", "Patient.name.invalid");

        let source_info = engine.source_manager().get_source(source_id).unwrap();
        assert_eq!(source_info.name, "test.fhirpath");
        assert_eq!(source_info.content, "Patient.name.invalid");
    }

    #[test]
    fn test_report_builder() {
        let diagnostic = ReportBuilder::new()
            .with_error_code(FP0055)
            .with_severity(DiagnosticSeverity::Error)
            .with_message("Property not found: 'invalid'")
            .with_span(13..20)
            .with_help("Check available properties for Patient.name")
            .with_note("Property access must be valid according to FHIR specification")
            .build();

        assert_eq!(diagnostic.error_code, FP0055);
        assert_eq!(diagnostic.message, "Property not found: 'invalid'");
        assert_eq!(diagnostic.span, 13..20);
        assert_eq!(
            diagnostic.help,
            Some("Check available properties for Patient.name".to_string())
        );
        assert_eq!(
            diagnostic.note,
            Some("Property access must be valid according to FHIR specification".to_string())
        );
    }

    #[test]
    fn test_color_scheme_configuration() {
        // Test that engine can be created with custom color scheme
        let custom_colors = ColorScheme {
            error: Color::Magenta,
            warning: Color::Blue,
            suggestion: Color::Green,
            note: Color::Yellow,
            primary: Color::Cyan,
            secondary: Color::Red,
        };

        let engine = DiagnosticEngine::with_colors(custom_colors);
        // Just verify the engine was created successfully
        // Color scheme is used internally by Ariadne
        assert!(engine.source_manager().source_count() == 0);
    }

    #[test]
    fn test_diagnostic_report_generation() {
        let mut engine = DiagnosticEngine::new();
        let source_id = engine.add_source("test.fhirpath", "Patient.name.invalid");

        let diagnostic = engine
            .builder()
            .with_error_code(FP0055)
            .with_message("Property 'invalid' not found on Patient.name")
            .with_span(13..20)
            .with_help("Available properties: family, given, use, text")
            .build();

        let report = engine.build_report(&diagnostic, source_id);
        assert!(report.is_ok());

        let formatted = engine.format_diagnostic(&diagnostic, source_id);
        assert!(formatted.is_ok());

        let output = formatted.unwrap();
        assert!(output.contains("FP0055"));
        assert!(output.contains("Property 'invalid' not found"));
    }

    #[test]
    fn test_related_diagnostics() {
        let diagnostic = ReportBuilder::new()
            .with_error_code(FP0010)
            .with_message("Undefined variable")
            .with_span(10..15)
            .with_related(
                "variable defined here".to_string(),
                5..10,
                DiagnosticSeverity::Info,
            )
            .build();

        assert_eq!(diagnostic.related.len(), 1);
        assert_eq!(diagnostic.related[0].message, "variable defined here");
        assert_eq!(diagnostic.related[0].span, 5..10);
        assert_eq!(diagnostic.related[0].severity, DiagnosticSeverity::Info);
    }
}
