//! Core FHIRPath static analyzer implementation
//!
//! This module provides comprehensive static analysis capabilities for FHIRPath expressions,
//! including syntax validation, type checking, semantic analysis, and optimization detection.

use crate::analyzer::optimization_detector::OptimizationDetector;
use crate::analyzer::property_validator::PropertyValidator;
use crate::analyzer::type_checker::{NodeId, TypeChecker, TypeInfo};
use crate::analyzer::visitor::{DefaultExpressionVisitor, ExpressionVisitor};
use crate::ast::expression::*;
use crate::ast::operator::BinaryOperator;
use crate::core::{ModelProvider, Result, SourceLocation};
use crate::diagnostics::{
    Diagnostic, DiagnosticProcessor, DiagnosticSeverity, ProcessedDiagnostic,
};
use crate::registry::FunctionRegistry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Comprehensive static analysis result
#[derive(Debug, Clone)]
pub struct StaticAnalysisResult {
    /// Diagnostic messages (errors and warnings)
    pub diagnostics: Vec<Diagnostic>,
    /// Analysis warnings with suggestions
    pub warnings: Vec<AnalysisWarning>,
    /// Optimization suggestions
    pub suggestions: Vec<OptimizationSuggestion>,
    /// Type information for each node
    pub type_info: HashMap<NodeId, TypeInfo>,
    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    /// Whether the analysis passed without errors
    pub is_valid: bool,
    /// Enhanced Ariadne diagnostics from PropertyValidator
    pub ariadne_diagnostics: Vec<crate::diagnostics::AriadneDiagnostic>,
}

/// Analysis warning with optional suggestion
#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub code: String,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub severity: DiagnosticSeverity,
    pub suggestion: Option<String>,
}

/// Optimization suggestion
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub kind: OptimizationKind,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub estimated_improvement: f32, // Performance improvement estimate (0.0-1.0)
}

/// Type of optimization suggestion
#[derive(Debug, Clone)]
pub enum OptimizationKind {
    /// Redundant condition that can be simplified
    RedundantCondition,
    /// Code that can never be reached
    UnreachableCode,
    /// Operation that may be computationally expensive
    ExpensiveOperation,
    /// Expression that could benefit from caching
    CachableExpression,
    /// Unnecessary type coercion
    TypeCoercion,
    /// Collection operation that can be optimized
    CollectionOptimization,
    /// Function call that can be simplified
    FunctionSimplification,
    /// Deep nesting that impacts performance
    DeepNesting,
    /// Property correction suggestion
    PropertyCorrection,
}

/// Complexity metrics for the analyzed expression
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity (number of decision points + 1)
    pub cyclomatic_complexity: usize,
    /// Maximum nesting depth in the expression
    pub expression_depth: usize,
    /// Number of function calls
    pub function_calls: usize,
    /// Number of property accesses
    pub property_accesses: usize,
    /// Number of collection operations
    pub collection_operations: usize,
    /// Estimated runtime cost (heuristic)
    pub estimated_runtime_cost: f32,
}

/// Comprehensive static analyzer for FHIRPath expressions
pub struct StaticAnalyzer {
    function_registry: Arc<FunctionRegistry>,
    type_checker: TypeChecker,
    property_validator: PropertyValidator,
    max_depth: usize,
    max_complexity: usize,
}

