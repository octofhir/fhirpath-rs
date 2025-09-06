//! Comprehensive optimization detector for FHIRPath expressions
//!
//! This module provides intelligent analysis of FHIRPath expressions to detect performance issues,
//! suggest optimizations, and identify common anti-patterns that can be improved.

use crate::ast::expression::*;
use crate::ast::operator::BinaryOperator;
use crate::analyzer::visitor::{ExpressionVisitor, DefaultExpressionVisitor};
use crate::analyzer::{OptimizationSuggestion, OptimizationKind, AnalysisWarning};
use crate::analyzer::type_checker::{TypeInfo, NodeId};
use crate::core::{Result, SourceLocation};
use crate::diagnostics::DiagnosticSeverity;
use std::collections::{HashMap, HashSet};

/// Comprehensive optimization analysis result
#[derive(Debug, Clone)]
pub struct OptimizationAnalysisResult {
    /// Optimization suggestions found
    pub suggestions: Vec<OptimizationSuggestion>,
    /// Performance score from 0.0 (poor) to 1.0 (excellent)
    pub performance_score: f32,
    /// Complex issues that impact performance
    pub complexity_issues: Vec<ComplexityIssue>,
    /// Optimization patterns that were matched
    pub pattern_matches: Vec<PatternMatch>,
    /// Function call statistics
    pub function_call_stats: FunctionCallStats,
    /// Expression depth analysis
    pub depth_analysis: DepthAnalysis,
}

/// Performance issue found in the expression
#[derive(Debug, Clone)]
pub struct ComplexityIssue {
    /// Type of complexity issue
    pub issue_type: ComplexityIssueType,
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Location in the source code
    pub location: Option<SourceLocation>,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggested_fix: Option<String>,
    /// Estimated performance impact
    pub performance_impact: f32,
}

/// Types of complexity issues
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComplexityIssueType {
    /// Expression nesting is too deep
    DeepNesting,
    /// Subexpression is repeated multiple times
    RepeatedSubexpression,
    /// Operation has high computational cost
    ExpensiveOperation,
    /// Condition is redundant or always true/false
    RedundantCondition,
    /// Code path can never be reached
    UnreachableCode,
    /// Filter operation is inefficient
    InefficientFilter,
    /// Missing index access optimization
    MissingIndex,
    /// Unnecessary iteration over collection
    UnnecessaryIteration,
    /// Function call can be simplified
    SimplifiableFunction,
    /// Property access can be optimized
    PropertyAccessOptimization,
}

/// Severity levels for performance issues
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    /// Minor impact, mostly for code quality
    Low,
    /// Noticeable performance impact
    Medium,
    /// Significant performance impact
    High,
    /// Major performance bottleneck
    Critical,
}

/// Pattern that can be optimized
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// Type of optimization pattern
    pub pattern_type: PatternType,
    /// Location in the source
    pub location: Option<SourceLocation>,
    /// Original code pattern
    pub original: String,
    /// Suggested replacement
    pub suggested: String,
    /// Benefit description
    pub benefit: String,
    /// Estimated performance improvement
    pub improvement_factor: f32,
}

/// Types of optimization patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternType {
    // Performance patterns
    /// Multiple filters can be combined
    FilterCombination,
    /// Direct index access is more efficient
    IndexAccess,
    /// Early exit can avoid unnecessary computation
    EarlyExit,
    /// Expression result can be cached
    CacheableExpression,
    /// Expensive function can be replaced
    ExpensiveFunctionReplacement,
    
    // Readability patterns
    /// Complex logic can be simplified
    SimplifyLogic,
    /// Complex expression should be split
    ExtractVariable,
    /// Reduce overall complexity
    ReduceComplexity,
    /// Combine related operations
    CombineOperations,
    
    // Correctness patterns
    /// Add null safety check
    NullSafety,
    /// Improve type safety
    TypeSafety,
    /// Add empty collection check
    EmptyCheck,
    /// Fix potential reference issues
    ReferenceCheck,
}

/// Statistics about function calls in the expression
#[derive(Debug, Clone)]
pub struct FunctionCallStats {
    /// Total number of function calls
    pub total_calls: usize,
    /// Number of expensive function calls
    pub expensive_calls: usize,
    /// Number of cacheable function calls
    pub cacheable_calls: usize,
    /// Most frequently called functions
    pub frequent_functions: Vec<(String, usize)>,
    /// Functions that could be replaced with more efficient alternatives
    pub replaceable_functions: Vec<String>,
}

/// Analysis of expression depth and nesting
#[derive(Debug, Clone)]
pub struct DepthAnalysis {
    /// Maximum depth of property access chains
    pub max_property_depth: usize,
    /// Maximum nesting level of expressions
    pub max_expression_depth: usize,
    /// Number of deeply nested expressions (depth > threshold)
    pub deep_expressions: usize,
    /// Locations where depth could be reduced
    pub depth_reduction_opportunities: Vec<SourceLocation>,
}

