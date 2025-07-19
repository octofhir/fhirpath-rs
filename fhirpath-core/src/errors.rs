// FHIRPath Error Types
//
// This module defines the error types used throughout the FHIRPath implementation.

use thiserror::Error;

/// Errors that can occur during FHIRPath parsing and evaluation
#[derive(Error, Debug)]
pub enum FhirPathError {
    /// Error during lexical analysis
    #[error("Lexer error: {0}")]
    LexerError(String),

    /// Error during parsing
    #[error("Parser error: {0}")]
    ParserError(String),

    /// Error during evaluation
    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    /// Type error during evaluation
    #[error("Type error: {0}")]
    TypeError(String),

    /// Feature isn't implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}
