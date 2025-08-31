//! Bridge API-enabled error analysis with enhanced diagnostics
//!
//! This module provides comprehensive error reporting that leverages the Bridge Support Architecture
//! for detailed error analysis, contextual suggestions, and similarity-based error recovery.

use octofhir_fhirpath_model::provider::ModelProvider;
use octofhir_fhirschema::FhirSchemaPackageManager;
use std::sync::Arc;

use crate::{
    bridge_field_validator::{AnalyzerError, SimilarityMatcher},
    types::AnalysisContext,
};

/// Enhanced error report with bridge-enabled analysis
#[derive(Debug, Clone)]
pub struct EnhancedErrorReport {
    /// The original error that was analyzed
    pub original_error: FhirPathError,
    /// Categorization of the error
    pub error_category: ErrorCategory,
    /// Contextual suggestions for fixing the error
    pub suggestions: Vec<ErrorSuggestion>,
    /// Automatic fixes that can be applied
    pub fixes: Vec<AutomaticFix>,
    /// Related documentation links
    pub related_documentation: Vec<DocumentationLink>,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Additional context information
    pub context_info: Option<String>,
}

/// FHIRPath error types for analysis
#[derive(Debug, Clone)]
pub enum FhirPathError {
    PropertyNotFound {
        type_name: String,
        property: String,
        source: Option<String>,
    },
    TypeMismatch {
        expected: String,
        actual: String,
    },
    InvalidPath(String),
    FunctionNotFound(String),
    InvalidFunction {
        function_name: String,
        reason: String,
    },
    ConstraintViolation {
        constraint: String,
        violation_type: String,
    },
    Generic {
        message: String,
        error_type: String,
    },
}

/// Error categorization for better handling
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Field or property access errors
    FieldAccess,
    /// Type-related errors
    TypeSystem,
    /// Function call errors
    FunctionCall,
    /// Path navigation errors
    Navigation,
    /// Constraint validation errors
    Constraint,
    /// Syntax or parsing errors
    Syntax,
    /// Generic errors
    Generic,
}

/// Error suggestion with contextual information
#[derive(Debug, Clone)]
pub struct ErrorSuggestion {
    /// The suggestion text
    pub suggestion: String,
    /// Confidence level of the suggestion (0.0 to 1.0)
    pub confidence: f64,
    /// Category of the suggestion
    pub category: SuggestionCategory,
    /// Example usage if applicable
    pub example: Option<String>,
}

/// Types of suggestions that can be provided
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionCategory {
    /// Suggest alternative field names
    AlternativeField,
    /// Suggest type conversions
    TypeConversion,
    /// Suggest similar functions
    SimilarFunction,
    /// Suggest path corrections
    PathCorrection,
    /// Suggest constraint fixes
    ConstraintFix,
    /// General usage guidance
    UsageGuidance,
}

/// Automatic fix that can be applied
#[derive(Debug, Clone)]
pub struct AutomaticFix {
    /// Description of the fix
    pub description: String,
    /// The original text to replace
    pub original: String,
    /// The replacement text
    pub replacement: String,
    /// Confidence level of the fix (0.0 to 1.0)
    pub confidence: f64,
}

/// Documentation link for additional help
#[derive(Debug, Clone)]
pub struct DocumentationLink {
    /// Title of the documentation
    pub title: String,
    /// URL to the documentation
    pub url: String,
    /// Brief description
    pub description: String,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ErrorSeverity {
    /// Error that prevents execution
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Info,
    /// Hint or suggestion
    Hint,
}

/// Property suggestion for field errors
#[derive(Debug, Clone)]
pub struct PropertySuggestion {
    /// Suggested property name
    pub property_name: String,
    /// Similarity score with the invalid property
    pub similarity_score: f64,
    /// Property information from bridge API
    pub property_info: super::bridge_field_validator::PropertyInfo,
    /// Usage examples for this property
    pub usage_examples: Vec<String>,
}

/// Bridge-enabled error reporter with enhanced analysis
pub struct AnalyzerErrorReporter {
    /// Schema manager for bridge API operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Similarity matcher for suggestions
    similarity_matcher: SimilarityMatcher,
}

impl AnalyzerErrorReporter {
    /// Create new analyzer error reporter with bridge support
    pub async fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Result<Self, AnalyzerError> {
        let model_provider: Arc<dyn ModelProvider> = Arc::new(
            octofhir_fhirpath_model::FhirSchemaModelProvider::new()
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create model provider: {}", e),
                })?,
        );