/// Advanced optimization detector
pub struct OptimizationDetector {
    /// Known expensive functions and their alternatives
    expensive_functions: HashMap<String, Vec<String>>,
    /// Functions that can be cached for better performance
    cacheable_functions: HashSet<String>,
    /// Pure functions that have no side effects
    pure_functions: HashSet<String>,
    /// Pattern matchers for common optimizations
    pattern_matchers: Vec<Box<dyn PatternMatcher>>,
    /// Function call frequency tracking
    function_calls: HashMap<String, usize>,
    /// Maximum allowed expression depth
    max_allowed_depth: usize,
    /// Performance impact weights
    impact_weights: HashMap<ComplexityIssueType, f32>,
}

impl OptimizationDetector {
    /// Create a new optimization detector with default configuration
    pub fn new() -> Self {
        let mut detector = Self {
            expensive_functions: HashMap::new(),
            cacheable_functions: HashSet::new(),
            pure_functions: HashSet::new(),
            pattern_matchers: Vec::new(),
            function_calls: HashMap::new(),
            max_allowed_depth: 5,
            impact_weights: HashMap::new(),
        };
        
        detector.initialize_function_classifications();
        detector.initialize_pattern_matchers();
        detector.initialize_impact_weights();
        detector
    }

    /// Perform comprehensive optimization analysis on an expression
    pub fn analyze(&mut self, expression: &ExpressionNode, type_info: &HashMap<NodeId, TypeInfo>) -> Result<OptimizationAnalysisResult> {
        // Reset state for new analysis
        self.function_calls.clear();
        
        let mut suggestions = Vec::new();
        let mut complexity_issues = Vec::new();
        let mut pattern_matches = Vec::new();

        // Phase 1: Detect anti-patterns and performance issues
        self.detect_performance_issues(expression, type_info, &mut suggestions, &mut complexity_issues)?;

        // Phase 2: Find optimization patterns
        self.detect_optimization_patterns(expression, &mut pattern_matches)?;

        // Phase 3: Analyze function call patterns
        let function_stats = self.analyze_function_calls(expression)?;

        // Phase 4: Analyze expression depth and complexity
        let depth_analysis = self.analyze_expression_depth(expression)?;

        // Phase 5: Calculate overall performance score
        let performance_score = self.calculate_performance_score(&suggestions, &complexity_issues, &depth_analysis);

        // Convert pattern matches to suggestions
        for pattern in &pattern_matches {
            suggestions.push(OptimizationSuggestion {
                kind: self.pattern_type_to_optimization_kind(&pattern.pattern_type),
                message: format!("{}: {} → {}", pattern.benefit, pattern.original, pattern.suggested),
                location: pattern.location.clone(),
                estimated_improvement: pattern.improvement_factor,
            });
        }

        Ok(OptimizationAnalysisResult {
            suggestions,
            performance_score,
            complexity_issues,
            pattern_matches,
            function_call_stats: function_stats,
            depth_analysis,
        })
    }

    /// Detect performance issues and anti-patterns
    fn detect_performance_issues(
        &mut self,
        expression: &ExpressionNode,
        type_info: &HashMap<NodeId, TypeInfo>,
        suggestions: &mut Vec<OptimizationSuggestion>,
        complexity_issues: &mut Vec<ComplexityIssue>,
    ) -> Result<()> {
        let mut visitor = PerformanceIssueVisitor {
            detector: self,
            type_info,
            suggestions,
            complexity_issues,
            current_depth: 0,
            property_access_depth: 0,
            filter_chain_length: 0,
            seen_expressions: HashMap::new(),
        };

        visitor.visit_expression(expression)?;
        Ok(())
    }

    /// Find common optimization patterns
    fn detect_optimization_patterns(
        &self,
        expression: &ExpressionNode,
        pattern_matches: &mut Vec<PatternMatch>,
    ) -> Result<()> {
        for matcher in &self.pattern_matchers {
            matcher.find_matches(expression, pattern_matches)?;
        }
        Ok(())
    }

    /// Analyze function call patterns for optimization opportunities
    fn analyze_function_calls(&self, expression: &ExpressionNode) -> Result<FunctionCallStats> {
        let mut visitor = FunctionCallAnalyzer {
            function_calls: HashMap::new(),
            expensive_calls: 0,
            cacheable_calls: 0,
            expensive_functions: &self.expensive_functions,
            cacheable_functions: &self.cacheable_functions,
        };

        visitor.analyze_expression(expression)?;

        // Sort functions by frequency
        let mut frequent_functions: Vec<(String, usize)> = visitor.function_calls.into_iter().collect();
        frequent_functions.sort_by(|a, b| b.1.cmp(&a.1));

        // Find replaceable functions
        let replaceable_functions: Vec<String> = self.expensive_functions.keys()
            .filter(|func| frequent_functions.iter().any(|(name, _)| name == *func))
            .cloned()
            .collect();

        Ok(FunctionCallStats {
            total_calls: frequent_functions.iter().map(|(_, count)| count).sum(),
            expensive_calls: visitor.expensive_calls,
            cacheable_calls: visitor.cacheable_calls,
            frequent_functions: frequent_functions.into_iter().take(10).collect(), // Top 10
            replaceable_functions,
        })
    }

