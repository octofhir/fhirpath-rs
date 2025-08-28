//! Simplified diagnostic reporting system
//!
//! This streamlined version removes over-engineered analysis features while maintaining
//! essential diagnostic collection and reporting capabilities.

use super::diagnostic::{Diagnostic, Severity};
use std::collections::BTreeMap;
use std::fmt;

/// Streamlined diagnostic report with essential information
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    /// All diagnostics in the report
    pub diagnostics: Vec<Diagnostic>,
    /// Summary statistics
    pub summary: DiagnosticSummary,
}

/// Essential summary statistics for diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticSummary {
    /// Total number of diagnostics
    pub total_count: usize,
    /// Count by severity
    pub error_count: usize,
    /// Number of warning diagnostics
    pub warning_count: usize,
    /// Number of info diagnostics
    pub info_count: usize,
    /// Number of hint diagnostics
    pub hint_count: usize,
    /// Overall severity (highest severity present)
    pub overall_severity: Severity,
}

/// Streamlined diagnostic reporter
#[derive(Debug, Default)]
pub struct DiagnosticReporter {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReporter {
    /// Create a new diagnostic reporter
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic to the report
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Add multiple diagnostics to the report
    pub fn add_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    /// Generate a complete diagnostic report
    pub fn generate_report(&self) -> DiagnosticReport {
        let summary = self.generate_summary();

        DiagnosticReport {
            diagnostics: self.diagnostics.clone(),
            summary,
        }
    }

    /// Generate summary statistics
    fn generate_summary(&self) -> DiagnosticSummary {
        let mut summary = DiagnosticSummary {
            total_count: self.diagnostics.len(),
            ..Default::default()
        };

        for diagnostic in &self.diagnostics {
            match diagnostic.severity {
                Severity::Error => {
                    summary.error_count += 1;
                    summary.overall_severity = Severity::Error;
                }
                Severity::Warning => {
                    summary.warning_count += 1;
                    if summary.overall_severity < Severity::Warning {
                        summary.overall_severity = Severity::Warning;
                    }
                }
                Severity::Info => {
                    summary.info_count += 1;
                    if summary.overall_severity < Severity::Info {
                        summary.overall_severity = Severity::Info;
                    }
                }
                Severity::Hint => {
                    summary.hint_count += 1;
                    if summary.overall_severity < Severity::Hint {
                        summary.overall_severity = Severity::Hint;
                    }
                }
            }
        }

        summary
    }

    /// Get all error diagnostics
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .collect()
    }

    /// Get all warning diagnostics
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .collect()
    }

    /// Get diagnostics grouped by severity
    pub fn group_by_severity(&self) -> BTreeMap<Severity, Vec<&Diagnostic>> {
        let mut grouped = BTreeMap::new();

        for diagnostic in &self.diagnostics {
            grouped
                .entry(diagnostic.severity)
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }

        grouped
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    /// Get total number of diagnostics
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Check if reporter is empty
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

impl fmt::Display for DiagnosticReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Diagnostic Report")?;
        writeln!(f, "==================")?;
        writeln!(f, "Total: {}", self.summary.total_count)?;
        writeln!(f, "Errors: {}", self.summary.error_count)?;
        writeln!(f, "Warnings: {}", self.summary.warning_count)?;
        writeln!(f, "Info: {}", self.summary.info_count)?;
        writeln!(f, "Hints: {}", self.summary.hint_count)?;
        writeln!(f)?;

        for diagnostic in &self.diagnostics {
            writeln!(f, "{diagnostic}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, DiagnosticCode};
    use crate::location::SourceLocation;

    #[test]
    fn test_diagnostic_reporter_basic() {
        let mut reporter = DiagnosticReporter::new();

        let diagnostic = Diagnostic::new(
            DiagnosticCode::UnknownFunction,
            Severity::Error,
            "count is not defined".to_string(),
            SourceLocation::from_line_column(1, 10, 9),
        );

        reporter.add_diagnostic(diagnostic);

        assert_eq!(reporter.len(), 1);
        assert!(reporter.has_errors());
        assert_eq!(reporter.errors().len(), 1);
        assert_eq!(reporter.warnings().len(), 0);
    }

    #[test]
    fn test_summary_generation() {
        let mut reporter = DiagnosticReporter::new();

        // Add error
        let error_diagnostic = Diagnostic::new(
            DiagnosticCode::UnknownFunction,
            Severity::Error,
            "error message".to_string(),
            SourceLocation::from_line_column(1, 1, 0),
        );
        reporter.add_diagnostic(error_diagnostic);

        // Add warning
        let warning_diagnostic = Diagnostic::new(
            DiagnosticCode::UnexpectedToken,
            Severity::Warning,
            "warning message".to_string(),
            SourceLocation::from_line_column(2, 1, 10),
        );
        reporter.add_diagnostic(warning_diagnostic);

        let report = reporter.generate_report();

        assert_eq!(report.summary.total_count, 2);
        assert_eq!(report.summary.error_count, 1);
        assert_eq!(report.summary.warning_count, 1);
        assert_eq!(report.summary.overall_severity, Severity::Error);
    }

    #[test]
    fn test_grouping_by_severity() {
        let mut reporter = DiagnosticReporter::new();

        // Add diagnostics of different severities
        for severity in &[Severity::Error, Severity::Warning, Severity::Error] {
            let diagnostic = Diagnostic::new(
                DiagnosticCode::UnexpectedToken,
                *severity,
                "test message".to_string(),
                SourceLocation::from_line_column(1, 1, 0),
            );
            reporter.add_diagnostic(diagnostic);
        }

        let grouped = reporter.group_by_severity();

        assert_eq!(grouped.get(&Severity::Error).unwrap().len(), 2);
        assert_eq!(grouped.get(&Severity::Warning).unwrap().len(), 1);
        assert!(grouped.get(&Severity::Info).is_none());
    }
}
