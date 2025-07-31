//! Diagnostic system for FHIRPath parsing and evaluation errors
//!
//! This crate provides a comprehensive diagnostic system that can produce
//! both human-friendly error messages and machine-readable diagnostics
//! suitable for IDE integration.

#![warn(missing_docs)]

pub mod builder;
pub mod diagnostic;
pub mod diagnostic_reporter;
pub mod enhanced_diagnostic;
pub mod formatter;
pub mod location;

pub use builder::DiagnosticBuilder;
pub use diagnostic::{Diagnostic, DiagnosticCode, Severity, Suggestion, RelatedInformation};
pub use diagnostic_reporter::{
    DiagnosticReporter, DiagnosticReport, DiagnosticSummary, GroupedDiagnostics,
    DiagnosticAnalysis, ErrorPattern, RootCause, WorkflowStep, ReporterConfig
};
pub use enhanced_diagnostic::{
    EnhancedDiagnostic, SmartSuggestion, SuggestionCategory, DocumentationLink, 
    QuickFix, SuggestionGenerator
};
pub use formatter::{DiagnosticFormatter, Format};
pub use location::{Position, SourceLocation, Span};

// Re-export LSP types when feature is enabled
#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "lsp")]
pub use lsp::to_lsp_diagnostic;