    /// Analyze expression depth and nesting
    fn analyze_expression_depth(&self, expression: &ExpressionNode) -> Result<DepthAnalysis> {
        let mut visitor = DepthAnalyzer {
            max_property_depth: 0,
            max_expression_depth: 0,
            current_property_depth: 0,
            current_expression_depth: 0,
            deep_expressions: 0,
            depth_reduction_opportunities: Vec::new(),
            max_allowed_depth: self.max_allowed_depth,
        };

        visitor.analyze_depth(expression)?;

        Ok(DepthAnalysis {
            max_property_depth: visitor.max_property_depth,
            max_expression_depth: visitor.max_expression_depth,
            deep_expressions: visitor.deep_expressions,
            depth_reduction_opportunities: visitor.depth_reduction_opportunities,
        })
    }

    /// Calculate overall performance score based on analysis results
    fn calculate_performance_score(
        &self,
        suggestions: &[OptimizationSuggestion],
        complexity_issues: &[ComplexityIssue],
        depth_analysis: &DepthAnalysis,
    ) -> f32 {
        let mut score = 1.0;

        // Deduct for optimization suggestions
        for suggestion in suggestions {
            let deduction = match suggestion.kind {
                OptimizationKind::ExpensiveOperation => 0.25,
                OptimizationKind::CollectionOptimization => 0.20,
                OptimizationKind::RedundantCondition => 0.15,
                OptimizationKind::DeepNesting => 0.15,
                OptimizationKind::CachableExpression => 0.10,
                OptimizationKind::FunctionSimplification => 0.10,
                OptimizationKind::UnreachableCode => 0.05,
                OptimizationKind::TypeCoercion => 0.05,
                OptimizationKind::PropertyCorrection => 0.0, // Correctness, not performance
            };
            score -= deduction * suggestion.estimated_improvement;
        }

        // Deduct for complexity issues
        for issue in complexity_issues {
            let base_deduction = match issue.severity {
                IssueSeverity::Critical => 0.30,
                IssueSeverity::High => 0.20,
                IssueSeverity::Medium => 0.10,
                IssueSeverity::Low => 0.05,
            };
            score -= base_deduction * issue.performance_impact;
        }

        // Deduct for excessive depth
        if depth_analysis.max_expression_depth > self.max_allowed_depth {
            let depth_penalty = (depth_analysis.max_expression_depth - self.max_allowed_depth) as f32 * 0.05;
            score -= depth_penalty;
        }

        score.max(0.0).min(1.0)
    }

    /// Convert pattern type to optimization kind
    fn pattern_type_to_optimization_kind(&self, pattern_type: &PatternType) -> OptimizationKind {
        match pattern_type {
            PatternType::FilterCombination => OptimizationKind::CollectionOptimization,
            PatternType::IndexAccess => OptimizationKind::CollectionOptimization,
            PatternType::EarlyExit => OptimizationKind::ExpensiveOperation,
            PatternType::CacheableExpression => OptimizationKind::CachableExpression,
            PatternType::ExpensiveFunctionReplacement => OptimizationKind::FunctionSimplification,
            PatternType::SimplifyLogic => OptimizationKind::RedundantCondition,
            PatternType::ExtractVariable => OptimizationKind::DeepNesting,
            PatternType::ReduceComplexity => OptimizationKind::DeepNesting,
            PatternType::CombineOperations => OptimizationKind::CollectionOptimization,
            PatternType::NullSafety => OptimizationKind::PropertyCorrection,
            PatternType::TypeSafety => OptimizationKind::TypeCoercion,
            PatternType::EmptyCheck => OptimizationKind::PropertyCorrection,
            PatternType::ReferenceCheck => OptimizationKind::PropertyCorrection,
        }
    }

    /// Initialize function classifications
    fn initialize_function_classifications(&mut self) {
        // Expensive functions with their alternatives
        self.expensive_functions.insert("resolve".to_string(), vec!["ofType".to_string()]);
        self.expensive_functions.insert("descendants".to_string(), vec!["children".to_string()]);
        self.expensive_functions.insert("descendantsAndSelf".to_string(), vec!["select".to_string()]);

        // Cacheable functions (pure functions with deterministic results)
        self.cacheable_functions.extend([
            "length".to_string(),
            "substring".to_string(),
            "abs".to_string(),
            "ceiling".to_string(),
            "floor".to_string(),
            "round".to_string(),
            "sqrt".to_string(),
            "ln".to_string(),
            "log".to_string(),
            "exp".to_string(),
            "power".to_string(),
        ]);

        // Pure functions (no side effects, consistent results)
        self.pure_functions.extend([
            "count".to_string(),
            "empty".to_string(),
            "exists".to_string(),
            "first".to_string(),
            "last".to_string(),
            "tail".to_string(),
            "take".to_string(),
            "skip".to_string(),
            "distinct".to_string(),
            "union".to_string(),
            "intersect".to_string(),
            "exclude".to_string(),
        ]);
    }

