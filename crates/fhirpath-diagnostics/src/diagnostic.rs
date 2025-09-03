// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Core diagnostic types

use crate::location::SourceLocation;
use std::fmt;

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default)]
pub enum Severity {
    /// Hint - subtle suggestion for improvement
    Hint,
    /// Information - provides helpful information
    #[default]
    Info,
    /// Warning - may indicate a problem but doesn't prevent execution
    Warning,
    /// Error - prevents successful execution
    Error,
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
        actual: String,
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
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Suggestion {
    /// Human-readable message describing the suggestion
    pub message: String,
    /// Optional replacement text edit
    pub replacement: Option<TextEdit>,
    /// Confidence level of this suggestion (0.0 to 1.0)
    pub confidence: f32,
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
}

/// Text edit for replacing content at a specific location
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextEdit {
    /// Location where the edit should be applied
    pub location: SourceLocation,
    /// New text to insert
    pub new_text: String,
}

/// Types of suggestions that can be provided
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuggestionType {
    /// Fix a typo in function or property name
    TypoFix,
    /// Alternative function suggestion
    AlternativeFunction,
    /// Alternative property suggestion
    AlternativeProperty,
    /// Type conversion suggestion
    TypeConversion,
    /// Syntax improvement suggestion
    SyntaxImprovement,
    /// Performance optimization suggestion
    PerformanceOptimization,
    /// General suggestion
    General,
}

/// Quick fix action that can be applied automatically
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuickFix {
    /// Title describing the quick fix
    pub title: String,
    /// Text edits to apply
    pub edits: Vec<TextEdit>,
    /// Kind of quick fix
    pub kind: QuickFixKind,
}

/// Types of quick fixes
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuickFixKind {
    /// Quick fix that corrects an error
    QuickFix,
    /// Refactoring that improves code
    Refactor,
    /// Source action that adds functionality
    Source,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(
        message: String,
        suggestion_type: SuggestionType,
        confidence: f32,
    ) -> Self {
        Self {
            message,
            replacement: None,
            confidence: confidence.clamp(0.0, 1.0),
            suggestion_type,
        }
    }
    
    /// Create a suggestion with a text replacement
    pub fn with_replacement(
        message: String,
        replacement: TextEdit,
        suggestion_type: SuggestionType,
        confidence: f32,
    ) -> Self {
        Self {
            message,
            replacement: Some(replacement),
            confidence: confidence.clamp(0.0, 1.0),
            suggestion_type,
        }
    }
    
    /// Create a typo fix suggestion
    pub fn typo_fix(_original: &str, replacement: &str, location: SourceLocation, confidence: f32) -> Self {
        Self::with_replacement(
            format!("Did you mean '{replacement}'?"),
            TextEdit {
                location,
                new_text: replacement.to_string(),
            },
            SuggestionType::TypoFix,
            confidence,
        )
    }
    
    /// Create an alternative function suggestion
    pub fn alternative_function(function_name: &str, location: SourceLocation) -> Self {
        Self::with_replacement(
            format!("Consider using '{function_name}()' instead"),
            TextEdit {
                location,
                new_text: function_name.to_string(),
            },
            SuggestionType::AlternativeFunction,
            0.8,
        )
    }
}

impl TextEdit {
    /// Create a new text edit
    pub fn new(location: SourceLocation, new_text: String) -> Self {
        Self { location, new_text }
    }
}

impl QuickFix {
    /// Create a new quick fix
    pub fn new(title: String, edits: Vec<TextEdit>, kind: QuickFixKind) -> Self {
        Self { title, edits, kind }
    }
    
    /// Create a simple single-edit quick fix
    pub fn simple_replacement(
        title: String,
        location: SourceLocation,
        new_text: String,
    ) -> Self {
        Self::new(
            title,
            vec![TextEdit::new(location, new_text)],
            QuickFixKind::QuickFix,
        )
    }
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
#[derive(Debug, Clone, PartialEq)]
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
    /// Simple contextual help message
    pub help: Option<String>,
    /// Suggestions for fixing the issue
    pub suggestions: Vec<Suggestion>,
    /// Related information
    pub related: Vec<RelatedInformation>,
    /// Quick fixes that can be applied automatically
    pub quick_fixes: Vec<QuickFix>,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(
        code: DiagnosticCode,
        severity: Severity,
        message: String,
        location: SourceLocation,
    ) -> Self {
        let help = Self::generate_help(&code);
        Self {
            severity,
            code,
            message,
            location,
            help,
            suggestions: Vec::new(),
            related: Vec::new(),
            quick_fixes: Vec::new(),
        }
    }

    /// Generate simple contextual help based on diagnostic code
    fn generate_help(code: &DiagnosticCode) -> Option<String> {
        match code {
            DiagnosticCode::UnknownFunction => {
                Some("Check function name spelling and available functions".to_string())
            }
            DiagnosticCode::ExpectedToken(_) => {
                Some("Check expression syntax for missing or incorrect tokens".to_string())
            }
            DiagnosticCode::TypeMismatch { .. } => {
                Some("Ensure arguments match expected types for the operation".to_string())
            }
            DiagnosticCode::InvalidArity => {
                Some("Check function documentation for correct number of arguments".to_string())
            }
            DiagnosticCode::UndefinedVariable => {
                Some("Define the variable or check variable name spelling".to_string())
            }
            DiagnosticCode::UnexpectedToken => {
                Some("Review expression syntax for unexpected characters or operators".to_string())
            }
            DiagnosticCode::DivisionByZero => {
                Some("Ensure divisor is not zero before performing division".to_string())
            }
            _ => None,
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

    /// Add a quick fix
    pub fn with_quick_fix(mut self, quick_fix: QuickFix) -> Self {
        self.quick_fixes.push(quick_fix);
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

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            match self.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN",
                Severity::Info => "INFO",
                Severity::Hint => "HINT",
            },
            self.code,
            self.message
        )
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticCode::UnexpectedToken => write!(f, "unexpected token"),
            DiagnosticCode::ExpectedToken(token) => write!(f, "expected '{token}'"),
            DiagnosticCode::UnclosedString => write!(f, "unclosed string literal"),
            DiagnosticCode::InvalidNumber => write!(f, "invalid number format"),
            DiagnosticCode::InvalidDateTime => write!(f, "invalid date/time format"),
            DiagnosticCode::UnknownOperator => write!(f, "unknown operator"),
            DiagnosticCode::UnknownFunction => write!(f, "unknown function"),
            DiagnosticCode::InvalidEscape => write!(f, "invalid escape sequence"),
            DiagnosticCode::TypeMismatch { expected, actual } => {
                write!(f, "type mismatch: expected {expected}, found {actual}")
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
            DiagnosticCode::Custom(msg) => write!(f, "{msg}"),
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
            DiagnosticCode::UnknownFunction,
            Severity::Error,
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
