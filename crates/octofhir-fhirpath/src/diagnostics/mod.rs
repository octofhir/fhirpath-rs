//! Diagnostic system with Ariadne integration for beautiful error reporting
//!
//! This module provides a comprehensive diagnostic system that creates beautiful,
//! Rust compiler-style error reports using Ariadne. It includes error code integration,
//! source span highlighting, and rich diagnostic information.

pub mod batch_formatter;
pub mod builder;
pub mod collector;
pub mod diagnostic;
pub mod engine;
pub mod formatter;
pub mod processor;
pub mod source;

// Re-export main types for backward compatibility
pub use builder::DiagnosticBuilder;
pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

// Re-export new Ariadne-based types
pub use engine::{ColorScheme, DiagnosticEngine};
pub use formatter::DiagnosticFormatter;
pub use processor::{
    ContextualSuggestion, DiagnosticProcessor, DiagnosticRelationship, ProcessedDiagnostic,
};
pub use source::{SourceInfo, SourceManager};

// Re-export multi-diagnostic collection types
pub use batch_formatter::BatchFormatter;
pub use collector::{DiagnosticBatch, DiagnosticStatistics, MultiDiagnosticCollector};

// Enhanced diagnostics with Ariadne support
use crate::core::error_code::ErrorCode;
use ariadne::{Color, ReportKind};
use std::ops::Range;

/// Enhanced diagnostic with Ariadne integration
#[derive(Debug, Clone)]
pub struct AriadneDiagnostic {
    /// The severity of this diagnostic
    pub severity: DiagnosticSeverity,
    /// The error code
    pub error_code: ErrorCode,
    /// The diagnostic message
    pub message: String,
    /// Source span for highlighting
    pub span: Range<usize>,
    /// Optional help text
    pub help: Option<String>,
    /// Optional note
    pub note: Option<String>,
    /// Related diagnostics
    pub related: Vec<RelatedDiagnostic>,
}

/// Related diagnostic information (like "note: defined here")
#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    /// Related message
    pub message: String,
    /// Source span
    pub span: Range<usize>,
    /// Severity level
    pub severity: DiagnosticSeverity,
}

impl DiagnosticSeverity {
    /// Get Ariadne ReportKind for this severity
    pub fn to_report_kind(&self) -> ReportKind<'static> {
        match self {
            DiagnosticSeverity::Error => ReportKind::Error,
            DiagnosticSeverity::Warning => ReportKind::Warning,
            DiagnosticSeverity::Info => ReportKind::Advice,
            DiagnosticSeverity::Hint => ReportKind::Custom("hint", Color::Blue),
        }
    }

    /// Get color for this severity level
    pub fn color(&self) -> Color {
        match self {
            DiagnosticSeverity::Error => Color::Red,
            DiagnosticSeverity::Warning => Color::Yellow,
            DiagnosticSeverity::Info => Color::Cyan,
            DiagnosticSeverity::Hint => Color::Blue,
        }
    }
}

/// Trait for types that can report diagnostics
pub trait DiagnosticReporter {
    /// Report a diagnostic
    fn report(&mut self, diagnostic: Diagnostic);

    /// Report multiple diagnostics
    fn report_all(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        for diagnostic in diagnostics {
            self.report(diagnostic);
        }
    }
}

/// Simple diagnostic collector that stores diagnostics in a vector
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    /// Collected diagnostics
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    /// Create a new diagnostic collector
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Get all collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Check if any errors were collected
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    /// Clear all collected diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

impl DiagnosticReporter for DiagnosticCollector {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}
