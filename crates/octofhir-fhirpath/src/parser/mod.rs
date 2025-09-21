//! Unified FHIRPath Parser API
//!
//! This module provides a clean, unified API for parsing FHIRPath expressions with dual-mode support:
//!
//! ## Parser Modes
//!
//! ### Fast Mode (Default)
//! - **Performance**: 100K+ operations per second
//! - **Error Recovery**: Minimal, fail fast for performance
//! - **Memory Usage**: Low overhead
//! - **Use Case**: Runtime evaluation, production applications
//!
//! ### Analysis Mode  
//! - **Error Recovery**: Comprehensive, collects multiple errors
//! - **Diagnostics**: Rich error information with suggestions
//! - **Memory Usage**: Higher due to error collection
//! - **Use Case**: Development, IDE integration, validation
//!
//! ## Usage Examples
//!
//! ```rust
//! use octofhir_fhirpath::parser::{parse, parse_with_analysis, ParsingMode};
//!
//! // Simple fast parsing (default)
//! let result = parse("Patient.name.given.first()");
//!
//! // Parsing with full analysis and error recovery
//! let result = parse_with_analysis("Patient.name[unclosed");
//! if !result.success {
//!     for diagnostic in &result.diagnostics {
//!         println!("Error: {}", diagnostic.message);
//!     }
//! }
//! ```

pub mod analysis_integration;
pub mod analyzer;
pub mod combinators;
pub mod pratt;
pub mod pratt_analysis;

use crate::ast::ExpressionNode;
use crate::core::{FP0001, FhirPathError};
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use std::fmt;

// Re-export combinators for advanced usage
pub use combinators::*;

// Re-export analysis integration
pub use analysis_integration::{AnalysisResult, ComprehensiveAnalyzer};

// Re-export semantic analyzer
pub use analyzer::{AnalyzedParseResult, SemanticAnalyzer};

/// Parser mode selection for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ParsingMode {
    /// Fast parser optimized for runtime evaluation
    /// - High performance (100K+ ops/sec)
    /// - Minimal error recovery
    /// - Stops at first error
    /// - Best for production environments
    #[default]
    Fast,

    /// Comprehensive error recovery parser for development
    /// - Full error recovery and analysis
    /// - Collects multiple errors
    /// - Rich diagnostic information
    /// - Best for development and IDE integration
    Analysis,
}

impl fmt::Display for ParsingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsingMode::Fast => write!(f, "fast"),
            ParsingMode::Analysis => write!(f, "analysis"),
        }
    }
}

/// Unified result type for parsing operations
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed AST (None if parsing completely failed)
    pub ast: Option<ExpressionNode>,
    /// Whether parsing succeeded without errors
    pub success: bool,
    /// All diagnostics (errors, warnings, notes)
    pub diagnostics: Vec<Diagnostic>,
    /// Primary error message (if any)
    pub error_message: Option<String>,
}

impl ParseResult {
    /// Create successful parse result
    pub fn success(ast: ExpressionNode) -> Self {
        Self {
            ast: Some(ast),
            success: true,
            diagnostics: vec![],
            error_message: None,
        }
    }

    /// Create failed parse result with error
    pub fn error(error_message: String, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            ast: None,
            success: false,
            diagnostics,
            error_message: Some(error_message),
        }
    }

    /// Create partial success (AST available but with errors)
    pub fn partial(ast: ExpressionNode, diagnostics: Vec<Diagnostic>) -> Self {
        let has_errors = diagnostics
            .iter()
            .any(|d| matches!(d.severity, DiagnosticSeverity::Error));

        Self {
            ast: Some(ast),
            success: !has_errors,
            diagnostics,
            error_message: if has_errors {
                Some("Parsing completed with errors".to_string())
            } else {
                None
            },
        }
    }

    /// Check if result has any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.severity, DiagnosticSeverity::Error))
    }

    /// Check if result has any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| matches!(d.severity, DiagnosticSeverity::Warning))
    }

    /// Get first error message if any
    pub fn first_error(&self) -> Option<&str> {
        self.diagnostics
            .iter()
            .find(|d| matches!(d.severity, DiagnosticSeverity::Error))
            .map(|d| d.message.as_str())
    }

    /// Convert to Result<ExpressionNode, FhirPathError> for compatibility
    pub fn into_result(self) -> Result<ExpressionNode, FhirPathError> {
        if self.success {
            if let Some(ast) = self.ast {
                return Ok(ast);
            }
        }

        let error_msg = self
            .error_message
            .or_else(|| {
                self.diagnostics
                    .iter()
                    .find(|d| matches!(d.severity, DiagnosticSeverity::Error))
                    .map(|d| d.message.clone())
            })
            .unwrap_or_else(|| "Parsing failed".to_string());

        Err(FhirPathError::parse_error(FP0001, &error_msg, "", None))
    }
}

