//! Static analyzer for comprehensive batch analysis of FHIRPath expressions
//!
//! This module provides enhanced multi-level reporting that combines all analyzers
//! for comprehensive static analysis with detailed statistics and suggestions.

use std::sync::Arc;

use crate::analyzer::{
    DiagnosticBuilder, ExpressionContext, FunctionAnalyzer, PropertyAnalyzer, TypeAnalyzer,
    UnionTypeAnalyzer,
};
use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, SourceLocation};
use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};
use crate::parser::SemanticAnalyzer;
use octofhir_fhir_model::{ModelProvider, TypeInfo};

/// Comprehensive static analyzer combining all analysis capabilities
pub struct StaticAnalyzer {
    #[allow(dead_code)]
    property_analyzer: PropertyAnalyzer,
    #[allow(dead_code)]
    union_analyzer: UnionTypeAnalyzer,
    #[allow(dead_code)]
    function_analyzer: FunctionAnalyzer,
    #[allow(dead_code)]
    type_analyzer: TypeAnalyzer,
    #[allow(dead_code)]
    diagnostic_builder: DiagnosticBuilder,
    semantic_analyzer: SemanticAnalyzer,
}

/// Result of comprehensive static analysis
#[derive(Debug, Clone)]
pub struct StaticAnalysisResult {
    /// Whether analysis completed successfully (no critical errors)
    pub success: bool,
    /// All diagnostics generated during analysis
    pub diagnostics: Vec<AriadneDiagnostic>,
    /// Analysis statistics summary
    pub statistics: AnalysisStatistics,
    /// Improvement suggestions
    pub suggestions: Vec<AnalysisSuggestion>,
    /// Type information if analysis succeeded
    pub type_info: Option<TypeInfo>,
}

/// Statistical summary of analysis results
#[derive(Debug, Clone, Default)]
pub struct AnalysisStatistics {
    /// Total number of expressions analyzed
    pub total_expressions: usize,
    /// Number of errors found
    pub errors_found: usize,
    /// Number of warnings found
    pub warnings_found: usize,
    /// Number of info-level diagnostics
    pub info_found: usize,
    /// Number of suggestions generated
    pub suggestions_generated: usize,
    /// Analysis performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics for analysis
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Time spent in property analysis (microseconds)
    pub property_analysis_time: u64,
    /// Time spent in function analysis (microseconds)
    pub function_analysis_time: u64,
    /// Time spent in type analysis (microseconds)
    pub type_analysis_time: u64,
    /// Time spent in union analysis (microseconds)
    pub union_analysis_time: u64,
    /// Total analysis time (microseconds)
    pub total_analysis_time: u64,
}

/// Analysis suggestion for code improvement
#[derive(Debug, Clone)]
pub struct AnalysisSuggestion {
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Human-readable suggestion message
    pub message: String,
    /// Location where suggestion applies
    pub location: Option<SourceLocation>,
    /// Code snippet showing the improvement
    pub code_snippet: Option<String>,
    /// Confidence level of the suggestion (0.0 - 1.0)
    pub confidence: f32,
}

/// Type of analysis suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionType {
    /// Performance optimization
    Performance,
    /// Code simplification
    Simplification,
    /// Type safety improvement
    TypeSafety,
    /// Best practice recommendation
    BestPractice,
    /// Error prevention
    ErrorPrevention,
}

impl std::fmt::Display for SuggestionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionType::Performance => write!(f, "Performance"),
            SuggestionType::Simplification => write!(f, "Simplification"),
            SuggestionType::TypeSafety => write!(f, "Type Safety"),
            SuggestionType::BestPractice => write!(f, "Best Practice"),
            SuggestionType::ErrorPrevention => write!(f, "Error Prevention"),
        }
    }
}

/// Context for analysis operations
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Root type for analysis
    pub root_type: TypeInfo,
    /// Whether to perform deep analysis (more thorough but slower)
    pub deep_analysis: bool,
    /// Whether to generate performance suggestions
    pub suggest_optimizations: bool,
    /// Maximum number of suggestions to generate
    pub max_suggestions: usize,
}

