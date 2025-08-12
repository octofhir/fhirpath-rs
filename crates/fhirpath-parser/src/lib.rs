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

//! Parser and tokenizer for FHIRPath expressions
//!
//! This crate provides parsing capabilities for FHIRPath expressions,
//! converting text into abstract syntax trees.

pub mod ast_cache;
pub mod error;
pub mod error_recovery;
pub mod lexer;
pub mod pratt;
pub mod span;
pub mod tokenizer;

// Re-export main types
pub use error::{ParseError, ParseResult};
pub use tokenizer::{Token, Tokenizer};
// pub use lexer::FhirPathParser; // May not exist
pub use ast_cache::{cache_ast, get_cached_ast};
pub use pratt::parse_expression_pratt;
pub use span::{Span, Spanned};

// Re-export from workspace crates for convenience
pub use fhirpath_ast::{ExpressionNode, LiteralValue};
pub use fhirpath_core::{FhirPathError, Result};
pub use fhirpath_diagnostics::{Diagnostic, DiagnosticBuilder};

/// Parse a FHIRPath expression from a string
pub fn parse_expression(input: &str) -> Result<ExpressionNode> {
    pratt::parse_expression_pratt(input)
        .map_err(|e| fhirpath_core::FhirPathError::parse_error(0, e.to_string()))
}