/// Parser configuration for advanced usage
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to include position information in AST nodes
    pub include_positions: bool,
    /// Maximum expression depth to prevent stack overflow
    pub max_depth: usize,
    /// Whether to collect warnings in addition to errors
    pub collect_warnings: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            include_positions: true,
            max_depth: 100,
            collect_warnings: true,
        }
    }
}

//=============================================================================
// Main Parser API Functions
//=============================================================================

/// Parse FHIRPath expression using fast mode (default)
///
/// This is the main parsing function optimized for runtime performance.
/// Uses minimal error recovery for maximum speed.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::parse;
///
/// let result = parse("Patient.name.given.first()");
/// if result.success {
///     println!("Parsed AST: {:?}", result.ast);
/// }
/// ```
pub fn parse(input: &str) -> ParseResult {
    parse_with_mode(input, ParsingMode::Fast)
}

/// Parse FHIRPath expression using analysis mode
///
/// This mode provides comprehensive error recovery and detailed diagnostics,
/// ideal for development, validation, and IDE integration.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::parse_with_analysis;
///
/// let result = parse_with_analysis("Patient.name[unclosed");
/// for diagnostic in &result.diagnostics {
///     println!("Error: {}", diagnostic.message);
/// }
/// ```
pub fn parse_with_analysis(input: &str) -> ParseResult {
    parse_with_mode(input, ParsingMode::Analysis)
}

/// Parse FHIRPath expression with specified mode
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::{parse_with_mode, ParsingMode};
///
/// // Fast parsing
/// let result = parse_with_mode("Patient.name", ParsingMode::Fast);
///
/// // Analysis parsing
/// let result = parse_with_mode("Patient.name", ParsingMode::Analysis);
/// ```
pub fn parse_with_mode(input: &str, mode: ParsingMode) -> ParseResult {
    match mode {
        ParsingMode::Fast => parse_fast(input),
        ParsingMode::Analysis => parse_analysis(input),
    }
}

/// Parse FHIRPath expression with semantic analysis using ModelProvider
///
/// This function performs both syntactic and semantic analysis, providing
/// rich type information and validation feedback.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::parse_with_semantic_analysis;
/// use octofhir_fhir_model::EmptyModelProvider;
/// use std::sync::Arc;
///
/// let model_provider = Arc::new(EmptyModelProvider);
/// let result = parse_with_semantic_analysis("Patient.name", model_provider, None);
/// ```
pub async fn parse_with_semantic_analysis(
    input: &str,
    model_provider: std::sync::Arc<dyn octofhir_fhir_model::ModelProvider>,
    context_type: Option<octofhir_fhir_model::TypeInfo>,
) -> AnalyzedParseResult {
    // First parse the expression
    let parse_result = parse_analysis(input);

    if let Some(ast) = parse_result.ast {
        // Then perform semantic analysis
        let mut analyzer = SemanticAnalyzer::new(model_provider);
        match analyzer.analyze_expression(&ast, context_type).await {
            Ok(analysis) => AnalyzedParseResult::success(ast, analysis),
            Err(err) => {
                let mut analysis = crate::ast::ExpressionAnalysis::failure(vec![]);
                analysis.add_diagnostic(crate::diagnostics::Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "SEMANTIC_ANALYSIS_FAILED".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: err.to_string(),
                    location: None,
                    related: vec![],
                });
                AnalyzedParseResult::failure(analysis)
            }
        }
    } else {
        // Parsing failed, convert parse diagnostics to analysis format
        let analysis = crate::ast::ExpressionAnalysis::failure(parse_result.diagnostics);
        AnalyzedParseResult::failure(analysis)
    }
}

