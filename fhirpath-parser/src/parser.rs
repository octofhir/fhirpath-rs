//! FHIRPath parser module using high-performance Pratt parser
//!
//! This module provides the primary parsing interface for FHIRPath expressions.
//! The parser uses the Pratt parsing algorithm (precedence climbing) to efficiently
//! parse complex expressions with proper operator precedence and associativity.
//!
//! ## Features
//!
//! - **High Performance**: Zero-allocation parsing with aggressive inlining
//! - **Correct Precedence**: Follows official FHIRPath specification exactly  
//! - **Comprehensive Support**: Handles all FHIRPath operators and constructs
//! - **Better Error Messages**: Enhanced diagnostics with precedence context
//!
//! ## Usage
//!
//! ```rust
//! use fhirpath_parser::parse_expression;
//!
//! let result = parse_expression("Patient.name.where(use = 'official').given.first()");
//! ```
//!
//! The parser delegates to the optimized Pratt parser implementation in `crate::pratt`.

use crate::error::ParseResult;
use fhirpath_ast::ExpressionNode;

/// Parse FHIRPath expression using the high-performance Pratt parser
pub fn parse_expression(input: &str) -> ParseResult<ExpressionNode> {
    crate::pratt::parse_expression_pratt(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let result = parse_expression("Patient").unwrap();
        assert!(matches!(result, ExpressionNode::Identifier { .. }));
    }

    #[test]
    fn test_path_expression() {
        let result = parse_expression("Patient.name").unwrap();
        assert!(matches!(result, ExpressionNode::Path { .. }));
    }

    #[test]
    fn test_complex_path() {
        let result = parse_expression("Patient.name.given").unwrap();
        assert!(matches!(result, ExpressionNode::Path { .. }));
    }

    #[test]
    fn test_function_call() {
        // For now, this will parse as path until we implement function calls
        let result = parse_expression("Patient.name.where");
        assert!(result.is_ok());
    }
}