impl StaticAnalyzer {
    /// Create a new static analyzer
    pub fn new(
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            function_registry: function_registry.clone(),
            type_checker: TypeChecker::new(function_registry.clone(), model_provider.clone()),
            property_validator: PropertyValidator::new(
                model_provider.clone(),
                function_registry.clone(),
            ),
            max_depth: 50,
            max_complexity: 100,
        }
    }

    /// Set the maximum allowed expression depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set the maximum allowed complexity
    pub fn with_max_complexity(mut self, max_complexity: usize) -> Self {
        self.max_complexity = max_complexity;
        self
    }

    /// Perform comprehensive static analysis
    pub async fn analyze(&self, expression: &ExpressionNode) -> Result<StaticAnalysisResult> {
        let mut diagnostics = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Phase 1: Syntax and Structure Analysis
        self.analyze_syntax(expression, &mut diagnostics, &mut warnings)?;

        // Phase 2: Type Analysis
        let type_analysis = self.type_checker.analyze(expression)?;
        let type_info = type_analysis.type_info;

        // Convert type warnings to analysis warnings
        for type_warning in type_analysis.warnings {
            warnings.push(AnalysisWarning {
                code: type_warning.code,
                message: type_warning.message,
                location: None, // Would need source location mapping
                severity: DiagnosticSeverity::Warning,
                suggestion: type_warning.suggestion,
            });
        }

        // Phase 3: Property Validation
        let property_analysis = self.property_validator.validate(expression).await?;
        warnings.extend(property_analysis.warnings);

        // Collect enhanced Ariadne diagnostics from PropertyValidator
        let ariadne_diagnostics = property_analysis.ariadne_diagnostics;

        // Convert property suggestions to optimization suggestions
        for prop_suggestion in property_analysis.suggestions {
            suggestions.push(OptimizationSuggestion {
                kind: OptimizationKind::PropertyCorrection,
                message: format!(
                    "Property '{}' not found. Did you mean '{}'?",
                    prop_suggestion.invalid_property,
                    prop_suggestion.suggested_properties.join("' or '")
                ),
                location: prop_suggestion.location,
                estimated_improvement: 0.0, // Correctness improvement, not performance
            });
        }

        // Phase 4: Semantic Analysis
        self.analyze_semantics(expression, &type_info, &mut warnings, &mut suggestions)?;

        // Phase 5: Comprehensive Optimization Analysis
        let mut optimization_detector = OptimizationDetector::new();
        let optimization_analysis = optimization_detector.analyze(expression, &type_info)?;
        suggestions.extend(optimization_analysis.suggestions);

        // Phase 6: Complexity Metrics
        let complexity_metrics = self.calculate_complexity_metrics(expression)?;

        // Check complexity limits
        if complexity_metrics.expression_depth > self.max_depth {
            warnings.push(AnalysisWarning {
                code: "W001".to_string(),
                message: format!(
                    "Expression depth {} exceeds recommended maximum of {}",
                    complexity_metrics.expression_depth, self.max_depth
                ),
                location: None,
                severity: DiagnosticSeverity::Warning,
                suggestion: Some("Consider breaking the expression into smaller parts".to_string()),
            });
        }

        if complexity_metrics.cyclomatic_complexity > self.max_complexity {
            warnings.push(AnalysisWarning {
                code: "W002".to_string(),
                message: format!(
                    "Expression complexity {} exceeds recommended maximum of {}",
                    complexity_metrics.cyclomatic_complexity, self.max_complexity
                ),
                location: None,
                severity: DiagnosticSeverity::Warning,
                suggestion: Some("Consider simplifying the expression logic".to_string()),
            });
        }

        let is_valid = diagnostics.iter().all(|d| !d.severity.is_error());

        Ok(StaticAnalysisResult {
            diagnostics,
            warnings,
            suggestions,
            type_info,
            complexity_metrics,
            is_valid,
            ariadne_diagnostics,
        })
    }

    /// Analyze syntax and structure
    fn analyze_syntax(
        &self,
        expression: &ExpressionNode,
        diagnostics: &mut Vec<Diagnostic>,
        warnings: &mut Vec<AnalysisWarning>,
    ) -> Result<()> {
        let mut visitor = SyntaxAnalysisVisitor::new(diagnostics, warnings, self.max_depth);
        visitor.visit_expression(expression)?;
        Ok(())
    }

    /// Analyze semantics and constraints
    fn analyze_semantics(
        &self,
        expression: &ExpressionNode,
        type_info: &HashMap<NodeId, TypeInfo>,
        warnings: &mut Vec<AnalysisWarning>,
        suggestions: &mut Vec<OptimizationSuggestion>,
    ) -> Result<()> {
        let mut visitor =
            SemanticAnalysisVisitor::new(&self.function_registry, type_info, warnings, suggestions);
        visitor.visit_expression(expression)?;
        Ok(())
    }

    /// Calculate complexity metrics
    fn calculate_complexity_metrics(
        &self,
        expression: &ExpressionNode,
    ) -> Result<ComplexityMetrics> {
        let mut visitor = ComplexityCalculator::new();
        visitor.visit_expression(expression)?;
        Ok(visitor.into_metrics())
    }

    /// Create processed diagnostics with rich error reporting
    pub fn create_processed_diagnostics(
        &self,
        analysis_result: &StaticAnalysisResult,
        source: &str,
        filename: Option<&str>,
    ) -> std::result::Result<Vec<ProcessedDiagnostic>, Box<dyn std::error::Error>> {
        let mut processor = DiagnosticProcessor::new();
        Ok(processor.process_analysis(analysis_result, source, filename))
    }

    /// Render processed diagnostics to string for CLI output
    pub fn render_diagnostics(
        &self,
        analysis_result: &StaticAnalysisResult,
        source: &str,
        filename: Option<&str>,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let mut processor = DiagnosticProcessor::new();
        let processed = processor.process_analysis(analysis_result, source, filename);
        processor.render_diagnostics(&processed, source, filename)
    }
}

