//! Diagnostic system for FHIRPath parsing and evaluation errors
//!
//! This crate provides a comprehensive diagnostic system that can produce
//! both human-friendly error messages and machine-readable diagnostics
//! suitable for IDE integration.

#![warn(missing_docs)]

pub mod diagnostic;
pub mod location;
pub mod builder;
pub mod formatter;

pub use diagnostic::{Diagnostic, Severity, DiagnosticCode};
pub use location::{SourceLocation, Position, Span};
pub use builder::DiagnosticBuilder;
pub use formatter::{DiagnosticFormatter, Format};

// Re-export LSP types when feature is enabled
#[cfg(feature = "lsp")]
pub mod lsp;

#[cfg(feature = "lsp")]
pub use lsp::to_lsp_diagnostic;