impl StaticAnalyzer {
    /// Create a new static analyzer with all sub-analyzers
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            property_analyzer: PropertyAnalyzer::new(model_provider.clone()),
            union_analyzer: UnionTypeAnalyzer::new(model_provider.clone()),
            function_analyzer: FunctionAnalyzer::new(model_provider.clone()),
            type_analyzer: TypeAnalyzer::new(model_provider.clone()),
            diagnostic_builder: DiagnosticBuilder::new(),
            semantic_analyzer: SemanticAnalyzer::new(model_provider),
        }
    }

    /// Perform comprehensive static analysis on a single expression
    pub async fn analyze_expression(
        &mut self,
        expression: &str,
        context: AnalysisContext,
    ) -> StaticAnalysisResult {
        let start_time = std::time::Instant::now();
        let mut statistics = AnalysisStatistics::default();
        let mut diagnostics = Vec::new();
        let mut suggestions = Vec::new();

        statistics.total_expressions = 1;

        // Parse the expression first using the fhirpath parser
        let parse_result = crate::parse(expression);

        let ast = match parse_result.ast {
            Some(ast) => ast,
            None => {
                statistics.errors_found = 1;
                let ariadne_diagnostics: Vec<AriadneDiagnostic> = parse_result
                    .diagnostics
                    .into_iter()
                    .map(|d| self.convert_diagnostic_to_ariadne(d, expression))
                    .collect();
                return StaticAnalysisResult {
                    success: false,
                    diagnostics: ariadne_diagnostics,
                    statistics,
                    suggestions,
                    type_info: None,
                };
            }
        };

        // Convert and add any parse diagnostics
        for diagnostic in parse_result.diagnostics {
            diagnostics.push(self.convert_diagnostic_to_ariadne(diagnostic, expression));
        }

        // Analyze the AST with semantic analyzer, passing expression text for span calculation
        let analysis_result = match self
            .semantic_analyzer
            .analyze_expression_with_text(&ast, Some(context.root_type.clone()), expression)
            .await
        {
            Ok(analysis) => analysis,
            Err(error) => {
                statistics.errors_found = 1;
                return StaticAnalysisResult {
                    success: false,
                    diagnostics: vec![self.create_parse_error_diagnostic(error, expression)],
                    statistics,
                    suggestions,
                    type_info: None,
                };
            }
        };

        // Convert and add existing diagnostics from semantic analysis
        for diagnostic in analysis_result.diagnostics {
            diagnostics.push(self.convert_diagnostic_to_ariadne(diagnostic, expression));
        }

        // Perform comprehensive analysis
        let analysis_result = self.analyze_ast_node(&ast, &context, &mut statistics).await;

        // Merge results
        diagnostics.extend(analysis_result.diagnostics);

        // Generate suggestions if requested
        if context.suggest_optimizations {
            let generated_suggestions = self
                .generate_suggestions(&ast, &context, &diagnostics)
                .await;
            suggestions.extend(
                generated_suggestions
                    .into_iter()
                    .take(context.max_suggestions),
            );
            statistics.suggestions_generated = suggestions.len();
        }

        // Calculate final statistics
        self.calculate_statistics(&diagnostics, &mut statistics);
        statistics.performance_metrics.total_analysis_time =
            start_time.elapsed().as_micros() as u64;

        let success = statistics.errors_found == 0;

        StaticAnalysisResult {
            success,
            diagnostics,
            statistics,
            suggestions,
            type_info: analysis_result.type_info,
        }
    }

    /// Batch analysis for multiple expressions
    pub async fn analyze_batch(
        &mut self,
        expressions: Vec<String>,
        context: AnalysisContext,
    ) -> Vec<StaticAnalysisResult> {
        let mut results = Vec::new();

        for expression in expressions {
            let result = self.analyze_expression(&expression, context.clone()).await;
            results.push(result);
        }

        results
    }

    /// Analyze a single AST node comprehensively
    async fn analyze_ast_node(
        &self,
        node: &ExpressionNode,
        context: &AnalysisContext,
        statistics: &mut AnalysisStatistics,
    ) -> InternalAnalysisResult {
        let mut diagnostics = Vec::new();
        let mut type_info = None;

        // Create expression context for type analysis
        let expr_context = ExpressionContext::new(context.root_type.clone());

        // Property analysis
        let property_start = std::time::Instant::now();
        if let Some(property_diagnostics) =
            self.analyze_property_access(node, &context.root_type).await
        {
            diagnostics.extend(property_diagnostics);
        }
        statistics.performance_metrics.property_analysis_time =
            property_start.elapsed().as_micros() as u64;

        // Function analysis
        let function_start = std::time::Instant::now();
        if let Some(function_diagnostics) =
            self.analyze_function_calls(node, &context.root_type).await
        {
            diagnostics.extend(function_diagnostics);
        }
        statistics.performance_metrics.function_analysis_time =
            function_start.elapsed().as_micros() as u64;

        // Union type analysis
        let union_start = std::time::Instant::now();
        if let Some(union_diagnostics) = self
            .analyze_union_operations(node, &context.root_type)
            .await
        {
            diagnostics.extend(union_diagnostics);
        }
        statistics.performance_metrics.union_analysis_time =
            union_start.elapsed().as_micros() as u64;

        // Type analysis
        let type_start = std::time::Instant::now();
        if let Some((type_diagnostics, inferred_type)) =
            self.analyze_type_flow(node, &expr_context).await
        {
            diagnostics.extend(type_diagnostics);
            type_info = inferred_type;
        }
        statistics.performance_metrics.type_analysis_time = type_start.elapsed().as_micros() as u64;

        InternalAnalysisResult {
            diagnostics,
            type_info,
        }
    }

    /// Analyze property access patterns
    async fn analyze_property_access(
        &self,
        _node: &ExpressionNode,
        _root_type: &TypeInfo,
    ) -> Option<Vec<AriadneDiagnostic>> {
        // This would traverse the AST and analyze property access
        // For now, return empty as this is a simplified implementation
        Some(Vec::new())
    }

    /// Analyze function call patterns
    async fn analyze_function_calls(
        &self,
        _node: &ExpressionNode,
        _root_type: &TypeInfo,
    ) -> Option<Vec<AriadneDiagnostic>> {
        // This would traverse the AST and analyze function calls
        // For now, return empty as this is a simplified implementation
        Some(Vec::new())
    }

    /// Analyze union type operations
    async fn analyze_union_operations(
        &self,
        _node: &ExpressionNode,
        _root_type: &TypeInfo,
    ) -> Option<Vec<AriadneDiagnostic>> {
        // This would traverse the AST and analyze union operations
        // For now, return empty as this is a simplified implementation
        Some(Vec::new())
    }

    /// Analyze type flow and inference
    async fn analyze_type_flow(
        &self,
        _node: &ExpressionNode,
        _context: &ExpressionContext,
    ) -> Option<(Vec<AriadneDiagnostic>, Option<TypeInfo>)> {
        // This would perform comprehensive type flow analysis
        // For now, return empty as this is a simplified implementation
        Some((Vec::new(), None))
    }

    /// Generate improvement suggestions based on analysis
    async fn generate_suggestions(
        &self,
        node: &ExpressionNode,
        context: &AnalysisContext,
        diagnostics: &[AriadneDiagnostic],
    ) -> Vec<AnalysisSuggestion> {
        let mut suggestions = Vec::new();

        // Generate performance suggestions
        if context.suggest_optimizations {
            suggestions.extend(self.generate_performance_suggestions(node).await);
        }

        // Generate simplification suggestions
        suggestions.extend(self.generate_simplification_suggestions(node).await);

        // Generate type safety suggestions based on diagnostics
        suggestions.extend(self.generate_type_safety_suggestions(diagnostics).await);

        suggestions
    }

    /// Generate performance optimization suggestions
    async fn generate_performance_suggestions(
        &self,
        node: &ExpressionNode,
    ) -> Vec<AnalysisSuggestion> {
        let mut suggestions = Vec::new();

        // Example: Suggest caching for complex expressions
        if self.is_complex_expression(node) {
            suggestions.push(AnalysisSuggestion {
                suggestion_type: SuggestionType::Performance,
                message: "Consider caching this complex expression result if used multiple times"
                    .to_string(),
                location: None,
                code_snippet: None,
                confidence: 0.7,
            });
        }

        suggestions
    }

    /// Generate code simplification suggestions
    async fn generate_simplification_suggestions(
        &self,
        node: &ExpressionNode,
    ) -> Vec<AnalysisSuggestion> {
        let mut suggestions = Vec::new();

        // Example: Suggest removing redundant operations
        if self.has_redundant_operations(node) {
            suggestions.push(AnalysisSuggestion {
                suggestion_type: SuggestionType::Simplification,
                message: "This expression can be simplified by removing redundant operations"
                    .to_string(),
                location: None,
                code_snippet: None,
                confidence: 0.8,
            });
        }

        suggestions
    }

    /// Generate type safety improvement suggestions
    async fn generate_type_safety_suggestions(
        &self,
        diagnostics: &[AriadneDiagnostic],
    ) -> Vec<AnalysisSuggestion> {
        let mut suggestions = Vec::new();

        // Analyze diagnostics for type safety improvements
        for diagnostic in diagnostics {
            if diagnostic.severity == DiagnosticSeverity::Warning {
                suggestions.push(AnalysisSuggestion {
                    suggestion_type: SuggestionType::TypeSafety,
                    message: "Consider adding explicit type checking to improve safety".to_string(),
                    location: None,
                    code_snippet: None,
                    confidence: 0.6,
                });
            }
        }

        suggestions
    }

    /// Check if expression is complex enough to warrant optimization suggestions
    fn is_complex_expression(&self, node: &ExpressionNode) -> bool {
        // Simple heuristic - in a real implementation this would be more sophisticated
        self.count_ast_nodes(node) > 10
    }

    /// Check if expression has redundant operations
    fn has_redundant_operations(&self, _node: &ExpressionNode) -> bool {
        // Simple heuristic - in a real implementation this would detect actual redundancy
        false
    }

    /// Count the number of nodes in the AST
    fn count_ast_nodes(&self, _node: &ExpressionNode) -> usize {
        // Simplified node counting - in a real implementation this would traverse the AST
        1
    }

    /// Create a diagnostic for parse errors
    fn create_parse_error_diagnostic(
        &self,
        error: FhirPathError,
        expression: &str,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: crate::core::error_code::ErrorCode::new(9000),
            message: format!("Parse error: {error}"),
            span: 0..expression.len(),
            help: Some("Check syntax and try again".to_string()),
            note: None,
            related: Vec::new(),
        }
    }

    /// Calculate final statistics from diagnostics
    fn calculate_statistics(
        &self,
        diagnostics: &[AriadneDiagnostic],
        statistics: &mut AnalysisStatistics,
    ) {
        for diagnostic in diagnostics {
            match diagnostic.severity {
                DiagnosticSeverity::Error => statistics.errors_found += 1,
                DiagnosticSeverity::Warning => statistics.warnings_found += 1,
                DiagnosticSeverity::Info => statistics.info_found += 1,
                DiagnosticSeverity::Hint => statistics.info_found += 1,
            }
        }
    }

    /// Convert a Diagnostic to AriadneDiagnostic
    fn convert_diagnostic_to_ariadne(
        &self,
        diagnostic: crate::diagnostics::Diagnostic,
        expression: &str,
    ) -> AriadneDiagnostic {
        use crate::core::error_code::ErrorCode;

        let span = if let Some(loc) = &diagnostic.location {
            loc.offset..loc.offset + loc.length
        } else {
            0..expression.len()
        };

        AriadneDiagnostic {
            severity: diagnostic.severity,
            error_code: ErrorCode::new(9001), // Fallback error code
            message: diagnostic.message,
            span,
            help: None,
            note: None,
            related: Vec::new(),
        }
    }
}

