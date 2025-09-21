//! Type analyzer for FHIRPath expressions
//!
//! This module provides type inference and validation capabilities
//! for FHIRPath expressions, handling cardinality and type compatibility.

use std::sync::Arc;

use crate::analyzer::ExpressionContext;
use crate::ast::ExpressionNode;
use crate::diagnostics::AriadneDiagnostic;
use octofhir_fhir_model::{ModelProvider, TypeInfo};

/// Analyzer for type inference and validation
#[derive(Debug)]
pub struct TypeAnalyzer {
    model_provider: Arc<dyn ModelProvider>,
}

/// Cardinality of a type (singleton or collection)
#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality {
    /// Single value (0..1)
    Singleton,
    /// Multiple values (0..*)
    Collection,
}

/// Result of type analysis
#[derive(Debug, Clone)]
pub struct ContextAnalysisResult {
    /// Inferred type information
    pub type_info: TypeInfo,
    /// Any diagnostics generated during analysis
    pub diagnostics: Vec<AriadneDiagnostic>,
    /// Updated context after analysis
    pub context: ExpressionContext,
}

/// Type information with expression context
#[derive(Debug, Clone)]
pub struct ExpressionContextResult {
    /// The resulting type
    pub result_type: TypeInfo,
    /// Cardinality information
    pub cardinality: Cardinality,
    /// Context changes
    pub context: ExpressionContext,
}

impl TypeAnalyzer {
    /// Create a new type analyzer with the given model provider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    /// Perform type analysis on an expression node
    pub async fn analyze_type_flow(
        &self,
        _node: &ExpressionNode,
        context: &ExpressionContext,
    ) -> Option<(Vec<AriadneDiagnostic>, Option<TypeInfo>)> {
        // Basic implementation - return the current context type
        // In a full implementation, this would traverse the AST and infer types
        Some((Vec::new(), Some(context.input_type.clone())))
    }

    /// Infer the result type of an operation
    pub fn infer_result_type(
        &self,
        input_type: &TypeInfo,
        operation: &str,
        operand_types: &[TypeInfo],
    ) -> TypeInfo {
        match operation {
            // Binary operators
            "+" | "-" | "*" | "/" => self.infer_arithmetic_result(input_type, operand_types),
            "=" | "!=" | "<" | ">" | "<=" | ">=" => TypeInfo {
                type_name: "Boolean".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Boolean".to_string()),
            },
            "and" | "or" | "xor" => TypeInfo {
                type_name: "Boolean".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Boolean".to_string()),
            },
            // Union operator preserves left type but makes it a collection
            "|" => {
                let mut result = input_type.clone();
                result.singleton = Some(false);
                result
            }
            // Default case
            _ => input_type.clone(),
        }
    }

    /// Infer arithmetic operation result type
    fn infer_arithmetic_result(
        &self,
        _input_type: &TypeInfo,
        operand_types: &[TypeInfo],
    ) -> TypeInfo {
        if operand_types.len() != 2 {
            return TypeInfo {
                type_name: "Any".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Any".to_string()),
            };
        }

        let left = &operand_types[0];
        let right = &operand_types[1];

        // Type promotion rules: Integer + Decimal = Decimal
        match (left.type_name.as_str(), right.type_name.as_str()) {
            ("Integer", "Integer") => TypeInfo {
                type_name: "Integer".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Integer".to_string()),
            },
            ("Integer", "Decimal") | ("Decimal", "Integer") | ("Decimal", "Decimal") => TypeInfo {
                type_name: "Decimal".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Decimal".to_string()),
            },
            _ => TypeInfo {
                type_name: "Any".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Any".to_string()),
            },
        }
    }

    /// Check if two types are compatible
    pub fn are_types_compatible(&self, type1: &TypeInfo, type2: &TypeInfo) -> bool {
        // Basic compatibility checking
        if type1.type_name == type2.type_name {
            return true;
        }

        // Integer is compatible with Decimal
        if (type1.type_name == "Integer" && type2.type_name == "Decimal")
            || (type1.type_name == "Decimal" && type2.type_name == "Integer")
        {
            return true;
        }

        // Any is compatible with everything
        if type1.type_name == "Any" || type2.type_name == "Any" {
            return true;
        }

        false
    }

    /// Get cardinality from type info
    pub fn get_cardinality(&self, type_info: &TypeInfo) -> Cardinality {
        if type_info.singleton.unwrap_or(false) {
            Cardinality::Singleton
        } else {
            Cardinality::Collection
        }
    }

    /// Create a singleton version of a type
    pub fn make_singleton(&self, type_info: &TypeInfo) -> TypeInfo {
        let mut result = type_info.clone();
        result.singleton = Some(true);
        result
    }

    /// Create a collection version of a type
    pub fn make_collection(&self, type_info: &TypeInfo) -> TypeInfo {
        let mut result = type_info.clone();
        result.singleton = Some(false);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;

    fn create_test_analyzer() -> TypeAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        TypeAnalyzer::new(provider)
    }

    #[test]
    fn test_type_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert_eq!(
            std::mem::size_of::<TypeAnalyzer>(),
            std::mem::size_of_val(&analyzer)
        );
    }

    #[test]
    fn test_arithmetic_type_inference() {
        let analyzer = create_test_analyzer();

        let int_type = TypeInfo {
            type_name: "Integer".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Integer".to_string()),
        };

        let decimal_type = TypeInfo {
            type_name: "Decimal".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Decimal".to_string()),
        };

        // Integer + Integer = Integer
        let result =
            analyzer.infer_arithmetic_result(&int_type, &[int_type.clone(), int_type.clone()]);
        assert_eq!(result.type_name, "Integer");

        // Integer + Decimal = Decimal
        let result =
            analyzer.infer_arithmetic_result(&int_type, &[int_type.clone(), decimal_type.clone()]);
        assert_eq!(result.type_name, "Decimal");
    }

    #[test]
    fn test_type_compatibility() {
        let analyzer = create_test_analyzer();

        let int_type = TypeInfo {
            type_name: "Integer".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Integer".to_string()),
        };

        let decimal_type = TypeInfo {
            type_name: "Decimal".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Decimal".to_string()),
        };

        let string_type = TypeInfo {
            type_name: "String".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String".to_string()),
        };

        assert!(analyzer.are_types_compatible(&int_type, &int_type));
        assert!(analyzer.are_types_compatible(&int_type, &decimal_type));
        assert!(!analyzer.are_types_compatible(&int_type, &string_type));
    }

    #[test]
    fn test_cardinality() {
        let analyzer = create_test_analyzer();

        let singleton_type = TypeInfo {
            type_name: "String".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String".to_string()),
        };

        let collection_type = TypeInfo {
            type_name: "String".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("String".to_string()),
        };

        assert_eq!(
            analyzer.get_cardinality(&singleton_type),
            Cardinality::Singleton
        );
        assert_eq!(
            analyzer.get_cardinality(&collection_type),
            Cardinality::Collection
        );
    }
}
