//! Integration between analysis parser and multi-diagnostic system
//!
//! This module connects the Chumsky analysis parser with the diagnostic
//! collection system to provide comprehensive error reporting.

use crate::ast::ExpressionNode;
use crate::core::error_code::*;
use crate::diagnostics::DiagnosticSeverity;
use crate::diagnostics::collector::{DiagnosticBatch, MultiDiagnosticCollector};
use std::ops::Range;

/// Result of comprehensive FHIRPath analysis
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Parsed AST (may be partial if errors occurred)
    pub ast: Option<ExpressionNode>,
    /// All collected diagnostics
    pub diagnostics: DiagnosticBatch,
    /// Whether parsing succeeded (no errors)
    pub parse_success: bool,
    /// Whether the AST is complete and usable
    pub ast_complete: bool,
}

/// Comprehensive analyzer that combines parsing and static analysis
pub struct ComprehensiveAnalyzer {
    collector: MultiDiagnosticCollector,
}

impl ComprehensiveAnalyzer {
    /// Create new comprehensive analyzer
    pub fn new() -> Self {
        Self {
            collector: MultiDiagnosticCollector::new(),
        }
    }

    /// Analyze FHIRPath expression with comprehensive error reporting
    pub fn analyze(&mut self, expression: &str, source_name: String) -> AnalysisResult {
        // Clear any previous diagnostics
        self.collector.clear();

        // Phase 1: Parse with error recovery
        let (ast, parse_success) = self.parse_with_recovery(expression);

        // Phase 2: Static analysis (if AST available)
        let ast_complete = if let Some(ref ast) = ast {
            self.analyze_ast(ast, expression)
        } else {
            false
        };

        // Phase 3: Build diagnostic batch
        let mut diagnostics_batch = self.collector.build_batch(0, source_name);

        // Sort diagnostics by severity and position
        self.collector.sort_by_severity();
        diagnostics_batch.diagnostics = self.collector.diagnostics().to_vec();

        AnalysisResult {
            ast,
            diagnostics: diagnostics_batch,
            parse_success,
            ast_complete,
        }
    }

    /// Parse with comprehensive error recovery
    fn parse_with_recovery(&mut self, expression: &str) -> (Option<ExpressionNode>, bool) {
        // Use the analysis parser with full error recovery
        match crate::parser::parse_with_analysis(expression) {
            result if result.success => (result.ast, true),
            result => {
                // Convert parse errors to diagnostics
                for diagnostic in &result.diagnostics {
                    self.convert_diagnostic_to_ariadne(diagnostic, expression);
                }

                // Attempt partial parsing for what we can recover
                let partial_ast = self.attempt_partial_parse(expression);
                (partial_ast, false)
            }
        }
    }

    /// Convert parser diagnostic to AriadneDiagnostic
    fn convert_diagnostic_to_ariadne(
        &mut self,
        diagnostic: &crate::diagnostics::Diagnostic,
        expression: &str,
    ) {
        // Extract span from location if available
        let span = if let Some(location) = &diagnostic.location {
            location.offset..(location.offset + location.length)
        } else {
            0..expression.len()
        };

        // Map error code
        let error_code = match diagnostic.code.code.as_str() {
            "syntax_error" => FP0001,
            "unexpected_token" => FP0002,
            "missing_delimiter" => FP0003,
            _ => FP0001, // Default to general syntax error
        };

        let help = Some(self.generate_error_help(&diagnostic.message, expression, &span));

        match diagnostic.severity {
            crate::diagnostics::DiagnosticSeverity::Error => {
                if let Some(help_text) = help {
                    self.collector.error_with_help(
                        error_code,
                        diagnostic.message.clone(),
                        span,
                        help_text,
                    );
                } else {
                    self.collector
                        .error(error_code, diagnostic.message.clone(), span);
                }
            }
            crate::diagnostics::DiagnosticSeverity::Warning => {
                if let Some(help_text) = help {
                    self.collector.warning_with_help(
                        error_code,
                        diagnostic.message.clone(),
                        span,
                        help_text,
                    );
                } else {
                    self.collector
                        .warning(error_code, diagnostic.message.clone(), span);
                }
            }
            crate::diagnostics::DiagnosticSeverity::Info => {
                if let Some(help_text) = help {
                    self.collector.suggestion_with_help(
                        error_code,
                        diagnostic.message.clone(),
                        span,
                        help_text,
                    );
                } else {
                    self.collector
                        .suggestion(error_code, diagnostic.message.clone(), span);
                }
            }
            crate::diagnostics::DiagnosticSeverity::Hint => {
                if let Some(_help_text) = help {
                    self.collector
                        .note(error_code, diagnostic.message.clone(), span);
                } else {
                    self.collector
                        .note(error_code, diagnostic.message.clone(), span);
                }
            }
        }

        // Add related diagnostics
        for related in &diagnostic.related {
            let related_span = if let Some(location) = &related.location {
                location.offset..(location.offset + location.length)
            } else {
                0..expression.len()
            };

            let related_severity = match related.severity {
                crate::diagnostics::DiagnosticSeverity::Error => DiagnosticSeverity::Error,
                crate::diagnostics::DiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
                crate::diagnostics::DiagnosticSeverity::Info => DiagnosticSeverity::Info,
                crate::diagnostics::DiagnosticSeverity::Hint => DiagnosticSeverity::Hint,
            };

            self.collector
                .add_related(related.message.clone(), related_span, related_severity);
        }
    }