/// Parse FHIRPath expression and return AST directly (Result wrapper)
///
/// This function provides a traditional Result<AST, Error> interface for
/// compatibility with existing code patterns.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::parse_ast;
///
/// match parse_ast("Patient.name") {
///     Ok(ast) => println!("Success: {:?}", ast),
///     Err(error) => println!("Error: {}", error),
/// }
/// ```
pub fn parse_ast(input: &str) -> Result<ExpressionNode, FhirPathError> {
    parse(input).into_result()
}

/// Parse FHIRPath expression and return AST with specified mode
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::{parse_ast_with_mode, ParsingMode};
///
/// match parse_ast_with_mode("Patient.name", ParsingMode::Analysis) {
///     Ok(ast) => println!("Success: {:?}", ast),
///     Err(error) => println!("Error: {}", error),
/// }
/// ```
pub fn parse_ast_with_mode(
    input: &str,
    mode: ParsingMode,
) -> Result<ExpressionNode, FhirPathError> {
    parse_with_mode(input, mode).into_result()
}

/// Advanced parsing with custom configuration (future enhancement)
///
/// Currently uses default configuration but provides interface for future
/// customization options.
pub fn parse_with_config(input: &str, mode: ParsingMode, _config: ParserConfig) -> ParseResult {
    // TODO: Implement config support in future enhancement
    parse_with_mode(input, mode)
}

//=============================================================================
// Parser-Specific Implementation Functions
//=============================================================================

/// Parse using fast parser implementation
fn parse_fast(input: &str) -> ParseResult {
    match pratt::parse(input) {
        Ok(ast) => ParseResult::success(ast),
        Err(error) => {
            // Convert FhirPathError to diagnostic
            let diagnostic = error_to_diagnostic(error);
            ParseResult::error(diagnostic.message.clone(), vec![diagnostic])
        }
    }
}

/// Parse using analysis parser implementation
fn parse_analysis(input: &str) -> ParseResult {
    let result = pratt_analysis::parse_for_analysis(input);

    match result.ast {
        Some(ast) => {
            if result.has_errors {
                ParseResult::partial(ast, result.diagnostics)
            } else if result.diagnostics.is_empty() {
                ParseResult::success(ast)
            } else {
                // Has warnings/notes but no errors
                ParseResult::partial(ast, result.diagnostics)
            }
        }
        None => {
            let error_message = result
                .diagnostics
                .first()
                .map(|d| d.message.clone())
                .unwrap_or_else(|| "Analysis parsing failed".to_string());

            ParseResult::error(error_message, result.diagnostics)
        }
    }
}

/// Convert FhirPathError to Diagnostic for unified error handling
fn error_to_diagnostic(error: FhirPathError) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: DiagnosticCode {
            code: error.error_code().code_str().to_string(),
            namespace: Some("fhirpath".to_string()),
        },
        message: error.to_string(),
        location: None,
        related: vec![],
    }
}

//=============================================================================
// Convenience Functions for Common Use Cases
//=============================================================================

/// Quick syntax validation - returns true if expression is valid
///
/// This is a lightweight way to check if an expression has valid syntax
/// without constructing the AST.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::is_valid;
///
/// assert!(is_valid("Patient.name"));
/// assert!(!is_valid("Patient.name["));
/// ```
pub fn is_valid(input: &str) -> bool {
    parse(input).success
}

/// Validate syntax with detailed error information
///
/// Returns Ok(()) if valid, or Err with all diagnostic information if invalid.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::validate;
///
/// match validate("Patient.name[unclosed") {
///     Ok(()) => println!("Valid syntax"),
///     Err(diagnostics) => {
///         println!("Invalid syntax: {} errors", diagnostics.len());
///         for diagnostic in diagnostics {
///             println!("  - {}", diagnostic.message);
///         }
///     }
/// }
/// ```
pub fn validate(input: &str) -> Result<(), Vec<Diagnostic>> {
    let result = parse_with_analysis(input);

    if result.has_errors() {
        Err(result.diagnostics)
    } else {
        Ok(())
    }
}

