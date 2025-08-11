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


//! Expression analysis for navigation paths, function calls, and complex operations
//!
//! This module provides detailed analysis of FHIRPath expressions focusing on
//! navigation validation, function signatures, and operational semantics.

use crate::ast::{ExpressionNode, BinaryOperator};
use crate::model::provider::{ModelProvider, TypeReflectionInfo, NavigationValidation};
use crate::analyzer::{AnalysisContext, AnalysisError};
// Removed Span import to avoid lifetime issues
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

/// Result of expression analysis
#[derive(Debug, Clone)]
pub struct ExpressionAnalysisResult {
    /// Additional types discovered during analysis
    pub additional_types: Vec<String>,
    /// Navigation paths validated
    pub navigation_paths: Vec<NavigationAnalysis>,
    /// Function calls analyzed
    pub function_calls: Vec<FunctionCallAnalysis>,
    /// Complexity metrics
    pub complexity: ExpressionComplexity,
    /// Performance characteristics
    pub performance_notes: Vec<PerformanceNote>,
}

impl Default for ExpressionAnalysisResult {
    fn default() -> Self {
        Self {
            additional_types: Vec::new(),
            navigation_paths: Vec::new(),
            function_calls: Vec::new(),
            complexity: ExpressionComplexity::default(),
            performance_notes: Vec::new(),
        }
    }
}

/// Analysis of a navigation path
#[derive(Debug, Clone)]
pub struct NavigationAnalysis {
    /// The navigation path (e.g., "name.given")
    pub path: String,
    /// Source type for the navigation
    pub source_type: Option<String>,
    /// Target type after navigation
    pub target_type: Option<String>,
    /// Validation result from ModelProvider
    pub validation: Option<NavigationValidation>,
    /// Start byte offset in the source code
    pub start_offset: Option<usize>,
    /// End byte offset in the source code
    pub end_offset: Option<usize>,
    /// Whether this navigation is valid
    pub is_valid: bool,
    /// Confidence in the analysis
    pub confidence: f32,
}

/// Analysis of a function call
#[derive(Debug, Clone)]
pub struct FunctionCallAnalysis {
    /// Function name
    pub name: String,
    /// Argument types
    pub arg_types: Vec<Option<TypeReflectionInfo>>,
    /// Expected return type
    pub expected_return_type: Option<TypeReflectionInfo>,
    /// Whether the function signature is valid
    pub signature_valid: bool,
    /// Start byte offset in the source code
    pub start_offset: Option<usize>,
    /// End byte offset in the source code
    pub end_offset: Option<usize>,
    /// Function category (builtin, extension, etc.)
    pub category: FunctionCategory,
}

/// Category of function
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionCategory {
    /// Built-in FHIRPath function
    Builtin,
    /// FHIR-specific function
    Fhir,
    /// CDA extension function
    Cda,
    /// Custom extension function
    Custom,
    /// Unknown function
    Unknown,
}

/// Complexity metrics for an expression
#[derive(Debug, Clone)]
pub struct ExpressionComplexity {
    /// Total depth of the expression tree
    pub depth: u32,
    /// Number of nodes in the expression tree
    pub node_count: u32,
    /// Number of navigation operations
    pub navigation_count: u32,
    /// Number of function calls
    pub function_call_count: u32,
    /// Number of collection operations
    pub collection_operations: u32,
    /// Estimated evaluation cost (0-100)
    pub estimated_cost: u32,
}

impl Default for ExpressionComplexity {
    fn default() -> Self {
        Self {
            depth: 0,
            node_count: 0,
            navigation_count: 0,
            function_call_count: 0,
            collection_operations: 0,
            estimated_cost: 0,
        }
    }
}

/// Performance characteristics note
#[derive(Debug, Clone)]
pub struct PerformanceNote {
    /// Type of performance consideration
    pub note_type: PerformanceNoteType,
    /// Description of the issue or optimization opportunity
    pub description: String,
    /// Severity level
    pub severity: PerformanceNoteSeverity,
    /// Start byte offset where this applies
    pub start_offset: Option<usize>,
    /// End byte offset where this applies
    pub end_offset: Option<usize>,
}

