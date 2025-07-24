//! Parser error types

use fhirpath_diagnostics::{Diagnostic, DiagnosticBuilder, DiagnosticCode};
use nom::error::{ErrorKind, ParseError as NomParseError};
use thiserror::Error;

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse error with location information
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Syntax error at a specific location
    #[error("Syntax error at position {position}: {message}")]
    SyntaxError { position: usize, message: String },
    
    /// Unexpected token
    #[error("Unexpected token '{token}' at position {position}")]
    UnexpectedToken { token: String, position: usize },
    
    /// Unexpected end of input
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    /// Invalid literal value
    #[error("Invalid {literal_type} literal at position {position}: {value}")]
    InvalidLiteral {
        literal_type: String,
        value: String,
        position: usize,
    },
    
    /// Invalid escape sequence
    #[error("Invalid escape sequence at position {position}: {sequence}")]
    InvalidEscape { sequence: String, position: usize },
    
    /// Unclosed string literal
    #[error("Unclosed string literal starting at position {position}")]
    UnclosedString { position: usize },
    
    /// Invalid identifier
    #[error("Invalid identifier at position {position}: {identifier}")]
    InvalidIdentifier {
        identifier: String,
        position: usize,
    },
    
    /// Generic nom error
    #[error("Parse error at position {position}: {kind:?}")]
    NomError { position: usize, kind: ErrorKind },
}

impl ParseError {
    /// Convert to a diagnostic
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            ParseError::SyntaxError { position, message } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(message)
                    .with_location(0, *position, 0, position + 1)
                    .build()
            }
            ParseError::UnexpectedToken { token, position } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(&format!("Unexpected token '{}'", token))
                    .with_location(0, *position, 0, position + token.len())
                    .build()
            }
            ParseError::UnexpectedEof => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message("Unexpected end of input")
                    .build()
            }
            ParseError::InvalidLiteral { literal_type, value, position } => {
                DiagnosticBuilder::error(DiagnosticCode::InvalidNumber)
                    .with_message(&format!(
                        "Invalid {} literal: {}",
                        literal_type, value
                    ))
                    .with_location(0, *position, 0, position + value.len())
                    .build()
            }
            ParseError::InvalidEscape { sequence, position } => {
                DiagnosticBuilder::error(DiagnosticCode::InvalidEscape)
                    .with_message(&format!(
                        "Invalid escape sequence: {}",
                        sequence
                    ))
                    .with_location(0, *position, 0, position + sequence.len())
                    .build()
            }
            ParseError::UnclosedString { position } => {
                DiagnosticBuilder::error(DiagnosticCode::UnclosedString)
                    .with_message("Unclosed string literal")
                    .with_location(0, *position, 0, *position)
                    .build()
            }
            ParseError::InvalidIdentifier { identifier, position } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(&format!(
                        "Invalid identifier: {}",
                        identifier
                    ))
                    .with_location(0, *position, 0, position + identifier.len())
                    .build()
            }
            ParseError::NomError { position, kind } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(&format!("Parse error: {:?}", kind))
                    .with_location(0, *position, 0, *position)
                    .build()
            }
        }
    }
}

/// Implement nom's ParseError trait
impl<I> NomParseError<I> for ParseError {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        ParseError::NomError {
            position: 0, // Will be updated with proper position tracking
            kind,
        }
    }
    
    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}