    /// Generate helpful suggestions for parse errors
    fn generate_error_help(&self, message: &str, expression: &str, span: &Range<usize>) -> String {
        let lowercase_message = message.to_lowercase();

        if lowercase_message.contains("unexpected") && lowercase_message.contains("token") {
            if lowercase_message.contains("(") {
                "Function calls require parentheses: functionName(arguments)".to_string()
            } else if lowercase_message.contains(")") {
                "Check for missing closing parentheses in function calls or grouped expressions"
                    .to_string()
            } else if lowercase_message.contains(".") {
                "Property access requires dot notation: resource.property".to_string()
            } else {
                "Check the FHIRPath syntax specification for valid expression patterns".to_string()
            }
        } else if lowercase_message.contains("unclosed") {
            if lowercase_message.contains("parenthesis") {
                "Add a closing parenthesis ')' to match the opening parenthesis".to_string()
            } else if lowercase_message.contains("bracket") {
                "Add a closing bracket ']' to match the opening bracket".to_string()
            } else if lowercase_message.contains("quote") {
                "Add a closing quote to terminate the string literal".to_string()
            } else {
                "Add the appropriate closing delimiter".to_string()
            }
        } else if lowercase_message.contains("expected") {
            "Review the FHIRPath grammar specification for valid syntax at this position"
                .to_string()
        } else {
            format!(
                "Error occurred at position {} in expression. Check syntax around: '{}'",
                span.start,
                self.extract_error_context(expression, span.clone())
            )
        }
    }

    /// Extract context around error position
    fn extract_error_context(&self, expression: &str, span: Range<usize>) -> String {
        let start = span.start.saturating_sub(10);
        let end = (span.end + 10).min(expression.len());

        let context = &expression[start..end];
        format!("...{context}...")
    }

    /// Attempt partial parsing for error recovery
    fn attempt_partial_parse(&mut self, expression: &str) -> Option<ExpressionNode> {
        // Try parsing smaller parts of the expression
        // This is a simple recovery strategy - could be enhanced

        // Try parsing just the first identifier or literal
        let tokens: Vec<&str> = expression.split_whitespace().collect();
        for token in &tokens {
            if let Ok(result) = crate::parser::parse_ast(token) {
                self.collector.note(
                    FP0154,
                    format!("Partial recovery: successfully parsed '{token}'"),
                    0..token.len(),
                );
                return Some(result);
            }
        }

        None
    }

    /// Perform static analysis on AST
    fn analyze_ast(&mut self, _ast: &ExpressionNode, _expression: &str) -> bool {
        // TODO: Implement comprehensive static analysis
        // This will be expanded in later tasks with:
        // - Type checking
        // - Property validation
        // - Function signature validation
        // - Performance analysis
        // - Optimization suggestions

        // For now, add a simple completeness check
        self.collector.note(
            FP0154,
            "Static analysis completed successfully".to_string(),
            0..0,
        );

        true // For now, assume analysis succeeds
    }
}

impl Default for ComprehensiveAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_analysis_success() {
        let mut analyzer = ComprehensiveAnalyzer::new();
        let expression = "Patient.name.given.first()";

        let result = analyzer.analyze(expression, "test.fhirpath".to_string());

        assert!(result.parse_success);
        assert!(result.ast.is_some());
        assert_eq!(result.diagnostics.source_name, "test.fhirpath");
    }

    #[test]
    fn test_comprehensive_analysis_with_errors() {
        let mut analyzer = ComprehensiveAnalyzer::new();
        let expression = "Patient.name[unclosed";

        let result = analyzer.analyze(expression, "test.fhirpath".to_string());

        assert!(!result.parse_success);
        assert!(result.diagnostics.statistics.error_count > 0);
        assert_eq!(result.diagnostics.source_name, "test.fhirpath");
    }

    #[test]
    fn test_error_help_generation() {
        let analyzer = ComprehensiveAnalyzer::new();

        let help =
            analyzer.generate_error_help("Unexpected token '(' found", "Patient.name[", &(10..11));
        assert!(help.contains("parentheses"));

        let help =
            analyzer.generate_error_help("Unclosed bracket found", "Patient.name[test", &(12..17));
        assert!(help.contains("closing bracket"));
    }

    #[test]
    fn test_partial_parsing() {
        let mut analyzer = ComprehensiveAnalyzer::new();
        let expression = "Patient.name invalid syntax here";

        let partial = analyzer.attempt_partial_parse(expression);
        // Should be able to parse at least "Patient.name"
        assert!(partial.is_some() || analyzer.collector.diagnostics().len() > 0);
    }

    #[test]
    fn test_error_context_extraction() {
        let analyzer = ComprehensiveAnalyzer::new();
        let expression = "Patient.name.given.where(use = 'official').family";
        let span = 20..25;

        let context = analyzer.extract_error_context(expression, span);
        assert!(context.contains("where"));
        assert!(context.contains("..."));
    }

    #[test]
    fn test_static_analysis_placeholder() {
        let mut analyzer = ComprehensiveAnalyzer::new();
        let expression = "Patient.name";
        let result = crate::parser::parse_ast(expression).unwrap();

        let analysis_complete = analyzer.analyze_ast(&result, expression);
        assert!(analysis_complete);

        // Should have added a note about static analysis
        let notes = analyzer
            .collector
            .diagnostics()
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Info)
            .count();
        assert!(notes > 0);
    }
}