/// Visitor for syntax analysis
struct SyntaxAnalysisVisitor<'a> {
    diagnostics: &'a mut Vec<Diagnostic>,
    warnings: &'a mut Vec<AnalysisWarning>,
    depth: usize,
    max_depth: usize,
    max_depth_seen: usize,
    function_names: HashSet<String>,
}

impl<'a> SyntaxAnalysisVisitor<'a> {
    fn new(
        diagnostics: &'a mut Vec<Diagnostic>,
        warnings: &'a mut Vec<AnalysisWarning>,
        max_depth: usize,
    ) -> Self {
        let mut function_names = HashSet::new();

        // Add known FHIRPath functions
        let known_functions = [
            "first",
            "last",
            "tail",
            "skip",
            "take",
            "single",
            "exists",
            "empty",
            "count",
            "length",
            "where",
            "select",
            "all",
            "any",
            "distinct",
            "union",
            "contains",
            "in",
            "startsWith",
            "endsWith",
            "matches",
            "replace",
            "replaceMatches",
            "substring",
            "indexOf",
            "split",
            "join",
            "lower",
            "upper",
            "trim",
            "toString",
            "convertsToBoolean",
            "convertsToInteger",
            "convertsToDecimal",
            "convertsToDateTime",
            "convertsToDate",
            "convertsToTime",
            "toBoolean",
            "toInteger",
            "toDecimal",
            "toDateTime",
            "toDate",
            "toTime",
            "toQuantity",
            "sum",
            "min",
            "max",
            "avg",
            "abs",
            "ceiling",
            "floor",
            "round",
            "sqrt",
            "ln",
            "log",
            "power",
            "exp",
            "truncate",
            "now",
            "today",
            "timeOfDay",
            "trace",
            "hasValue",
            "getValue",
            "extension",
            "resolve",
            "descendants",
            "children",
            "binary",
            "encode",
            "decode",
        ];

        for func in &known_functions {
            function_names.insert(func.to_string());
        }

        Self {
            diagnostics,
            warnings,
            depth: 0,
            max_depth,
            max_depth_seen: 0,
            function_names,
        }
    }

    fn add_diagnostic(
        &mut self,
        code: &str,
        message: String,
        location: Option<SourceLocation>,
        severity: DiagnosticSeverity,
    ) {
        let mut diagnostic = Diagnostic::new(severity, code, message);
        if let Some(loc) = location {
            diagnostic = diagnostic.with_location(loc);
        }
        self.diagnostics.push(diagnostic);
    }