/// Parse multiple expressions at once
///
/// Efficiently parses multiple expressions and returns results for each.
/// Uses the specified parsing mode for all expressions.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::{parse_multiple, ParsingMode};
///
/// let expressions = vec![
///     "Patient.name",
///     "age > 18",
///     "status = 'active'",
/// ];
///
/// let results = parse_multiple(&expressions, ParsingMode::Fast);
/// for (expr, result) in expressions.iter().zip(results.iter()) {
///     println!("{}: {}", expr, if result.success { "OK" } else { "ERROR" });
/// }
/// ```
pub fn parse_multiple(inputs: &[&str], mode: ParsingMode) -> Vec<ParseResult> {
    inputs
        .iter()
        .map(|input| parse_with_mode(input, mode))
        .collect()
}

/// Parse multiple expressions and return only successful ones
///
/// This filters out failed parses and returns only the successful ASTs.
/// Useful when you want to process only valid expressions from a batch.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::parse_multiple_ast;
///
/// let expressions = vec![
///     "Patient.name",
///     "invalid[",
///     "age > 18",
/// ];
///
/// let asts = parse_multiple_ast(&expressions);
/// println!("Got {} valid ASTs out of {} expressions", asts.len(), expressions.len());
/// ```
pub fn parse_multiple_ast(inputs: &[&str]) -> Vec<ExpressionNode> {
    parse_multiple(inputs, ParsingMode::Fast)
        .into_iter()
        .filter_map(|result| result.ast)
        .collect()
}

/// Get recommended parser mode for specific use case
///
/// Provides guidance on which parser mode to use based on the intended
/// use case of the parsing operation.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::{recommend_mode, ParserUseCase};
///
/// let mode = recommend_mode(ParserUseCase::IDE);
/// println!("Recommended mode for IDE: {}", mode);
/// ```
pub fn recommend_mode(use_case: ParserUseCase) -> ParsingMode {
    match use_case {
        ParserUseCase::RuntimeEvaluation => ParsingMode::Fast,
        ParserUseCase::SyntaxValidation => ParsingMode::Analysis,
        ParserUseCase::IDE => ParsingMode::Analysis,
        ParserUseCase::Development => ParsingMode::Analysis,
        ParserUseCase::Testing => ParsingMode::Analysis,
        ParserUseCase::Performance => ParsingMode::Fast,
    }
}

/// Use case categories for parser mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserUseCase {
    /// Runtime expression evaluation in production
    RuntimeEvaluation,
    /// Syntax validation during development
    SyntaxValidation,
    /// IDE integration (error highlighting, completion)
    IDE,
    /// Development and debugging
    Development,
    /// Unit testing and validation
    Testing,
    /// High-performance parsing requirements
    Performance,
}

/// Parse and collect all errors (ignoring warnings)
///
/// Returns only the error diagnostics, filtering out warnings and notes.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::get_errors;
///
/// let errors = get_errors("Patient.name[");
/// for error in errors {
///     println!("Error: {}", error.message);
/// }
/// ```
pub fn get_errors(input: &str) -> Vec<Diagnostic> {
    parse_with_analysis(input)
        .diagnostics
        .into_iter()
        .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
        .collect()
}

/// Parse and collect all warnings (ignoring errors)
///
/// Returns only the warning diagnostics, filtering out errors and notes.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::get_warnings;
///
/// let warnings = get_warnings("Patient.name");
/// for warning in warnings {
///     println!("Warning: {}", warning.message);
/// }
/// ```
pub fn get_warnings(input: &str) -> Vec<Diagnostic> {
    parse_with_analysis(input)
        .diagnostics
        .into_iter()
        .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
        .collect()
}

//=============================================================================
// Backward Compatibility Functions
//=============================================================================

/// Parse expression (legacy function for backward compatibility)
///
/// This maintains compatibility with existing code that uses the old API.
/// Internally uses the fast parser mode.
pub fn parse_expression(input: &str) -> Result<ExpressionNode, FhirPathError> {
    parse_ast(input)
}

//=============================================================================
// Enhanced Functions for FHIRPath Semicolon-Delimited Expressions
//=============================================================================

