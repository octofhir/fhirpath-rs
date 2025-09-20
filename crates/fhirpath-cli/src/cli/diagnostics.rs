//! CLI diagnostic integration for beautiful error reporting
//!
//! This module integrates the diagnostic engine with CLI commands,
//! providing consistent error reporting across all CLI interfaces.

use octofhir_fhirpath::core::error_code::ErrorCode;
use octofhir_fhirpath::diagnostics::batch_formatter::BatchFormatter;
use octofhir_fhirpath::diagnostics::{
    AriadneDiagnostic, DiagnosticEngine, DiagnosticFormatter, DiagnosticSeverity,
};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::{self, Write};
use std::ops::Range;

use super::output::OutputFormat;

/// CLI diagnostic handler for consistent error reporting
pub struct CliDiagnosticHandler {
    engine: DiagnosticEngine,
    output_format: OutputFormat,
    quiet_mode: bool,
    verbose_mode: bool,
}

impl CliDiagnosticHandler {
    /// Create new CLI diagnostic handler
    pub fn new(output_format: OutputFormat) -> Self {
        Self {
            engine: DiagnosticEngine::new(),
            output_format,
            quiet_mode: false,
            verbose_mode: false,
        }
    }

    /// Set quiet mode (suppress informational messages)
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet_mode = quiet;
        self
    }

    /// Set verbose mode (show additional diagnostic details)
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose_mode = verbose;
        self
    }

    /// Add source text for diagnostic reporting
    pub fn add_source(&mut self, name: String, content: String) -> usize {
        self.engine.add_source(name, content)
    }

    /// Report a single diagnostic based on output format
    pub fn report_diagnostic(
        &self,
        diagnostic: &AriadneDiagnostic,
        source_id: usize,
        writer: &mut dyn Write,
    ) -> io::Result<()> {
        match self.output_format {
            OutputFormat::Json => {
                // JSON mode: NO diagnostics to stderr, only structured data to stdout
                // Error information will be included in the JSON response itself
                Ok(())
            }
            OutputFormat::Raw => {
                let output = DiagnosticFormatter::format_raw(diagnostic);
                write!(writer, "{output}")?;
                Ok(())
            }
            OutputFormat::Pretty => {
                let output =
                    DiagnosticFormatter::format_pretty(&self.engine, diagnostic, source_id)
                        .map_err(|e| io::Error::other(e.to_string()))?;
                write!(writer, "{output}")?;
                Ok(())
            }
        }
    }

    /// Report multiple diagnostics in batch (for static analysis)
    pub fn report_diagnostics(
        &self,
        diagnostics: &[AriadneDiagnostic],
        source_id: usize,
        writer: &mut dyn Write,
    ) -> io::Result<()> {
        if diagnostics.is_empty() && !self.verbose_mode {
            return Ok(());
        }

        match self.output_format {
            OutputFormat::Json => {
                // JSON mode: Include diagnostics in structured format
                let json_diagnostics: Vec<Value> = diagnostics
                    .iter()
                    .map(DiagnosticFormatter::format_json)
                    .collect();

                let output = json!({
                    "diagnostics": json_diagnostics,
                    "count": diagnostics.len(),
                    "has_errors": diagnostics.iter().any(|d| matches!(d.severity, DiagnosticSeverity::Error)),
                    "has_warnings": diagnostics.iter().any(|d| matches!(d.severity, DiagnosticSeverity::Warning)),
                });

                writeln!(writer, "{}", serde_json::to_string_pretty(&output)?)?;
                Ok(())
            }
            OutputFormat::Raw => {
                // Raw mode: Show single consolidated diagnostic with ALL error codes
                if !diagnostics.is_empty() {
                    let consolidated = self.consolidate_diagnostics(diagnostics);
                    self.report_diagnostic(&consolidated, source_id, writer)?;
                }
                Ok(())
            }
            OutputFormat::Pretty => {
                // Pretty mode: Show ALL diagnostics as ONE unified report, then add single help message
                if !diagnostics.is_empty() {
                    // Show unified report with all diagnostics combined
                    self.engine
                        .emit_unified_report(diagnostics, source_id, writer)
                        .map_err(|e| io::Error::other(e.to_string()))?;

                    // Then show a single consolidated help message with all error codes
                    let mut error_codes: Vec<String> = diagnostics
                        .iter()
                        .map(|d| d.error_code.code_str())
                        .collect::<HashSet<_>>() // Remove duplicates
                        .into_iter()
                        .collect();
                    error_codes.sort(); // Ensure consistent ordering

                    if error_codes.len() == 1 {
                        writeln!(writer, "\n  = help: for more information about this error, try `octofhir-fhirpath docs {}`", error_codes[0])?;
                    } else {
                        writeln!(writer, "\n  = help: for more information about these errors, try `octofhir-fhirpath docs {}` (or other codes: {})", error_codes[0], error_codes[1..].join(", "))?;
                    }
                }
                Ok(())
            }
        }
    }

    /// Consolidate multiple diagnostics into a single diagnostic with all error codes
    fn consolidate_diagnostics(&self, diagnostics: &[AriadneDiagnostic]) -> AriadneDiagnostic {
        if diagnostics.is_empty() {
            return AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: ErrorCode::new(1),
                message: "No diagnostics to consolidate".to_string(),
                span: 0..0,
                help: None,
                note: None,
                related: Vec::new(),
            };
        }

        if diagnostics.len() == 1 {
            return diagnostics[0].clone();
        }

        // Collect all unique error codes
        let mut error_codes: Vec<String> = diagnostics
            .iter()
            .map(|d| d.error_code.code_str())
            .collect();
        error_codes.sort();
        error_codes.dedup();

        // Use the most severe severity
        let most_severe = diagnostics
            .iter()
            .map(|d| &d.severity)
            .max_by_key(|s| match s {
                DiagnosticSeverity::Error => 3,
                DiagnosticSeverity::Warning => 2,
                DiagnosticSeverity::Info => 1,
                DiagnosticSeverity::Hint => 0,
            })
            .unwrap_or(&DiagnosticSeverity::Error);

        // Create consolidated message
        let first_message = &diagnostics[0].message;
        let message = if diagnostics.len() == 1 {
            first_message.clone()
        } else {
            format!("{} (and {} more issues)", first_message, diagnostics.len() - 1)
        };

        // Use span of first diagnostic
        let span = diagnostics[0].span.clone();

        // Create help text with docs command suggestion
        let help_text = if error_codes.len() == 1 {
            format!("for more information about this error, try `octofhir-fhirpath docs {}`", error_codes[0])
        } else {
            format!(
                "for more information about these errors, try `octofhir-fhirpath docs {}` (or other codes: {})",
                error_codes[0],
                error_codes[1..].join(", ")
            )
        };

        AriadneDiagnostic {
            severity: most_severe.clone(),
            error_code: diagnostics[0].error_code.clone(), // Use first error code as primary
            message,
            span,
            help: Some(help_text),
            note: Some(format!("found {} error(s) with codes: {}", diagnostics.len(), error_codes.join(", "))),
            related: Vec::new(),
        }
    }

    /// Create a diagnostic from parser/evaluator error
    pub fn create_diagnostic_from_error(
        &self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
        help: Option<String>,
    ) -> AriadneDiagnostic {
        self.engine
            .builder()
            .with_error_code(error_code)
            .with_severity(DiagnosticSeverity::Error)
            .with_message(message)
            .with_span(span)
            .with_help(help.unwrap_or_default())
            .build()
    }

    /// Create a warning diagnostic
    pub fn create_warning_diagnostic(
        &self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
        help: Option<String>,
    ) -> AriadneDiagnostic {
        self.engine
            .builder()
            .with_error_code(error_code)
            .with_severity(DiagnosticSeverity::Warning)
            .with_message(message)
            .with_span(span)
            .with_help(help.unwrap_or_default())
            .build()
    }

    /// Create an info diagnostic
    pub fn create_info_diagnostic(
        &self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
        help: Option<String>,
    ) -> AriadneDiagnostic {
        self.engine
            .builder()
            .with_error_code(error_code)
            .with_severity(DiagnosticSeverity::Info)
            .with_message(message)
            .with_span(span)
            .with_help(help.unwrap_or_default())
            .build()
    }

    /// Show informational message (respects quiet mode)
    pub fn info(&self, message: &str, writer: &mut dyn Write) -> io::Result<()> {
        if !self.quiet_mode && self.output_format != OutputFormat::Json {
            writeln!(writer, "ℹ️  {message}")?;
        }
        Ok(())
    }

    /// Show success message (respects quiet mode)
    pub fn success(&self, message: &str, writer: &mut dyn Write) -> io::Result<()> {
        if !self.quiet_mode && self.output_format != OutputFormat::Json {
            writeln!(writer, "✅ {message}")?;
        }
        Ok(())
    }

    /// Show warning message
    pub fn warning(&self, message: &str, writer: &mut dyn Write) -> io::Result<()> {
        if self.output_format != OutputFormat::Json {
            writeln!(writer, "⚠️  {message}")?;
        }
        Ok(())
    }

    /// Show error message
    pub fn error(&self, message: &str, writer: &mut dyn Write) -> io::Result<()> {
        if self.output_format != OutputFormat::Json {
            writeln!(writer, "❌ {message}")?;
        }
        Ok(())
    }

    /// Get the diagnostic engine for advanced operations
    pub fn engine(&self) -> &DiagnosticEngine {
        &self.engine
    }

    /// Get the output format
    pub fn output_format(&self) -> &OutputFormat {
        &self.output_format
    }

    /// Check if we're in quiet mode
    pub fn is_quiet(&self) -> bool {
        self.quiet_mode
    }

    /// Check if we're in verbose mode
    pub fn is_verbose(&self) -> bool {
        self.verbose_mode
    }

    /// Report comprehensive analysis results with multi-error display
    pub fn report_analysis_result(
        &self,
        result: &octofhir_fhirpath::parser::analysis_integration::AnalysisResult,
        writer: &mut dyn Write,
    ) -> io::Result<()> {
        match self.output_format {
            OutputFormat::Json => {
                let json_output = BatchFormatter::format_json_report(&result.diagnostics);
                writeln!(writer, "{}", serde_json::to_string_pretty(&json_output)?)?;
            }
            OutputFormat::Pretty => {
                let pretty_output =
                    BatchFormatter::format_comprehensive_report(&self.engine, &result.diagnostics)
                        .map_err(|e| io::Error::other(e.to_string()))?;
                write!(writer, "{pretty_output}")?;
            }
            OutputFormat::Raw => {
                // Raw format shows compact list
                for (i, diagnostic) in result.diagnostics.diagnostics.iter().enumerate() {
                    writeln!(
                        writer,
                        "{}. {}",
                        i + 1,
                        DiagnosticFormatter::format_raw(diagnostic)
                    )?;
                }

                let summary =
                    BatchFormatter::format_compact_summary(&result.diagnostics.statistics);
                writeln!(writer, "\n{summary}")?;
            }
        }

        Ok(())
    }

    /// Show progress indicator for analysis (respects quiet mode)
    pub fn show_analysis_progress(&self, phase: &str, writer: &mut dyn Write) -> io::Result<()> {
        if !self.quiet_mode && self.output_format != OutputFormat::Json {
            writeln!(writer, "🔄 {phase}")?;
        }
        Ok(())
    }

    /// Show analysis completion with statistics
    pub fn show_analysis_completion(
        &self,
        stats: &octofhir_fhirpath::diagnostics::collector::DiagnosticStatistics,
        writer: &mut dyn Write,
    ) -> io::Result<()> {
        if !self.quiet_mode && self.output_format != OutputFormat::Json {
            let summary = BatchFormatter::format_compact_summary(stats);
            writeln!(writer, "✅ Analysis complete: {summary}")?;
        }
        Ok(())
    }
}

