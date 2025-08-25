//! Simplified function analyzer that works with the new unified registry system

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_model::types::TypeInfo;
use octofhir_fhirpath_registry::FunctionRegistry;
use std::sync::Arc;

use crate::{
    error::{AnalysisError, ValidationError, ValidationErrorType},
    types::{FunctionCallAnalysis, FunctionSignature},
};

/// Simplified function analyzer that integrates with the new registry system
pub struct FunctionAnalyzer {
    registry: Arc<FunctionRegistry>,
}

impl FunctionAnalyzer {
    /// Create a new function analyzer with the given unified registry
    pub fn new(registry: Arc<FunctionRegistry>) -> Self {
        Self { registry }
    }

    /// Analyze function call for basic validation
    pub async fn analyze_function(
        &self,
        name: &str,
        _args: &[ExpressionNode],
        arg_types: &[TypeInfo],
    ) -> Result<FunctionCallAnalysis, AnalysisError> {
        // Check if function exists in registry
        let function_exists = self.registry.has_function(name);
        
        let mut validation_errors = vec![];
        
        if !function_exists {
            validation_errors.push(ValidationError {
                message: format!("Function '{}' not found in registry", name),
                error_type: ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: self.get_function_suggestions(name),
            });
        }

        // Create a simplified signature for analysis
        let signature = FunctionSignature {
            name: name.to_string(),
            parameters: vec![], // Simplified - no detailed parameter validation for now
            return_type: TypeInfo::Any, // Default to any type
            is_aggregate: self.is_aggregate_function(name),
            description: format!("Function {} from unified registry", name),
        };

        let node_id = 0; // Will be set by external mapping

        Ok(FunctionCallAnalysis {
            node_id,
            function_name: name.to_string(),
            signature,
            parameter_types: arg_types.to_vec(),
            return_type: TypeInfo::Any, // Use Any for both cases since we don't have detailed type inference yet
            validation_errors,
        })
    }

    /// Get function suggestions for unknown functions
    fn get_function_suggestions(&self, unknown_function: &str) -> Vec<String> {
        let available_functions = self.registry.function_names();
        
        // Simple similarity matching - find functions that start with similar characters
        let mut suggestions: Vec<String> = available_functions
            .into_iter()
            .filter(|func| {
                // Check if function name is similar (starts with same letter or contains substring)
                func.starts_with(&unknown_function[..1.min(unknown_function.len())])
                    || func.contains(unknown_function)
                    || unknown_function.contains(func)
            })
            .take(3) // Limit to 3 suggestions
            .collect();
        
        if suggestions.is_empty() {
            suggestions.push("Check available functions in registry".to_string());
        }
        
        suggestions
    }

    /// Check if a function is an aggregate function based on common patterns
    fn is_aggregate_function(&self, name: &str) -> bool {
        matches!(name, "count" | "sum" | "avg" | "min" | "max" | "distinct")
    }

    /// Get available functions from registry for validation
    pub fn get_available_functions(&self) -> Vec<String> {
        self.registry.function_names()
    }

    /// Check if function supports sync evaluation
    pub fn supports_sync(&self, name: &str) -> bool {
        self.registry.supports_sync(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::mock_provider::MockModelProvider;
    use octofhir_fhirpath_registry::create_standard_registry;

    #[tokio::test]
    async fn test_function_analyzer_basic() {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(create_standard_registry());
        let analyzer = FunctionAnalyzer::new(registry);

        // Test existing function
        let result = analyzer.analyze_function("count", &[], &[]).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert_eq!(analysis.function_name, "count");
        assert!(analysis.validation_errors.is_empty());
    }

    #[tokio::test]
    async fn test_function_analyzer_unknown_function() {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(create_standard_registry());
        let analyzer = FunctionAnalyzer::new(registry);

        // Test unknown function
        let result = analyzer.analyze_function("unknownFunc", &[], &[]).await;
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert_eq!(analysis.function_name, "unknownFunc");
        assert!(!analysis.validation_errors.is_empty());
        assert_eq!(analysis.validation_errors[0].error_type, ValidationErrorType::InvalidFunction);
    }
}