    fn add_warning(&mut self, code: &str, message: String, location: Option<SourceLocation>) {
        self.warnings.push(AnalysisWarning {
            code: code.to_string(),
            message,
            location,
            severity: DiagnosticSeverity::Warning,
            suggestion: None,
        });
    }
}

impl<'a> DefaultExpressionVisitor for SyntaxAnalysisVisitor<'a> {}

impl<'a> ExpressionVisitor for SyntaxAnalysisVisitor<'a> {
    type Output = Result<()>;

    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output {
        // Check if function is known
        if !self.function_names.contains(&call.name) {
            self.add_diagnostic(
                "E001",
                format!("Unknown function '{}'", call.name),
                call.location.clone(),
                DiagnosticSeverity::Error,
            );
        }

        // Check basic argument validation
        match call.name.as_str() {
            "substring" => {
                if call.arguments.len() < 1 || call.arguments.len() > 2 {
                    self.add_diagnostic(
                        "E002",
                        "substring() requires 1 or 2 arguments".to_string(),
                        call.location.clone(),
                        DiagnosticSeverity::Error,
                    );
                }
            }
            "skip" | "take" => {
                if call.arguments.len() != 1 {
                    self.add_diagnostic(
                        "E003",
                        format!("{}() requires exactly 1 argument", call.name),
                        call.location.clone(),
                        DiagnosticSeverity::Error,
                    );
                }
            }
            _ => {}
        }

        // Visit arguments
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }

        Ok(())
    }

    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output {
        self.depth += 1;
        if self.depth > self.max_depth_seen {
            self.max_depth_seen = self.depth;
        }

        // Check for excessive nesting
        if self.depth > self.max_depth {
            self.add_warning(
                "W003",
                format!(
                    "Deep property nesting may impact performance (depth: {})",
                    self.depth
                ),
                access.location.clone(),
            );
        }

        self.visit_expression(&access.object)?;
        self.depth -= 1;
        Ok(())
    }

    fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output {
        self.visit_expression(&filter.base)?;
        self.visit_expression(&filter.condition)?;
        Ok(())
    }

    fn visit_literal(&mut self, _literal: &LiteralNode) -> Self::Output {
        Ok(())
    }

    fn visit_identifier(&mut self, _identifier: &IdentifierNode) -> Self::Output {
        Ok(())
    }

    fn visit_method_call(&mut self, call: &MethodCallNode) -> Self::Output {
        self.visit_expression(&call.object)?;
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    fn visit_index_access(&mut self, access: &IndexAccessNode) -> Self::Output {
        self.visit_expression(&access.object)?;
        self.visit_expression(&access.index)?;
        Ok(())
    }

    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output {
        self.visit_expression(&binary.left)?;
        self.visit_expression(&binary.right)?;
        Ok(())
    }

    fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Self::Output {
        self.visit_expression(&unary.operand)?;
        Ok(())
    }

    fn visit_lambda(&mut self, lambda: &LambdaNode) -> Self::Output {
        self.visit_expression(&lambda.body)?;
        Ok(())
    }

    fn visit_collection(&mut self, collection: &CollectionNode) -> Self::Output {
        for element in &collection.elements {
            self.visit_expression(element)?;
        }
        Ok(())
    }

    fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Self::Output {
        self.visit_expression(expr)
    }

    fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Self::Output {
        self.visit_expression(&cast.expression)?;
        Ok(())
    }

    fn visit_union(&mut self, union: &UnionNode) -> Self::Output {
        self.visit_expression(&union.left)?;
        self.visit_expression(&union.right)?;
        Ok(())
    }

    fn visit_type_check(&mut self, check: &TypeCheckNode) -> Self::Output {
        self.visit_expression(&check.expression)?;
        Ok(())
    }

    fn visit_variable(&mut self, _variable: &VariableNode) -> Self::Output {
        Ok(())
    }

    fn visit_path(&mut self, path: &PathNode) -> Self::Output {
        self.visit_expression(&path.base)
    }
}

