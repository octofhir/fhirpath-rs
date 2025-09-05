//! Core error types with rich error code system

use std::fmt;
use thiserror::Error;
use serde::{Deserialize, Serialize};

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
    pub fn new(line: usize, column: usize, offset: usize, length: usize) -> Self {
        Self { line, column, offset, length }
    }

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
        error_code: ErrorCode,
        message: String,
        expression: String,
        location: Option<SourceLocation>,
        context: Option<String>,
    },

    /// Evaluation error during expression execution
    #[error("{error_code}: {message}")]
    EvaluationError {
        error_code: ErrorCode,
        message: String,
        expression: Option<String>,
        location: Option<SourceLocation>,
        context: Option<String>,
    },

    /// Type checking or validation error
    #[error("{error_code}: {message}")]
    TypeError {
        error_code: ErrorCode,
        message: String,
        expected_type: Option<String>,
        actual_type: Option<String>,
        location: Option<SourceLocation>,
    },

    /// Model provider error
    #[error("{error_code}: {message}")]
    ModelError {
        error_code: ErrorCode,
        message: String,
        resource_type: Option<String>,
        context: Option<String>,
    },

    /// Function registry error
    #[error("{error_code}: {message}")]
    FunctionError {
        error_code: ErrorCode,
        message: String,
        function_name: Option<String>,
        context: Option<String>,
    },

    /// System or configuration error
    #[error("{error_code}: {message}")]
    SystemError {
        error_code: ErrorCode,
        message: String,
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
    pub fn evaluation_error(
        error_code: ErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self::EvaluationError {
            error_code,
            message: message.into(),
            expression: None,
            location: None,
            context: None,
        }
    }

    /// Create a model error
    pub fn model_error(
        error_code: ErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self::ModelError {
            error_code,
            message: message.into(),
            resource_type: None,
            context: None,
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
}

/// Specialized evaluation error for the evaluation engine
#[derive(Debug, Error)]
pub enum EvaluationError {
    /// General evaluation failure
    #[error("Evaluation failed: {message}")]
    Failed {
        message: String,
        error_code: Option<ErrorCode>,
    },
}

// Additional error codes for system errors (using new FP0001-style codes)
pub const FP0200: ErrorCode = ErrorCode::new(200);  // System external error

/// Result type for FHIRPath operations
pub type Result<T> = std::result::Result<T, FhirPathError>;

/// Result type for evaluation operations
pub type EvaluationResult<T> = std::result::Result<T, EvaluationError>;