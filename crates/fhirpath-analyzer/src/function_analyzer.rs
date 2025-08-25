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

    /// Analyze function call with comprehensive signature validation
    pub async fn analyze_function(
        &self,
        name: &str,
        args: &[ExpressionNode],
        arg_types: &[TypeInfo],
    ) -> Result<FunctionCallAnalysis, AnalysisError> {
        let mut validation_errors = vec![];
        
        // Check if function exists in registry and get its signature
        if let Some(registry_signature) = self.registry.get_function_signature(name) {
            // Validate argument count
            self.validate_argument_count(
                name, 
                registry_signature, 
                args.len(), 
                &mut validation_errors
            );
            
            // Validate argument types
            self.validate_argument_types(
                name,
                registry_signature,
                args,
                arg_types,
                &mut validation_errors
            );
            
            // Convert registry signature to analyzer signature for compatibility
            let analyzer_signature = FunctionSignature {
                name: name.to_string(),
                parameters: registry_signature.parameters.iter().map(|p| {
                    crate::types::ParameterInfo {
                        name: "param".to_string(), // Registry doesn't have param names
                        type_constraint: crate::types::TypeConstraint::Any, // Simplified for now
                        cardinality: crate::types::Cardinality::ZeroToOne,
                        is_optional: false,
                    }
                }).collect(),
                return_type: self.convert_value_type(&registry_signature.return_type),
                is_aggregate: self.is_aggregate_function(name),
                description: format!("{}({}) -> {}", name, self.format_parameters(&registry_signature.parameters), self.format_return_type(&registry_signature.return_type)),
            };
            
            let node_id = 0; // Will be set by external mapping
            
            Ok(FunctionCallAnalysis {
                node_id,
                function_name: name.to_string(),
                signature: analyzer_signature,
                parameter_types: arg_types.to_vec(),
                return_type: self.convert_value_type(&registry_signature.return_type),
                validation_errors,
            })
        } else {
            // Function not found - provide suggestions
            validation_errors.push(ValidationError {
                message: format!("Function '{}' not found in registry", name),
                error_type: ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: self.get_function_suggestions(name),
            });
            
            // Create a fallback signature
            let signature = FunctionSignature {
                name: name.to_string(),
                parameters: vec![],
                return_type: TypeInfo::Any,
                is_aggregate: false,
                description: format!("Unknown function: {}", name),
            };
            
            let node_id = 0;
            
            Ok(FunctionCallAnalysis {
                node_id,
                function_name: name.to_string(),
                signature,
                parameter_types: arg_types.to_vec(),
                return_type: TypeInfo::Any,
                validation_errors,
            })
        }
    }

    /// Get function suggestions for unknown functions
    fn get_function_suggestions(&self, unknown_function: &str) -> Vec<String> {
        let mut all_suggestions = Vec::new();
        
        // First, check lambda functions (these are more commonly used as methods)
        let lambda_functions = vec![
            "where", "select", "sort", "repeat", "aggregate", "all", "exists", "iif"
        ];
        
        let lambda_suggestions: Vec<String> = lambda_functions
            .into_iter()
            .filter(|func| {
                self.is_similar_function_name(unknown_function, func)
            })
            .map(|s| s.to_string())
            .collect();
        
        all_suggestions.extend(lambda_suggestions);
        
        // Then check registry functions if we don't have enough suggestions
        if all_suggestions.len() < 3 {
            let available_functions = self.registry.function_names();
            let registry_suggestions: Vec<String> = available_functions
                .into_iter()
                .filter(|func| {
                    self.is_similar_function_name(unknown_function, func)
                })
                .take(3 - all_suggestions.len()) // Fill up to 3 total suggestions
                .collect();
            
            all_suggestions.extend(registry_suggestions);
        }
        
        // If still no suggestions, provide helpful message
        if all_suggestions.is_empty() {
            all_suggestions.push("Check available functions in registry or lambda functions (where, select, all, exists, etc.)".to_string());
        }
        
        all_suggestions.truncate(3); // Limit to 3 suggestions
        all_suggestions
    }
    
    /// Check if two function names are similar (for typo detection)
    fn is_similar_function_name(&self, input: &str, candidate: &str) -> bool {
        let input_lower = input.to_lowercase();
        let candidate_lower = candidate.to_lowercase();
        
        // Exact match
        if input_lower == candidate_lower {
            return true;
        }
        
        // Check for common typo patterns
        if input_lower.len() >= 2 && candidate_lower.len() >= 2 {
            // Check if one is a prefix of the other (missing letters)
            if input_lower.starts_with(&candidate_lower) || candidate_lower.starts_with(&input_lower) {
                return true;
            }
            
            // Check edit distance (simple approximation)
            if self.simple_edit_distance(&input_lower, &candidate_lower) <= 2 {
                return true;
            }
        }
        
        // Contains substring check
        candidate_lower.contains(&input_lower) || input_lower.contains(&candidate_lower)
    }
    
    /// Simple edit distance calculation (Levenshtein approximation)
    fn simple_edit_distance(&self, s1: &str, s2: &str) -> usize {
        if s1 == s2 {
            return 0;
        }
        
        let len1 = s1.len();
        let len2 = s2.len();
        
        // Quick approximation for performance
        if (len1 as isize - len2 as isize).abs() > 2 {
            return 3; // Too different
        }
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        let mut prev_row: Vec<usize> = (0..=len2).collect();
        
        for (i, &c1) in s1_chars.iter().enumerate() {
            let mut curr_row = vec![i + 1];
            
            for (j, &c2) in s2_chars.iter().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                curr_row.push([
                    prev_row[j + 1] + 1, // deletion
                    curr_row[j] + 1,     // insertion
                    prev_row[j] + cost   // substitution
                ].iter().min().unwrap().clone());
            }
            
            prev_row = curr_row;
        }
        
        prev_row[len2]
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

    /// Validate argument count against signature
    fn validate_argument_count(
        &self,
        function_name: &str,
        signature: &octofhir_fhirpath_registry::signature::FunctionSignature,
        actual_count: usize,
        validation_errors: &mut Vec<ValidationError>,
    ) {
        let min_args = signature.min_args();
        let max_args = signature.max_args();
        
        if actual_count < min_args {
            validation_errors.push(ValidationError {
                message: format!(
                    "Function '{}()' requires at least {} argument{}, got {}",
                    function_name,
                    min_args,
                    if min_args == 1 { "" } else { "s" },
                    actual_count
                ),
                error_type: ValidationErrorType::InvalidFunction,
                location: None,
                suggestions: vec![format!("Add {} more argument{}", min_args - actual_count, if min_args - actual_count == 1 { "" } else { "s" })],
            });
        } else if let Some(max) = max_args {
            if actual_count > max {
                validation_errors.push(ValidationError {
                    message: format!(
                        "Function '{}()' accepts at most {} argument{}, got {}",
                        function_name,
                        max,
                        if max == 1 { "" } else { "s" },
                        actual_count
                    ),
                    error_type: ValidationErrorType::InvalidFunction,
                    location: None,
                    suggestions: vec![format!("Remove {} argument{}", actual_count - max, if actual_count - max == 1 { "" } else { "s" })],
                });
            }
        }
    }

    /// Validate argument types against signature
    fn validate_argument_types(
        &self,
        function_name: &str,
        signature: &octofhir_fhirpath_registry::signature::FunctionSignature,
        args: &[ExpressionNode],
        arg_types: &[TypeInfo],
        validation_errors: &mut Vec<ValidationError>,
    ) {
        // Basic type validation - can be enhanced later with more sophisticated type checking
        for (i, param_type) in signature.parameters.iter().enumerate() {
            if i >= args.len() {
                break; // Argument count validation handles missing args
            }
            
            // Check for obvious type mismatches
            if let Some(arg_type) = arg_types.get(i) {
                if !self.is_type_compatible(param_type, arg_type) {
                    validation_errors.push(ValidationError {
                        message: format!(
                            "Function '{}()' parameter {} expects {}, got {}",
                            function_name,
                            i + 1,
                            self.format_parameter_type(param_type),
                            self.format_type_info(arg_type)
                        ),
                        error_type: ValidationErrorType::TypeMismatch,
                        location: None,
                        suggestions: vec![format!("Provide a {} value for parameter {}", self.format_parameter_type(param_type), i + 1)],
                    });
                }
            }
        }
    }

    /// Check if an actual type is compatible with expected parameter type
    fn is_type_compatible(
        &self,
        expected: &octofhir_fhirpath_registry::signature::ParameterType,
        actual: &TypeInfo,
    ) -> bool {
        use octofhir_fhirpath_registry::signature::ParameterType;
        
        match (expected, actual) {
            // Any parameter accepts any type
            (ParameterType::Any, _) => true,
            (ParameterType::Collection, _) => true, // Collections can hold any type
            
            // Exact matches
            (ParameterType::String, TypeInfo::String) => true,
            (ParameterType::Integer, TypeInfo::Integer) => true,
            (ParameterType::Decimal, TypeInfo::Decimal) => true,
            (ParameterType::Boolean, TypeInfo::Boolean) => true,
            (ParameterType::Date, TypeInfo::Date) => true,
            (ParameterType::DateTime, TypeInfo::DateTime) => true,
            (ParameterType::Time, TypeInfo::Time) => true,
            (ParameterType::Quantity, TypeInfo::Quantity) => true,
            
            // Numeric compatibility
            (ParameterType::Numeric, TypeInfo::Integer) => true,
            (ParameterType::Numeric, TypeInfo::Decimal) => true,
            
            // Lambda parameters are flexible
            (ParameterType::Lambda, _) => true,
            
            // Default to compatible for Any actual type
            (_, TypeInfo::Any) => true,
            
            // Everything else is incompatible
            _ => false,
        }
    }

    /// Convert registry ParameterType to analyzer TypeInfo
    fn convert_parameter_type(&self, param_type: &octofhir_fhirpath_registry::signature::ParameterType) -> TypeInfo {
        use octofhir_fhirpath_registry::signature::ParameterType;
        
        match param_type {
            ParameterType::String => TypeInfo::String,
            ParameterType::Integer => TypeInfo::Integer,
            ParameterType::Decimal => TypeInfo::Decimal,
            ParameterType::Boolean => TypeInfo::Boolean,
            ParameterType::Date => TypeInfo::Date,
            ParameterType::DateTime => TypeInfo::DateTime,
            ParameterType::Time => TypeInfo::Time,
            ParameterType::Quantity => TypeInfo::Quantity,
            ParameterType::Any => TypeInfo::Any,
            ParameterType::Collection => TypeInfo::Any, // Collections can hold any type
            ParameterType::Numeric => TypeInfo::Decimal, // Use decimal as default numeric type
            ParameterType::Resource => TypeInfo::Any, // Generic resource type
            ParameterType::Lambda => TypeInfo::Any, // Lambda expressions can be any type
        }
    }

    /// Convert registry ValueType to analyzer TypeInfo
    fn convert_value_type(&self, value_type: &octofhir_fhirpath_registry::signature::ValueType) -> TypeInfo {
        use octofhir_fhirpath_registry::signature::ValueType;
        
        match value_type {
            ValueType::String => TypeInfo::String,
            ValueType::Integer => TypeInfo::Integer,
            ValueType::Decimal => TypeInfo::Decimal,
            ValueType::Boolean => TypeInfo::Boolean,
            ValueType::Date => TypeInfo::Date,
            ValueType::DateTime => TypeInfo::DateTime,
            ValueType::Time => TypeInfo::Time,
            ValueType::Quantity => TypeInfo::Quantity,
            ValueType::Collection => TypeInfo::Any, // Collections can hold any type
            ValueType::Any => TypeInfo::Any,
            ValueType::Resource => TypeInfo::Any, // Resource type maps to Any
            ValueType::Empty => TypeInfo::Any, // Empty type maps to Any
        }
    }

    /// Format parameter type for display
    fn format_parameter_type(&self, param_type: &octofhir_fhirpath_registry::signature::ParameterType) -> String {
        use octofhir_fhirpath_registry::signature::ParameterType;
        
        match param_type {
            ParameterType::String => "string".to_string(),
            ParameterType::Integer => "integer".to_string(),
            ParameterType::Decimal => "decimal".to_string(),
            ParameterType::Boolean => "boolean".to_string(),
            ParameterType::Date => "date".to_string(),
            ParameterType::DateTime => "dateTime".to_string(),
            ParameterType::Time => "time".to_string(),
            ParameterType::Quantity => "quantity".to_string(),
            ParameterType::Any => "any".to_string(),
            ParameterType::Collection => "collection".to_string(),
            ParameterType::Numeric => "numeric".to_string(),
            ParameterType::Resource => "resource".to_string(),
            ParameterType::Lambda => "expression".to_string(),
        }
    }

    /// Format TypeInfo for display
    fn format_type_info(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::String => "string".to_string(),
            TypeInfo::Integer => "integer".to_string(),
            TypeInfo::Decimal => "decimal".to_string(),
            TypeInfo::Boolean => "boolean".to_string(),
            TypeInfo::Date => "date".to_string(),
            TypeInfo::DateTime => "dateTime".to_string(),
            TypeInfo::Time => "time".to_string(),
            TypeInfo::Quantity => "quantity".to_string(),
            TypeInfo::Any => "any".to_string(),
            TypeInfo::Collection(_) => "collection".to_string(),
            TypeInfo::Resource(name) => format!("resource({})", name),
            TypeInfo::Union(_) => "union".to_string(),
            TypeInfo::Optional(_) => "optional".to_string(),
            TypeInfo::SimpleType => "simple".to_string(),
            // Handle any other variants that might exist
            _ => "unknown".to_string(),
        }
    }

    /// Format parameters list for signature display
    fn format_parameters(&self, parameters: &[octofhir_fhirpath_registry::signature::ParameterType]) -> String {
        parameters.iter()
            .map(|p| self.format_parameter_type(p))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format return type for signature display
    fn format_return_type(&self, return_type: &octofhir_fhirpath_registry::signature::ValueType) -> String {
        use octofhir_fhirpath_registry::signature::ValueType;
        
        match return_type {
            ValueType::String => "string".to_string(),
            ValueType::Integer => "integer".to_string(),
            ValueType::Decimal => "decimal".to_string(),
            ValueType::Boolean => "boolean".to_string(),
            ValueType::Date => "date".to_string(),
            ValueType::DateTime => "dateTime".to_string(),
            ValueType::Time => "time".to_string(),
            ValueType::Quantity => "quantity".to_string(),
            ValueType::Collection => "collection".to_string(),
            ValueType::Any => "any".to_string(),
            ValueType::Resource => "resource".to_string(),
            ValueType::Empty => "empty".to_string(),
        }
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