/// Type of performance note
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceNoteType {
    /// Potential optimization opportunity
    Optimization,
    /// Performance warning
    Warning,
    /// Known expensive operation
    Expensive,
    /// Caching opportunity
    Caching,
    /// Memory usage concern
    Memory,
}

/// Severity of performance note
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceNoteSeverity {
    /// Informational note
    Info,
    /// Minor performance impact
    Minor,
    /// Moderate performance impact
    Moderate,
    /// Major performance impact
    Major,
    /// Critical performance issue
    Critical,
}

/// Expression analyzer for detailed analysis of FHIRPath expressions
pub struct ExpressionAnalyzer<P: ModelProvider> {
    provider: Arc<P>,
    function_registry: HashMap<String, FunctionInfo>,
}

/// Information about a known function
#[derive(Debug, Clone)]
struct FunctionInfo {
    category: FunctionCategory,
    min_args: usize,
    max_args: Option<usize>,
    return_type_hint: Option<String>,
}

impl<P: ModelProvider> ExpressionAnalyzer<P> {
    /// Create a new expression analyzer
    pub fn new(provider: Arc<P>) -> Self {
        let mut function_registry = HashMap::new();
        
        // Register built-in functions
        Self::register_builtin_functions(&mut function_registry);
        Self::register_fhir_functions(&mut function_registry);
        
        Self {
            provider,
            function_registry,
        }
    }

    /// Analyze an expression
    pub async fn analyze(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
    ) -> Result<ExpressionAnalysisResult, AnalysisError> {
        let context = AnalysisContext::new(context_type.map(String::from));
        self.analyze_with_context(expression, &context).await
    }

    /// Analyze expression with full context
    pub async fn analyze_with_context(
        &self,
        expression: &ExpressionNode,
        context: &AnalysisContext,
    ) -> Result<ExpressionAnalysisResult, AnalysisError> {
        if context.depth > 50 {
            return Err(AnalysisError::MaxDepthExceeded { max_depth: 50 });
        }

        let mut result = ExpressionAnalysisResult::default();
        let child_context = context.child();

        match expression {
            ExpressionNode::Path { base, path } => {
                self.analyze_navigation_path(base, path, &child_context, &mut result).await?;
            }
            ExpressionNode::FunctionCall(data) => {
                self.analyze_function_call(&data.name, &data.args, &child_context, &mut result).await?;
            }
            ExpressionNode::MethodCall(data) => {
                self.analyze_method_call(&data.base, &data.method, &data.args, &child_context, &mut result).await?;
            }
            ExpressionNode::BinaryOp(data) => {
                self.analyze_binary_operation(&data.op, &data.left, &data.right, &child_context, &mut result).await?;
            }
            ExpressionNode::Filter { base, condition } => {
                self.analyze_filter_operation(base, condition, &child_context, &mut result).await?;
            }
            ExpressionNode::Union { left, right } => {
                self.analyze_union_operation(left, right, &child_context, &mut result).await?;
            }
            _ => {
                // For other expression types, recursively analyze children
                self.analyze_children(expression, &child_context, &mut result).await?;
            }
        }

        // Calculate complexity metrics
        result.complexity = self.calculate_complexity(expression);

        // Add performance notes
        self.add_performance_notes(expression, &mut result);

        Ok(result)
    }

