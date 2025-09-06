//! Diagnostic builder for fluent diagnostic creation

use crate::core::SourceLocation;
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

/// Builder for creating diagnostics with fluent API
#[derive(Debug)]
pub struct DiagnosticBuilder {
    /// The diagnostic being built
    diagnostic: Diagnostic,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder with the given severity
    pub fn new(
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            diagnostic: Diagnostic {
                severity,
                code: DiagnosticCode {
                    code: code.into(),
                    namespace: None,
                },
                message: message.into(),
                location: None,
                related: Vec::new(),
            },
        }
    }

    /// Set the location for this diagnostic
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.diagnostic.location = Some(location);
        self
    }

    /// Add a related diagnostic
    pub fn with_related(mut self, related: Diagnostic) -> Self {
        self.diagnostic.related.push(related);
        self
    }

    /// Set the namespace for the diagnostic code
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.diagnostic.code.namespace = Some(namespace.into());
        self
    }

    /// Build the final diagnostic
    pub fn build(self) -> Diagnostic {
        self.diagnostic
    }
}