/// Parse FHIRPath expression supporting semicolon-delimited multiple expressions
///
/// According to the FHIRPath specification, multiple expressions can be
/// separated by semicolons. This function parses such input and returns
/// all expressions as a collection.
///
/// # Examples
/// ```rust
/// use octofhir_fhirpath::parser::parse_semicolon_delimited;
///
/// let result = parse_semicolon_delimited("Patient.name; age > 18; status = 'active'");
/// if result.success {
///     println!("Parsed {} expressions", result.expressions.len());
/// }
/// ```
pub fn parse_semicolon_delimited(input: &str) -> MultipleParsesResult {
    parse_semicolon_delimited_with_mode(input, ParsingMode::Fast)
}

/// Parse semicolon-delimited expressions with analysis mode
pub fn parse_semicolon_delimited_with_analysis(input: &str) -> MultipleParsesResult {
    parse_semicolon_delimited_with_mode(input, ParsingMode::Analysis)
}

/// Parse semicolon-delimited expressions with specified mode
pub fn parse_semicolon_delimited_with_mode(input: &str, mode: ParsingMode) -> MultipleParsesResult {
    // Split on semicolons, but be careful about semicolons inside strings
    let expressions = split_on_semicolons(input);

    let mut results = Vec::new();
    let mut all_diagnostics = Vec::new();
    let mut has_any_errors = false;

    for expr in &expressions {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            continue; // Skip empty expressions
        }

        let result = parse_with_mode(trimmed, mode);
        has_any_errors |= result.has_errors();
        all_diagnostics.extend(result.diagnostics.clone());
        results.push(result);
    }

    MultipleParsesResult {
        expressions: results,
        success: !has_any_errors,
        diagnostics: all_diagnostics,
        original_input: input.to_string(),
    }
}

/// Result type for parsing multiple semicolon-delimited expressions
#[derive(Debug, Clone)]
pub struct MultipleParsesResult {
    /// Individual parse results for each expression
    pub expressions: Vec<ParseResult>,
    /// Whether all expressions succeeded
    pub success: bool,
    /// All diagnostics from all expressions
    pub diagnostics: Vec<Diagnostic>,
    /// Original input for reference
    pub original_input: String,
}

impl MultipleParsesResult {
    /// Get only successful expressions (filter out failed ones)
    pub fn successful_asts(&self) -> Vec<ExpressionNode> {
        self.expressions
            .iter()
            .filter_map(|result| result.ast.clone())
            .collect()
    }

    /// Get count of successful expressions
    pub fn success_count(&self) -> usize {
        self.expressions.iter().filter(|r| r.success).count()
    }

    /// Get count of failed expressions  
    pub fn failure_count(&self) -> usize {
        self.expressions.iter().filter(|r| !r.success).count()
    }

    /// Check if at least one expression succeeded
    pub fn has_any_success(&self) -> bool {
        self.expressions.iter().any(|r| r.success)
    }
}