    /// Analyze navigation paths
    async fn analyze_navigation_path(
        &self,
        base: &ExpressionNode,
        path: &str,
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // First analyze the base expression
        let base_result = self.analyze_with_context(base, context).await?;
        result.additional_types.extend(base_result.additional_types);
        result.navigation_paths.extend(base_result.navigation_paths);
        result.function_calls.extend(base_result.function_calls);

        // Determine the source type for navigation
        let source_type = if let ExpressionNode::Identifier(name) = base {
            if name == "$this" || Some(name) == context.root_type.as_ref() {
                context.root_type.clone()
            } else {
                None
            }
        } else {
            // For complex base expressions, we'd need type inference
            None
        };

        if let Some(source_type) = &source_type {
            // Validate the navigation path using the ModelProvider
            let validation_result = self.provider
                .validate_navigation_path(source_type, path)
                .await;

            match validation_result {
                Ok(validation) => {
                    let navigation = NavigationAnalysis {
                        path: format!("{}.{}", source_type, path),
                        source_type: Some(source_type.clone()),
                        target_type: validation.result_type.as_ref().map(|t| t.name().to_string()),
                        validation: Some(validation.clone()),
                        start_offset: context.start_offset,
                        end_offset: context.end_offset,
                        is_valid: validation.is_valid,
                        confidence: if validation.is_valid { 0.9 } else { 0.3 },
                    };

                    if let Some(target_type) = &validation.result_type {
                        result.additional_types.push(target_type.name().to_string());
                    }

                    result.navigation_paths.push(navigation);
                }
                Err(_) => {
                    // Navigation validation failed - add as invalid
                    let navigation = NavigationAnalysis {
                        path: format!("{}.{}", source_type, path),
                        source_type: Some(source_type.clone()),
                        target_type: None,
                        validation: None,
                        start_offset: context.start_offset,
                        end_offset: context.end_offset,
                        is_valid: false,
                        confidence: 0.1,
                    };
                    result.navigation_paths.push(navigation);
                }
            }
        } else {
            // Unknown source type - low confidence navigation
            let navigation = NavigationAnalysis {
                path: path.to_string(),
                source_type: None,
                target_type: None,
                validation: None,
                start_offset: context.start_offset,
            end_offset: context.end_offset,
                is_valid: false,
                confidence: 0.1,
            };
            result.navigation_paths.push(navigation);
        }

        result.complexity.navigation_count += 1;

        Ok(())
    }

    /// Analyze function calls
    async fn analyze_function_call(
        &self,
        name: &str,
        args: &[ExpressionNode],
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // Analyze all arguments first
        for arg in args {
            let arg_result = self.analyze_with_context(arg, context).await?;
            result.additional_types.extend(arg_result.additional_types);
            result.navigation_paths.extend(arg_result.navigation_paths);
            result.function_calls.extend(arg_result.function_calls);
        }

        // Look up function information
        let function_info = self.function_registry.get(name);
        
        let (category, signature_valid) = if let Some(info) = function_info {
            let arg_count_valid = args.len() >= info.min_args && 
                info.max_args.map_or(true, |max| args.len() <= max);
            (info.category.clone(), arg_count_valid)
        } else {
            (FunctionCategory::Unknown, false)
        };

        // Create function call analysis
        let analysis = FunctionCallAnalysis {
            name: name.to_string(),
            arg_types: vec![None; args.len()], // Would need type inference for this
            expected_return_type: function_info.and_then(|f| f.return_type_hint.as_ref())
                .and_then(|type_name| {
                    // This would be async in a real implementation
                    None // Placeholder
                }),
            signature_valid,
            start_offset: context.start_offset,
            end_offset: context.end_offset,
            category,
        };

        result.function_calls.push(analysis);
        result.complexity.function_call_count += 1;

        // Add performance notes for expensive functions
        if self.is_expensive_function(name) {
            result.performance_notes.push(PerformanceNote {
                note_type: PerformanceNoteType::Expensive,
                description: format!("Function '{}' may have significant performance cost", name),
                severity: PerformanceNoteSeverity::Moderate,
                start_offset: context.start_offset,
            end_offset: context.end_offset,
            });
        }

        Ok(())
    }

    /// Analyze method calls
    async fn analyze_method_call(
        &self,
        base: &ExpressionNode,
        method: &str,
        args: &[ExpressionNode],
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // Analyze base and arguments
        let base_result = self.analyze_with_context(base, context).await?;
        result.additional_types.extend(base_result.additional_types);
        result.navigation_paths.extend(base_result.navigation_paths);
        result.function_calls.extend(base_result.function_calls);

        for arg in args {
            let arg_result = self.analyze_with_context(arg, context).await?;
            result.additional_types.extend(arg_result.additional_types);
            result.navigation_paths.extend(arg_result.navigation_paths);
            result.function_calls.extend(arg_result.function_calls);
        }

        // Treat method call as function call for analysis
        self.analyze_function_call(method, args, context, result).await?;

        // Add specific performance notes for method calls
        if self.is_collection_method(method) {
            result.complexity.collection_operations += 1;
            
            if method == "where" && args.len() > 0 {
                result.performance_notes.push(PerformanceNote {
                    note_type: PerformanceNoteType::Warning,
                    description: "Complex where() conditions can be expensive on large collections".to_string(),
                    severity: PerformanceNoteSeverity::Minor,
                    start_offset: context.start_offset,
            end_offset: context.end_offset,
                });
            }
        }

        Ok(())
    }

