//! Core error types with rich error code system

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub use super::error_code::*;

/// Source location for error reporting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Character offset from start (0-based)
    pub offset: usize,
    /// Length of the problematic text
    pub length: usize,
}

impl SourceLocation {
    /// Create new source location
    pub fn new(line: usize, column: usize, offset: usize, length: usize) -> Self {
        Self {
            line,
            column,
            offset,
            length,
        }
    }

    /// Create point source location with length 1
    pub fn point(line: usize, column: usize, offset: usize) -> Self {
        Self::new(line, column, offset, 1)
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Main FHIRPath error type with rich error codes and context
#[derive(Debug, Clone, Error)]
pub enum FhirPathError {
    /// Parse error with source location
    #[error("{error_code}: {message}")]
    ParseError {
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
        /// Expression being parsed
        expression: String,
        /// Source location
        location: Option<SourceLocation>,
        /// Additional context
        context: Option<String>,
    },

    /// Evaluation error during expression execution
    #[error("{error_code}: {message}")]
    EvaluationError {
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
        /// Expression being evaluated
        expression: Option<String>,
        /// Source location
        location: Option<SourceLocation>,
        /// Additional context
        context: Option<String>,
    },

    /// Type checking or validation error
    #[error("{error_code}: {message}")]
    TypeError {
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
        /// Expected type
        expected_type: Option<String>,
        /// Actual type
        actual_type: Option<String>,
        /// Source location
        location: Option<SourceLocation>,
    },

    /// Model provider error
    #[error("{error_code}: {message}")]
    ModelError {
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
        /// Resource type
        resource_type: Option<String>,
        /// Additional context
        context: Option<String>,
    },

    /// Function registry error
    #[error("{error_code}: {message}")]
    FunctionError {
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
        /// Function name that caused the error
        function_name: Option<String>,
        /// Additional context
        context: Option<String>,
    },

    /// System or configuration error
    #[error("{error_code}: {message}")]
    SystemError {
        /// Error code
        error_code: ErrorCode,
        /// Error message
        message: String,
        /// Additional context
        context: Option<String>,
    },
}

impl FhirPathError {
    /// Create a parse error
    pub fn parse_error(
        error_code: ErrorCode,
        message: impl Into<String>,
        expression: impl Into<String>,
        location: Option<SourceLocation>,
    ) -> Self {
        Self::ParseError {
            error_code,
            message: message.into(),
            expression: expression.into(),
            location,
            context: None,
        }
    }

    /// Create an evaluation error
    pub fn evaluation_error(error_code: ErrorCode, message: impl Into<String>) -> Self {
        Self::EvaluationError {
            error_code,
            message: message.into(),
            expression: None,
            location: None,
            context: None,
        }
    }

    /// Create a model error
    pub fn model_error(error_code: ErrorCode, message: impl Into<String>) -> Self {
        Self::ModelError {
            error_code,
            message: message.into(),
            resource_type: None,
            context: None,
        }
    }

    /// Create a resource type mismatch error
    pub fn resource_type_mismatch(
        expected_type: impl Into<String>,
        actual_type: impl Into<String>,
        location: Option<SourceLocation>,
    ) -> Self {
        let expected = expected_type.into();
        let actual = actual_type.into();
        let message = format!(
            "Resource type mismatch: expression expects '{expected}' but context contains '{actual}' resource"
        );
        Self::EvaluationError {
            error_code: super::error_code::FP0061,
            message,
            expression: None,
            location,
            context: Some(format!("Expected: {expected}, Actual: {actual}")),
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &ErrorCode {
        match self {
            Self::ParseError { error_code, .. } => error_code,
            Self::EvaluationError { error_code, .. } => error_code,
            Self::TypeError { error_code, .. } => error_code,
            Self::ModelError { error_code, .. } => error_code,
            Self::FunctionError { error_code, .. } => error_code,
            Self::SystemError { error_code, .. } => error_code,
        }
    }

    /// Get error information with help and documentation
    pub fn error_info(&self) -> &'static ErrorInfo {
        self.error_code().info()
    }

    /// Get the source location for this error, if available
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            Self::ParseError { location, .. } => location.as_ref(),
            Self::EvaluationError { location, .. } => location.as_ref(),
            Self::TypeError { location, .. } => location.as_ref(),
            Self::ModelError { .. } => None,
            Self::FunctionError { .. } => None,
            Self::SystemError { .. } => None,
        }
    }

    /// Get the source span (offset and length) for this error, if available
    pub fn span(&self) -> Option<std::ops::Range<usize>> {
        self.location()
            .map(|loc| loc.offset..(loc.offset + loc.length))
    }

    /// Get the expression being evaluated when this error occurred
    pub fn expression(&self) -> Option<&str> {
        match self {
            Self::ParseError { expression, .. } => Some(expression.as_str()),
            Self::EvaluationError { expression, .. } => expression.as_ref().map(|s| s.as_str()),
            _ => None,
        }
    }
}

/// Specialized evaluation error for the evaluation engine
#[derive(Debug, Error)]
pub enum EvaluationError {
    /// General evaluation failure
    #[error("Evaluation failed: {message}")]
    Failed {
        /// Error message describing the failure
        message: String,
        /// Optional error code for categorization
        error_code: Option<ErrorCode>,
    },
}

// Additional error codes for system errors (using new FP0001-style codes)
/// System external error code
pub const FP0200: ErrorCode = ErrorCode::new(200);

/// Result type for FHIRPath operations
pub type Result<T> = std::result::Result<T, FhirPathError>;

/// Result type for evaluation operations
pub type EvaluationResult<T> = std::result::Result<T, EvaluationError>;