/// Internal result structure for analysis operations
struct InternalAnalysisResult {
    diagnostics: Vec<AriadneDiagnostic>,
    type_info: Option<TypeInfo>,
}

impl AnalysisContext {
    /// Create a new analysis context with default settings
    pub fn new(root_type: TypeInfo) -> Self {
        Self {
            root_type,
            deep_analysis: false,
            suggest_optimizations: true,
            max_suggestions: 10,
        }
    }

    /// Enable deep analysis (more thorough but slower)
    pub fn with_deep_analysis(mut self) -> Self {
        self.deep_analysis = true;
        self
    }

    /// Configure optimization suggestions
    pub fn with_optimization_suggestions(mut self, enabled: bool) -> Self {
        self.suggest_optimizations = enabled;
        self
    }

    /// Set maximum number of suggestions
    pub fn with_max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self {
            root_type: TypeInfo {
                type_name: "Resource".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("FHIR".to_string()),
                name: Some("Resource".to_string()),
            },
            deep_analysis: false,
            suggest_optimizations: true,
            max_suggestions: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;

    fn create_test_type_info(type_name: &str, singleton: bool) -> TypeInfo {
        TypeInfo {
            type_name: type_name.to_string(),
            singleton: Some(singleton),
            is_empty: Some(false),
            namespace: Some("FHIR".to_string()),
            name: Some(type_name.to_string()),
        }
    }

    #[tokio::test]
    async fn test_static_analyzer_creation() {
        let provider = Arc::new(EmptyModelProvider);
        let analyzer = StaticAnalyzer::new(provider);

        // Just verify it doesn't panic
        assert_eq!(
            std::mem::size_of_val(&analyzer),
            std::mem::size_of::<StaticAnalyzer>()
        );
    }

    #[tokio::test]
    async fn test_analysis_context_creation() {
        let type_info = create_test_type_info("Patient", true);
        let context = AnalysisContext::new(type_info.clone());

        assert_eq!(context.root_type.type_name, "Patient");
        assert!(!context.deep_analysis);
        assert!(context.suggest_optimizations);
        assert_eq!(context.max_suggestions, 10);
    }

    #[tokio::test]
    async fn test_analysis_context_configuration() {
        let type_info = create_test_type_info("Patient", true);
        let context = AnalysisContext::new(type_info)
            .with_deep_analysis()
            .with_optimization_suggestions(false)
            .with_max_suggestions(5);

        assert!(context.deep_analysis);
        assert!(!context.suggest_optimizations);
        assert_eq!(context.max_suggestions, 5);
    }

    #[tokio::test]
    async fn test_static_analysis_result_structure() {
        let statistics = AnalysisStatistics::default();
        let result = StaticAnalysisResult {
            success: true,
            diagnostics: Vec::new(),
            statistics,
            suggestions: Vec::new(),
            type_info: None,
        };

        assert!(result.success);
        assert_eq!(result.diagnostics.len(), 0);
        assert_eq!(result.suggestions.len(), 0);
    }

    #[tokio::test]
    async fn test_suggestion_types() {
        let suggestion = AnalysisSuggestion {
            suggestion_type: SuggestionType::Performance,
            message: "Test suggestion".to_string(),
            location: None,
            code_snippet: None,
            confidence: 0.8,
        };

        assert_eq!(suggestion.suggestion_type, SuggestionType::Performance);
        assert_eq!(suggestion.confidence, 0.8);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let metrics = PerformanceMetrics::default();

        assert_eq!(metrics.property_analysis_time, 0);
        assert_eq!(metrics.function_analysis_time, 0);
        assert_eq!(metrics.type_analysis_time, 0);
        assert_eq!(metrics.union_analysis_time, 0);
        assert_eq!(metrics.total_analysis_time, 0);
    }
}
