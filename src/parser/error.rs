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

//! Parser error types

use crate::diagnostics::{Diagnostic, DiagnosticBuilder, DiagnosticCode};
use nom::error::{ErrorKind, ParseError as NomParseError};
use std::borrow::Cow;
use thiserror::Error;

/// Pre-allocated common error messages for performance
pub mod common_messages {
    /// Common literal type error messages
    pub const INTEGER: &str = "integer";
    /// Decimal literal type name
    pub const DECIMAL: &str = "decimal";
    /// String literal type name
    pub const STRING: &str = "string";
    /// Boolean literal type name
    pub const BOOLEAN: &str = "boolean";
    /// Date literal type name
    pub const DATE: &str = "date";
    /// DateTime literal type name
    pub const DATETIME: &str = "datetime";
    /// Time literal type name
    pub const TIME: &str = "time";
    /// Quantity literal type name
    pub const QUANTITY: &str = "quantity";

    /// Common token expectation messages
    pub const EXPECTED_IDENTIFIER: &str = "identifier";
    /// Expected expression error message
    pub const EXPECTED_EXPRESSION: &str = "expression";
    /// Expected operator error message
    pub const EXPECTED_OPERATOR: &str = "operator";
    /// Expected function name error message
    pub const EXPECTED_FUNCTION_NAME: &str = "function name";
    /// Expected left parenthesis error message
    pub const EXPECTED_LEFT_PAREN: &str = "'('";
    /// Expected right parenthesis error message
    pub const EXPECTED_RIGHT_PAREN: &str = "')'";
    /// Expected left bracket error message
    pub const EXPECTED_LEFT_BRACKET: &str = "'['";
    /// Expected right bracket error message
    pub const EXPECTED_RIGHT_BRACKET: &str = "']'";
    /// Expected dot operator error message
    pub const EXPECTED_DOT: &str = "'.'";
    /// Expected comma error message
    pub const EXPECTED_COMMA: &str = "','";
    /// Expected string literal error message
    pub const EXPECTED_STRING_LITERAL: &str = "string literal";
    /// Expected number error message
    pub const EXPECTED_NUMBER: &str = "number";

    /// Common syntax error messages  
    pub const INVALID_ESCAPE_SEQUENCE: &str = "Invalid escape sequence";
    /// Unclosed string literal error message
    pub const UNCLOSED_STRING: &str = "Unclosed string literal";
    /// Invalid number format error message
    pub const INVALID_NUMBER_FORMAT: &str = "Invalid number format";
    /// Unexpected end of file error message
    pub const UNEXPECTED_EOF: &str = "Unexpected end of input";
    /// Invalid identifier character error message
    pub const INVALID_IDENTIFIER_CHAR: &str = "Invalid character in identifier";
}

/// Result type for parser operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse error with location information
#[derive(Error)]
pub enum ParseError {
    /// Syntax error at a specific location
    #[error("Syntax error at position {position}: {message}")]
    SyntaxError {
        /// Position where the error occurred
        position: usize,
        /// Error message describing the syntax error (zero-allocation for static strings)
        message: Cow<'static, str>,
    },

    /// Unexpected token
    #[error("Unexpected token '{token}' at position {position}")]
    UnexpectedToken {
        /// The unexpected token that was found
        token: Cow<'static, str>,
        /// Position where the token was found
        position: usize,
    },

    /// Unexpected end of input
    #[error("Unexpected end of input")]
    UnexpectedEof,

    /// Expected token
    #[error("Expected {expected} at position {position}")]
    ExpectedToken {
        /// The expected token description (zero-allocation for static strings)
        expected: Cow<'static, str>,
        /// Position where the token was expected  
        position: usize,
    },

    /// Unexpected end of input at specific position
    #[error("Unexpected end of input at position {position}")]
    UnexpectedEndOfInput {
        /// Position where more input was expected
        position: usize,
    },
    /// Invalid literal value
    #[error("Invalid {literal_type} literal at position {position}: {value}")]
    InvalidLiteral {
        /// Type of literal that failed to parse (zero-allocation for static strings)
        literal_type: Cow<'static, str>,
        /// The invalid value that was encountered
        value: Cow<'static, str>,
        /// Position where the invalid literal was found
        position: usize,
    },

    /// Invalid escape sequence
    #[error("Invalid escape sequence at position {position}: {sequence}")]
    InvalidEscape {
        /// The invalid escape sequence
        sequence: Cow<'static, str>,
        /// Position where the escape sequence was found
        position: usize,
    },

    /// Unclosed string literal
    #[error("Unclosed string literal starting at position {position}")]
    UnclosedString {
        /// Position where the unclosed string started
        position: usize,
    },

