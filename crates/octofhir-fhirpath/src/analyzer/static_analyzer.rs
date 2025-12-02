//! Static analyzer for comprehensive batch analysis of FHIRPath expressions
//!
//! This module provides enhanced multi-level reporting that combines all analyzers
//! for comprehensive static analysis with detailed statistics and suggestions.

use std::sync::Arc;

use octofhir_fhir_model::{ModelProvider, TypeInfo};

use crate::analyzer::{
    DiagnosticBuilder, ExpressionContext, FunctionAnalyzer, PropertyAnalyzer, TypeAnalyzer,
    UnionTypeAnalyzer,
};
use crate::ast::ExpressionNode;
use crate::core::error_code::ErrorCode;
use crate::core::{FhirPathError, SourceLocation};
use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};
use crate::parser::SemanticAnalyzer;

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
    #[allow(dead_code)]
    semantic_analyzer: SemanticAnalyzer,
    /// Current source expression being analyzed (for span calculation)
    current_source: Option<String>,
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
    pub fn new(model_provider: Arc<dyn ModelProvider + Send + Sync>) -> Self {
        let function_registry = Arc::new(crate::evaluator::create_function_registry());
        Self {
            property_analyzer: PropertyAnalyzer::new(model_provider.clone()),
            union_analyzer: UnionTypeAnalyzer::new(model_provider.clone()),
            function_analyzer: FunctionAnalyzer::new(model_provider.clone(), function_registry),
            type_analyzer: TypeAnalyzer::new(model_provider.clone()),
            diagnostic_builder: DiagnosticBuilder::new(),
            semantic_analyzer: SemanticAnalyzer::new(model_provider),
            current_source: None,
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

        // Store current source for span calculation
        self.current_source = Some(expression.to_string());

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

        // Extract resource type from expression and use it as context
        let extracted_type = self.extract_resource_type_from_ast(&ast, &context.root_type);

        // Skip semantic analyzer - static analyzer handles all validation
        // The semantic analyzer is causing incorrect property validation with wrong context
        // let analysis_result = match self
        //     .semantic_analyzer
        //     .analyze_expression_with_text(&ast, Some(extracted_type), expression)
        //     .await
        // {
        //     Ok(analysis) => analysis,
        //     Err(error) => {
        //         statistics.errors_found = 1;
        //         return StaticAnalysisResult {
        //             success: false,
        //             diagnostics: vec![self.create_parse_error_diagnostic(error, expression)],
        //             statistics,
        //             suggestions,
        //             type_info: None,
        //         };
        //     }
        // };

        // Skip semantic analyzer diagnostics - using static analyzer only
        // for diagnostic in analysis_result.diagnostics {
        //     diagnostics.push(self.convert_diagnostic_to_ariadne(diagnostic, expression));
        // }

        // Perform comprehensive analysis with extracted resource type context
        let mut updated_context = context.clone();
        updated_context.root_type = extracted_type;
        let analysis_result = self
            .analyze_ast_node(&ast, &updated_context, &mut statistics)
            .await;

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
        node: &ExpressionNode,
        root_type: &TypeInfo,
    ) -> Option<Vec<AriadneDiagnostic>> {
        let mut diagnostics = Vec::new();
        self.validate_properties_recursive(node, root_type, &mut diagnostics)
            .await;
        Some(diagnostics)
    }

    /// Validate properties recursively through the AST
    fn validate_properties_recursive<'a>(
        &'a self,
        node: &'a ExpressionNode,
        current_type: &'a TypeInfo,
        diagnostics: &'a mut Vec<AriadneDiagnostic>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a + Send>> {
        Box::pin(async move {
            match node {
                ExpressionNode::PropertyAccess(property_access) => {
                    let property_name = &property_access.property;

                    // Skip the root resource type (e.g., don't validate "Patient" in "Patient.name")
                    if !self.is_likely_resource_type(property_name) {
                        // Validate this property exists on current type
                        if !self.property_exists_on_type(current_type, property_name) {
                            let suggestions =
                                self.suggest_property_names(current_type, property_name);
                            let message = if !suggestions.is_empty() {
                                format!(
                                    "Unknown property '{}', did you mean '{}'?",
                                    property_name, suggestions[0]
                                )
                            } else {
                                format!("Unknown property '{property_name}'")
                            };

                            let span = property_access
                                .location
                                .clone()
                                .map(|l| l.offset..l.offset + property_name.len())
                                .unwrap_or_else(|| {
                                    // Calculate span by finding property in source text
                                    self.calculate_property_span(property_name)
                                });

                            diagnostics.push(AriadneDiagnostic {
                                severity: DiagnosticSeverity::Error,
                                error_code: crate::core::error_code::FP0052,
                                message,
                                span,
                                help: None,
                                note: None,
                                related: Vec::new(),
                            });
                        }
                    }

                    // Continue validating the object part
                    self.validate_properties_recursive(
                        &property_access.object,
                        current_type,
                        diagnostics,
                    )
                    .await;
                }
                ExpressionNode::MethodCall(method_call) => {
                    // Validate the object part
                    self.validate_properties_recursive(
                        &method_call.object,
                        current_type,
                        diagnostics,
                    )
                    .await;

                    // Validate function arguments that might contain properties
                    for arg in &method_call.arguments {
                        self.validate_properties_recursive(arg, current_type, diagnostics)
                            .await;
                        // Also check for resourceType parameter validation
                        self.validate_method_parameters(&method_call.method, arg, diagnostics)
                            .await;
                    }
                }
                ExpressionNode::FunctionCall(function_call) => {
                    // Validate function arguments
                    for arg in &function_call.arguments {
                        self.validate_properties_recursive(arg, current_type, diagnostics)
                            .await;
                        self.validate_method_parameters(&function_call.name, arg, diagnostics)
                            .await;
                    }
                }
                _ => {}
            }
        })
    }

    /// Validate function parameters like resourceType values
    async fn validate_method_parameters(
        &self,
        _function_name: &str,
        arg: &ExpressionNode,
        diagnostics: &mut Vec<AriadneDiagnostic>,
    ) {
        // Check for resourceType parameter assignments
        if let ExpressionNode::BinaryOperation(comparison) = arg
            && comparison.operator == crate::ast::BinaryOperator::Equal
            && let (ExpressionNode::Identifier(left), ExpressionNode::Literal(literal)) =
                (&*comparison.left, &*comparison.right)
            && left.name == "resourceType"
            && let crate::ast::LiteralValue::String(resource_type) = &literal.value
            && !self.is_valid_resource_type(resource_type)
        {
            let valid_types = self.get_valid_resource_types();
            let message = format!(
                "Invalid resourceType '{}', valid types: {}",
                resource_type,
                valid_types.join(", ")
            );

            let span = literal
                .location
                .clone()
                .map(|l| l.offset..l.offset + l.length)
                .unwrap_or_else(|| self.calculate_literal_span(resource_type));

            diagnostics.push(AriadneDiagnostic {
                severity: DiagnosticSeverity::Error,
                error_code: crate::core::error_code::FP0001,
                message,
                span,
                help: None,
                note: None,
                related: Vec::new(),
            });
        }
    }

    /// Check if property exists on given type
    fn property_exists_on_type(&self, type_info: &TypeInfo, property_name: &str) -> bool {
        // Basic implementation - in real case would check model provider
        match type_info.type_name.as_str() {
            "Patient" => {
                let patient_properties = [
                    "id",
                    "meta",
                    "implicitRules",
                    "language",
                    "text",
                    "contained",
                    "extension",
                    "modifierExtension",
                    "identifier",
                    "active",
                    "name",
                    "telecom",
                    "gender",
                    "birthDate",
                    "deceased",
                    "address",
                    "maritalStatus",
                    "multipleBirth",
                    "photo",
                    "contact",
                    "communication",
                    "generalPractitioner",
                    "managingOrganization",
                    "link",
                ];
                patient_properties.contains(&property_name)
            }
            _ => true, // For other types, assume valid for now
        }
    }

    /// Suggest property names using Levenshtein distance
    fn suggest_property_names(&self, type_info: &TypeInfo, attempted_name: &str) -> Vec<String> {
        match type_info.type_name.as_str() {
            "Patient" => {
                let patient_properties = [
                    "id",
                    "meta",
                    "implicitRules",
                    "language",
                    "text",
                    "contained",
                    "extension",
                    "modifierExtension",
                    "identifier",
                    "active",
                    "name",
                    "telecom",
                    "gender",
                    "birthDate",
                    "deceased",
                    "address",
                    "maritalStatus",
                    "multipleBirth",
                    "photo",
                    "contact",
                    "communication",
                    "generalPractitioner",
                    "managingOrganization",
                    "link",
                ];

                let mut suggestions = Vec::new();
                for property in &patient_properties {
                    let distance = self.levenshtein_distance(attempted_name, property);
                    let max_distance = std::cmp::max(2, attempted_name.len() / 2);
                    if distance <= max_distance && distance > 0 {
                        suggestions.push((property.to_string(), distance));
                    }
                }
                suggestions.sort_by_key(|&(_, distance)| distance);
                suggestions
                    .into_iter()
                    .map(|(name, _)| name)
                    .take(3)
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    /// Check if resource type is valid
    fn is_valid_resource_type(&self, resource_type: &str) -> bool {
        let valid_types = [
            "Patient",
            "Observation",
            "Condition",
            "Procedure",
            "MedicationRequest",
            "DiagnosticReport",
            "Encounter",
            "Practitioner",
            "Organization",
            "Location",
        ];
        valid_types.contains(&resource_type)
    }

    /// Get list of valid resource types
    fn get_valid_resource_types(&self) -> Vec<String> {
        vec![
            "Patient".to_string(),
            "Observation".to_string(),
            "Condition".to_string(),
            "Procedure".to_string(),
            "MedicationRequest".to_string(),
        ]
    }

    /// Calculate Levenshtein distance (reuse from function analyzer)
    fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        #[allow(clippy::needless_range_loop)]
        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        #[allow(clippy::needless_range_loop)]
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        #[allow(clippy::needless_range_loop)]
        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        matrix[a_len][b_len]
    }

    /// Analyze function call patterns
    async fn analyze_function_calls(
        &self,
        node: &ExpressionNode,
        root_type: &TypeInfo,
    ) -> Option<Vec<AriadneDiagnostic>> {
        let mut diagnostics = Vec::new();
        self.validate_function_calls(node, root_type, &mut diagnostics)
            .await;
        Some(diagnostics)
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

    /// Recursively validate properties in an AST node
    #[allow(dead_code)]
    fn validate_node_properties<'a>(
        &'a self,
        node: &'a ExpressionNode,
        current_type: &'a TypeInfo,
        diagnostics: &'a mut Vec<AriadneDiagnostic>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a + Send>> {
        Box::pin(async move {
            match node {
                ExpressionNode::Identifier(identifier_node) => {
                    if current_type.type_name == "Resource" || current_type.type_name == "Any" {
                        self.validate_resource_type_literal(&identifier_node.name, diagnostics)
                            .await;
                    } else {
                        self.validate_property_name(
                            current_type,
                            &identifier_node.name,
                            diagnostics,
                        )
                        .await;
                    }
                }
                ExpressionNode::Literal(literal_node) => {
                    // Check if this is a resourceType literal
                    if let crate::ast::LiteralValue::String(value) = &literal_node.value {
                        // We'll validate resource type literals when we see them in type functions
                        // For now, just store that we saw this literal
                        let _ = value; // Prevent unused variable warning
                    }
                }
                ExpressionNode::FunctionCall(function_node) => {
                    // Validate function calls, especially type-related ones
                    self.validate_function_call(
                        &function_node.name,
                        &function_node.arguments,
                        current_type,
                        diagnostics,
                    )
                    .await;

                    // Recursively validate arguments
                    for arg in &function_node.arguments {
                        self.validate_node_properties(arg, current_type, diagnostics)
                            .await;
                    }
                }
                ExpressionNode::PropertyAccess(property_node) => {
                    match property_node.object.as_ref() {
                        ExpressionNode::Identifier(obj_identifier) => {
                            if current_type.type_name == "Resource"
                                || current_type.type_name == "Any"
                            {
                                self.validate_resource_type_literal(
                                    &obj_identifier.name,
                                    diagnostics,
                                )
                                .await;

                                let patient_type = TypeInfo {
                                    type_name: obj_identifier.name.clone(),
                                    singleton: Some(true),
                                    is_empty: Some(false),
                                    namespace: Some("FHIR".to_string()),
                                    name: Some(obj_identifier.name.clone()),
                                };
                                self.validate_property_name(
                                    &patient_type,
                                    &property_node.property,
                                    diagnostics,
                                )
                                .await;
                            } else {
                                self.validate_node_properties(
                                    &property_node.object,
                                    current_type,
                                    diagnostics,
                                )
                                .await;
                            }
                        }
                        _ => {
                            self.validate_node_properties(
                                &property_node.object,
                                current_type,
                                diagnostics,
                            )
                            .await;
                        }
                    }
                }
                ExpressionNode::BinaryOperation(binary_node) => {
                    // Recursively validate both sides
                    self.validate_node_properties(&binary_node.left, current_type, diagnostics)
                        .await;
                    self.validate_node_properties(&binary_node.right, current_type, diagnostics)
                        .await;
                }
                _ => {
                    // For other expressions, we don't validate for now
                    // In a full implementation, we'd handle all node types
                }
            }
        })
    }

    /// Validate a property name against the current type
    #[allow(dead_code)]
    async fn validate_property_name(
        &self,
        parent_type: &TypeInfo,
        property_name: &str,
        diagnostics: &mut Vec<AriadneDiagnostic>,
    ) {
        // Don't validate if this is a root-level resource type identifier (e.g., "Patient" in "Patient.name")
        // These are handled differently as they represent resource type selection, not property access
        if parent_type.type_name == "Resource" || parent_type.type_name == "Any" {
            // This might be a resource type identifier, not a property - skip validation
            return;
        }

        // Check if this is a resourceType property on a Reference type
        if property_name == "resourceType" && self.is_reference_type(parent_type) {
            // This is valid - resourceType is always valid on Reference types
            return;
        }

        if let Ok(analysis) = self
            .property_analyzer
            .validate_property_access(parent_type, property_name, None)
            .await
        {
            for diagnostic in analysis.diagnostics {
                if let Ok(ariadne_diagnostic) =
                    self.convert_legacy_diagnostic_to_ariadne(diagnostic)
                {
                    diagnostics.push(ariadne_diagnostic);
                }
            }
        }
    }

    /// Validate a resource type literal (used in ofType, is, as functions)
    #[allow(dead_code)]
    async fn validate_resource_type_literal(
        &self,
        resource_type: &str,
        diagnostics: &mut Vec<AriadneDiagnostic>,
    ) {
        // Use the property analyzer to validate the resource type
        if let Ok(analysis) = self
            .property_analyzer
            .validate_resource_type(resource_type, None)
            .await
        {
            // Convert any diagnostics from the property analyzer
            for diagnostic in analysis.diagnostics {
                if let Ok(ariadne_diagnostic) =
                    self.convert_legacy_diagnostic_to_ariadne(diagnostic)
                {
                    diagnostics.push(ariadne_diagnostic);
                }
            }
        }
    }

    /// Recursively validate function calls in AST
    fn validate_function_calls<'a>(
        &'a self,
        node: &'a ExpressionNode,
        current_type: &'a TypeInfo,
        diagnostics: &'a mut Vec<AriadneDiagnostic>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a + Send>> {
        Box::pin(async move {
            match node {
                ExpressionNode::FunctionCall(function_node) => {
                    let span = function_node
                        .location
                        .clone()
                        .map(|l| l.offset..l.offset + function_node.name.len())
                        .unwrap_or_else(|| self.calculate_function_span(&function_node.name));
                    let function_diagnostics = self
                        .function_analyzer
                        .validate_function_call_new(
                            &function_node.name,
                            current_type,
                            &function_node.arguments,
                            span,
                        )
                        .await;
                    diagnostics.extend(function_diagnostics);

                    for arg in &function_node.arguments {
                        self.validate_function_calls(arg, current_type, diagnostics)
                            .await;
                    }
                }
                ExpressionNode::MethodCall(method_node) => {
                    let span = method_node
                        .location
                        .clone()
                        .map(|l| l.offset..l.offset + method_node.method.len())
                        .unwrap_or_else(|| self.calculate_function_span(&method_node.method));
                    let method_diagnostics = self
                        .function_analyzer
                        .validate_function_call_new(
                            &method_node.method,
                            current_type,
                            &method_node.arguments,
                            span,
                        )
                        .await;
                    diagnostics.extend(method_diagnostics);

                    self.validate_function_calls(&method_node.object, current_type, diagnostics)
                        .await;
                    for arg in &method_node.arguments {
                        self.validate_function_calls(arg, current_type, diagnostics)
                            .await;
                    }
                }
                ExpressionNode::PropertyAccess(property_node) => {
                    self.validate_function_calls(&property_node.object, current_type, diagnostics)
                        .await;
                }
                ExpressionNode::BinaryOperation(binary_node) => {
                    self.validate_function_calls(&binary_node.left, current_type, diagnostics)
                        .await;
                    self.validate_function_calls(&binary_node.right, current_type, diagnostics)
                        .await;
                }
                _ => {}
            }
        })
    }

    /// Validate function calls for type-related operations
    #[allow(dead_code)]
    async fn validate_function_call(
        &self,
        function_name: &str,
        arguments: &[ExpressionNode],
        current_type: &TypeInfo,
        diagnostics: &mut Vec<AriadneDiagnostic>,
    ) {
        match function_name {
            "ofType" | "is" | "as" => {
                // These functions should have exactly one argument that's a type literal
                if arguments.len() != 1 {
                    return; // Function analyzer will handle argument count validation
                }

                // Check if the argument is a string literal representing a type
                if let ExpressionNode::Literal(literal_node) = &arguments[0]
                    && let crate::ast::LiteralValue::String(type_name) = &literal_node.value
                {
                    // Validate that the resource type value is a valid FHIR resource type

                    // Also validate if this type operation makes sense for the current type
                    self.validate_type_operation(
                        function_name,
                        current_type,
                        type_name,
                        diagnostics,
                    )
                    .await;
                }
            }
            _ => {
                // Other function validations can be added here
            }
        }
    }

    /// Validate type operations (ofType, is, as) against the current type context
    #[allow(dead_code)]
    async fn validate_type_operation(
        &self,
        operation: &str,
        input_type: &TypeInfo,
        target_type: &str,
        diagnostics: &mut Vec<AriadneDiagnostic>,
    ) {
        // Use the union analyzer to validate type filter operations
        let union_diagnostics =
            self.union_analyzer
                .validate_type_filter(input_type, target_type, operation);

        diagnostics.extend(union_diagnostics);
    }

    /// Check if a type is a Reference type
    #[allow(dead_code)]
    fn is_reference_type(&self, type_info: &TypeInfo) -> bool {
        type_info.type_name == "Reference"
            || type_info.type_name.ends_with("Reference")
            || type_info.namespace == Some("FHIR".to_string())
                && type_info.name == Some("Reference".to_string())
    }

    /// Convert legacy diagnostic to AriadneDiagnostic
    #[allow(dead_code)]
    fn convert_legacy_diagnostic_to_ariadne(
        &self,
        diagnostic: crate::diagnostics::Diagnostic,
    ) -> Result<AriadneDiagnostic, String> {
        let span = if let Some(loc) = &diagnostic.location {
            loc.offset..loc.offset + loc.length
        } else {
            0..0
        };

        Ok(AriadneDiagnostic {
            severity: diagnostic.severity,
            error_code: ErrorCode::new(9002), // Generic conversion error code
            message: diagnostic.message,
            span,
            help: None,
            note: None,
            related: Vec::new(),
        })
    }

    /// Create a diagnostic for parse errors
    #[allow(dead_code)]
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

    /// Convert a Diagnostic to AriadneDiagnostic with accurate span calculation
    fn convert_diagnostic_to_ariadne(
        &self,
        diagnostic: crate::diagnostics::Diagnostic,
        expression: &str,
    ) -> AriadneDiagnostic {
        let span = if let Some(loc) = &diagnostic.location {
            // Calculate accurate span based on the source location
            let start = std::cmp::min(loc.offset, expression.len());
            let end = std::cmp::min(loc.offset + loc.length, expression.len());
            start..end
        } else {
            // If no location is provided, try to find the span based on the diagnostic message
            self.calculate_span_from_diagnostic(&diagnostic, expression)
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

    /// Extract resource type from AST for proper context
    fn extract_resource_type_from_ast(
        &self,
        ast: &ExpressionNode,
        default_type: &TypeInfo,
    ) -> TypeInfo {
        match ast {
            ExpressionNode::Identifier(identifier) => {
                // Check if this identifier is a known resource type
                if self.is_likely_resource_type(&identifier.name) {
                    TypeInfo {
                        type_name: identifier.name.clone(),
                        singleton: Some(true),
                        is_empty: Some(false),
                        namespace: Some("FHIR".to_string()),
                        name: Some(identifier.name.clone()),
                    }
                } else {
                    default_type.clone()
                }
            }
            ExpressionNode::PropertyAccess(property_access) => {
                // For "Patient.name", extract "Patient" from the object
                self.extract_resource_type_from_ast(&property_access.object, default_type)
            }
            ExpressionNode::MethodCall(method_call) => {
                // For "Patient.name.where(...)", extract from the object
                self.extract_resource_type_from_ast(&method_call.object, default_type)
            }
            _ => default_type.clone(),
        }
    }

    /// Check if a name is likely a FHIR resource type (capitalized)
    fn is_likely_resource_type(&self, name: &str) -> bool {
        // Basic heuristic: FHIR resource types start with capital letter
        // In a full implementation, this would check against the model provider
        name.chars().next().is_some_and(|c| c.is_uppercase())
    }

    /// Calculate span from diagnostic message when no location is available
    fn calculate_span_from_diagnostic(
        &self,
        diagnostic: &crate::diagnostics::Diagnostic,
        expression: &str,
    ) -> std::ops::Range<usize> {
        // Try to extract identifiers or function names from the diagnostic message
        // and find their position in the expression

        // Look for property names in quotes
        if let Some(property_start) = diagnostic.message.find("'")
            && let Some(property_end) = diagnostic.message[property_start + 1..].find("'")
        {
            let property_name =
                &diagnostic.message[property_start + 1..property_start + 1 + property_end];
            if let Some(pos) = expression.find(property_name) {
                return pos..pos + property_name.len();
            }
        }

        // Look for function names in the message (not hardcoded resource types)
        let function_names = [
            "resourceType",
            "ofType",
            "is",
            "as",
            "where",
            "first",
            "last",
        ];
        for word in &function_names {
            if diagnostic.message.contains(word)
                && let Some(pos) = expression.find(word)
            {
                return pos..pos + word.len();
            }
        }

        // Fallback: span the entire expression
        0..expression.len()
    }

    /// Calculate span for a property name by finding its position in the source
    fn calculate_property_span(&self, property_name: &str) -> std::ops::Range<usize> {
        if let Some(source) = &self.current_source {
            // Look for the property name after a dot
            let pattern = format!(".{property_name}");
            if let Some(pos) = source.find(&pattern) {
                let start = pos + 1; // Skip the dot
                return start..start + property_name.len();
            }

            // If not found after dot, try to find it anywhere in the expression
            if let Some(pos) = source.find(property_name) {
                return pos..pos + property_name.len();
            }
        }

        // Fallback to default span
        0..0
    }

    /// Calculate span for a function name by finding its position in the source
    fn calculate_function_span(&self, function_name: &str) -> std::ops::Range<usize> {
        if let Some(source) = &self.current_source {
            // Look for the function name followed by an opening parenthesis
            let pattern = format!("{function_name}(");
            if let Some(pos) = source.find(&pattern) {
                return pos..pos + function_name.len();
            }

            // If not found with parenthesis, try to find it anywhere
            if let Some(pos) = source.find(function_name) {
                return pos..pos + function_name.len();
            }
        }

        // Fallback to default span
        0..0
    }

    /// Calculate span for a literal value by finding its position in the source
    fn calculate_literal_span(&self, literal_value: &str) -> std::ops::Range<usize> {
        if let Some(source) = &self.current_source {
            // Look for the literal value in quotes
            let patterns = [format!("'{literal_value}'"), format!("\"{literal_value}\"")];

            for pattern in &patterns {
                if let Some(pos) = source.find(pattern) {
                    let start = pos + 1; // Skip the opening quote
                    return start..start + literal_value.len();
                }
            }

            // If not found in quotes, try to find it anywhere
            if let Some(pos) = source.find(literal_value) {
                return pos..pos + literal_value.len();
            }
        }

        // Fallback to default span
        0..0
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
