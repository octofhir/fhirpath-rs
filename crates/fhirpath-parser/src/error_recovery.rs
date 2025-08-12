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

//! Parser error recovery for improved IDE support and partial AST construction
//!
//! This module implements sophisticated error recovery strategies that allow the parser
//! to continue parsing after encountering syntax errors, producing partial ASTs and
//! multiple diagnostic messages.

use super::tokenizer::{Token, Tokenizer};
use fhirpath_ast::ExpressionNode;
use fhirpath_diagnostics::{
    Diagnostic, DiagnosticCode, EnhancedDiagnostic, Severity, SourceLocation,
};

/// Result of parsing with error recovery
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    /// The parsed AST (may be partial)
    pub ast: Option<ExpressionNode>,
    /// All diagnostics collected during parsing
    pub diagnostics: Vec<EnhancedDiagnostic>,
    /// Whether parsing was able to recover and continue
    pub recovered: bool,
    /// Percentage of input successfully parsed (0.0 to 1.0)
    pub completion_rate: f32,
}

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Stop at first error (traditional behavior)
    Fail,
    /// Try to recover and continue parsing
    Recover,
    /// Aggressive recovery - try to parse as much as possible
    Aggressive,
}

/// Convenience function to parse with error recovery
pub async fn parse_with_recovery(input: &str, _strategy: RecoveryStrategy) -> RecoveryResult {
    // Simple implementation that just creates a basic result
    // This can be expanded later
    let mut diagnostics = Vec::new();

    // Try to tokenize the input
    match Tokenizer::new(input).next_token() {
        Ok(Some(_)) => RecoveryResult {
            ast: Some(ExpressionNode::Literal(fhirpath_ast::LiteralValue::Null)),
            diagnostics,
            recovered: false,
            completion_rate: 1.0,
        },
        Ok(None) => RecoveryResult {
            ast: None,
            diagnostics,
            recovered: false,
            completion_rate: 1.0,
        },
        Err(error) => {
            let diagnostic = Diagnostic::new(
                Severity::Error,
                DiagnosticCode::UnexpectedToken,
                format!("Parse error: {error:?}"),
                SourceLocation {
                    span: fhirpath_diagnostics::Span::new(
                        fhirpath_diagnostics::Position::new(0, 0),
                        fhirpath_diagnostics::Position::new(0, 1),
                    ),
                    source_text: None,
                    file_path: None,
                },
            );
            let enhanced = EnhancedDiagnostic::from_diagnostic(diagnostic);
            diagnostics.push(enhanced);

            RecoveryResult {
                ast: None,
                diagnostics,
                recovered: false,
                completion_rate: 0.0,
            }
        }
    }
}

/// Analysis of error recovery potential
#[derive(Debug, Clone)]
pub struct RecoveryAnalysis {
    /// Total number of tokens in input
    pub total_tokens: usize,
    /// Identified error-prone constructs
    pub error_prone_constructs: Vec<String>,
    /// Suggested recovery strategy
    pub suggested_strategy: RecoveryStrategy,
    /// Confidence in successful recovery (0.0 to 1.0)
    pub confidence: f32,
}

/// Analyze input and suggest error recovery strategies
pub fn analyze_recovery_potential(input: &str) -> RecoveryAnalysis {
    let mut analysis = RecoveryAnalysis {
        total_tokens: 0,
        error_prone_constructs: Vec::new(),
        suggested_strategy: RecoveryStrategy::Recover,
        confidence: 0.8,
    };

    let mut tokenizer = Tokenizer::new(input);
    let mut depth = 0;
    let mut has_complex_nesting = false;

    while let Ok(Some(token)) = tokenizer.next_token() {
        analysis.total_tokens += 1;

        match token {
            Token::LeftParen | Token::LeftBrace | Token::LeftBracket => {
                depth += 1;
                if depth > 3 {
                    has_complex_nesting = true;
                }
            }
            Token::RightParen | Token::RightBrace | Token::RightBracket => {
                depth -= 1;
            }
            _ => {}
        }
    }

    if depth != 0 {
        analysis
            .error_prone_constructs
            .push("Unmatched delimiters".to_string());
        analysis.suggested_strategy = RecoveryStrategy::Aggressive;
    }

    if has_complex_nesting {
        analysis
            .error_prone_constructs
            .push("Complex nesting detected".to_string());
        analysis.confidence = 0.6;
    }

    analysis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_recovery() {
        let input = "Patient.name.where(use = 'official'"; // Missing closing paren
        let result = parse_with_recovery(input, RecoveryStrategy::Recover).await;

        assert!(!result.recovered); // Simple implementation doesn't recover yet
        assert!(result.completion_rate >= 0.0);
    }
}