    /// Invalid identifier
    #[error("Invalid identifier at position {position}: {identifier}")]
    InvalidIdentifier {
        /// The invalid identifier
        identifier: Cow<'static, str>,
        /// Position where the identifier was found
        position: usize,
    },

    /// Generic nom error
    #[error("Parse error at position {position}: {kind:?}")]
    NomError {
        /// Position where the parse error occurred
        position: usize,
        /// The nom error kind
        kind: ErrorKind,
    },

    /// Lazy formatted error - defers expensive string formatting until display time
    #[error("{}", .format_fn())]
    LazyFormatted {
        /// Lazy formatting function that creates the error message on demand
        format_fn: Box<dyn Fn() -> String + Send + Sync>,
        /// Position where the error occurred
        position: usize,
    },
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SyntaxError { position, message } => f
                .debug_struct("SyntaxError")
                .field("position", position)
                .field("message", message)
                .finish(),
            Self::UnexpectedToken { token, position } => f
                .debug_struct("UnexpectedToken")
                .field("token", token)
                .field("position", position)
                .finish(),
            Self::UnexpectedEof => write!(f, "UnexpectedEof"),
            Self::ExpectedToken { expected, position } => f
                .debug_struct("ExpectedToken")
                .field("expected", expected)
                .field("position", position)
                .finish(),
            Self::UnexpectedEndOfInput { position } => f
                .debug_struct("UnexpectedEndOfInput")
                .field("position", position)
                .finish(),
            Self::InvalidLiteral {
                literal_type,
                value,
                position,
            } => f
                .debug_struct("InvalidLiteral")
                .field("literal_type", literal_type)
                .field("value", value)
                .field("position", position)
                .finish(),
            Self::InvalidEscape { sequence, position } => f
                .debug_struct("InvalidEscape")
                .field("sequence", sequence)
                .field("position", position)
                .finish(),
            Self::UnclosedString { position } => f
                .debug_struct("UnclosedString")
                .field("position", position)
                .finish(),
            Self::InvalidIdentifier {
                identifier,
                position,
            } => f
                .debug_struct("InvalidIdentifier")
                .field("identifier", identifier)
                .field("position", position)
                .finish(),
            Self::NomError { position, kind } => f
                .debug_struct("NomError")
                .field("position", position)
                .field("kind", kind)
                .finish(),
            Self::LazyFormatted { position, .. } => f
                .debug_struct("LazyFormatted")
                .field("position", position)
                .field("format_fn", &"<closure>")
                .finish(),
        }
    }
}

impl Clone for ParseError {
    fn clone(&self) -> Self {
        match self {
            Self::SyntaxError { position, message } => Self::SyntaxError {
                position: *position,
                message: message.clone(),
            },
            Self::UnexpectedToken { token, position } => Self::UnexpectedToken {
                token: token.clone(),
                position: *position,
            },
            Self::UnexpectedEof => Self::UnexpectedEof,
            Self::ExpectedToken { expected, position } => Self::ExpectedToken {
                expected: expected.clone(),
                position: *position,
            },
            Self::UnexpectedEndOfInput { position } => Self::UnexpectedEndOfInput {
                position: *position,
            },
            Self::InvalidLiteral {
                literal_type,
                value,
                position,
            } => Self::InvalidLiteral {
                literal_type: literal_type.clone(),
                value: value.clone(),
                position: *position,
            },
            Self::InvalidEscape { sequence, position } => Self::InvalidEscape {
                sequence: sequence.clone(),
                position: *position,
            },
            Self::UnclosedString { position } => Self::UnclosedString {
                position: *position,
            },
            Self::InvalidIdentifier {
                identifier,
                position,
            } => Self::InvalidIdentifier {
                identifier: identifier.clone(),
                position: *position,
            },
            Self::NomError { position, kind } => Self::NomError {
                position: *position,
                kind: *kind,
            },
            Self::LazyFormatted {
                format_fn,
                position,
            } => {
                // Convert lazy error to a syntax error when cloning to avoid function pointer issues
                Self::SyntaxError {
                    message: Cow::Owned(format_fn()),
                    position: *position,
                }
            }
        }
    }
}