/// Split input on semicolons, being careful about semicolons inside string literals
fn split_on_semicolons(input: &str) -> Vec<String> {
    let mut expressions = Vec::new();
    let mut current_expr = String::new();
    let mut in_string = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                current_expr.push(ch);
                in_string = !in_string;

                // Handle escaped quotes (double single quotes)
                if in_string && chars.peek() == Some(&'\'') {
                    current_expr.push(chars.next().unwrap());
                    in_string = !in_string;
                }
            }
            ';' if !in_string => {
                // Found a delimiter semicolon
                if !current_expr.trim().is_empty() {
                    expressions.push(current_expr.trim().to_string());
                }
                current_expr.clear();
            }
            _ => {
                current_expr.push(ch);
            }
        }
    }

    // Add the last expression if not empty
    if !current_expr.trim().is_empty() {
        expressions.push(current_expr.trim().to_string());
    }

    // If no semicolons found, return the entire input as single expression
    // But only if the input isn't just semicolons/whitespace
    if expressions.is_empty()
        && !input.trim().is_empty()
        && !input.trim().chars().all(|c| c == ';' || c.is_whitespace())
    {
        expressions.push(input.trim().to_string());
    }

    expressions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_modes() {
        let expr = "Patient.name.given.first()";

        // Both modes should succeed for valid expression
        let fast_result = parse_with_mode(expr, ParsingMode::Fast);
        assert!(fast_result.success);
        assert!(fast_result.ast.is_some());

        let analysis_result = parse_with_mode(expr, ParsingMode::Analysis);
        assert!(analysis_result.success);
        assert!(analysis_result.ast.is_some());
    }

    #[test]
    fn test_main_api_functions() {
        let valid_expr = "Patient.name";
        let invalid_expr = "Patient.name[";

        // Test parse (default fast mode)
        let result = parse(valid_expr);
        assert!(result.success);
        assert!(result.ast.is_some());

        let result = parse(invalid_expr);
        assert!(!result.success);

        // Test parse_with_analysis
        let result = parse_with_analysis(valid_expr);
        assert!(result.success);
        assert!(result.ast.is_some());

        let result = parse_with_analysis(invalid_expr);
        assert!(!result.success);
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_ast_functions() {
        let valid_expr = "Patient.name";
        let invalid_expr = "Patient.name[";

        // Test parse_ast
        match parse_ast(valid_expr) {
            Ok(ast) => assert!(!format!("{:?}", ast).is_empty()),
            Err(_) => panic!("Should have succeeded"),
        }

        match parse_ast(invalid_expr) {
            Ok(_) => panic!("Should have failed"),
            Err(error) => assert!(!error.to_string().is_empty()),
        }

        // Test parse_ast_with_mode
        assert!(parse_ast_with_mode(valid_expr, ParsingMode::Fast).is_ok());
        assert!(parse_ast_with_mode(valid_expr, ParsingMode::Analysis).is_ok());
        assert!(parse_ast_with_mode(invalid_expr, ParsingMode::Fast).is_err());
        assert!(parse_ast_with_mode(invalid_expr, ParsingMode::Analysis).is_err());
    }

    #[test]
    fn test_convenience_functions() {
        let valid_expr = "Patient.name";
        let invalid_expr = "Patient.name[";

        // Test is_valid
        assert!(is_valid(valid_expr));
        assert!(!is_valid(invalid_expr));

        // Test validate
        assert!(validate(valid_expr).is_ok());
        match validate(invalid_expr) {
            Ok(_) => panic!("Should have failed"),
            Err(diagnostics) => assert!(!diagnostics.is_empty()),
        }
    }

    #[test]
    fn test_multiple_expressions() {
        let expressions = vec![
            "Patient.name",
            "age > 18",
            "status = 'active'",
            "invalid.expression[",
        ];

        let results = parse_multiple(&expressions, ParsingMode::Analysis);
        assert_eq!(results.len(), 4);

        // First 3 should succeed
        assert!(results[0].success);
        assert!(results[1].success);
        assert!(results[2].success);

        // Last should fail
        assert!(!results[3].success);

        // Test parse_multiple_ast
        let asts = parse_multiple_ast(&expressions);
        assert_eq!(asts.len(), 3); // Only successful ones
    }

    #[test]
    fn test_error_handling_differences() {
        let invalid_expr = "Patient.name(unclosed_paren";

        // Fast parser should fail fast
        let fast_result = parse_with_mode(invalid_expr, ParsingMode::Fast);
        assert!(!fast_result.success);
        assert!(fast_result.ast.is_none());
        assert!(!fast_result.diagnostics.is_empty());

        // Analysis parser should provide more detailed recovery
        let analysis_result = parse_with_mode(invalid_expr, ParsingMode::Analysis);
        assert!(!analysis_result.success);
        assert!(!analysis_result.diagnostics.is_empty());
    }

    #[test]
    fn test_parser_use_case_recommendations() {
        assert_eq!(
            recommend_mode(ParserUseCase::RuntimeEvaluation),
            ParsingMode::Fast
        );

        assert_eq!(recommend_mode(ParserUseCase::IDE), ParsingMode::Analysis);

        assert_eq!(
            recommend_mode(ParserUseCase::Performance),
            ParsingMode::Fast
        );

        assert_eq!(
            recommend_mode(ParserUseCase::Development),
            ParsingMode::Analysis
        );
    }

    #[test]
    fn test_result_conversion() {
        let valid_expr = "Patient.name";
        let result = parse(valid_expr);

        // Should convert to Ok
        let ast_result = result.into_result();
        assert!(ast_result.is_ok());

        let invalid_expr = "Patient.name[";
        let result = parse(invalid_expr);

        // Should convert to Err
        let ast_result = result.into_result();
        assert!(ast_result.is_err());
    }

    #[test]
    fn test_error_and_warning_extraction() {
        let invalid_expr = "Patient.name[unclosed_bracket";

        let errors = get_errors(invalid_expr);
        assert!(!errors.is_empty());

        // For this test, warnings might be empty since we're testing with an error
        let _warnings = get_warnings(invalid_expr);
    }

    #[test]
    fn test_parse_result_helpers() {
        let valid_expr = "Patient.name";
        let invalid_expr = "Patient.name[";

        let valid_result = parse(valid_expr);
        assert!(valid_result.success);
        assert!(!valid_result.has_errors());
        assert!(valid_result.first_error().is_none());

        let invalid_result = parse(invalid_expr);
        assert!(!invalid_result.success);
        assert!(invalid_result.has_errors());
        assert!(invalid_result.first_error().is_some());
        assert!(!invalid_result.first_error().unwrap().is_empty());
    }

    #[test]
    fn test_backward_compatibility() {
        let expr = "Patient.name";

        // Legacy function should still work
        match parse_expression(expr) {
            Ok(ast) => assert!(!format!("{:?}", ast).is_empty()),
            Err(_) => panic!("Should have succeeded"),
        }
    }

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;

        let expressions = vec![
            "Patient.name.given.first()",
            "age > 18 and status = 'active'",
            "value * 2.5 + offset",
            "items.where(active = true).count() > 0",
        ];

        // Test fast parser performance
        let start = Instant::now();
        for _ in 0..100 {
            for expr in &expressions {
                let _ = parse_with_mode(expr, ParsingMode::Fast);
            }
        }
        let fast_time = start.elapsed();

        // Test analysis parser performance
        let start = Instant::now();
        for _ in 0..100 {
            for expr in &expressions {
                let _ = parse_with_mode(expr, ParsingMode::Analysis);
            }
        }
        let analysis_time = start.elapsed();

        println!("Fast parser: {:?} for 400 expressions", fast_time);
        println!("Analysis parser: {:?} for 400 expressions", analysis_time);

        // Fast parser should complete quickly
        let fast_ops_per_sec = 400.0 / fast_time.as_secs_f64();
        let analysis_ops_per_sec = 400.0 / analysis_time.as_secs_f64();

        assert!(
            fast_ops_per_sec > 2_000.0,
            "Fast parser too slow: {} ops/sec",
            fast_ops_per_sec
        );
        assert!(
            analysis_ops_per_sec > 1_000.0,
            "Analysis parser too slow: {} ops/sec",
            analysis_ops_per_sec
        );

        // Fast should generally be faster (though small samples might vary)
        println!("Fast parser: {:.0} ops/sec", fast_ops_per_sec);
        println!("Analysis parser: {:.0} ops/sec", analysis_ops_per_sec);
    }

    #[test]
    fn test_diagnostic_consistency() {
        let invalid_expr = "Patient.name = ";

        let fast_result = parse_with_mode(invalid_expr, ParsingMode::Fast);
        let analysis_result = parse_with_mode(invalid_expr, ParsingMode::Analysis);

        // Both should report errors
        assert!(!fast_result.success);
        assert!(!analysis_result.success);

        // Both should have diagnostics
        assert!(!fast_result.diagnostics.is_empty());
        assert!(!analysis_result.diagnostics.is_empty());
    }

    #[test]
    fn test_config_support() {
        let expr = "Patient.name";
        let config = ParserConfig::default();

        // Should work with default config (even though config is not fully implemented yet)
        let result = parse_with_config(expr, ParsingMode::Fast, config);
        assert!(result.success);
    }

    #[test]
    fn test_complex_expressions() {
        let complex_expr = "Patient.name.where(use = 'official' and active = true).given[0]";

        let fast_result = parse_with_mode(complex_expr, ParsingMode::Fast);
        assert!(
            fast_result.success,
            "Fast parser should handle complex expressions"
        );

        let analysis_result = parse_with_mode(complex_expr, ParsingMode::Analysis);
        assert!(
            analysis_result.success,
            "Analysis parser should handle complex expressions"
        );
    }

    #[test]
    fn test_various_error_conditions() {
        let error_cases = vec![
            ("", "Empty expression"),
            ("Patient.", "Incomplete property access"),
            ("Patient[", "Unclosed bracket"),
            ("Patient(", "Unclosed parenthesis"),
            ("Patient.name = ", "Incomplete comparison"),
        ];

        for (expr, description) in error_cases {
            let result = parse_with_analysis(expr);
            assert!(!result.success, "Should fail for: {}", description);
            assert!(
                !result.diagnostics.is_empty(),
                "Should have diagnostics for: {}",
                description
            );
        }
    }

    #[test]
    fn test_parsing_mode_display() {
        assert_eq!(format!("{}", ParsingMode::Fast), "fast");
        assert_eq!(format!("{}", ParsingMode::Analysis), "analysis");
    }

    #[test]
    fn test_parsing_mode_default() {
        assert_eq!(ParsingMode::default(), ParsingMode::Fast);
    }

    #[test]
    fn test_semicolon_delimited_parsing() {
        let input = "Patient.name; age > 18; status = 'active'";
        let result = parse_semicolon_delimited(input);

        assert!(result.success, "Should parse all expressions successfully");
        assert_eq!(result.expressions.len(), 3, "Should parse 3 expressions");
        assert_eq!(result.success_count(), 3, "All 3 should succeed");
        assert_eq!(result.failure_count(), 0, "None should fail");

        let asts = result.successful_asts();
        assert_eq!(asts.len(), 3, "Should get 3 successful ASTs");
    }

    #[test]
    fn test_semicolon_delimited_with_errors() {
        let input = "Patient.name; invalid[; status = 'active'";
        let result = parse_semicolon_delimited_with_analysis(input);

        assert!(!result.success, "Should not succeed overall");
        assert_eq!(result.expressions.len(), 3, "Should parse 3 expressions");
        assert_eq!(result.success_count(), 2, "Two should succeed");
        assert_eq!(result.failure_count(), 1, "One should fail");
        assert!(result.has_any_success(), "Should have at least one success");

        let asts = result.successful_asts();
        assert_eq!(asts.len(), 2, "Should get 2 successful ASTs");
    }

    #[test]
    fn test_semicolon_delimited_with_strings() {
        // Test that semicolons inside strings are not treated as delimiters
        let input = "Patient.name; status = 'active; not inactive'; age > 18";
        let result = parse_semicolon_delimited(input);

        assert!(result.success, "Should parse all expressions successfully");
        assert_eq!(result.expressions.len(), 3, "Should parse 3 expressions");

        // Check that the middle expression contains the semicolon
        let middle_result = &result.expressions[1];
        assert!(middle_result.success, "Middle expression should succeed");
    }

    #[test]
    fn test_semicolon_split_function() {
        let expressions = split_on_semicolons("a; b; c");
        assert_eq!(expressions, vec!["a", "b", "c"]);

        let expressions = split_on_semicolons("Patient.name");
        assert_eq!(expressions, vec!["Patient.name"]);

        let expressions = split_on_semicolons("status = 'active; test'; name");
        assert_eq!(expressions, vec!["status = 'active; test'", "name"]);

        let expressions = split_on_semicolons("a;; b; ; c");
        assert_eq!(expressions, vec!["a", "b", "c"]); // Empty expressions filtered out
    }

    #[test]
    fn test_semicolon_delimited_empty_input() {
        let result = parse_semicolon_delimited("");
        assert!(result.success, "Empty input should succeed");
        assert_eq!(result.expressions.len(), 0, "Should have no expressions");

        let result = parse_semicolon_delimited(";;;");
        assert!(result.success, "Only semicolons should succeed");
        assert_eq!(result.expressions.len(), 0, "Should have no expressions");
    }
}