/// Visitor for semantic analysis
#[allow(dead_code)]
struct SemanticAnalysisVisitor<'a> {
    function_registry: &'a FunctionRegistry,
    type_info: &'a HashMap<NodeId, TypeInfo>,
    warnings: &'a mut Vec<AnalysisWarning>,
    suggestions: &'a mut Vec<OptimizationSuggestion>,
}

impl<'a> SemanticAnalysisVisitor<'a> {
    fn new(
        function_registry: &'a FunctionRegistry,
        type_info: &'a HashMap<NodeId, TypeInfo>,
        warnings: &'a mut Vec<AnalysisWarning>,
        suggestions: &'a mut Vec<OptimizationSuggestion>,
    ) -> Self {
        Self {
            function_registry,
            type_info,
            warnings,
            suggestions,
        }
    }
}

impl<'a> DefaultExpressionVisitor for SemanticAnalysisVisitor<'a> {}

impl<'a> ExpressionVisitor for SemanticAnalysisVisitor<'a> {
    type Output = Result<()>;

    fn visit_literal(&mut self, _literal: &LiteralNode) -> Self::Output {
        Ok(())
    }
    fn visit_identifier(&mut self, _identifier: &IdentifierNode) -> Self::Output {
        Ok(())
    }
    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output {
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }
    fn visit_method_call(&mut self, call: &MethodCallNode) -> Self::Output {
        self.visit_expression(&call.object)?;
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }
    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output {
        self.visit_expression(&access.object)
    }
    fn visit_index_access(&mut self, access: &IndexAccessNode) -> Self::Output {
        self.visit_expression(&access.object)?;
        self.visit_expression(&access.index)
    }
    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output {
        self.visit_expression(&binary.left)?;
        self.visit_expression(&binary.right)
    }
    fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Self::Output {
        self.visit_expression(&unary.operand)
    }
    fn visit_lambda(&mut self, lambda: &LambdaNode) -> Self::Output {
        self.visit_expression(&lambda.body)
    }
    fn visit_collection(&mut self, collection: &CollectionNode) -> Self::Output {
        for element in &collection.elements {
            self.visit_expression(element)?;
        }
        Ok(())
    }
    fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Self::Output {
        self.visit_expression(expr)
    }
    fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Self::Output {
        self.visit_expression(&cast.expression)
    }
    fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output {
        self.visit_expression(&filter.base)?;
        self.visit_expression(&filter.condition)
    }
    fn visit_union(&mut self, union: &UnionNode) -> Self::Output {
        self.visit_expression(&union.left)?;
        self.visit_expression(&union.right)
    }
    fn visit_type_check(&mut self, check: &TypeCheckNode) -> Self::Output {
        self.visit_expression(&check.expression)
    }
    fn visit_variable(&mut self, _variable: &VariableNode) -> Self::Output {
        Ok(())
    }
    fn visit_path(&mut self, path: &PathNode) -> Self::Output {
        self.visit_expression(&path.base)
    }
}

/// Visitor for complexity calculation
struct ComplexityCalculator {
    function_calls: usize,
    property_accesses: usize,
    collection_operations: usize,
    depth: usize,
    max_depth: usize,
    conditional_branches: usize,
}

impl ComplexityCalculator {
    fn new() -> Self {
        Self {
            function_calls: 0,
            property_accesses: 0,
            collection_operations: 0,
            depth: 0,
            max_depth: 0,
            conditional_branches: 0,
        }
    }

    fn into_metrics(self) -> ComplexityMetrics {
        let cyclomatic_complexity = 1 + self.conditional_branches;
        let estimated_runtime_cost = (self.function_calls as f32 * 10.0)
            + (self.property_accesses as f32 * 2.0)
            + (self.collection_operations as f32 * 15.0)
            + (self.max_depth as f32 * 1.5)
            + (cyclomatic_complexity as f32 * 5.0);

        ComplexityMetrics {
            cyclomatic_complexity,
            expression_depth: self.max_depth,
            function_calls: self.function_calls,
            property_accesses: self.property_accesses,
            collection_operations: self.collection_operations,
            estimated_runtime_cost,
        }
    }
}