    /// Analyze binary operations
    async fn analyze_binary_operation(
        &self,
        op: &BinaryOperator,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // Analyze both operands
        let left_result = self.analyze_with_context(left, context).await?;
        let right_result = self.analyze_with_context(right, context).await?;

        result.additional_types.extend(left_result.additional_types);
        result.additional_types.extend(right_result.additional_types);
        result.navigation_paths.extend(left_result.navigation_paths);
        result.navigation_paths.extend(right_result.navigation_paths);
        result.function_calls.extend(left_result.function_calls);
        result.function_calls.extend(right_result.function_calls);

        // Add performance notes for expensive operations
        match op {
            BinaryOperator::Union => {
                result.complexity.collection_operations += 1;
                result.performance_notes.push(PerformanceNote {
                    note_type: PerformanceNoteType::Optimization,
                    description: "Union operations can be optimized if operands are pre-sorted".to_string(),
                    severity: PerformanceNoteSeverity::Info,
                    start_offset: context.start_offset,
            end_offset: context.end_offset,
                });
            }
            BinaryOperator::In => {
                result.performance_notes.push(PerformanceNote {
                    note_type: PerformanceNoteType::Warning,
                    description: "'in' operator performance depends on collection size".to_string(),
                    severity: PerformanceNoteSeverity::Minor,
                    start_offset: context.start_offset,
            end_offset: context.end_offset,
                });
            }
            _ => {}
        }

        Ok(())
    }

    /// Analyze filter operations
    async fn analyze_filter_operation(
        &self,
        base: &ExpressionNode,
        condition: &ExpressionNode,
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // Analyze base and condition
        let base_result = self.analyze_with_context(base, context).await?;
        let condition_result = self.analyze_with_context(condition, context).await?;

        result.additional_types.extend(base_result.additional_types);
        result.additional_types.extend(condition_result.additional_types);
        result.navigation_paths.extend(base_result.navigation_paths);
        result.navigation_paths.extend(condition_result.navigation_paths);
        result.function_calls.extend(base_result.function_calls);
        result.function_calls.extend(condition_result.function_calls);

        result.complexity.collection_operations += 1;

        // Add performance note for complex filter conditions
        if condition_result.complexity.function_call_count > 0 || 
           condition_result.complexity.navigation_count > 2 {
            result.performance_notes.push(PerformanceNote {
                note_type: PerformanceNoteType::Warning,
                description: "Complex filter conditions can significantly impact performance on large collections".to_string(),
                severity: PerformanceNoteSeverity::Moderate,
                start_offset: context.start_offset,
            end_offset: context.end_offset,
            });
        }

        Ok(())
    }

    /// Analyze union operations
    async fn analyze_union_operation(
        &self,
        left: &ExpressionNode,
        right: &ExpressionNode,
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // Analyze both operands
        let left_result = self.analyze_with_context(left, context).await?;
        let right_result = self.analyze_with_context(right, context).await?;

        result.additional_types.extend(left_result.additional_types);
        result.additional_types.extend(right_result.additional_types);
        result.navigation_paths.extend(left_result.navigation_paths);
        result.navigation_paths.extend(right_result.navigation_paths);
        result.function_calls.extend(left_result.function_calls);
        result.function_calls.extend(right_result.function_calls);

        result.complexity.collection_operations += 1;

        // Performance considerations for union
        result.performance_notes.push(PerformanceNote {
            note_type: PerformanceNoteType::Optimization,
            description: "Union operations may benefit from deduplication optimizations".to_string(),
            severity: PerformanceNoteSeverity::Info,
            start_offset: context.start_offset,
            end_offset: context.end_offset,
        });

        Ok(())
    }