/// Helper function to convert FhirPathError to AriadneDiagnostic
pub fn error_to_diagnostic(error: &octofhir_fhirpath::core::FhirPathError) -> AriadneDiagnostic {
    let error_code = error.error_code();
    let message = error.to_string();
    let span = 0..0; // TODO: Extract actual span from error when available

    AriadneDiagnostic {
        severity: DiagnosticSeverity::Error,
        error_code: error_code.clone(),
        message,
        span,
        help: None,
        note: None,
        related: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath::core::error_code::*;
    use std::io::Cursor;

    #[test]
    fn test_diagnostic_handler_creation() {
        let handler = CliDiagnosticHandler::new(OutputFormat::Pretty);
        assert!(!handler.is_quiet());
        assert!(!handler.is_verbose());
        assert_eq!(handler.output_format(), &OutputFormat::Pretty);
    }

    #[test]
    fn test_diagnostic_handler_modes() {
        let handler = CliDiagnosticHandler::new(OutputFormat::Raw)
            .with_quiet(true)
            .with_verbose(false);

        assert!(handler.is_quiet());
        assert!(!handler.is_verbose());
    }

    #[test]
    fn test_create_diagnostic() {
        let handler = CliDiagnosticHandler::new(OutputFormat::Pretty);
        let diagnostic = handler.create_diagnostic_from_error(
            FP0001,
            "Test error".to_string(),
            0..5,
            Some("Test help".to_string()),
        );

        assert_eq!(diagnostic.error_code, FP0001);
        assert_eq!(diagnostic.message, "Test error");
        assert_eq!(diagnostic.span, 0..5);
        assert_eq!(diagnostic.help, Some("Test help".to_string()));
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_diagnostic_types() {
        let handler = CliDiagnosticHandler::new(OutputFormat::Pretty);

        let error_diag =
            handler.create_diagnostic_from_error(FP0001, "Error".to_string(), 0..5, None);
        assert_eq!(error_diag.severity, DiagnosticSeverity::Error);

        let warning_diag =
            handler.create_warning_diagnostic(FP0153, "Warning".to_string(), 0..5, None);
        assert_eq!(warning_diag.severity, DiagnosticSeverity::Warning);

        let info_diag = handler.create_info_diagnostic(FP0153, "Info".to_string(), 0..5, None);
        assert_eq!(info_diag.severity, DiagnosticSeverity::Info);
    }

    #[test]
    fn test_quiet_mode() {
        let handler = CliDiagnosticHandler::new(OutputFormat::Pretty).with_quiet(true);
        let mut buffer = Cursor::new(Vec::new());

        handler.info("Test info message", &mut buffer).unwrap();
        assert!(buffer.get_ref().is_empty());

        handler.warning("Test warning", &mut buffer).unwrap();
        assert!(!buffer.get_ref().is_empty());
    }

    #[test]
    fn test_json_mode_no_diagnostics_to_stderr() {
        let handler = CliDiagnosticHandler::new(OutputFormat::Json);
        let diagnostic = handler.create_diagnostic_from_error(
            FP0001,
            "Test error".to_string(),
            0..5,
            Some("Test help".to_string()),
        );

        let mut buffer = Cursor::new(Vec::new());
        handler
            .report_diagnostic(&diagnostic, 0, &mut buffer)
            .unwrap();

        // JSON mode should not output diagnostics to stderr
        assert!(buffer.get_ref().is_empty());
    }

    #[test]
    fn test_source_management() {
        let mut handler = CliDiagnosticHandler::new(OutputFormat::Pretty);
        let source_id = handler.add_source("test.fhirpath".to_string(), "Patient.name".to_string());

        assert_eq!(source_id, 0);

        // Verify source is stored in the engine
        let source_info = handler
            .engine()
            .source_manager()
            .get_source(source_id)
            .unwrap();
        assert_eq!(source_info.name, "test.fhirpath");
        assert_eq!(source_info.content, "Patient.name");
    }
}
