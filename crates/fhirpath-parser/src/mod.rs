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

//! FHIRPath expression parser
//!
//! This crate provides a nom-based parser for FHIRPath expressions,
//! converting text expressions into an Abstract Syntax Tree (AST).

#![warn(missing_docs)]

pub mod ast_cache;
pub mod error;
pub mod error_recovery;
pub mod lexer;
pub mod pratt;
pub mod span;
pub mod tokenizer;

pub use ast_cache::{
    AstCache, AstCacheConfig, AstCacheStats, SharedAst, cache_ast, get_cached_ast,
    global_ast_cache, global_ast_cache_stats,
};
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
pub async fn parse(input: &str) -> ParseResult<fhirpath_ast::ExpressionNode> {
    parse_expression_pratt(input)
}

/// Parse with IDE-friendly error recovery and enhanced diagnostics
pub async fn parse_for_ide(input: &str) -> RecoveryResult {
    parse_with_recovery(input, RecoveryStrategy::Aggressive).await
}

/// Parse with custom recovery strategy
pub async fn parse_with_strategy(input: &str, strategy: RecoveryStrategy) -> RecoveryResult {
    parse_with_recovery(input, strategy).await
}
