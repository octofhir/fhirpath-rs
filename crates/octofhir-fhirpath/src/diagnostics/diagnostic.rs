//! Core diagnostic types and functionality
//!
//! This module defines the core diagnostic system used throughout the FHIRPath
//! implementation for error reporting, warnings, and informational messages.

use serde::{Deserialize, Serialize};
use crate::core::SourceLocation;

/// Severity level for diagnostics
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// Error that prevents processing
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Info,
    /// Hint for optimization or improvement
    Hint,
}

/// Diagnostic code for categorizing issues
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiagnosticCode {
    /// The diagnostic code identifier
    pub code: String,
    /// Optional namespace for the code
    pub namespace: Option<String>,
}

/// A diagnostic message with location and severity information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// The severity of this diagnostic
    pub severity: DiagnosticSeverity,
    /// The diagnostic code
    pub code: DiagnosticCode,
    /// The diagnostic message
    pub message: String,
    /// Optional source location
    pub location: Option<SourceLocation>,
    /// Optional related diagnostics
    pub related: Vec<Diagnostic>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: code.into(),
                namespace: None,
            },
            message: message.into(),
            location: None,
            related: Vec::new(),
        }
    }

    /// Create a new warning diagnostic
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: DiagnosticCode {
                code: code.into(),
                namespace: None,
            },
            message: message.into(),
            location: None,
            related: Vec::new(),
        }
    }

    /// Create a new info diagnostic
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            code: DiagnosticCode {
                code: code.into(),
                namespace: None,
            },
            message: message.into(),
            location: None,
            related: Vec::new(),
        }
    }

    /// Create a new hint diagnostic
    pub fn hint(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Hint,
            code: DiagnosticCode {
                code: code.into(),
                namespace: None,
            },
            message: message.into(),
            location: None,
            related: Vec::new(),
        }
    }

    /// Set the location for this diagnostic
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add a related diagnostic
    pub fn with_related(mut self, related: Diagnostic) -> Self {
        self.related.push(related);
        self
    }
}