impl PartialEq for ParseError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::SyntaxError {
                    position: p1,
                    message: m1,
                },
                Self::SyntaxError {
                    position: p2,
                    message: m2,
                },
            ) => p1 == p2 && m1 == m2,
            (
                Self::UnexpectedToken {
                    token: t1,
                    position: p1,
                },
                Self::UnexpectedToken {
                    token: t2,
                    position: p2,
                },
            ) => t1 == t2 && p1 == p2,
            (Self::UnexpectedEof, Self::UnexpectedEof) => true,
            (
                Self::ExpectedToken {
                    expected: e1,
                    position: p1,
                },
                Self::ExpectedToken {
                    expected: e2,
                    position: p2,
                },
            ) => e1 == e2 && p1 == p2,
            (
                Self::UnexpectedEndOfInput { position: p1 },
                Self::UnexpectedEndOfInput { position: p2 },
            ) => p1 == p2,
            (
                Self::InvalidLiteral {
                    literal_type: lt1,
                    value: v1,
                    position: p1,
                },
                Self::InvalidLiteral {
                    literal_type: lt2,
                    value: v2,
                    position: p2,
                },
            ) => lt1 == lt2 && v1 == v2 && p1 == p2,
            (
                Self::InvalidEscape {
                    sequence: s1,
                    position: p1,
                },
                Self::InvalidEscape {
                    sequence: s2,
                    position: p2,
                },
            ) => s1 == s2 && p1 == p2,
            (Self::UnclosedString { position: p1 }, Self::UnclosedString { position: p2 }) => {
                p1 == p2
            }
            (
                Self::InvalidIdentifier {
                    identifier: i1,
                    position: p1,
                },
                Self::InvalidIdentifier {
                    identifier: i2,
                    position: p2,
                },
            ) => i1 == i2 && p1 == p2,
            (
                Self::NomError {
                    position: p1,
                    kind: k1,
                },
                Self::NomError {
                    position: p2,
                    kind: k2,
                },
            ) => p1 == p2 && k1 == k2,
            (
                Self::LazyFormatted {
                    format_fn: f1,
                    position: p1,
                },
                Self::LazyFormatted {
                    format_fn: f2,
                    position: p2,
                },
            ) => {
                // Compare by formatting the messages (expensive but necessary for equality)
                p1 == p2 && f1() == f2()
            }
            _ => false,
        }
    }
}