    /// Initialize pattern matchers
    fn initialize_pattern_matchers(&mut self) {
        self.pattern_matchers.push(Box::new(FilterCombinationMatcher));
        self.pattern_matchers.push(Box::new(IndexAccessOptimizationMatcher));
        self.pattern_matchers.push(Box::new(NullSafetyMatcher));
        self.pattern_matchers.push(Box::new(LogicSimplificationMatcher));
        self.pattern_matchers.push(Box::new(CacheableExpressionMatcher::new(&self.cacheable_functions)));
        self.pattern_matchers.push(Box::new(ExpensiveFunctionMatcher::new(&self.expensive_functions)));
    }

    /// Initialize performance impact weights
    fn initialize_impact_weights(&mut self) {
        self.impact_weights.insert(ComplexityIssueType::ExpensiveOperation, 0.8);
        self.impact_weights.insert(ComplexityIssueType::InefficientFilter, 0.6);
        self.impact_weights.insert(ComplexityIssueType::UnnecessaryIteration, 0.5);
        self.impact_weights.insert(ComplexityIssueType::DeepNesting, 0.4);
        self.impact_weights.insert(ComplexityIssueType::RepeatedSubexpression, 0.4);
        self.impact_weights.insert(ComplexityIssueType::SimplifiableFunction, 0.3);
        self.impact_weights.insert(ComplexityIssueType::PropertyAccessOptimization, 0.3);
        self.impact_weights.insert(ComplexityIssueType::MissingIndex, 0.2);
        self.impact_weights.insert(ComplexityIssueType::RedundantCondition, 0.2);
        self.impact_weights.insert(ComplexityIssueType::UnreachableCode, 0.1);
    }
}

impl Default for OptimizationDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Visitor to detect performance issues
struct PerformanceIssueVisitor<'a> {
    detector: &'a mut OptimizationDetector,
    type_info: &'a HashMap<NodeId, TypeInfo>,
    suggestions: &'a mut Vec<OptimizationSuggestion>,
    complexity_issues: &'a mut Vec<ComplexityIssue>,
    current_depth: usize,
    property_access_depth: usize,
    filter_chain_length: usize,
    seen_expressions: HashMap<String, Vec<SourceLocation>>, // For detecting repeated expressions
}

impl<'a> ExpressionVisitor for PerformanceIssueVisitor<'a> {
    type Output = Result<()>;

    fn visit_function_call(&mut self, call: &FunctionCallNode) -> Self::Output {
        // Track function call frequency
        *self.detector.function_calls.entry(call.name.clone()).or_insert(0) += 1;

        // Check for expensive functions
        if let Some(alternatives) = self.detector.expensive_functions.get(&call.name) {
            self.complexity_issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::ExpensiveOperation,
                severity: IssueSeverity::High,
                location: call.location.clone(),
                description: format!("Function '{}' is computationally expensive", call.name),
                suggested_fix: Some(format!("Consider using alternatives: {}", alternatives.join(", "))),
                performance_impact: 0.7,
            });
        }

        // Check for functions in filters that could be optimized
        if self.filter_chain_length > 0 {
            match call.name.as_str() {
                "count" => {
                    self.suggestions.push(OptimizationSuggestion {
                        kind: OptimizationKind::CollectionOptimization,
                        message: "Using count() in filter conditions is inefficient. Consider exists() or empty()".to_string(),
                        location: call.location.clone(),
                        estimated_improvement: 0.4,
                    });
                },
                "length" => {
                    self.suggestions.push(OptimizationSuggestion {
                        kind: OptimizationKind::CollectionOptimization,
                        message: "Using length() in filter conditions can be optimized with empty()".to_string(),
                        location: call.location.clone(),
                        estimated_improvement: 0.3,
                    });
                },
                _ => {}
            }
        }

        // Visit arguments
        for arg in &call.arguments {
            self.visit_expression(arg)?;
        }

        Ok(())
    }

    fn visit_property_access(&mut self, access: &PropertyAccessNode) -> Self::Output {
        self.property_access_depth += 1;

        // Check for excessive property chaining
        if self.property_access_depth > self.detector.max_allowed_depth {
            self.complexity_issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::DeepNesting,
                severity: if self.property_access_depth > 8 { IssueSeverity::High } else { IssueSeverity::Medium },
                location: access.location.clone(),
                description: format!("Deep property chaining ({} levels) impacts performance", self.property_access_depth),
                suggested_fix: Some("Consider using intermediate variables or where() clauses".to_string()),
                performance_impact: (self.property_access_depth as f32 - 5.0) * 0.1,
            });
        }

        // Check for common null-prone properties
        if matches!(access.property.as_str(), "family" | "given" | "text" | "display" | "value") {
            self.suggestions.push(OptimizationSuggestion {
                kind: OptimizationKind::PropertyCorrection,
                message: format!("Property '{}' might be null. Consider adding exists() check", access.property),
                location: access.location.clone(),
                estimated_improvement: 0.0, // Correctness improvement
            });
        }

        self.visit_expression(&access.object)?;
        self.property_access_depth -= 1;
        Ok(())
    }

    fn visit_filter(&mut self, filter: &FilterNode) -> Self::Output {
        self.filter_chain_length += 1;

        // Detect chained filters that could be combined
        if self.filter_chain_length > 1 {
            self.suggestions.push(OptimizationSuggestion {
                kind: OptimizationKind::CollectionOptimization,
                message: "Multiple sequential filters can be combined with 'and' for better performance".to_string(),
                location: filter.location.clone(),
                estimated_improvement: 0.25,
            });
        }

        // Check for complex filter conditions
        let condition_complexity = self.estimate_expression_complexity(&filter.condition);
        if condition_complexity > 10.0 {
            self.complexity_issues.push(ComplexityIssue {
                issue_type: ComplexityIssueType::InefficientFilter,
                severity: IssueSeverity::Medium,
                location: filter.location.clone(),
                description: "Complex filter condition may impact performance".to_string(),
                suggested_fix: Some("Consider simplifying the filter condition or breaking into multiple steps".to_string()),
                performance_impact: condition_complexity * 0.05,
            });
        }

        self.visit_expression(&filter.base)?;
        self.visit_expression(&filter.condition)?;

        self.filter_chain_length -= 1;
        Ok(())
    }

    fn visit_binary_operation(&mut self, binary: &BinaryOperationNode) -> Self::Output {
        // Check for redundant conditions
        if matches!(binary.operator, BinaryOperator::And | BinaryOperator::Or) {
            let left_str = self.expression_to_string(&binary.left);
            let right_str = self.expression_to_string(&binary.right);
            
            if left_str == right_str {
                self.suggestions.push(OptimizationSuggestion {
                    kind: OptimizationKind::RedundantCondition,
                    message: "Redundant condition: both sides are identical".to_string(),
                    location: binary.location.clone(),
                    estimated_improvement: 0.15,
                });
            }
        }

        // Check for always true/false conditions
        if let (ExpressionNode::Literal(left), ExpressionNode::Literal(right)) = (binary.left.as_ref(), binary.right.as_ref()) {
            self.check_static_boolean_conditions(binary, left, right);
        }

        self.visit_expression(&binary.left)?;
        self.visit_expression(&binary.right)?;
        Ok(())
    }
}

