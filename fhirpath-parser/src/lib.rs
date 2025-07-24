//! FHIRPath expression parser
//!
//! This crate provides a nom-based parser for FHIRPath expressions,
//! converting text expressions into an Abstract Syntax Tree (AST).

#![warn(missing_docs)]

pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod tokenizer;

pub use error::{ParseError, ParseResult};
pub use parser::parse_expression;
pub use span::{Span, Spanned};

/// Parse an FHIRPath expression string into an AST
pub fn parse(input: &str) -> ParseResult<fhirpath_ast::ExpressionNode> {
    parse_expression(input)
}
