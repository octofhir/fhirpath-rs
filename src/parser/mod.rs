//! FHIRPath expression parser
//!
//! This crate provides a nom-based parser for FHIRPath expressions,
//! converting text expressions into an Abstract Syntax Tree (AST).

#![warn(missing_docs)]

pub mod error;
pub mod error_recovery;
pub mod lexer;
pub mod pratt;
pub mod span;
pub mod tokenizer;

pub use error::{ParseError, ParseResult};
pub use error_recovery::{
    RecoveryAnalysis, RecoveryResult, RecoveryStrategy, analyze_recovery_potential,
    parse_with_recovery,
};
pub use pratt::parse_expression_pratt;
pub use span::{Span, Spanned};

// Re-export parser function for compatibility
pub use pratt::parse_expression_pratt as parse_expression;

/// Parse an FHIRPath expression string into an AST using the optimized Pratt parser
pub fn parse(input: &str) -> ParseResult<crate::ast::ExpressionNode> {
    parse_expression_pratt(input)
}

/// Parse with IDE-friendly error recovery and enhanced diagnostics
pub fn parse_for_ide(input: &str) -> RecoveryResult {
    parse_with_recovery(input, RecoveryStrategy::Aggressive)
}

/// Parse with custom recovery strategy
pub fn parse_with_strategy(input: &str, strategy: RecoveryStrategy) -> RecoveryResult {
    parse_with_recovery(input, strategy)
}
