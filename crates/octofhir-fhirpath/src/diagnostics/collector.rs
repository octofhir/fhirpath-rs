//! Multi-diagnostic collection system for comprehensive error reporting
//!
//! This module provides tools for collecting multiple diagnostics during
//! parsing and analysis, enabling comprehensive error reporting in a single pass.

use super::{AriadneDiagnostic, DiagnosticSeverity, RelatedDiagnostic};
use crate::core::error_code::ErrorCode;
use std::collections::HashMap;
use std::ops::Range;

/// Collector for multiple diagnostics during analysis
#[derive(Debug, Default)]
pub struct MultiDiagnosticCollector {
    diagnostics: Vec<AriadneDiagnostic>,
    source_spans: HashMap<usize, Range<usize>>,
    max_errors: usize,
    max_warnings: usize,
}

/// Batch of collected diagnostics with metadata
#[derive(Debug, Clone)]
pub struct DiagnosticBatch {
    /// Collection of diagnostics found during analysis
    pub diagnostics: Vec<AriadneDiagnostic>,
    /// Unique identifier for the source being analyzed
    pub source_id: usize,
    /// Human-readable name of the source (e.g., filename)
    pub source_name: String,
    /// Summary statistics about the collected diagnostics
    pub statistics: DiagnosticStatistics,
}

/// Statistics about collected diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticStatistics {
    /// Number of error diagnostics
    pub error_count: usize,
    /// Number of warning diagnostics
    pub warning_count: usize,
    /// Number of suggestion diagnostics
    pub suggestion_count: usize,
    /// Number of note diagnostics
    pub note_count: usize,
    /// Total number of diagnostics
    pub total_count: usize,
}

