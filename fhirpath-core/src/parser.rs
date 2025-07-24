//! FHIRPath expression parser bridge
//!
//! This module bridges to the dedicated parser crate.

use crate::error::{FhirPathError, Result};
use fhirpath_ast::ExpressionNode;
use fhirpath_parser::ParseError;

/// Parse a FHIRPath expression string into an AST
pub fn parse_expression(expression: &str) -> Result<ExpressionNode> {
    fhirpath_parser::parse(expression)
        .map_err(|e: ParseError| FhirPathError::parse_error(0, e.to_string()))
}