impl<'a> PerformanceIssueVisitor<'a> {
    fn estimate_expression_complexity(&self, expr: &ExpressionNode) -> f32 {
        // Simplified complexity estimation
        match expr {
            ExpressionNode::Literal(_) | ExpressionNode::Identifier(_) => 1.0,
            ExpressionNode::PropertyAccess(access) => 2.0 + self.estimate_expression_complexity(&access.object),
            ExpressionNode::FunctionCall(call) => {
                let base_cost = if self.detector.expensive_functions.contains_key(&call.name) { 5.0 } else { 2.0 };
                base_cost + call.arguments.iter().map(|arg| self.estimate_expression_complexity(arg)).sum::<f32>()
            },
            ExpressionNode::BinaryOperation(binary) => {
                2.0 + self.estimate_expression_complexity(&binary.left) + self.estimate_expression_complexity(&binary.right)
            },
            ExpressionNode::Filter(filter) => {
                3.0 + self.estimate_expression_complexity(&filter.base) + self.estimate_expression_complexity(&filter.condition)
            },
            _ => 1.5, // Default complexity for other node types
        }
    }

    fn expression_to_string(&self, expr: &ExpressionNode) -> String {
        // Simplified expression string representation for comparison
        match expr {
            ExpressionNode::Identifier(id) => id.name.clone(),
            ExpressionNode::PropertyAccess(access) => format!("{}.{}", self.expression_to_string(&access.object), access.property),
            ExpressionNode::Literal(literal) => format!("{:?}", literal.value),
            _ => "complex_expression".to_string(),
        }
    }

    fn check_static_boolean_conditions(&mut self, binary: &BinaryOperationNode, left: &LiteralNode, right: &LiteralNode) {
        if let (crate::ast::literal::LiteralValue::Boolean(left_val), crate::ast::literal::LiteralValue::Boolean(right_val)) = (&left.value, &right.value) {
            match binary.operator {
                BinaryOperator::And => {
                    if !left_val || !right_val {
                        self.suggestions.push(OptimizationSuggestion {
                            kind: OptimizationKind::UnreachableCode,
                            message: "Condition is always false and can be simplified to 'false'".to_string(),
                            location: binary.location.clone(),
                            estimated_improvement: 0.1,
                        });
                    } else {
                        self.suggestions.push(OptimizationSuggestion {
                            kind: OptimizationKind::RedundantCondition,
                            message: "Condition is always true and can be simplified to 'true'".to_string(),
                            location: binary.location.clone(),
                            estimated_improvement: 0.1,
                        });
                    }
                },
                BinaryOperator::Or => {
                    if *left_val || *right_val {
                        self.suggestions.push(OptimizationSuggestion {
                            kind: OptimizationKind::RedundantCondition,
                            message: "Condition is always true and can be simplified to 'true'".to_string(),
                            location: binary.location.clone(),
                            estimated_improvement: 0.1,
                        });
                    }
                },
                _ => {}
            }
        }
    }