    /// Analyze children of complex expressions
    async fn analyze_children(
        &self,
        expression: &ExpressionNode,
        context: &AnalysisContext,
        result: &mut ExpressionAnalysisResult,
    ) -> Result<(), AnalysisError> {
        match expression {
            ExpressionNode::UnaryOp { operand, .. } => {
                let child_result = self.analyze_with_context(operand, context).await?;
                result.additional_types.extend(child_result.additional_types);
                result.navigation_paths.extend(child_result.navigation_paths);
                result.function_calls.extend(child_result.function_calls);
            }
            ExpressionNode::Index { base, index } => {
                let base_result = self.analyze_with_context(base, context).await?;
                let index_result = self.analyze_with_context(index, context).await?;
                
                result.additional_types.extend(base_result.additional_types);
                result.additional_types.extend(index_result.additional_types);
                result.navigation_paths.extend(base_result.navigation_paths);
                result.navigation_paths.extend(index_result.navigation_paths);
                result.function_calls.extend(base_result.function_calls);
                result.function_calls.extend(index_result.function_calls);
            }
            ExpressionNode::TypeCheck { expression, type_name } |
            ExpressionNode::TypeCast { expression, type_name } => {
                let expr_result = self.analyze_with_context(expression, context).await?;
                result.additional_types.extend(expr_result.additional_types);
                result.additional_types.push(type_name.clone());
                result.navigation_paths.extend(expr_result.navigation_paths);
                result.function_calls.extend(expr_result.function_calls);
            }
            ExpressionNode::Lambda(data) => {
                let body_result = self.analyze_with_context(&data.body, context).await?;
                result.additional_types.extend(body_result.additional_types);
                result.navigation_paths.extend(body_result.navigation_paths);
                result.function_calls.extend(body_result.function_calls);
            }
            ExpressionNode::Conditional(data) => {
                let condition_result = self.analyze_with_context(&data.condition, context).await?;
                let then_result = self.analyze_with_context(&data.then_expr, context).await?;
                
                result.additional_types.extend(condition_result.additional_types);
                result.additional_types.extend(then_result.additional_types);
                result.navigation_paths.extend(condition_result.navigation_paths);
                result.navigation_paths.extend(then_result.navigation_paths);
                result.function_calls.extend(condition_result.function_calls);
                result.function_calls.extend(then_result.function_calls);

                if let Some(else_expr) = &data.else_expr {
                    let else_result = self.analyze_with_context(else_expr, context).await?;
                    result.additional_types.extend(else_result.additional_types);
                    result.navigation_paths.extend(else_result.navigation_paths);
                    result.function_calls.extend(else_result.function_calls);
                }
            }
            _ => {} // Literals, identifiers, variables don't need child analysis
        }

        Ok(())
    }

    /// Calculate complexity metrics for an expression
    fn calculate_complexity(&self, expression: &ExpressionNode) -> ExpressionComplexity {
        let mut complexity = ExpressionComplexity::default();
        self.calculate_complexity_recursive(expression, &mut complexity, 1);
        
        // Estimate cost based on various factors
        complexity.estimated_cost = (complexity.node_count * 2 +
                                    complexity.navigation_count * 5 +
                                    complexity.function_call_count * 10 +
                                    complexity.collection_operations * 15).min(100);
        
        complexity
    }

