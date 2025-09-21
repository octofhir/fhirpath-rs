//! Union type analyzer for FHIRPath expressions
//!
//! This module handles union type operations and validates union choices
//! in FHIRPath expressions.

use std::sync::Arc;

use crate::analyzer::ExpressionContext;
use crate::ast::ExpressionNode;
use crate::core::error_code::{FP0156, FP0157};
use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};
use octofhir_fhir_model::{ModelProvider, TypeInfo};

/// Analyzer for union type operations
#[derive(Debug)]
pub struct UnionTypeAnalyzer {
    model_provider: Arc<dyn ModelProvider>,
}

/// Result of union type analysis
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Resulting type after union operation
    pub result_type: TypeInfo,
    /// Any diagnostics generated
    pub diagnostics: Vec<AriadneDiagnostic>,
    /// Updated context
    pub context: ExpressionContext,
}

/// Union operation types
#[derive(Debug, Clone, PartialEq)]
pub enum UnionOperation {
    /// Union operator (|)
    Union,
    /// Type filtering (ofType, is, as)
    TypeFilter,
}

impl UnionTypeAnalyzer {
    /// Create a new union type analyzer
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    /// Analyze union operations in an expression
    pub async fn analyze_union_operations(
        &self,
        _node: &ExpressionNode,
        _context: &ExpressionContext,
    ) -> Option<Vec<AriadneDiagnostic>> {
        // Basic implementation - no union-specific diagnostics for now
        // In a full implementation, this would check for union type operations
        Some(Vec::new())
    }

    /// Analyze a union operation between two types
    pub fn analyze_union(
        &self,
        left_type: &TypeInfo,
        right_type: &TypeInfo,
        context: &ExpressionContext,
    ) -> AnalysisResult {
        let diagnostics = Vec::new();

        // Union operation preserves the left type but makes it a collection
        let mut result_type = left_type.clone();
        result_type.singleton = Some(false);

        // Check if types are compatible for union
        if !self.are_union_compatible(left_type, right_type) {
            // In a full implementation, we might generate warnings here
        }

        AnalysisResult {
            result_type,
            diagnostics,
            context: context.clone(),
        }
    }

    /// Check if two types can be unioned
    fn are_union_compatible(&self, _left: &TypeInfo, _right: &TypeInfo) -> bool {
        // For now, allow all unions - FHIRPath is quite permissive
        true
    }

    /// Validate a type filter operation (ofType, is, as)
    pub fn validate_type_filter(
        &self,
        input_type: &TypeInfo,
        target_type: &str,
        operation: &str,
    ) -> Vec<AriadneDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check if the input type is a union type
        if !self.is_union_type(input_type) {
            return diagnostics;
        }

        // Get union choices if available
        let choices = self.get_union_choices(input_type);
        if !choices.is_empty() && !choices.contains(&target_type.to_string()) {
            // Create a warning diagnostic
            let message = format!(
                "Type {} '{}' may always be empty - type not present in union. Available types: {}",
                operation,
                target_type,
                choices.join(", ")
            );

            diagnostics.push(AriadneDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message,
                span: 0..0, // Would be set by caller with proper span
                error_code: FP0156,
                help: None,
                note: None,
                related: Vec::new(),
            });
        }

        diagnostics
    }

    /// Check if a type is a union type
    fn is_union_type(&self, type_info: &TypeInfo) -> bool {
        // Check if the type has union metadata
        // This would be determined by the model provider in a full implementation
        type_info.type_name.contains("|")
        // TODO: Add model_context when it becomes available in TypeInfo
    }

    /// Get the choices available in a union type
    fn get_union_choices(&self, type_info: &TypeInfo) -> Vec<String> {
        // Extract union choices from type info
        if type_info.type_name.contains("|") {
            return type_info
                .type_name
                .split("|")
                .map(|s| s.trim().to_string())
                .collect();
        }

        // TODO: Check model context for union choices when available
        Vec::new()
    }

    /// Validate a union choice against available options
    pub fn validate_union_choice(
        &self,
        union_type: &TypeInfo,
        choice: &str,
        operation: &str,
    ) -> Option<AriadneDiagnostic> {
        if !self.is_union_type(union_type) {
            return None;
        }

        let choices = self.get_union_choices(union_type);
        if !choices.is_empty() && !choices.contains(&choice.to_string()) {
            let message = format!(
                "Type {} '{}' will always be false - type not present in union. Available types: {}",
                operation,
                choice,
                choices.join(", ")
            );

            return Some(AriadneDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message,
                span: 0..0,
                error_code: FP0157,
                help: None,
                note: None,
                related: Vec::new(),
            });
        }

        None
    }

    /// Create a union type from multiple types
    pub fn create_union_type(&self, types: &[TypeInfo]) -> TypeInfo {
        if types.is_empty() {
            return TypeInfo {
                type_name: "Any".to_string(),
                singleton: Some(false),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Any".to_string()),
            };
        }

        if types.len() == 1 {
            return types[0].clone();
        }

        // Create a union type string
        let union_type = types
            .iter()
            .map(|t| t.type_name.as_str())
            .collect::<Vec<_>>()
            .join(" | ");

        TypeInfo {
            type_name: union_type,
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Union".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;

    fn create_test_analyzer() -> UnionTypeAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        UnionTypeAnalyzer::new(provider)
    }

    #[test]
    fn test_union_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert_eq!(
            std::mem::size_of::<UnionTypeAnalyzer>(),
            std::mem::size_of_val(&analyzer)
        );
    }

    #[test]
    fn test_union_operation() {
        let analyzer = create_test_analyzer();
        let context = ExpressionContext::default();

        let string_type = TypeInfo {
            type_name: "String".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String".to_string()),
        };

        let integer_type = TypeInfo {
            type_name: "Integer".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Integer".to_string()),
        };

        let result = analyzer.analyze_union(&string_type, &integer_type, &context);

        // Union should preserve left type but make it a collection
        assert_eq!(result.result_type.type_name, "String");
        assert_eq!(result.result_type.singleton, Some(false));
    }

    #[test]
    fn test_union_type_detection() {
        let analyzer = create_test_analyzer();

        let simple_type = TypeInfo {
            type_name: "String".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String".to_string()),
        };

        let union_type = TypeInfo {
            type_name: "String | Integer".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String | Integer".to_string()),
        };

        assert!(!analyzer.is_union_type(&simple_type));
        assert!(analyzer.is_union_type(&union_type));
    }

    #[test]
    fn test_union_choices() {
        let analyzer = create_test_analyzer();

        let union_type = TypeInfo {
            type_name: "String | Integer | Boolean".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String | Integer | Boolean".to_string()),
        };

        let choices = analyzer.get_union_choices(&union_type);
        assert_eq!(choices, vec!["String", "Integer", "Boolean"]);
    }

    #[test]
    fn test_create_union_type() {
        let analyzer = create_test_analyzer();

        let types = vec![
            TypeInfo {
                type_name: "String".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("String".to_string()),
            },
            TypeInfo {
                type_name: "Integer".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Integer".to_string()),
            },
        ];

        let union = analyzer.create_union_type(&types);
        assert_eq!(union.type_name, "String | Integer");
        assert_eq!(union.singleton, Some(false));
    }
}
