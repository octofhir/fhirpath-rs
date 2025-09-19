//! Semantic analyzer for FHIRPath expressions
//!
//! This module provides semantic analysis capabilities that can enhance AST nodes
//! with type information, path tracking, and validation. It's designed to be used
//! optionally during analysis parsing mode without affecting fast parsing performance.

use std::sync::Arc;

use crate::ast::{
    AnalysisMetadata, BinaryOperationNode, BinaryOperator, ExpressionAnalysis, ExpressionNode,
    FunctionCallNode, IdentifierNode, LiteralNode, LiteralValue, MethodCallNode, PropertyAccessNode,
};
use crate::core::FhirPathError;
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use octofhir_fhir_model::{ModelProvider, TypeInfo};

/// Semantic analyzer for FHIRPath expressions
pub struct SemanticAnalyzer {
    /// Model provider for type resolution
    model_provider: Arc<dyn ModelProvider>,
    /// Current input type context
    input_type: Option<TypeInfo>,
    /// Whether we're at the head of a navigation chain
    is_chain_head: bool,
}

impl SemanticAnalyzer {
    /// Create new semantic analyzer
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            model_provider,
            input_type: None,
            is_chain_head: true,
        }
    }

    /// Analyze an expression and return analysis metadata
    pub async fn analyze_expression(
        &mut self,
        expr: &ExpressionNode,
        context_type: Option<TypeInfo>,
    ) -> Result<ExpressionAnalysis, FhirPathError> {
        // Initialize context if provided
        self.input_type = context_type;
        self.is_chain_head = true;

        let mut analysis = ExpressionAnalysis::success(None);

        // Perform semantic analysis on the expression
        if let Err(err) = self.analyze_node(expr, &mut analysis).await {
            analysis.success = false;
            analysis.add_diagnostic(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "ANALYSIS_ERROR".to_string(),
                    namespace: Some("fhirpath".to_string()),
                },
                message: err.to_string(),
                location: expr.location().cloned(),
                related: vec![],
            });
        }

        Ok(analysis)
    }

    /// Analyze a single AST node
    fn analyze_node<'a>(
        &'a mut self,
        node: &'a ExpressionNode,
        analysis: &'a mut ExpressionAnalysis,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<AnalysisMetadata, FhirPathError>> + 'a>,
    > {
        Box::pin(async move {
            let result = match node {
                ExpressionNode::Identifier(identifier) => {
                    self.analyze_identifier(identifier, analysis).await
                }
                ExpressionNode::PropertyAccess(prop) => {
                    self.analyze_property_access(prop, analysis).await
                }
                ExpressionNode::Literal(literal) => Ok(self.analyze_literal(literal)),
                ExpressionNode::BinaryOperation(binary_op) => {
                    self.analyze_binary_operation(binary_op, analysis).await
                }
                ExpressionNode::FunctionCall(func_call) => {
                    self.analyze_function_call(func_call, analysis).await
                }
                ExpressionNode::MethodCall(method_call) => {
                    self.analyze_method_call(method_call, analysis).await
                }
                _ => {
                    // For now, return empty metadata for other node types
                    Ok(AnalysisMetadata::new())
                }
            };

            // After analyzing a node, we're no longer at the chain head
            self.is_chain_head = false;
            result
        })
    }

    /// Analyze identifier node
    async fn analyze_identifier(
        &mut self,
        identifier: &IdentifierNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        let mut metadata = AnalysisMetadata::new();
        let name = &identifier.name;

        // Try to use model provider for accurate type information
        if let Some(ref input_type) = self.input_type {
            // First try to navigate from input type (property access)
            if let Ok(Some(element_type)) =
                self.model_provider.get_element_type(input_type, name).await
            {
                metadata.type_info = Some(element_type);
                self.input_type = metadata.type_info.clone();
                self.is_chain_head = false;
                return Ok(metadata);
            }

            // If property not found and we have a concrete type, this is a semantic error
            // C# implementation: "prop 'given1' not found on HumanName[]"
            let type_display = if input_type.singleton.unwrap_or(true) {
                input_type.type_name.clone()
            } else {
                format!("{}[]", input_type.type_name)
            };

            let diagnostic = Diagnostic {
                severity: DiagnosticSeverity::Error, // Make this an error like C# implementation
                code: DiagnosticCode {
                    code: "PROPERTY_NOT_FOUND".to_string(),
                    namespace: Some("fhirpath".to_string()),
                },
                message: format!("prop '{}' not found on {}", name, type_display),
                location: identifier.location.clone(),
                related: vec![],
            };
            metadata.add_diagnostic(diagnostic.clone());
            analysis.add_diagnostic(diagnostic);

            // Return empty collection type but mark analysis as failed
            metadata.type_info = Some(TypeInfo {
                type_name: "Any".to_string(),
                singleton: Some(false),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Any".to_string()),
            });
            return Ok(metadata);
        }

        // Chain-head rule: at the head of a navigation chain, allow treating the
        // identifier as a known type to seed the chain (e.g., Patient.name)
        if self.is_chain_head {
            if let Ok(Some(type_info)) = self.model_provider.get_type(name).await {
                metadata.type_info = Some(type_info.clone());
                self.input_type = Some(type_info);
                self.is_chain_head = false;
                return Ok(metadata);
            }
        }

        // Without a model provider or context, we can't know the type
        // Return Any type - don't make assumptions
        metadata.type_info = Some(TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        });
        Ok(metadata)
    }

    /// Analyze property access node
    async fn analyze_property_access(
        &mut self,
        prop: &PropertyAccessNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // First analyze the object to establish context
        let _object_metadata = self.analyze_node(&prop.object, analysis).await?;

        // Mark that we're no longer at chain head
        self.is_chain_head = false;

        // Then analyze the property access
        let identifier = IdentifierNode {
            name: prop.property.clone(),
            location: prop.location.clone(),
        };

        self.analyze_identifier(&identifier, analysis).await
    }

    /// Analyze literal node (always valid)
    fn analyze_literal(&self, _literal: &crate::ast::LiteralNode) -> AnalysisMetadata {
        let mut metadata = AnalysisMetadata::new();

        // TODO: Infer type from literal value
        metadata.type_info = Some(TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        });

        metadata
    }

    /// Analyze binary operation node for semantic errors
    async fn analyze_binary_operation(
        &mut self,
        binary_op: &BinaryOperationNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // First, analyze the left and right operands
        let _left_metadata = self.analyze_node(&binary_op.left, analysis).await?;
        let _right_metadata = self.analyze_node(&binary_op.right, analysis).await?;

        // Check for semantic errors in addition operations
        if binary_op.operator == BinaryOperator::Add {
            // Check if left is a date/datetime/time literal and right is a plain number
            if let (ExpressionNode::Literal(left_literal), ExpressionNode::Literal(right_literal)) =
                (&*binary_op.left, &*binary_op.right)
            {
                // Check if left is a temporal literal and right is a number
                if self.is_temporal_literal(left_literal) && self.is_number_literal(right_literal) {
                    // Extract the actual number value for a better error message
                    let number_str = self.get_number_string(right_literal);
                    let suggestion = format!("+ {} 'days'", number_str);

                    // This is a semantic error: date + plain number is not allowed
                    analysis.add_diagnostic(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0082".to_string(),
                            namespace: None,
                        },
                        message: format!("Cannot add a plain number to a date/time value. Use a quantity with units instead (e.g., {})", suggestion),
                        location: binary_op.location.clone(),
                        related: Vec::new(),
                    });
                    analysis.success = false;
                }
            }
        }

        Ok(AnalysisMetadata::new())
    }

    /// Check if a literal is a temporal type (date, datetime, time)
    fn is_temporal_literal(&self, literal: &LiteralNode) -> bool {
        use crate::ast::LiteralValue;
        matches!(
            literal.value,
            LiteralValue::Date(_) | LiteralValue::DateTime(_) | LiteralValue::Time(_)
        )
    }

    /// Check if a literal is a number (integer or decimal)
    fn is_number_literal(&self, literal: &LiteralNode) -> bool {
        use crate::ast::LiteralValue;
        matches!(
            literal.value,
            LiteralValue::Integer(_) | LiteralValue::Decimal(_)
        )
    }

    /// Get the string representation of a number literal
    fn get_number_string(&self, literal: &LiteralNode) -> String {
        use crate::ast::LiteralValue;
        match &literal.value {
            LiteralValue::Integer(i) => i.to_string(),
            LiteralValue::Decimal(d) => d.to_string(),
            _ => "number".to_string(), // fallback
        }
    }

    /// Suggest property name fixes for typos
    async fn suggest_property_fixes(
        &self,
        property_name: &str,
        context_type: &TypeInfo,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get available properties for the current type
        let properties = self.model_provider.get_element_names(context_type);
        // Simple distance-based suggestions (could be enhanced with better algorithms)
        for prop in properties.iter().take(5) {
            if self.string_distance(property_name, prop) <= 2 {
                suggestions.push(prop.clone());
            }
        }

        suggestions
    }

    /// Calculate simple edit distance between strings
    fn string_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        if a_chars.is_empty() {
            return b_chars.len();
        }
        if b_chars.is_empty() {
            return a_chars.len();
        }

        let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

        for i in 0..=a_chars.len() {
            matrix[i][0] = i;
        }
        for j in 0..=b_chars.len() {
            matrix[0][j] = j;
        }

        for i in 1..=a_chars.len() {
            for j in 1..=b_chars.len() {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[a_chars.len()][b_chars.len()]
    }

    /// Analyze function call node
    async fn analyze_function_call(
        &mut self,
        func_call: &FunctionCallNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // Check for functions that require an input context but are called without one
        let functions_requiring_context = [
            "length", "empty", "count", "first", "last", "tail", "skip", "take",
            "exists", "all", "any", "allTrue", "anyTrue", "distinct", "children",
            "descendants", "where", "select", "single", "hasValue"
        ];

        if functions_requiring_context.contains(&func_call.name.as_str()) && self.is_chain_head {
            // This function requires an input context but is being called at the start of a chain
            analysis.success = false;
            analysis.add_diagnostic(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: DiagnosticCode {
                    code: "CONTEXT_REQUIRED".to_string(),
                    namespace: Some("fhirpath".to_string()),
                },
                message: format!("{} function requires an input context", func_call.name),
                location: func_call.location.clone(),
                related: vec![],
            });
        }

        // Check for specific function type validation
        if func_call.name == "iif" && func_call.arguments.len() >= 1 {
            // Analyze the first argument (condition) - should be boolean
            let condition_expr = &func_call.arguments[0];
            let metadata = self.analyze_node(condition_expr, analysis).await?;

            // Check if it's a literal non-boolean value
            if let ExpressionNode::Literal(literal) = condition_expr {
                if !matches!(literal.value, LiteralValue::Boolean(_)) {
                    analysis.success = false;
                    analysis.add_diagnostic(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "TYPE_MISMATCH".to_string(),
                            namespace: Some("fhirpath".to_string()),
                        },
                        message: "iif function condition must be boolean".to_string(),
                        location: condition_expr.location().cloned(),
                        related: vec![],
                    });
                }
            }

            // Check if condition involves union operations (creates collections)
            if self.contains_union_operation(condition_expr) {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "COLLECTION_IN_BOOLEAN_CONTEXT".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: "iif function condition cannot be a collection, must be a single boolean value".to_string(),
                    location: condition_expr.location().cloned(),
                    related: vec![],
                });
            }
        }

        // Analyze function arguments recursively
        for arg in &func_call.arguments {
            // Set context for analyzing arguments - they are not at chain head
            let prev_chain_head = self.is_chain_head;
            self.is_chain_head = true; // Arguments start their own chains
            let _arg_metadata = self.analyze_node(arg, analysis).await?;
            self.is_chain_head = prev_chain_head;
        }

        Ok(AnalysisMetadata::new())
    }

    /// Analyze method call node
    async fn analyze_method_call(
        &mut self,
        method_call: &MethodCallNode,
        analysis: &mut ExpressionAnalysis,
    ) -> Result<AnalysisMetadata, FhirPathError> {
        // Analyze the object first - this provides the context for the method
        let prev_chain_head = self.is_chain_head;
        let _object_metadata = self.analyze_node(&method_call.object, analysis).await?;

        // The method call is not at the chain head since it has an object
        self.is_chain_head = false;

        // Check for specific method type validation
        if method_call.method == "iif" && method_call.arguments.len() >= 1 {
            // Analyze the first argument (condition) - should be boolean
            let condition_expr = &method_call.arguments[0];

            // Arguments start their own chains
            self.is_chain_head = true;
            let _metadata = self.analyze_node(condition_expr, analysis).await?;

            // Check if it's a literal non-boolean value
            if let ExpressionNode::Literal(literal) = condition_expr {
                if !matches!(literal.value, LiteralValue::Boolean(_)) {
                    analysis.success = false;
                    analysis.add_diagnostic(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "TYPE_MISMATCH".to_string(),
                            namespace: Some("fhirpath".to_string()),
                        },
                        message: "iif function condition must be boolean".to_string(),
                        location: condition_expr.location().cloned(),
                        related: vec![],
                    });
                }
            }

            // Check if condition involves union operations (creates collections)
            if self.contains_union_operation(condition_expr) {
                analysis.success = false;
                analysis.add_diagnostic(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "COLLECTION_IN_BOOLEAN_CONTEXT".to_string(),
                        namespace: Some("fhirpath".to_string()),
                    },
                    message: "iif function condition cannot be a collection, must be a single boolean value".to_string(),
                    location: condition_expr.location().cloned(),
                    related: vec![],
                });
            }
        }

        // Analyze any other method arguments
        for arg in &method_call.arguments {
            self.is_chain_head = true; // Arguments start their own chains
            let _arg_metadata = self.analyze_node(arg, analysis).await?;
        }

        self.is_chain_head = prev_chain_head;
        Ok(AnalysisMetadata::new())
    }

    /// Reset analyzer state
    pub fn reset(&mut self) {
        self.input_type = None;
        self.is_chain_head = true;
    }

    /// Check if an expression contains union operations (creates collections)
    fn contains_union_operation(&self, expr: &ExpressionNode) -> bool {
        match expr {
            ExpressionNode::BinaryOperation(binary_op) => {
                if binary_op.operator == BinaryOperator::Union {
                    true
                } else {
                    // Recursively check both operands
                    self.contains_union_operation(&binary_op.left) ||
                    self.contains_union_operation(&binary_op.right)
                }
            }
            ExpressionNode::FunctionCall(func_call) => {
                // Check function arguments
                func_call.arguments.iter().any(|arg| self.contains_union_operation(arg))
            }
            ExpressionNode::MethodCall(method_call) => {
                // Check object and method arguments
                self.contains_union_operation(&method_call.object) ||
                method_call.arguments.iter().any(|arg| self.contains_union_operation(arg))
            }
            ExpressionNode::PropertyAccess(prop_access) => {
                // Check the object being accessed
                self.contains_union_operation(&prop_access.object)
            }
            // Literals and identifiers don't contain unions
            ExpressionNode::Literal(_) | ExpressionNode::Identifier(_) => false,
            // For other node types, assume no union for now
            _ => false,
        }
    }
}