    /// Recursively calculate complexity
    fn calculate_complexity_recursive(
        &self,
        expression: &ExpressionNode,
        complexity: &mut ExpressionComplexity,
        depth: u32,
    ) {
        complexity.depth = complexity.depth.max(depth);
        complexity.node_count += 1;

        match expression {
            ExpressionNode::Path { base, .. } => {
                complexity.navigation_count += 1;
                self.calculate_complexity_recursive(base, complexity, depth + 1);
            }
            ExpressionNode::FunctionCall(data) => {
                complexity.function_call_count += 1;
                for arg in &data.args {
                    self.calculate_complexity_recursive(arg, complexity, depth + 1);
                }
            }
            ExpressionNode::MethodCall(data) => {
                complexity.function_call_count += 1;
                if self.is_collection_method(&data.method) {
                    complexity.collection_operations += 1;
                }
                self.calculate_complexity_recursive(&data.base, complexity, depth + 1);
                for arg in &data.args {
                    self.calculate_complexity_recursive(arg, complexity, depth + 1);
                }
            }
            ExpressionNode::BinaryOp(data) => {
                if matches!(data.op, BinaryOperator::Union) {
                    complexity.collection_operations += 1;
                }
                self.calculate_complexity_recursive(&data.left, complexity, depth + 1);
                self.calculate_complexity_recursive(&data.right, complexity, depth + 1);
            }
            ExpressionNode::Filter { base, condition } => {
                complexity.collection_operations += 1;
                self.calculate_complexity_recursive(base, complexity, depth + 1);
                self.calculate_complexity_recursive(condition, complexity, depth + 1);
            }
            ExpressionNode::Union { left, right } => {
                complexity.collection_operations += 1;
                self.calculate_complexity_recursive(left, complexity, depth + 1);
                self.calculate_complexity_recursive(right, complexity, depth + 1);
            }
            ExpressionNode::UnaryOp { operand, .. } => {
                self.calculate_complexity_recursive(operand, complexity, depth + 1);
            }
            ExpressionNode::Index { base, index } => {
                self.calculate_complexity_recursive(base, complexity, depth + 1);
                self.calculate_complexity_recursive(index, complexity, depth + 1);
            }
            ExpressionNode::TypeCheck { expression, .. } |
            ExpressionNode::TypeCast { expression, .. } => {
                self.calculate_complexity_recursive(expression, complexity, depth + 1);
            }
            ExpressionNode::Lambda(data) => {
                self.calculate_complexity_recursive(&data.body, complexity, depth + 1);
            }
            ExpressionNode::Conditional(data) => {
                self.calculate_complexity_recursive(&data.condition, complexity, depth + 1);
                self.calculate_complexity_recursive(&data.then_expr, complexity, depth + 1);
                if let Some(else_expr) = &data.else_expr {
                    self.calculate_complexity_recursive(else_expr, complexity, depth + 1);
                }
            }
            _ => {} // Literals, identifiers, variables are leaf nodes
        }
    }

    /// Add performance notes based on expression patterns
    fn add_performance_notes(&self, expression: &ExpressionNode, result: &mut ExpressionAnalysisResult) {
        // Add notes based on complexity
        if result.complexity.depth > 10 {
            result.performance_notes.push(PerformanceNote {
                note_type: PerformanceNoteType::Warning,
                description: "Deep expression nesting may impact performance and readability".to_string(),
                severity: PerformanceNoteSeverity::Minor,
                start_offset: None,
                end_offset: None,
            });
        }

        if result.complexity.estimated_cost > 50 {
            result.performance_notes.push(PerformanceNote {
                note_type: PerformanceNoteType::Expensive,
                description: "High-cost expression - consider optimization or caching".to_string(),
                severity: PerformanceNoteSeverity::Moderate,
                start_offset: None,
                end_offset: None,
            });
        }

        // Add caching suggestion for expensive expressions with no side effects
        if result.complexity.estimated_cost > 30 && self.is_cacheable(expression) {
            result.performance_notes.push(PerformanceNote {
                note_type: PerformanceNoteType::Caching,
                description: "Expression appears cacheable - consider memoization".to_string(),
                severity: PerformanceNoteSeverity::Info,
                start_offset: None,
                end_offset: None,
            });
        }
    }