    // Default implementations for missing methods
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
        self.visit_expression(expr)?;
        Ok(())
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
        self.visit_expression(&path.base)?;
        Ok(())
    }
}

/// Visitor to analyze function calls
struct FunctionCallAnalyzer<'a> {
    function_calls: HashMap<String, usize>,
    expensive_calls: usize,
    cacheable_calls: usize,
    expensive_functions: &'a HashMap<String, Vec<String>>,
    cacheable_functions: &'a HashSet<String>,
}

impl<'a> FunctionCallAnalyzer<'a> {
    fn analyze_expression(&mut self, expression: &ExpressionNode) -> Result<()> {
        match expression {
            ExpressionNode::FunctionCall(call) => {
                *self.function_calls.entry(call.name.clone()).or_insert(0) += 1;

                if self.expensive_functions.contains_key(&call.name) {
                    self.expensive_calls += 1;
                }

                if self.cacheable_functions.contains(&call.name) {
                    self.cacheable_calls += 1;
                }

                for arg in &call.arguments {
                    self.analyze_expression(arg)?;
                }
            },
            ExpressionNode::MethodCall(call) => {
                self.analyze_expression(&call.object)?;
                for arg in &call.arguments {
                    self.analyze_expression(arg)?;
                }
            },
            ExpressionNode::PropertyAccess(access) => {
                self.analyze_expression(&access.object)?;
            },
            ExpressionNode::IndexAccess(access) => {
                self.analyze_expression(&access.object)?;
                self.analyze_expression(&access.index)?;
            },
            ExpressionNode::BinaryOperation(binary) => {
                self.analyze_expression(&binary.left)?;
                self.analyze_expression(&binary.right)?;
            },
            ExpressionNode::UnaryOperation(unary) => {
                self.analyze_expression(&unary.operand)?;
            },
            ExpressionNode::Lambda(lambda) => {
                self.analyze_expression(&lambda.body)?;
            },
            ExpressionNode::Collection(collection) => {
                for element in &collection.elements {
                    self.analyze_expression(element)?;
                }
            },
            ExpressionNode::Parenthesized(expr) => {
                self.analyze_expression(expr)?;
            },
            ExpressionNode::TypeCast(cast) => {
                self.analyze_expression(&cast.expression)?;
            },
            ExpressionNode::Filter(filter) => {
                self.analyze_expression(&filter.base)?;
                self.analyze_expression(&filter.condition)?;
            },
            ExpressionNode::Union(union) => {
                self.analyze_expression(&union.left)?;
                self.analyze_expression(&union.right)?;
            },
            ExpressionNode::TypeCheck(check) => {
                self.analyze_expression(&check.expression)?;
            },
            ExpressionNode::Variable(_) | ExpressionNode::Identifier(_) | ExpressionNode::Literal(_) => {
                // Leaf nodes, no further analysis needed
            },
            ExpressionNode::Path(path) => {
                self.analyze_expression(&path.base)?;
            },
        }
        Ok(())
    }
}

/// Visitor to analyze expression depth
struct DepthAnalyzer {
    max_property_depth: usize,
    max_expression_depth: usize,
    current_property_depth: usize,
    current_expression_depth: usize,
    deep_expressions: usize,
    depth_reduction_opportunities: Vec<SourceLocation>,
    max_allowed_depth: usize,
}

impl DepthAnalyzer {
    fn analyze_depth(&mut self, expr: &ExpressionNode) -> Result<()> {
        self.current_expression_depth += 1;
        self.max_expression_depth = self.max_expression_depth.max(self.current_expression_depth);

        match expr {
            ExpressionNode::PropertyAccess(access) => {
                self.current_property_depth += 1;
                self.max_property_depth = self.max_property_depth.max(self.current_property_depth);

                if self.current_property_depth > self.max_allowed_depth {
                    self.deep_expressions += 1;
                    if let Some(location) = access.location.clone() {
                        self.depth_reduction_opportunities.push(location);
                    }
                }

                self.analyze_depth(&access.object)?;
                self.current_property_depth -= 1;
            },
            ExpressionNode::FunctionCall(call) => {
                for arg in &call.arguments {
                    self.analyze_depth(arg)?;
                }
            },
            ExpressionNode::MethodCall(call) => {
                self.analyze_depth(&call.object)?;
                for arg in &call.arguments {
                    self.analyze_depth(arg)?;
                }
            },
            ExpressionNode::IndexAccess(access) => {
                self.analyze_depth(&access.object)?;
                self.analyze_depth(&access.index)?;
            },
            ExpressionNode::BinaryOperation(binary) => {
                self.analyze_depth(&binary.left)?;
                self.analyze_depth(&binary.right)?;
            },
            ExpressionNode::UnaryOperation(unary) => {
                self.analyze_depth(&unary.operand)?;
            },
            ExpressionNode::Lambda(lambda) => {
                self.analyze_depth(&lambda.body)?;
            },
            ExpressionNode::Collection(collection) => {
                for element in &collection.elements {
                    self.analyze_depth(element)?;
                }
            },
            ExpressionNode::Parenthesized(expr) => {
                self.analyze_depth(expr)?;
            },
            ExpressionNode::TypeCast(cast) => {
                self.analyze_depth(&cast.expression)?;
            },
            ExpressionNode::Filter(filter) => {
                self.analyze_depth(&filter.base)?;
                self.analyze_depth(&filter.condition)?;
            },
            ExpressionNode::Union(union) => {
                self.analyze_depth(&union.left)?;
                self.analyze_depth(&union.right)?;
            },
            ExpressionNode::TypeCheck(check) => {
                self.analyze_depth(&check.expression)?;
            },
            ExpressionNode::Path(path) => {
                self.analyze_depth(&path.base)?;
            },
            ExpressionNode::Literal(_) | ExpressionNode::Identifier(_) | ExpressionNode::Variable(_) => {
                // Leaf nodes, no further analysis needed
            },
        }

        self.current_expression_depth -= 1;
        Ok(())
    }
}