/// Enhanced parsing result that includes semantic analysis
#[derive(Debug, Clone)]
pub struct AnalyzedParseResult {
    /// The parsed AST
    pub ast: Option<ExpressionNode>,
    /// Semantic analysis results
    pub analysis: ExpressionAnalysis,
    /// Whether parsing succeeded
    pub success: bool,
}

impl AnalyzedParseResult {
    /// Create successful result
    pub fn success(ast: ExpressionNode, analysis: ExpressionAnalysis) -> Self {
        Self {
            ast: Some(ast),
            analysis,
            success: true,
        }
    }

    /// Create failed result
    pub fn failure(analysis: ExpressionAnalysis) -> Self {
        Self {
            ast: None,
            analysis,
            success: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ExpressionNode;
    use octofhir_fhir_model::EmptyModelProvider;

    #[tokio::test]
    async fn test_semantic_analyzer_creation() {
        let model_provider = Arc::new(EmptyModelProvider);
        let analyzer = SemanticAnalyzer::new(model_provider);
        assert!(analyzer.input_type.is_none());
        assert!(analyzer.is_chain_head);
    }

    #[tokio::test]
    async fn test_resource_type_analysis() {
        let model_provider = Arc::new(EmptyModelProvider);
        let mut analyzer = SemanticAnalyzer::new(model_provider);

        let identifier = ExpressionNode::identifier("Patient");
        let result = analyzer
            .analyze_expression(&identifier, None)
            .await
            .unwrap();

        // Should succeed for known resource type
        assert!(result.success);
    }

    #[test]
    fn test_string_distance() {
        let model_provider = Arc::new(EmptyModelProvider);
        let analyzer = SemanticAnalyzer::new(model_provider);

        assert_eq!(analyzer.string_distance("test", "test"), 0);
        assert_eq!(analyzer.string_distance("test", "best"), 1);
        assert_eq!(analyzer.string_distance("kitten", "sitting"), 3);
    }
}
