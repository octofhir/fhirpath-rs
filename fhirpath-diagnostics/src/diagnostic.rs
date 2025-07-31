//! Core diagnostic types

use crate::location::SourceLocation;
use std::fmt;

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Hint - subtle suggestion for improvement
    Hint,
    /// Information - provides helpful information
    Info,
    /// Warning - may indicate a problem but doesn't prevent execution
    Warning,
    /// Error - prevents successful execution
    Error,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Info
    }
}
/// Diagnostic error codes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticCode {
    // Parsing errors
    /// Unexpected token in expression
    UnexpectedToken,
    /// Expected a specific token
    ExpectedToken(String),
    /// Unclosed string literal
    UnclosedString,
    /// Invalid number format
    InvalidNumber,
    /// Invalid date/time format
    InvalidDateTime,
    /// Unknown operator
    UnknownOperator,
    /// Unknown function
    UnknownFunction,
    /// Invalid escape sequence
    InvalidEscape,

    // Type errors
    /// Type mismatch
    TypeMismatch { 
        /// Expected type name
        expected: String, 
        /// Actual type found
        actual: String 
    },
    /// Invalid operand types for operator
    InvalidOperandTypes,
    /// Invalid argument types for function
    InvalidArgumentTypes,
    /// Cannot convert between types
    ConversionError,

    // Semantic errors
    /// Wrong number of arguments
    InvalidArity,
    /// Property not found
    PropertyNotFound,
    /// Variable not defined
    UndefinedVariable,
    /// Invalid type specifier
    InvalidTypeSpecifier,

    // Runtime errors
    /// Division by zero
    DivisionByZero,
    /// Index out of bounds
    IndexOutOfBounds,
    /// Arithmetic overflow
    ArithmeticOverflow,
    /// Invalid regular expression
    InvalidRegex,

    // Custom error code
    /// Custom error with a string code
    Custom(String),
}

/// A suggestion for fixing a diagnostic
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Suggestion {
    /// Human-readable message describing the suggestion
    pub message: String,
    /// Optional replacement text
    pub replacement: Option<String>,
    /// Location where the replacement should be applied
    pub location: SourceLocation,
}

/// Related information for a diagnostic
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RelatedInformation {
    /// Location of the related information
    pub location: SourceLocation,
    /// Message describing the relation
    pub message: String,
}

/// A diagnostic message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// Severity of the diagnostic
    pub severity: Severity,
    /// Error code
    pub code: DiagnosticCode,
    /// Human-readable message
    pub message: String,
    /// Source location
    pub location: SourceLocation,
    /// Suggestions for fixing the issue
    pub suggestions: Vec<Suggestion>,
    /// Related information
    pub related: Vec<RelatedInformation>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(
        severity: Severity,
        code: DiagnosticCode,
        message: String,
        location: SourceLocation,
    ) -> Self {
        Self {
            severity,
            code,
            message,
            location,
            suggestions: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add related information
    pub fn with_related(mut self, related: RelatedInformation) -> Self {
        self.related.push(related);
        self
    }

    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }

    /// Check if this is a warning
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, Severity::Warning)
    }

    /// Get the diagnostic code as a string
    pub fn code_string(&self) -> String {
        match &self.code {
            DiagnosticCode::UnexpectedToken => "E001".to_string(),
            DiagnosticCode::ExpectedToken(_) => "E002".to_string(),
            DiagnosticCode::UnclosedString => "E003".to_string(),
            DiagnosticCode::InvalidNumber => "E004".to_string(),
            DiagnosticCode::InvalidDateTime => "E005".to_string(),
            DiagnosticCode::UnknownOperator => "E006".to_string(),
            DiagnosticCode::UnknownFunction => "E007".to_string(),
            DiagnosticCode::InvalidEscape => "E008".to_string(),
            DiagnosticCode::TypeMismatch { .. } => "E100".to_string(),
            DiagnosticCode::InvalidOperandTypes => "E101".to_string(),
            DiagnosticCode::InvalidArgumentTypes => "E102".to_string(),
            DiagnosticCode::ConversionError => "E103".to_string(),
            DiagnosticCode::InvalidArity => "E200".to_string(),
            DiagnosticCode::PropertyNotFound => "E201".to_string(),
            DiagnosticCode::UndefinedVariable => "E202".to_string(),
            DiagnosticCode::InvalidTypeSpecifier => "E203".to_string(),
            DiagnosticCode::DivisionByZero => "E300".to_string(),
            DiagnosticCode::IndexOutOfBounds => "E301".to_string(),
            DiagnosticCode::ArithmeticOverflow => "E302".to_string(),
            DiagnosticCode::InvalidRegex => "E303".to_string(),
            DiagnosticCode::Custom(code) => code.clone(),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticCode::UnexpectedToken => write!(f, "unexpected token"),
            DiagnosticCode::ExpectedToken(token) => write!(f, "expected '{}'", token),
            DiagnosticCode::UnclosedString => write!(f, "unclosed string literal"),
            DiagnosticCode::InvalidNumber => write!(f, "invalid number format"),
            DiagnosticCode::InvalidDateTime => write!(f, "invalid date/time format"),
            DiagnosticCode::UnknownOperator => write!(f, "unknown operator"),
            DiagnosticCode::UnknownFunction => write!(f, "unknown function"),
            DiagnosticCode::InvalidEscape => write!(f, "invalid escape sequence"),
            DiagnosticCode::TypeMismatch { expected, actual } => {
                write!(f, "type mismatch: expected {}, found {}", expected, actual)
            }
            DiagnosticCode::InvalidOperandTypes => write!(f, "invalid operand types"),
            DiagnosticCode::InvalidArgumentTypes => write!(f, "invalid argument types"),
            DiagnosticCode::ConversionError => write!(f, "conversion error"),
            DiagnosticCode::InvalidArity => write!(f, "invalid number of arguments"),
            DiagnosticCode::PropertyNotFound => write!(f, "property not found"),
            DiagnosticCode::UndefinedVariable => write!(f, "undefined variable"),
            DiagnosticCode::InvalidTypeSpecifier => write!(f, "invalid type specifier"),
            DiagnosticCode::DivisionByZero => write!(f, "division by zero"),
            DiagnosticCode::IndexOutOfBounds => write!(f, "index out of bounds"),
            DiagnosticCode::ArithmeticOverflow => write!(f, "arithmetic overflow"),
            DiagnosticCode::InvalidRegex => write!(f, "invalid regular expression"),
            DiagnosticCode::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::{Position, Span};

    #[test]
    fn test_diagnostic_creation() {
        let location = SourceLocation {
            span: Span::new(Position::new(0, 0), Position::new(0, 5)),
            source_text: Some("error".to_string()),
            file_path: None,
        };

        let diagnostic = Diagnostic::new(
            Severity::Error,
            DiagnosticCode::UnknownFunction,
            "Unknown function 'foo'".to_string(),
            location,
        );

        assert!(diagnostic.is_error());
        assert!(!diagnostic.is_warning());
        assert_eq!(diagnostic.code_string(), "E007");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Info > Severity::Hint);
    }
}