impl MultiDiagnosticCollector {
    /// Create new diagnostic collector
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            source_spans: HashMap::new(),
            max_errors: 50,    // Reasonable limit to prevent spam
            max_warnings: 100, // More warnings allowed than errors
        }
    }

    /// Create collector with custom limits
    pub fn with_limits(max_errors: usize, max_warnings: usize) -> Self {
        Self {
            diagnostics: Vec::new(),
            source_spans: HashMap::new(),
            max_errors,
            max_warnings,
        }
    }

    /// Add an error diagnostic
    pub fn error(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
    ) -> &mut Self {
        if self.error_count() < self.max_errors {
            self.diagnostics.push(AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code,
                message,
                span,
                help: None,
                note: None,
                related: Vec::new(),
            });
        }
        self
    }

    /// Add an error diagnostic with help text
    pub fn error_with_help(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
        help: String,
    ) -> &mut Self {
        if self.error_count() < self.max_errors {
            self.diagnostics.push(AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code,
                message,
                span,
                help: Some(help),
                note: None,
                related: Vec::new(),
            });
        }
        self
    }

    /// Add a warning diagnostic
    pub fn warning(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
    ) -> &mut Self {
        if self.warning_count() < self.max_warnings {
            self.diagnostics.push(AriadneDiagnostic {
                severity: DiagnosticSeverity::Warning,
                error_code,
                message,
                span,
                help: None,
                note: None,
                related: Vec::new(),
            });
        }
        self
    }

    /// Add a warning diagnostic with help text
    pub fn warning_with_help(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
        help: String,
    ) -> &mut Self {
        if self.warning_count() < self.max_warnings {
            self.diagnostics.push(AriadneDiagnostic {
                severity: DiagnosticSeverity::Warning,
                error_code,
                message,
                span,
                help: Some(help),
                note: None,
                related: Vec::new(),
            });
        }
        self
    }

    /// Add a suggestion diagnostic
    pub fn suggestion(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
    ) -> &mut Self {
        self.diagnostics.push(AriadneDiagnostic {
            severity: DiagnosticSeverity::Hint, // Map suggestion to hint in current system
            error_code,
            message,
            span,
            help: None,
            note: None,
            related: Vec::new(),
        });
        self
    }

    /// Add a suggestion diagnostic with help text
    pub fn suggestion_with_help(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
        help: String,
    ) -> &mut Self {
        self.diagnostics.push(AriadneDiagnostic {
            severity: DiagnosticSeverity::Hint, // Map suggestion to hint in current system
            error_code,
            message,
            span,
            help: Some(help),
            note: None,
            related: Vec::new(),
        });
        self
    }

    /// Add a note diagnostic
    pub fn note(
        &mut self,
        error_code: ErrorCode,
        message: String,
        span: Range<usize>,
    ) -> &mut Self {
        self.diagnostics.push(AriadneDiagnostic {
            severity: DiagnosticSeverity::Info, // Map note to info in current system
            error_code,
            message,
            span,
            help: None,
            note: None,
            related: Vec::new(),
        });
        self
    }

    /// Add a related diagnostic to the last added diagnostic
    pub fn add_related(
        &mut self,
        message: String,
        span: Range<usize>,
        severity: DiagnosticSeverity,
    ) -> &mut Self {
        if let Some(last_diagnostic) = self.diagnostics.last_mut() {
            last_diagnostic.related.push(RelatedDiagnostic {
                message,
                span,
                severity,
            });
        }
        self
    }

    /// Get count of errors
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    /// Get count of warnings  
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }

    /// Get count of suggestions (mapped to hints)
    pub fn suggestion_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Hint)
            .count()
    }

    /// Get count of notes (mapped to info)
    pub fn note_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Info)
            .count()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.warning_count() > 0
    }

    /// Check if collection limits were exceeded
    pub fn has_exceeded_limits(&self) -> bool {
        self.error_count() >= self.max_errors || self.warning_count() >= self.max_warnings
    }

    /// Build final diagnostic batch
    pub fn build_batch(&self, source_id: usize, source_name: String) -> DiagnosticBatch {
        let statistics = DiagnosticStatistics {
            error_count: self.error_count(),
            warning_count: self.warning_count(),
            suggestion_count: self.suggestion_count(),
            note_count: self.note_count(),
            total_count: self.diagnostics.len(),
        };

        DiagnosticBatch {
            diagnostics: self.diagnostics.clone(),
            source_id,
            source_name,
            statistics,
        }
    }

    /// Sort diagnostics by severity (errors first, then warnings, then suggestions)
    pub fn sort_by_severity(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            use DiagnosticSeverity::*;
            match (&a.severity, &b.severity) {
                (Error, Error) => a.span.start.cmp(&b.span.start),
                (Error, _) => std::cmp::Ordering::Less,
                (_, Error) => std::cmp::Ordering::Greater,
                (Warning, Warning) => a.span.start.cmp(&b.span.start),
                (Warning, _) => std::cmp::Ordering::Less,
                (_, Warning) => std::cmp::Ordering::Greater,
                (Hint, Hint) => a.span.start.cmp(&b.span.start), // Suggestions mapped to Hint
                (Hint, _) => std::cmp::Ordering::Less,
                (_, Hint) => std::cmp::Ordering::Greater,
                (Info, Info) => a.span.start.cmp(&b.span.start), // Notes mapped to Info
            }
        });
    }

    /// Clear all collected diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.source_spans.clear();
    }

    /// Get all diagnostics (sorted by severity)
    pub fn diagnostics(&self) -> &[AriadneDiagnostic] {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error_code::*;

    #[test]
    fn test_multi_diagnostic_collection() {
        let mut collector = MultiDiagnosticCollector::new();

        collector.error(FP0001, "First error".to_string(), 0..5);
        collector.warning(FP0153, "Warning message".to_string(), 10..15);
        collector.suggestion(FP0154, "Suggestion message".to_string(), 20..25);

        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);
        assert_eq!(collector.suggestion_count(), 1);
        assert!(collector.has_errors());
        assert!(collector.has_warnings());
    }

    #[test]
    fn test_diagnostic_sorting_by_severity() {
        let mut collector = MultiDiagnosticCollector::new();

        // Add in mixed order
        collector.suggestion(FP0154, "Suggestion".to_string(), 20..25);
        collector.error(FP0001, "Error".to_string(), 0..5);
        collector.warning(FP0153, "Warning".to_string(), 10..15);

        collector.sort_by_severity();
        let diagnostics = collector.diagnostics();

        // Should be sorted: Error, Warning, Suggestion (Hint)
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostics[1].severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostics[2].severity, DiagnosticSeverity::Hint);
    }

    #[test]
    fn test_diagnostic_limits() {
        let mut collector = MultiDiagnosticCollector::with_limits(2, 3);

        // Add more than the limits
        for i in 0..5 {
            collector.error(FP0001, format!("Error {}", i), i * 5..(i * 5 + 3));
            collector.warning(FP0153, format!("Warning {}", i), i * 5..(i * 5 + 3));
        }

        // Should respect limits
        assert!(collector.error_count() <= 2);
        assert!(collector.warning_count() <= 3);
        assert!(collector.has_exceeded_limits());
    }

    #[test]
    fn test_batch_formatting() {
        let mut collector = MultiDiagnosticCollector::new();
        collector.error(FP0001, "Test error".to_string(), 0..5);
        collector.warning(FP0153, "Test warning".to_string(), 10..15);

        let batch = collector.build_batch(0, "test.fhirpath".to_string());

        assert_eq!(batch.statistics.total_count, 2);
        assert_eq!(batch.statistics.error_count, 1);
        assert_eq!(batch.statistics.warning_count, 1);
        assert_eq!(batch.source_name, "test.fhirpath");
    }
}