impl DefaultExpressionVisitor for ComplexityCalculator {}

impl ExpressionVisitor for ComplexityCalculator {
    type Output = Result<()>;

    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output {
        self.function_calls += 1;

        // Count collection operations
        if matches!(
            call.name.as_str(),
            "where" | "select" | "all" | "any" | "exists" | "distinct"
        ) {
            self.collection_operations += 1;
        }

        // Count conditional complexity
        if matches!(
            call.name.as_str(),
            "where" | "select" | "all" | "any" | "exists"
        ) {
            self.conditional_branches += 1;
        }

        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }

    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output {
        self.property_accesses += 1;
        self.depth += 1;
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
        }

        self.visit_expression(&access.object)?;
        self.depth -= 1;
        Ok(())
    }

    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output {
        if matches!(binary.operator, BinaryOperator::And | BinaryOperator::Or) {
            self.conditional_branches += 1;
        }

        self.visit_expression(&binary.left)?;
        self.visit_expression(&binary.right)?;
        Ok(())
    }

    fn visit_literal(&mut self, _literal: &LiteralNode) -> Self::Output {
        Ok(())
    }
    fn visit_identifier(&mut self, _identifier: &IdentifierNode) -> Self::Output {
        Ok(())
    }
    fn visit_method_call(&mut self, call: &MethodCallNode) -> Self::Output {
        self.visit_expression(&call.object)?;
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }
        Ok(())
    }
    fn visit_index_access(&mut self, access: &IndexAccessNode) -> Self::Output {
        self.visit_expression(&access.object)?;
        self.visit_expression(&access.index)
    }
    fn visit_unary_operation(&mut self, unary: &UnaryOperationNode) -> Self::Output {
        self.visit_expression(&unary.operand)
    }
    fn visit_lambda(&mut self, lambda: &LambdaNode) -> Self::Output {
        self.visit_expression(&lambda.body)
    }
    fn visit_collection(&mut self, collection: &CollectionNode) -> Self::Output {
        for element in &collection.elements {
            self.visit_expression(element)?;
        }
        Ok(())
    }
    fn visit_parenthesized(&mut self, expr: &ExpressionNode) -> Self::Output {
        self.visit_expression(expr)
    }
    fn visit_type_cast(&mut self, cast: &TypeCastNode) -> Self::Output {
        self.visit_expression(&cast.expression)
    }
    fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output {
        self.visit_expression(&filter.base)?;
        self.visit_expression(&filter.condition)
    }
    fn visit_union(&mut self, union: &UnionNode) -> Self::Output {
        self.visit_expression(&union.left)?;
        self.visit_expression(&union.right)
    }
    fn visit_type_check(&mut self, check: &TypeCheckNode) -> Self::Output {
        self.visit_expression(&check.expression)
    }
    fn visit_variable(&mut self, _variable: &VariableNode) -> Self::Output {
        Ok(())
    }
    fn visit_path(&mut self, path: &PathNode) -> Self::Output {
        self.visit_expression(&path.base)
    }
}

/// Legacy analyzer for backward compatibility
#[derive(Debug, Default)]
pub struct Analyzer {
    _placeholder: (),
}

impl Analyzer {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    pub fn analyze(&self, _expression: &ExpressionNode) -> Result<LegacyAnalysisResult> {
        Ok(LegacyAnalysisResult::default())
    }
}

/// Legacy analysis result for backward compatibility
#[derive(Debug, Default)]
pub struct LegacyAnalysisResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub complexity: usize,
}

impl LegacyAnalysisResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            warnings: Vec::new(),
            complexity: 0,
        }
    }
}

// Alias for backward compatibility
pub type AnalysisResult = LegacyAnalysisResult;