    /// Register built-in functions
    fn register_builtin_functions(registry: &mut HashMap<String, FunctionInfo>) {
        let functions = [
            ("empty", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: Some("Boolean".to_string()) }),
            ("exists", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(1), return_type_hint: Some("Boolean".to_string()) }),
            ("count", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: Some("Integer".to_string()) }),
            ("length", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: Some("Integer".to_string()) }),
            ("first", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: None }),
            ("last", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: None }),
            ("tail", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: None }),
            ("where", FunctionInfo { category: FunctionCategory::Builtin, min_args: 1, max_args: Some(1), return_type_hint: None }),
            ("select", FunctionInfo { category: FunctionCategory::Builtin, min_args: 1, max_args: Some(1), return_type_hint: None }),
            ("all", FunctionInfo { category: FunctionCategory::Builtin, min_args: 1, max_args: Some(1), return_type_hint: Some("Boolean".to_string()) }),
            ("distinct", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: None }),
            ("substring", FunctionInfo { category: FunctionCategory::Builtin, min_args: 1, max_args: Some(2), return_type_hint: Some("String".to_string()) }),
            ("toString", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: Some("String".to_string()) }),
            ("toInteger", FunctionInfo { category: FunctionCategory::Builtin, min_args: 0, max_args: Some(0), return_type_hint: Some("Integer".to_string()) }),
        ];

        for (name, info) in functions {
            registry.insert(name.to_string(), info);
        }
    }

    /// Register FHIR-specific functions
    fn register_fhir_functions(registry: &mut HashMap<String, FunctionInfo>) {
        let functions = [
            ("resolve", FunctionInfo { category: FunctionCategory::Fhir, min_args: 0, max_args: Some(0), return_type_hint: None }),
            ("extension", FunctionInfo { category: FunctionCategory::Fhir, min_args: 1, max_args: Some(1), return_type_hint: None }),
            ("hasValue", FunctionInfo { category: FunctionCategory::Fhir, min_args: 0, max_args: Some(0), return_type_hint: Some("Boolean".to_string()) }),
            ("conformsTo", FunctionInfo { category: FunctionCategory::Fhir, min_args: 1, max_args: Some(1), return_type_hint: Some("Boolean".to_string()) }),
        ];

        for (name, info) in functions {
            registry.insert(name.to_string(), info);
        }
    }

    /// Check if a function is expensive
    fn is_expensive_function(&self, name: &str) -> bool {
        matches!(name, "resolve" | "conformsTo" | "all" | "select" | "where")
    }

    /// Check if a method is a collection operation
    fn is_collection_method(&self, method: &str) -> bool {
        matches!(method, "where" | "select" | "distinct" | "union" | "intersect")
    }

    /// Check if an expression is cacheable (no side effects)
    fn is_cacheable(&self, expression: &ExpressionNode) -> bool {
        match expression {
            ExpressionNode::Literal(_) | 
            ExpressionNode::Identifier(_) | 
            ExpressionNode::Variable(_) => true,
            
            ExpressionNode::Path { base, .. } => self.is_cacheable(base),
            
            ExpressionNode::FunctionCall(data) => {
                // Most built-in functions are pure, but some may have side effects
                !matches!(data.name.as_str(), "trace" | "resolve") &&
                data.args.iter().all(|arg| self.is_cacheable(arg))
            }
            
            ExpressionNode::BinaryOp(data) => {
                self.is_cacheable(&data.left) && self.is_cacheable(&data.right)
            }
            
            _ => false, // Conservative approach for other expression types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mock_provider::MockModelProvider;
    use crate::ast::ExpressionNode;

    #[tokio::test]
    async fn test_navigation_analysis() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = ExpressionAnalyzer::new(provider);

        let base = ExpressionNode::identifier("Patient");
        let expr = ExpressionNode::path(base, "name");
        
        let result = analyzer.analyze(&expr, Some("Patient")).await.unwrap();

        assert_eq!(result.navigation_paths.len(), 1);
        assert_eq!(result.complexity.navigation_count, 1);
    }

    #[tokio::test]
    async fn test_function_call_analysis() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = ExpressionAnalyzer::new(provider);

        let expr = ExpressionNode::function_call("count", vec![]);
        
        let result = analyzer.analyze(&expr, None).await.unwrap();

        assert_eq!(result.function_calls.len(), 1);
        assert_eq!(result.function_calls[0].category, FunctionCategory::Builtin);
        assert!(result.function_calls[0].signature_valid);
    }

    #[tokio::test]
    async fn test_complexity_calculation() {
        let provider = Arc::new(MockModelProvider::empty());
        let analyzer = ExpressionAnalyzer::new(provider);

        // Create a complex expression: Patient.name.where($this.given.count() > 0).first()
        let base = ExpressionNode::identifier("Patient");
        let name_path = ExpressionNode::path(base, "name");
        let count_call = ExpressionNode::function_call("count", vec![]);
        let zero = ExpressionNode::literal(crate::ast::LiteralValue::Integer(0));
        let comparison = ExpressionNode::binary_op(crate::ast::BinaryOperator::GreaterThan, count_call, zero);
        let where_call = ExpressionNode::method_call(name_path, "where", vec![comparison]);
        let expr = ExpressionNode::method_call(where_call, "first", vec![]);
        
        let result = analyzer.analyze(&expr, Some("Patient")).await.unwrap();

        assert!(result.complexity.depth > 3);
        assert!(result.complexity.function_call_count >= 3);
        assert!(result.complexity.estimated_cost > 20);
    }
}