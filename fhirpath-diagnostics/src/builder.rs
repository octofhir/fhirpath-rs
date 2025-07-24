//! Builder pattern for constructing diagnostics

use crate::diagnostic::{Diagnostic, DiagnosticCode, RelatedInformation, Severity, Suggestion};
use crate::location::{Position, SourceLocation, Span};

/// Builder for constructing diagnostics fluently
#[derive(Debug, Clone)]
pub struct DiagnosticBuilder {
    severity: Severity,
    code: DiagnosticCode,
    message: String,
    location: Option<SourceLocation>,
    suggestions: Vec<Suggestion>,
    related: Vec<RelatedInformation>,
}

impl DiagnosticBuilder {
    /// Create a new error diagnostic builder
    pub fn error(code: DiagnosticCode) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: String::new(),
            location: None,
            suggestions: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Create a new warning diagnostic builder
    pub fn warning(code: DiagnosticCode) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            message: String::new(),
            location: None,
            suggestions: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Create a new info diagnostic builder
    pub fn info(code: DiagnosticCode) -> Self {
        Self {
            severity: Severity::Info,
            code,
            message: String::new(),
            location: None,
            suggestions: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Create a new hint diagnostic builder
    pub fn hint(code: DiagnosticCode) -> Self {
        Self {
            severity: Severity::Hint,
            code,
            message: String::new(),
            location: None,
            suggestions: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Set the message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the location from a span
    pub fn with_span(mut self, span: Span) -> Self {
        self.location = Some(SourceLocation::new(span));
        self
    }

    /// Set the location from start and end positions
    pub fn with_positions(mut self, start: Position, end: Position) -> Self {
        self.with_span(Span::new(start, end))
    }

    /// Set the location from line/column coordinates
    pub fn with_location(mut self, start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        self.with_positions(
            Position::new(start_line, start_col),
            Position::new(end_line, end_col),
        )
    }

    /// Set the location from byte offsets
    pub fn with_offsets(mut self, source: &str, start_offset: usize, end_offset: usize) -> Self {
        self.with_span(Span::from_offsets(source, start_offset, end_offset))
    }

    /// Set the complete source location
    pub fn with_source_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add source text to the location
    pub fn with_source_text(mut self, text: impl Into<String>) -> Self {
        if let Some(loc) = &mut self.location {
            loc.source_text = Some(text.into());
        }
        self
    }

    /// Add file path to the location
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        if let Some(loc) = &mut self.location {
            loc.file_path = Some(path.into());
        }
        self
    }

    /// Add a suggestion
    pub fn suggest(mut self, message: impl Into<String>, replacement: Option<String>) -> Self {
        let location = self.location.clone().unwrap_or_default();
        self.suggestions.push(Suggestion {
            message: message.into(),
            replacement,
            location,
        });
        self
    }

    /// Add a suggestion with a specific location
    pub fn suggest_at(
        mut self,
        message: impl Into<String>,
        replacement: Option<String>,
        location: SourceLocation,
    ) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            replacement,
            location,
        });
        self
    }

    /// Add related information
    pub fn related(mut self, location: SourceLocation, message: impl Into<String>) -> Self {
        self.related.push(RelatedInformation {
            location,
            message: message.into(),
        });
        self
    }

    /// Build the diagnostic
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            severity: self.severity,
            code: self.code,
            message: self.message,
            location: self.location.unwrap_or_default(),
            suggestions: self.suggestions,
            related: self.related,
        }
    }
}

// Convenience functions for common diagnostics

impl DiagnosticBuilder {
    /// Create an "unknown function" error
    pub fn unknown_function(name: &str) -> Self {
        Self::error(DiagnosticCode::UnknownFunction)
            .with_message(format!("Unknown function '{}'", name))
    }

    /// Create an "unknown operator" error
    pub fn unknown_operator(op: &str) -> Self {
        Self::error(DiagnosticCode::UnknownOperator)
            .with_message(format!("Unknown operator '{}'", op))
    }

    /// Create a "type mismatch" error
    pub fn type_mismatch(expected: &str, actual: &str) -> Self {
        Self::error(DiagnosticCode::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
        .with_message(format!("Type mismatch: expected {}, found {}", expected, actual))
    }

    /// Create an "undefined variable" error
    pub fn undefined_variable(name: &str) -> Self {
        Self::error(DiagnosticCode::UndefinedVariable)
            .with_message(format!("Undefined variable '{}'", name))
    }

    /// Create a "property not found" error
    pub fn property_not_found(property: &str, type_name: &str) -> Self {
        Self::error(DiagnosticCode::PropertyNotFound)
            .with_message(format!("Property '{}' not found on type '{}'", property, type_name))
    }

    /// Create an "expected token" error
    pub fn expected_token(token: &str) -> Self {
        Self::error(DiagnosticCode::ExpectedToken(token.to_string()))
            .with_message(format!("Expected '{}'", token))
    }

    /// Create a "division by zero" error
    pub fn division_by_zero() -> Self {
        Self::error(DiagnosticCode::DivisionByZero)
            .with_message("Division by zero")
    }

    /// Create an "index out of bounds" error
    pub fn index_out_of_bounds(index: i64, size: usize) -> Self {
        Self::error(DiagnosticCode::IndexOutOfBounds)
            .with_message(format!("Index {} out of bounds for collection of size {}", index, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diagnostic = DiagnosticBuilder::error(DiagnosticCode::UnknownFunction)
            .with_message("Unknown function 'foo'")
            .with_location(0, 10, 0, 13)
            .with_source_text("foo()")
            .suggest("Did you mean 'for'?", Some("for".to_string()))
            .build();

        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(diagnostic.message, "Unknown function 'foo'");
        assert_eq!(diagnostic.suggestions.len(), 1);
        assert_eq!(diagnostic.suggestions[0].message, "Did you mean 'for'?");
    }

    #[test]
    fn test_convenience_builders() {
        let diagnostic = DiagnosticBuilder::unknown_function("foo")
            .with_location(0, 0, 0, 3)
            .build();

        assert_eq!(diagnostic.message, "Unknown function 'foo'");
        assert!(matches!(diagnostic.code, DiagnosticCode::UnknownFunction));

        let diagnostic = DiagnosticBuilder::type_mismatch("String", "Integer")
            .build();

        assert_eq!(diagnostic.message, "Type mismatch: expected String, found Integer");
    }

    #[test]
    fn test_multiple_suggestions() {
        let diagnostic = DiagnosticBuilder::unknown_function("whre")
            .suggest("Did you mean 'where'?", Some("where".to_string()))
            .suggest("Did you mean 'when'?", Some("when".to_string()))
            .build();

        assert_eq!(diagnostic.suggestions.len(), 2);
    }
}