impl ParseError {
    /// Convert to a diagnostic
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            ParseError::SyntaxError { position, message } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(message.to_string())
                    .with_location(0, *position, 0, position + 1)
                    .build()
            }
            ParseError::UnexpectedToken { token, position } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(format!("Unexpected token '{token}'"))
                    .with_location(0, *position, 0, position + token.len())
                    .build()
            }
            ParseError::UnexpectedEof => DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                .with_message("Unexpected end of input")
                .build(),
            ParseError::ExpectedToken { expected, position } => {
                DiagnosticBuilder::error(DiagnosticCode::ExpectedToken(expected.to_string()))
                    .with_message(format!("Expected {expected}"))
                    .with_location(0, *position, 0, *position)
                    .build()
            }
            ParseError::UnexpectedEndOfInput { position } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message("Unexpected end of input")
                    .with_location(0, *position, 0, *position)
                    .build()
            }
            ParseError::InvalidLiteral {
                literal_type,
                value,
                position,
            } => DiagnosticBuilder::error(DiagnosticCode::InvalidNumber)
                .with_message(format!("Invalid {literal_type} literal: {value}"))
                .with_location(0, *position, 0, position + value.len())
                .build(),
            ParseError::InvalidEscape { sequence, position } => {
                DiagnosticBuilder::error(DiagnosticCode::InvalidEscape)
                    .with_message(format!("Invalid escape sequence: {sequence}"))
                    .with_location(0, *position, 0, position + sequence.len())
                    .build()
            }
            ParseError::UnclosedString { position } => {
                DiagnosticBuilder::error(DiagnosticCode::UnclosedString)
                    .with_message("Unclosed string literal")
                    .with_location(0, *position, 0, *position)
                    .build()
            }
            ParseError::InvalidIdentifier {
                identifier,
                position,
            } => DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                .with_message(format!("Invalid identifier: {identifier}"))
                .with_location(0, *position, 0, position + identifier.len())
                .build(),
            ParseError::NomError { position, kind } => {
                DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                    .with_message(format!("Parse error: {kind:?}"))
                    .with_location(0, *position, 0, *position)
                    .build()
            }
            ParseError::LazyFormatted {
                format_fn,
                position,
            } => DiagnosticBuilder::error(DiagnosticCode::UnexpectedToken)
                .with_message(format_fn())
                .with_location(0, *position, 0, *position)
                .build(),
        }
    }

    /// Convenience constructors using pre-allocated messages for common error patterns
    ///
    /// Create an invalid literal error with pre-allocated literal type (zero-allocation)
    pub fn invalid_literal_prealloc(
        literal_type: &'static str,
        value: String,
        position: usize,
    ) -> Self {
        ParseError::InvalidLiteral {
            literal_type: Cow::Borrowed(literal_type),
            value: Cow::Owned(value),
            position,
        }
    }

    /// Create an expected token error with pre-allocated token type (zero-allocation)
    pub fn expected_token_prealloc(expected: &'static str, position: usize) -> Self {
        ParseError::ExpectedToken {
            expected: Cow::Borrowed(expected),
            position,
        }
    }

    /// Create a syntax error with pre-allocated message (zero-allocation)
    pub fn syntax_error_prealloc(message: &'static str, position: usize) -> Self {
        ParseError::SyntaxError {
            message: Cow::Borrowed(message),
            position,
        }
    }

    /// Create error variants with dynamic strings when needed
    pub fn syntax_error_dynamic(message: String, position: usize) -> Self {
        ParseError::SyntaxError {
            message: Cow::Owned(message),
            position,
        }
    }

    /// Create an unexpected token error with a dynamically allocated token string
    pub fn unexpected_token_dynamic(token: String, position: usize) -> Self {
        ParseError::UnexpectedToken {
            token: Cow::Owned(token),
            position,
        }
    }

    /// Create an expected token error with a dynamically allocated expected string
    pub fn expected_token_dynamic(expected: String, position: usize) -> Self {
        ParseError::ExpectedToken {
            expected: Cow::Owned(expected),
            position,
        }
    }

    /// Create common error patterns with zero-allocation static strings
    /// These methods avoid String allocation for the most common cases
    ///
    /// Create an error for when an identifier was expected but not found
    pub fn expected_identifier(position: usize) -> Self {
        Self::expected_token_prealloc(common_messages::EXPECTED_IDENTIFIER, position)
    }

    /// Create an error for when an expression was expected but not found
    pub fn expected_expression(position: usize) -> Self {
        Self::expected_token_prealloc(common_messages::EXPECTED_EXPRESSION, position)
    }

    /// Create an error for when a left parenthesis '(' was expected but not found
    pub fn expected_left_paren(position: usize) -> Self {
        Self::expected_token_prealloc(common_messages::EXPECTED_LEFT_PAREN, position)
    }

    /// Create an error for when a right parenthesis ')' was expected but not found
    pub fn expected_right_paren(position: usize) -> Self {
        Self::expected_token_prealloc(common_messages::EXPECTED_RIGHT_PAREN, position)
    }

    /// Create an error for an invalid integer literal
    pub fn invalid_integer_literal(value: String, position: usize) -> Self {
        Self::invalid_literal_prealloc(common_messages::INTEGER, value, position)
    }

    /// Create an error for an invalid decimal literal
    pub fn invalid_decimal_literal(value: String, position: usize) -> Self {
        Self::invalid_literal_prealloc(common_messages::DECIMAL, value, position)
    }

    /// Create an error for an invalid string literal
    pub fn invalid_string_literal(value: String, position: usize) -> Self {
        Self::invalid_literal_prealloc(common_messages::STRING, value, position)
    }

    /// Create an error for an unclosed string literal
    pub fn unclosed_string_literal(position: usize) -> Self {
        Self::syntax_error_prealloc(common_messages::UNCLOSED_STRING, position)
    }

    /// Create an error for an invalid number format
    pub fn invalid_number_format(position: usize) -> Self {
        Self::syntax_error_prealloc(common_messages::INVALID_NUMBER_FORMAT, position)
    }

    /// Lazy formatting constructors - defer expensive string operations until display time
    ///
    /// Create a lazy formatted error with a closure that generates the message on demand
    pub fn lazy_format<F>(position: usize, format_fn: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        ParseError::LazyFormatted {
            format_fn: Box::new(format_fn),
            position,
        }
    }

    /// Create a lazy "expected one of" error message
    pub fn lazy_expected_one_of(position: usize, tokens: Vec<&'static str>) -> Self {
        Self::lazy_format(position, move || {
            format!("Expected one of: {}", tokens.join(", "))
        })
    }

    /// Create a lazy context error with dynamic context information  
    pub fn lazy_context_error(position: usize, context: String, details: String) -> Self {
        Self::lazy_format(position, move || format!("Error in {context}: {details}"))
    }

    /// Create a lazy error for complex validation failures
    pub fn lazy_validation_error(
        position: usize,
        field: String,
        expected: String,
        actual: String,
    ) -> Self {
        Self::lazy_format(position, move || {
            format!("Validation failed for '{field}': expected '{expected}', got '{actual}'")
        })
    }

    /// Create a lazy error for missing required elements with context
    pub fn lazy_missing_required(position: usize, required_elements: Vec<String>) -> Self {
        Self::lazy_format(position, move || {
            format!(
                "Missing required elements: {}",
                required_elements.join(", ")
            )
        })
    }
}

/// Implement nom's ParseError trait
impl<I> NomParseError<I> for ParseError {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        ParseError::NomError {
            position: 0, // Will be updated with proper position tracking
            kind,
        }
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}
