//! Semantic analyzer for FHIRPath expressions
//!
//! This module provides semantic analysis capabilities that can enhance AST nodes
//! with type information, path tracking, and validation. It's designed to be used
//! optionally during analysis parsing mode without affecting fast parsing performance.

use std::sync::Arc;

use crate::ast::{
    AnalysisMetadata, ExpressionAnalysis, ExpressionNode, IdentifierNode, PropertyAccessNode,
};
use octofhir_fhir_model::{ModelProvider, TypeInfo};
use crate::core::FhirPathError;
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};

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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AnalysisMetadata, FhirPathError>> + 'a>> {
        Box::pin(async move {
            match node {
                ExpressionNode::Identifier(identifier) => {
                    self.analyze_identifier(identifier, analysis).await
                }
                ExpressionNode::PropertyAccess(prop) => {
                    self.analyze_property_access(prop, analysis).await
                }
                ExpressionNode::Literal(literal) => {
                    Ok(self.analyze_literal(literal))
                }
                _ => {
                    // For now, return empty metadata for other node types
                    Ok(AnalysisMetadata::new())
                }
            }
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
            if let Ok(Some(element_type)) = self.model_provider.get_element_type(input_type, name).await {
                metadata.type_info = Some(element_type);
                self.input_type = metadata.type_info.clone();
                self.is_chain_head = false;
                return Ok(metadata);
            }

            // If property not found and we have a concrete type, this is a semantic error
            // C# implementation: "prop 'given1' not found on HumanName[]"
            let type_display = if input_type.singleton {
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
            metadata.type_info = Some(TypeInfo::system_type("Any".to_string(), false));
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
        metadata.type_info = Some(TypeInfo::system_type("Any".to_string(), false));
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
        metadata.type_info = Some(TypeInfo::system_type("Any".to_string(), true));

        metadata
    }

    /// Suggest property name fixes for typos
    async fn suggest_property_fixes(
        &self,
        property_name: &str,
        context_type: &TypeInfo,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Get available properties for the current type
        if let Ok(properties) = self.model_provider
            .get_element_names(context_type)
            .await
        {
            // Simple distance-based suggestions (could be enhanced with better algorithms)
            for prop in properties.iter().take(5) {
                if self.string_distance(property_name, prop) <= 2 {
                    suggestions.push(prop.clone());
                }
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
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[a_chars.len()][b_chars.len()]
    }

    /// Reset analyzer state
    pub fn reset(&mut self) {
        self.input_type = None;
        self.is_chain_head = true;
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
    use octofhir_fhir_model::EmbeddedModelProvider;
    use crate::ast::ExpressionNode;

    #[tokio::test]
    async fn test_semantic_analyzer_creation() {
        let model_provider = Arc::new(EmbeddedModelProvider::new());
        let analyzer = SemanticAnalyzer::new(model_provider);
        assert!(analyzer.input_type.is_none());
        assert!(analyzer.is_chain_head);
    }

    #[tokio::test]
    async fn test_resource_type_analysis() {
        let model_provider = Arc::new(EmbeddedModelProvider::new());
        let mut analyzer = SemanticAnalyzer::new(model_provider);

        let identifier = ExpressionNode::identifier("Patient");
        let result = analyzer.analyze_expression(&identifier, None).await.unwrap();

        // Should succeed for known resource type
        assert!(result.success);
    }

    #[test]
    fn test_string_distance() {
        let model_provider = Arc::new(EmbeddedModelProvider::new());
        let analyzer = SemanticAnalyzer::new(model_provider);

        assert_eq!(analyzer.string_distance("test", "test"), 0);
        assert_eq!(analyzer.string_distance("test", "best"), 1);
        assert_eq!(analyzer.string_distance("kitten", "sitting"), 3);
    }
}