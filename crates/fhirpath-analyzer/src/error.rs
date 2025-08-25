//! Error types for the FHIRPath analyzer

use octofhir_fhirpath_model as fhirpath_model;
use thiserror::Error;

/// Main error type for analysis operations
/// Main error type for analysis operations
#[derive(Debug, Error, Clone)]
pub enum AnalysisError {
    /// Type inference operation failed
    #[error("Type inference failed: {message}")]
    TypeInferenceFailed {
        /// Error message describing the failure
        message: String,
    },

    /// Function analysis operation failed
    #[error("Function analysis failed: {function_name} - {message}")]
    FunctionAnalysisError {
        /// Name of the function that failed analysis
        function_name: String,
        /// Error message describing the failure
        message: String,
    },

    /// Union type creation failed
    #[error("Union type creation failed: {message}")]
    UnionTypeError {
        /// Error message describing the failure
        message: String,
    },

    /// Model provider error
    #[error("Model provider error: {source}")]
    ModelProviderError {
        #[from]
        /// The underlying model error
        source: fhirpath_model::error::ModelError,
    },

    /// Invalid expression error
    #[error("Invalid expression: {message}")]
    InvalidExpression {
        /// Error message describing why the expression is invalid
        message: String,
    },
}

/// Validation error with precise location information
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// Human-readable error message
    pub message: String,
    /// Classification of the validation error
    pub error_type: ValidationErrorType,
    /// Optional source location where the error occurred
    pub location: Option<SourceLocation>,
    /// Suggested fixes or alternatives
    pub suggestions: Vec<String>,
}

/// Classification of validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    /// Type mismatch between expected and actual types
    TypeMismatch,
    /// Invalid function usage or unknown function
    InvalidFunction,
    /// Invalid property access on a type
    InvalidProperty,
    /// Invalid type operation or cast
    InvalidTypeOperation,
    /// Constraint violation in type or value
    ConstraintViolation,
    /// Invalid FHIR resource type
    InvalidResourceType,
}

/// Source location information for errors
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// Starting character position
    pub start: usize,
    /// Ending character position  
    pub end: usize,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
}
