//! Error types for FHIRPath evaluation
//!
//! This module defines the error types used throughout the FHIRPath engine.

use thiserror::Error;

/// Result type alias for FHIRPath operations
pub type Result<T> = std::result::Result<T, FhirPathError>;

/// Comprehensive error type for FHIRPath operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum FhirPathError {
    /// Parsing errors
    #[error("Parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },

    /// Type errors during evaluation
    #[error("Type error: {message}")]
    TypeError { message: String },

    /// Runtime evaluation errors
    #[error("Evaluation error: {message}")]
    EvaluationError { message: String },

    /// Function call errors
    #[error("Function '{function_name}' error: {message}")]
    FunctionError {
        function_name: String,
        message: String,
    },

    /// Invalid expression structure
    #[error("Invalid expression: {message}")]
    InvalidExpression { message: String },

    /// Division by zero or other arithmetic errors
    #[error("Arithmetic error: {message}")]
    ArithmeticError { message: String },

    /// Index out of bounds
    #[error("Index out of bounds: {index} for collection of size {size}")]
    IndexOutOfBounds { index: i64, size: usize },

    /// Unknown function
    #[error("Unknown function: {function_name}")]
    UnknownFunction { function_name: String },

    /// Invalid argument count
    #[error("Function '{function_name}' expects {expected} arguments, got {actual}")]
    InvalidArgumentCount {
        function_name: String,
        expected: usize,
        actual: usize,
    },

    /// Conversion errors
    #[error("Conversion error: cannot convert {from} to {to}")]
    ConversionError { from: String, to: String },

    /// Generic error for compatibility
    #[error("FHIRPath error: {message}")]
    Generic { message: String },
}

impl FhirPathError {
    /// Create a parse error
    pub fn parse_error(position: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            position,
            message: message.into(),
        }
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
        }
    }

    /// Create an evaluation error
    pub fn evaluation_error(message: impl Into<String>) -> Self {
        Self::EvaluationError {
            message: message.into(),
        }
    }

    /// Create a function error
    pub fn function_error(function_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FunctionError {
            function_name: function_name.into(),
            message: message.into(),
        }
    }

    /// Create an invalid expression error
    pub fn invalid_expression(message: impl Into<String>) -> Self {
        Self::InvalidExpression {
            message: message.into(),
        }
    }

    /// Create an arithmetic error
    pub fn arithmetic_error(message: impl Into<String>) -> Self {
        Self::ArithmeticError {
            message: message.into(),
        }
    }

    /// Create an index out of bounds error
    pub fn index_out_of_bounds(index: i64, size: usize) -> Self {
        Self::IndexOutOfBounds { index, size }
    }

    /// Create an unknown function error
    pub fn unknown_function(function_name: impl Into<String>) -> Self {
        Self::UnknownFunction {
            function_name: function_name.into(),
        }
    }

    /// Create an invalid argument count error
    pub fn invalid_argument_count(
        function_name: impl Into<String>,
        expected: usize,
        actual: usize,
    ) -> Self {
        Self::InvalidArgumentCount {
            function_name: function_name.into(),
            expected,
            actual,
        }
    }

    /// Create a conversion error
    pub fn conversion_error(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::ConversionError {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}

/// Convert from Box<dyn std::error::Error> for compatibility with tests
impl From<Box<dyn std::error::Error>> for FhirPathError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::Generic {
            message: err.to_string(),
        }
    }
}

// Note: From<FhirPathError> for Box<dyn std::error::Error> is automatically provided by Rust
// since FhirPathError implements std::error::Error via thiserror