        let similarity_matcher = SimilarityMatcher::new();

        Ok(Self {
            schema_manager,
            model_provider,
            similarity_matcher,
        })
    }

    /// Analyze FHIRPath error and provide enhanced report
    pub async fn analyze_fhirpath_error(
        &self,
        expression: &str,
        error: &FhirPathError,
        context: &AnalysisContext,
    ) -> Result<EnhancedErrorReport, AnalyzerError> {
        let error_category = self.categorize_error(error);
        let severity = self.determine_severity(error);

        let mut report = EnhancedErrorReport {
            original_error: error.clone(),
            error_category,
            suggestions: Vec::new(),
            fixes: Vec::new(),
            related_documentation: Vec::new(),
            severity,
            context_info: None,
        };

        // Generate specific analysis based on error type
        match error {
            FhirPathError::PropertyNotFound {
                type_name,
                property,
                ..
            } => {
                report.suggestions = self
                    .generate_property_suggestions(type_name, property, context)
                    .await?;
                report.fixes = self
                    .generate_property_fixes(type_name, property, expression)
                    .await?;
                report.context_info = Some(format!(
                    "Property '{}' was not found in type '{}'. Check the FHIR specification for available properties.",
                    property, type_name
                ));
            }
            FhirPathError::TypeMismatch { expected, actual } => {
                report.suggestions = self
                    .generate_type_mismatch_suggestions(expected, actual)
                    .await?;
                report.fixes = self
                    .generate_type_conversion_fixes(expected, actual, expression)
                    .await?;
                report.context_info = Some(format!(
                    "Expected type '{}' but got '{}'. Consider type conversion or different navigation path.",
                    expected, actual
                ));
            }
            FhirPathError::InvalidPath(path) => {
                report.suggestions = self.analyze_invalid_path(path, context).await?;
                report.context_info = Some(format!(
                    "Path '{}' is invalid. Check for typos or incorrect property names.",
                    path
                ));
            }
            FhirPathError::FunctionNotFound(function_name) => {
                report.suggestions = self
                    .suggest_similar_functions(function_name, context)
                    .await?;
                report.context_info = Some(format!(
                    "Function '{}' not found. Check function name spelling and availability.",
                    function_name
                ));
            }
            FhirPathError::InvalidFunction {
                function_name,
                reason,
            } => {
                report.suggestions = self
                    .generate_function_usage_suggestions(function_name, reason)
                    .await?;
                report.context_info = Some(format!(
                    "Function '{}' used incorrectly: {}",
                    function_name, reason
                ));
            }
            FhirPathError::ConstraintViolation {
                constraint,
                violation_type,
            } => {
                report.suggestions = self
                    .generate_constraint_suggestions(constraint, violation_type)
                    .await?;
                report.context_info = Some(format!(
                    "Constraint violation in '{}': {}",
                    constraint, violation_type
                ));
            }
            FhirPathError::Generic {
                message,
                error_type,
            } => {
                report.suggestions = self
                    .generate_generic_suggestions(message, error_type, context)
                    .await?;
                report.context_info = Some(format!("Generic error ({}): {}", error_type, message));
            }
        }

        // Add contextual documentation links
        report.related_documentation = self.find_relevant_documentation(error, context).await?;

        Ok(report)
    }

    /// Categorize error for better handling
    fn categorize_error(&self, error: &FhirPathError) -> ErrorCategory {
        match error {
            FhirPathError::PropertyNotFound { .. } => ErrorCategory::FieldAccess,
            FhirPathError::TypeMismatch { .. } => ErrorCategory::TypeSystem,
            FhirPathError::FunctionNotFound(_) | FhirPathError::InvalidFunction { .. } => {
                ErrorCategory::FunctionCall
            }
            FhirPathError::InvalidPath(_) => ErrorCategory::Navigation,
            FhirPathError::ConstraintViolation { .. } => ErrorCategory::Constraint,
            FhirPathError::Generic { .. } => ErrorCategory::Generic,
        }
    }

    /// Determine error severity
    fn determine_severity(&self, error: &FhirPathError) -> ErrorSeverity {
        match error {
            FhirPathError::PropertyNotFound { .. }
            | FhirPathError::FunctionNotFound(_)
            | FhirPathError::InvalidPath(_) => ErrorSeverity::Error,
            FhirPathError::TypeMismatch { .. } => ErrorSeverity::Warning,
            FhirPathError::InvalidFunction { .. } => ErrorSeverity::Warning,
            FhirPathError::ConstraintViolation { .. } => ErrorSeverity::Error,
            FhirPathError::Generic { .. } => ErrorSeverity::Error,
        }
    }

    /// Generate property suggestions for field errors
    async fn generate_property_suggestions(
        &self,
        resource_type: &str,
        invalid_property: &str,
        _context: &AnalysisContext,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        if let Some(type_info) = self.model_provider.get_type_reflection(resource_type).await {
            if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                elements,
                ..
            } = type_info
            {
                // Find similar properties using similarity matching
                for element in elements {
                    let similarity = self
                        .similarity_matcher
                        .calculate_similarity(invalid_property, &element.name);

                    if similarity > 0.5 {
                        let suggestion = ErrorSuggestion {
                            suggestion: format!("Did you mean '{}'?", element.name),
                            confidence: similarity,
                            category: SuggestionCategory::AlternativeField,
                            example: Some(format!("{}.{}", resource_type, element.name)),
                        };
                        suggestions.push(suggestion);
                    }
                }
            }
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 5 suggestions
        suggestions.truncate(5);

        Ok(suggestions)
    }

    /// Generate property fixes
    async fn generate_property_fixes(
        &self,
        resource_type: &str,
        invalid_property: &str,
        _expression: &str,
    ) -> Result<Vec<AutomaticFix>, AnalyzerError> {
        let mut fixes = Vec::new();

        if let Some(type_info) = self.model_provider.get_type_reflection(resource_type).await {
            if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                elements,
                ..
            } = type_info
            {
                // Find the best match for automatic fix
                let mut best_match: Option<(&str, f64)> = None;

                for element in elements.iter() {
                    let similarity = self
                        .similarity_matcher
                        .calculate_similarity(invalid_property, &element.name);

                    if similarity > 0.7 {
                        // High confidence threshold for automatic fixes
                        if let Some((_, best_score)) = best_match {
                            if similarity > best_score {
                                best_match = Some((&element.name, similarity));
                            }
                        } else {
                            best_match = Some((&element.name, similarity));
                        }
                    }
                }

                if let Some((best_property, confidence)) = best_match {
                    let fix = AutomaticFix {
                        description: format!(
                            "Replace '{}' with '{}'",
                            invalid_property, best_property
                        ),
                        original: invalid_property.to_string(),
                        replacement: best_property.to_string(),
                        confidence,
                    };
                    fixes.push(fix);
                }
            }
        }

        Ok(fixes)
    }

    /// Generate type mismatch suggestions
    async fn generate_type_mismatch_suggestions(
        &self,
        expected: &str,
        actual: &str,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        // Common type conversion suggestions
        match (expected, actual) {
            ("string", "integer") | ("string", "decimal") => {
                suggestions.push(ErrorSuggestion {
                    suggestion: "Use toString() to convert number to string".to_string(),
                    confidence: 0.9,
                    category: SuggestionCategory::TypeConversion,
                    example: Some("someField.toString()".to_string()),
                });
            }
            ("integer", "string") => {
                suggestions.push(ErrorSuggestion {
                    suggestion: "Use toInteger() to convert string to integer".to_string(),
                    confidence: 0.9,
                    category: SuggestionCategory::TypeConversion,
                    example: Some("someField.toInteger()".to_string()),
                });
            }
            ("boolean", _) => {
                suggestions.push(ErrorSuggestion {
                    suggestion: "Use exists() to check if field has a value".to_string(),
                    confidence: 0.8,
                    category: SuggestionCategory::TypeConversion,
                    example: Some("someField.exists()".to_string()),
                });
            }
            _ => {
                suggestions.push(ErrorSuggestion {
                    suggestion: format!(
                        "Check if the field returns {} instead of {}",
                        expected, actual
                    ),
                    confidence: 0.6,
                    category: SuggestionCategory::UsageGuidance,
                    example: None,
                });
            }
        }

        Ok(suggestions)
    }

    /// Generate type conversion fixes
    async fn generate_type_conversion_fixes(
        &self,
        expected: &str,
        _actual: &str,
        _expression: &str,
    ) -> Result<Vec<AutomaticFix>, AnalyzerError> {
        let mut fixes = Vec::new();

        // Find the last identifier in the expression to apply conversion
        if let Some(last_word) = _expression.split('.').last() {
            let conversion_function = match expected {
                "string" => Some("toString()"),
                "integer" => Some("toInteger()"),
                "decimal" => Some("toDecimal()"),
                "boolean" => Some("exists()"),
                _ => None,
            };

            if let Some(func) = conversion_function {
                let fix = AutomaticFix {
                    description: format!("Add {} to convert to {}", func, expected),
                    original: last_word.to_string(),
                    replacement: format!("{}.{}", last_word, func),
                    confidence: 0.8,
                };
                fixes.push(fix);
            }
        }

        Ok(fixes)
    }

    /// Analyze invalid path and suggest corrections
    async fn analyze_invalid_path(
        &self,
        path: &str,
        _context: &AnalysisContext,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        // Split path and analyze each segment
        let segments: Vec<&str> = path.split('.').collect();

        if let Some(first_segment) = segments.first() {
            // Check if first segment is a valid resource type
            if let Some(_type_info) = self.model_provider.get_type_reflection(first_segment).await {
                // Valid resource type, check remaining path
                if segments.len() > 1 {
                    suggestions.push(ErrorSuggestion {
                        suggestion: format!("Check property names after '{}'", first_segment),
                        confidence: 0.7,
                        category: SuggestionCategory::PathCorrection,
                        example: Some(format!("{}.name or {}.id", first_segment, first_segment)),
                    });
                }
            } else {
                // Invalid resource type - suggest alternatives
                // TODO: Get list of valid resource types and find similar ones
                suggestions.push(ErrorSuggestion {
                    suggestion: "Check if resource type name is spelled correctly".to_string(),
                    confidence: 0.6,
                    category: SuggestionCategory::PathCorrection,
                    example: Some("Patient, Observation, Bundle, etc.".to_string()),
                });
            }
        }

        Ok(suggestions)
    }

    /// Suggest similar functions
    async fn suggest_similar_functions(
        &self,
        function_name: &str,
        _context: &AnalysisContext,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        // Common FHIRPath functions for similarity matching
        let common_functions = vec![
            "count",
            "length",
            "exists",
            "empty",
            "all",
            "any",
            "select",
            "where",
            "first",
            "last",
            "single",
            "skip",
            "take",
            "distinct",
            "union",
            "toString",
            "toInteger",
            "toDecimal",
            "matches",
            "contains",
        ];

        for func in &common_functions {
            let similarity = self
                .similarity_matcher
                .calculate_similarity(function_name, func);

            if similarity > 0.6 {
                suggestions.push(ErrorSuggestion {
                    suggestion: format!("Did you mean '{}()'?", func),
                    confidence: similarity,
                    category: SuggestionCategory::SimilarFunction,
                    example: Some(format!("collection.{}()", func)),
                });
            }
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions.truncate(3); // Top 3 suggestions

        Ok(suggestions)
    }

    /// Generate function usage suggestions
    async fn generate_function_usage_suggestions(
        &self,
        function_name: &str,
        _reason: &str,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        match function_name {
            "count" | "length" => {
                suggestions.push(ErrorSuggestion {
                    suggestion: "count() and length() work on collections. Use on array fields or after where() filters".to_string(),
                    confidence: 0.9,
                    category: SuggestionCategory::UsageGuidance,
                    example: Some("Patient.name.count() or Patient.telecom.where(use='work').count()".to_string()),
                });
            }
            "first" | "last" | "single" => {
                suggestions.push(ErrorSuggestion {
                    suggestion: "These functions extract elements from collections".to_string(),
                    confidence: 0.9,
                    category: SuggestionCategory::UsageGuidance,
                    example: Some(
                        "Patient.name.first().given or Bundle.entry.first().resource".to_string(),
                    ),
                });
            }
            _ => {
                suggestions.push(ErrorSuggestion {
                    suggestion: format!(
                        "Check the documentation for proper usage of {}()",
                        function_name
                    ),
                    confidence: 0.6,
                    category: SuggestionCategory::UsageGuidance,
                    example: None,
                });
            }
        }

        Ok(suggestions)
    }

    /// Generate constraint suggestions
    async fn generate_constraint_suggestions(
        &self,
        constraint: &str,
        violation_type: &str,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        suggestions.push(ErrorSuggestion {
            suggestion: format!(
                "Constraint '{}' violation: {}. Review constraint logic",
                constraint, violation_type
            ),
            confidence: 0.7,
            category: SuggestionCategory::ConstraintFix,
            example: None,
        });

        Ok(suggestions)
    }

    /// Generate generic suggestions
    async fn generate_generic_suggestions(
        &self,
        message: &str,
        error_type: &str,
        _context: &AnalysisContext,
    ) -> Result<Vec<ErrorSuggestion>, AnalyzerError> {
        let mut suggestions = Vec::new();

        suggestions.push(ErrorSuggestion {
            suggestion: format!(
                "Review the expression for syntax errors. Error type: {}",
                error_type
            ),
            confidence: 0.5,
            category: SuggestionCategory::UsageGuidance,
            example: None,
        });

        if message.contains("syntax") {
            suggestions.push(ErrorSuggestion {
                suggestion: "Check for missing quotes, parentheses, or operators".to_string(),
                confidence: 0.8,
                category: SuggestionCategory::UsageGuidance,
                example: Some("Patient.name.where(family='Smith')".to_string()),
            });
        }

        Ok(suggestions)
    }

    /// Find relevant documentation links
    async fn find_relevant_documentation(
        &self,
        error: &FhirPathError,
        _context: &AnalysisContext,
    ) -> Result<Vec<DocumentationLink>, AnalyzerError> {
        let mut docs = Vec::new();

        match error {
            FhirPathError::PropertyNotFound { .. } => {
                docs.push(DocumentationLink {
                    title: "FHIR Resource Definitions".to_string(),
                    url: "http://hl7.org/fhir/resourcelist.html".to_string(),
                    description: "Official FHIR resource structure documentation".to_string(),
                });
            }
            FhirPathError::FunctionNotFound(_) => {
                docs.push(DocumentationLink {
                    title: "FHIRPath Function Reference".to_string(),
                    url: "http://hl7.org/fhirpath/functions.html".to_string(),
                    description: "Complete list of available FHIRPath functions".to_string(),
                });
            }
            _ => {
                docs.push(DocumentationLink {
                    title: "FHIRPath Specification".to_string(),
                    url: "http://hl7.org/fhirpath/".to_string(),
                    description: "FHIRPath language specification and examples".to_string(),
                });
            }
        }

        Ok(docs)
    }

    /// Get schema manager reference
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_canonical_manager::FcmConfig;
    use octofhir_fhirschema::PackageManagerConfig;

    async fn create_test_reporter() -> Result<AnalyzerErrorReporter, AnalyzerError> {
        let fcm_config = FcmConfig::default();
        let config = PackageManagerConfig::default();
        let schema_manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create schema manager: {}", e),
                })?,
        );

        AnalyzerErrorReporter::new(schema_manager).await
    }

    #[tokio::test]
    async fn test_error_categorization() -> Result<(), Box<dyn std::error::Error>> {
        let reporter = create_test_reporter().await?;

        let error = FhirPathError::PropertyNotFound {
            type_name: "Patient".to_string(),
            property: "invalidField".to_string(),
            source: None,
        };

        let category = reporter.categorize_error(&error);
        assert_eq!(category, ErrorCategory::FieldAccess);

        Ok(())
    }

    #[tokio::test]
    async fn test_property_suggestions() -> Result<(), Box<dyn std::error::Error>> {
        let reporter = create_test_reporter().await?;
        let context = AnalysisContext::default();

        let suggestions = reporter
            .generate_property_suggestions("Patient", "nam", &context)
            .await?;

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.suggestion.contains("name")));

        Ok(())
    }

    #[tokio::test]
    async fn test_function_suggestions() -> Result<(), Box<dyn std::error::Error>> {
        let reporter = create_test_reporter().await?;
        let context = AnalysisContext::default();

        let suggestions = reporter
            .suggest_similar_functions("coutn", &context) // Typo in "count"
            .await?;

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.suggestion.contains("count")));

        Ok(())
    }
}