/// Trait for pattern matchers
pub trait PatternMatcher: Send + Sync {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()>;
}

/// Matcher for filter combination patterns
pub struct FilterCombinationMatcher;

impl PatternMatcher for FilterCombinationMatcher {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()> {
        // Look for .where(A).where(B) patterns that can be combined as .where(A and B)
        if let ExpressionNode::MethodCall(outer_call) = expression {
            if outer_call.method == "where" && outer_call.arguments.len() == 1 {
                if let ExpressionNode::MethodCall(inner_call) = outer_call.object.as_ref() {
                    if inner_call.method == "where" && inner_call.arguments.len() == 1 {
                        matches.push(PatternMatch {
                            pattern_type: PatternType::FilterCombination,
                            location: outer_call.location.clone(),
                            original: ".where(A).where(B)".to_string(),
                            suggested: ".where(A and B)".to_string(),
                            benefit: "Combine filters for better performance".to_string(),
                            improvement_factor: 0.25,
                        });
                    }
                }
            }
        }

        // Recursively check sub-expressions
        match expression {
            ExpressionNode::FunctionCall(call) => {
                for arg in &call.arguments {
                    self.find_matches(arg, matches)?;
                }
            },
            ExpressionNode::MethodCall(call) => {
                self.find_matches(&call.object, matches)?;
                for arg in &call.arguments {
                    self.find_matches(arg, matches)?;
                }
            },
            ExpressionNode::PropertyAccess(access) => {
                self.find_matches(&access.object, matches)?;
            },
            ExpressionNode::BinaryOperation(binary) => {
                self.find_matches(&binary.left, matches)?;
                self.find_matches(&binary.right, matches)?;
            },
            ExpressionNode::Filter(filter) => {
                self.find_matches(&filter.base, matches)?;
                self.find_matches(&filter.condition, matches)?;
            },
            _ => {},
        }

        Ok(())
    }
}

/// Matcher for index access optimization
pub struct IndexAccessOptimizationMatcher;

impl PatternMatcher for IndexAccessOptimizationMatcher {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()> {
        // Look for .first() on filtered collections that could use indexing
        if let ExpressionNode::MethodCall(call) = expression {
            if call.method == "first" && call.arguments.is_empty() {
                if let ExpressionNode::Filter(_) = call.object.as_ref() {
                    matches.push(PatternMatch {
                        pattern_type: PatternType::IndexAccess,
                        location: call.location.clone(),
                        original: ".where(condition).first()".to_string(),
                        suggested: "[0] or direct access pattern".to_string(),
                        benefit: "Use direct index access when possible".to_string(),
                        improvement_factor: 0.3,
                    });
                }
            }
        }

        Ok(())
    }
}

/// Matcher for null safety patterns
pub struct NullSafetyMatcher;

impl PatternMatcher for NullSafetyMatcher {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()> {
        if let ExpressionNode::PropertyAccess(access) = expression {
            if matches!(access.property.as_str(), "family" | "given" | "text" | "display" | "value") {
                matches.push(PatternMatch {
                    pattern_type: PatternType::NullSafety,
                    location: access.location.clone(),
                    original: format!(".{}", access.property),
                    suggested: format!(".{}.exists().{}", access.property, access.property),
                    benefit: "Add null safety check".to_string(),
                    improvement_factor: 0.0, // Correctness, not performance
                });
            }
        }

        Ok(())
    }
}

/// Matcher for logic simplification
pub struct LogicSimplificationMatcher;

impl PatternMatcher for LogicSimplificationMatcher {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()> {
        if let ExpressionNode::BinaryOperation(binary) = expression {
            if binary.operator == BinaryOperator::Or {
                // Look for patterns like: A = 'X' or A = 'Y'
                if self.is_equality_chain(binary) {
                    matches.push(PatternMatch {
                        pattern_type: PatternType::SimplifyLogic,
                        location: binary.location.clone(),
                        original: "A = 'X' or A = 'Y' or A = 'Z'".to_string(),
                        suggested: "A in ('X', 'Y', 'Z')".to_string(),
                        benefit: "Simplify multiple equality checks".to_string(),
                        improvement_factor: 0.2,
                    });
                }
            }
        }

        Ok(())
    }
}

impl LogicSimplificationMatcher {
    fn is_equality_chain(&self, binary: &BinaryOperationNode) -> bool {
        // Simplified check - in real implementation, this would be more sophisticated
        matches!(binary.operator, BinaryOperator::Or) &&
            self.is_equality(&binary.left) && self.is_equality(&binary.right)
    }

    fn is_equality(&self, expr: &ExpressionNode) -> bool {
        if let ExpressionNode::BinaryOperation(binary) = expr {
            matches!(binary.operator, BinaryOperator::Equal)
        } else {
            false
        }
    }
}

/// Matcher for cacheable expressions
pub struct CacheableExpressionMatcher {
    cacheable_functions: HashSet<String>,
}

impl CacheableExpressionMatcher {
    pub fn new(cacheable_functions: &HashSet<String>) -> Self {
        Self {
            cacheable_functions: cacheable_functions.clone(),
        }
    }
}

impl PatternMatcher for CacheableExpressionMatcher {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()> {
        if let ExpressionNode::FunctionCall(call) = expression {
            if self.cacheable_functions.contains(&call.name) && self.has_constant_arguments(&call.arguments) {
                matches.push(PatternMatch {
                    pattern_type: PatternType::CacheableExpression,
                    location: call.location.clone(),
                    original: format!("{}(...)", call.name),
                    suggested: format!("Cached {}(...)", call.name),
                    benefit: "Cache function result for better performance".to_string(),
                    improvement_factor: 0.2,
                });
            }
        }

        Ok(())
    }
}

impl CacheableExpressionMatcher {
    fn has_constant_arguments(&self, args: &[ExpressionNode]) -> bool {
        args.iter().all(|arg| self.is_constant_expression(arg))
    }

    fn is_constant_expression(&self, expr: &ExpressionNode) -> bool {
        matches!(expr, ExpressionNode::Literal(_))
    }
}

/// Matcher for expensive function replacement
pub struct ExpensiveFunctionMatcher {
    expensive_functions: HashMap<String, Vec<String>>,
}

impl ExpensiveFunctionMatcher {
    pub fn new(expensive_functions: &HashMap<String, Vec<String>>) -> Self {
        Self {
            expensive_functions: expensive_functions.clone(),
        }
    }
}

impl PatternMatcher for ExpensiveFunctionMatcher {
    fn find_matches(&self, expression: &ExpressionNode, matches: &mut Vec<PatternMatch>) -> Result<()> {
        if let ExpressionNode::FunctionCall(call) = expression {
            if let Some(alternatives) = self.expensive_functions.get(&call.name) {
                matches.push(PatternMatch {
                    pattern_type: PatternType::ExpensiveFunctionReplacement,
                    location: call.location.clone(),
                    original: format!("{}(...)", call.name),
                    suggested: format!("{}(...)", alternatives.first().unwrap_or(&"alternative".to_string())),
                    benefit: format!("Replace expensive '{}' with faster alternatives", call.name),
                    improvement_factor: 0.4,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expression::*;

    #[test]
    fn test_optimization_detector_creation() {
        let detector = OptimizationDetector::new();
        assert!(!detector.expensive_functions.is_empty());
        assert!(!detector.cacheable_functions.is_empty());
        assert!(!detector.pure_functions.is_empty());
    }

    #[test]
    fn test_performance_score_calculation() {
        let detector = OptimizationDetector::new();
        
        // Empty suggestions should give perfect score
        let score = detector.calculate_performance_score(
            &[],
            &[],
            &DepthAnalysis {
                max_property_depth: 2,
                max_expression_depth: 3,
                deep_expressions: 0,
                depth_reduction_opportunities: vec![],
            }
        );
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_pattern_type_to_optimization_kind() {
        let detector = OptimizationDetector::new();
        assert_eq!(
            detector.pattern_type_to_optimization_kind(&PatternType::FilterCombination),
            OptimizationKind::CollectionOptimization
        );
        assert_eq!(
            detector.pattern_type_to_optimization_kind(&PatternType::ExpensiveFunctionReplacement),
            OptimizationKind::FunctionSimplification
        );
    }

    #[test]
    fn test_filter_combination_matcher() {
        let matcher = FilterCombinationMatcher;
        let mut matches = Vec::new();
        
        // Create a simple expression for testing - this would normally be parsed from FHIRPath
        // For this test, we'll just verify that the matcher can be called without panicking
        let literal = ExpressionNode::Literal(LiteralNode {
            value: crate::ast::literal::LiteralValue::String("test".to_string()),
            location: None,
        });
        
        assert!(matcher.find_matches(&literal, &mut matches).is_ok());
    }
}