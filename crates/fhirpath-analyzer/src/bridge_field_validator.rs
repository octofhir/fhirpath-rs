//! Bridge API-enabled field validation with enhanced suggestions
//!
//! This module provides field validation that leverages the Bridge Support Architecture
//! for O(1) field validation operations and similarity-based suggestion generation.

use octofhir_canonical_manager::FcmConfig;
use octofhir_fhirpath_model::{FhirSchemaModelProvider, provider::ModelProvider};
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use std::sync::Arc;
use thiserror::Error;

use crate::error::{ValidationError, ValidationErrorType};

/// Bridge validation result with enhanced information
#[derive(Debug, Clone)]
pub struct BridgeValidationResult {
    /// Whether the field is valid
    pub is_valid: bool,
    /// The field path that was validated
    pub field_path: String,
    /// The resource type that was validated
    pub resource_type: String,
    /// Suggestions for invalid fields
    pub suggestions: Vec<String>,
    /// Context information for the validation
    pub context_info: Option<String>,
    /// Performance optimization hints
    pub optimization_hints: Vec<String>,
    /// Additional property information if available
    pub property_info: Option<PropertyInfo>,
}

/// Property information from bridge API
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    /// Property name
    pub name: String,
    /// Property type
    pub property_type: String,
    /// Cardinality information
    pub cardinality: BridgeCardinality,
    /// Whether this is a choice type
    pub is_choice_type: bool,
    /// Choice alternatives if applicable
    pub choice_alternatives: Vec<String>,
}

/// Bridge cardinality information
#[derive(Debug, Clone)]
pub struct BridgeCardinality {
    /// Minimum occurrences
    pub min: u32,
    /// Maximum occurrences (None for unlimited)
    pub max: Option<u32>,
    /// String representation (e.g., "0..1", "1..*")
    pub cardinality_string: String,
}

/// Error types for bridge field validator
#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("Validation error for field '{field_path}' in resource '{resource_type}': {source}")]
    ValidationError {
        resource_type: String,
        field_path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Navigation error for path '{path}' in resource '{resource_type}': {source}")]
    NavigationError {
        resource_type: String,
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Constraint validation error for constraint '{constraint}': {source}")]
    ConstraintValidationError {
        constraint: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Bridge API error: {message}")]
    BridgeApiError { message: String },

    #[error("Initialization error: {message}")]
    InitializationError { message: String },
}

/// Bridge-enabled field validator with enhanced suggestions
pub struct AnalyzerFieldValidator {
    /// Schema manager for bridge API operations
    schema_manager: Arc<FhirSchemaPackageManager>,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Similarity matcher for suggestions
    similarity_matcher: SimilarityMatcher,
}

/// Similarity matcher for generating field suggestions
pub struct SimilarityMatcher {
    /// Threshold for similarity matching
    similarity_threshold: f64,
}

impl SimilarityMatcher {
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.5,
        }
    }

    /// Calculate similarity between two strings using Levenshtein distance
    pub fn calculate_similarity(&self, a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }

        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let len_a = a_chars.len();
        let len_b = b_chars.len();

        if len_a == 0 {
            return 0.0;
        }
        if len_b == 0 {
            return 0.0;
        }

        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

        // Initialize base cases
        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };

                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        let max_len = std::cmp::max(len_a, len_b);
        1.0 - (matrix[len_a][len_b] as f64 / max_len as f64)
    }

    /// Check if two strings are similar enough
    pub fn is_similar(&self, a: &str, b: &str) -> bool {
        self.calculate_similarity(a, b) >= self.similarity_threshold
    }
}

impl AnalyzerFieldValidator {
    /// Create new analyzer field validator with bridge support
    pub async fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Result<Self, AnalyzerError> {
        // Create model provider from schema manager
        let model_provider: Arc<dyn ModelProvider> =
            Arc::new(FhirSchemaModelProvider::new().await.map_err(|e| {
                AnalyzerError::InitializationError {
                    message: format!("Failed to create model provider: {}", e),
                }
            })?);

        let similarity_matcher = SimilarityMatcher::new();

        Ok(Self {
            schema_manager,
            model_provider,
            similarity_matcher,
        })
    }

    /// Create from bridge configuration
    pub async fn from_bridge_config(
        fcm_config: FcmConfig,
        config: PackageManagerConfig,
    ) -> Result<Self, AnalyzerError> {
        let schema_manager = Arc::new(
            FhirSchemaPackageManager::new(fcm_config, config)
                .await
                .map_err(|e| AnalyzerError::InitializationError {
                    message: format!("Failed to create schema manager: {}", e),
                })?,
        );

        Self::new(schema_manager).await
    }

    /// Validate field with comprehensive bridge-enabled analysis
    pub async fn validate_field(
        &self,
        resource_type: &str,
        field_path: &str,
    ) -> Result<BridgeValidationResult, AnalyzerError> {
        // Use bridge API to validate field existence
        let field_exists = self.check_field_exists(resource_type, field_path).await?;

        let mut result = BridgeValidationResult {
            is_valid: field_exists,
            field_path: field_path.to_string(),
            resource_type: resource_type.to_string(),
            suggestions: Vec::new(),
            context_info: None,
            optimization_hints: Vec::new(),
            property_info: None,
        };

        if field_exists {
            // Field is valid - get property information
            result.property_info = self.get_property_info(resource_type, field_path).await?;
            result.optimization_hints = self.generate_optimization_hints(&result).await?;
        } else {
            // Field is invalid - generate suggestions
            result.suggestions = self
                .generate_field_suggestions(resource_type, field_path)
                .await?;
            result.context_info = Some(self.build_context_info(resource_type).await?);
        }

        Ok(result)
    }

    /// Check if a field exists using bridge API
    async fn check_field_exists(
        &self,
        resource_type: &str,
        field_path: &str,
    ) -> Result<bool, AnalyzerError> {
        // Use model provider to check field existence
        match self.model_provider.get_type_reflection(resource_type).await {
            Some(type_info) => Ok(self.field_exists_in_type_info(&type_info, field_path)),
            None => Ok(false), // Resource type doesn't exist
        }
    }

    /// Check if field exists in type information
    fn field_exists_in_type_info(
        &self,
        type_info: &octofhir_fhirpath_model::provider::TypeReflectionInfo,
        field_path: &str,
    ) -> bool {
        use octofhir_fhirpath_model::provider::TypeReflectionInfo;

        match type_info {
            TypeReflectionInfo::ClassInfo { elements, .. } => {
                // Check if any element matches the field path
                elements.iter().any(|element| {
                    element.name == field_path
                        || self.matches_choice_type(&element.name, field_path)
                })
            }
            _ => false,
        }
    }

    /// Check if field matches a choice type pattern
    fn matches_choice_type(&self, schema_field: &str, requested_field: &str) -> bool {
        // Handle FHIR choice types like value[x] -> valueQuantity, valueString, etc.
        if schema_field.ends_with("[x]") {
            let base = &schema_field[..schema_field.len() - 3]; // Remove "[x]"
            return requested_field.starts_with(base) && requested_field.len() > base.len();
        }
        false
    }

    /// Get property information for a valid field
    async fn get_property_info(
        &self,
        resource_type: &str,
        field_path: &str,
    ) -> Result<Option<PropertyInfo>, AnalyzerError> {
        match self.model_provider.get_type_reflection(resource_type).await {
            Some(type_info) => Ok(self.extract_property_info(&type_info, field_path)),
            None => Ok(None),
        }
    }

    /// Extract property information from type reflection
    fn extract_property_info(
        &self,
        type_info: &octofhir_fhirpath_model::provider::TypeReflectionInfo,
        field_path: &str,
    ) -> Option<PropertyInfo> {
        use octofhir_fhirpath_model::provider::TypeReflectionInfo;

        if let TypeReflectionInfo::ClassInfo { elements, .. } = type_info {
            for element in elements {
                if element.name == field_path {
                    let cardinality = BridgeCardinality {
                        min: if element.is_required() { 1 } else { 0 },
                        max: element.max_cardinality,
                        cardinality_string: format!(
                            "{}..{}",
                            if element.is_required() { 1 } else { 0 },
                            element
                                .max_cardinality
                                .map_or("*".to_string(), |m| m.to_string())
                        ),
                    };

                    return Some(PropertyInfo {
                        name: element.name.clone(),
                        property_type: self.extract_type_name(&element.type_info),
                        cardinality,
                        is_choice_type: element.name.ends_with("[x]"),
                        choice_alternatives: if element.name.ends_with("[x]") {
                            self.get_choice_alternatives(&element.name)
                        } else {
                            Vec::new()
                        },
                    });
                }
            }
        }
        None
    }

    /// Extract type name from type reflection info
    fn extract_type_name(
        &self,
        type_info: &octofhir_fhirpath_model::provider::TypeReflectionInfo,
    ) -> String {
        use octofhir_fhirpath_model::provider::TypeReflectionInfo;

        match type_info {
            TypeReflectionInfo::SimpleType { name, .. } => name.clone(),
            TypeReflectionInfo::ClassInfo { name, .. } => name.clone(),
            TypeReflectionInfo::ListType { element_type } => {
                format!("List<{}>", self.extract_type_name(element_type))
            }
            TypeReflectionInfo::TupleType { .. } => "Tuple".to_string(),
        }
    }

    /// Get choice alternatives for choice types
    fn get_choice_alternatives(&self, field_name: &str) -> Vec<String> {
        if !field_name.ends_with("[x]") {
            return Vec::new();
        }

        let base = &field_name[..field_name.len() - 3];

        // Common FHIR choice type alternatives
        match base {
            "value" => vec![
                format!("{}String", base),
                format!("{}Boolean", base),
                format!("{}Integer", base),
                format!("{}Decimal", base),
                format!("{}DateTime", base),
                format!("{}Quantity", base),
                format!("{}CodeableConcept", base),
            ],
            "effective" => vec![
                format!("{}DateTime", base),
                format!("{}Period", base),
                format!("{}Instant", base),
            ],
            "onset" => vec![
                format!("{}DateTime", base),
                format!("{}Age", base),
                format!("{}Period", base),
                format!("{}Range", base),
                format!("{}String", base),
            ],
            "deceased" => vec![format!("{}Boolean", base), format!("{}DateTime", base)],
            "multipleBirth" => vec![format!("{}Boolean", base), format!("{}Integer", base)],
            _ => Vec::new(),
        }
    }

    /// Generate field suggestions using similarity matching
    async fn generate_field_suggestions(
        &self,
        resource_type: &str,
        invalid_field: &str,
    ) -> Result<Vec<String>, AnalyzerError> {
        let mut suggestions = Vec::new();

        if let Some(type_info) = self.model_provider.get_type_reflection(resource_type).await {
            if let octofhir_fhirpath_model::provider::TypeReflectionInfo::ClassInfo {
                elements,
                ..
            } = type_info
            {
                for element in elements {
                    let similarity = self
                        .similarity_matcher
                        .calculate_similarity(invalid_field, &element.name);

                    if similarity > 0.5 {
                        // Configurable threshold
                        suggestions.push(element.name.clone());
                    }
                }

                // Sort by similarity score (this is a simplified version)
                suggestions.sort_by(|a, b| {
                    let sim_a = self
                        .similarity_matcher
                        .calculate_similarity(invalid_field, a);
                    let sim_b = self
                        .similarity_matcher
                        .calculate_similarity(invalid_field, b);
                    sim_b
                        .partial_cmp(&sim_a)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // Limit to top 5 suggestions
                suggestions.truncate(5);
            }
        }

        Ok(suggestions)
    }

    /// Build context information for validation errors
    async fn build_context_info(&self, resource_type: &str) -> Result<String, AnalyzerError> {
        Ok(format!(
            "Validating fields in FHIR resource type '{}'. Available fields are determined by the FHIR specification.",
            resource_type
        ))
    }

    /// Generate optimization hints for valid fields
    async fn generate_optimization_hints(
        &self,
        _result: &BridgeValidationResult,
    ) -> Result<Vec<String>, AnalyzerError> {
        // TODO: Add optimization hints based on field usage patterns
        Ok(vec![
            "Consider using caching for repeated field validations".to_string(),
            "Path navigation can be optimized with bridge API".to_string(),
        ])
    }

    /// Get schema manager reference
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }

    /// Convert to validation errors for compatibility with existing analyzer
    pub fn to_validation_errors(&self, result: &BridgeValidationResult) -> Vec<ValidationError> {
        if result.is_valid {
            Vec::new()
        } else {
            vec![ValidationError {
                message: format!(
                    "Field '{}' does not exist in FHIR resource type '{}'",
                    result.field_path, result.resource_type
                ),
                error_type: ValidationErrorType::InvalidField,
                location: None,
                suggestions: result.suggestions.clone(),
            }]
        }
    }
}

impl Default for SimilarityMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_similarity_matcher() {
        let matcher = SimilarityMatcher::new();

        // Test exact match
        assert_eq!(matcher.calculate_similarity("name", "name"), 1.0);

        // Test similar strings
        assert!(matcher.calculate_similarity("name", "nam") > 0.5);
        assert!(matcher.is_similar("patient", "patien"));

        // Test dissimilar strings
        assert!(matcher.calculate_similarity("name", "xyz") < 0.5);
        assert!(!matcher.is_similar("name", "completely_different"));
    }

    #[test]
    fn test_choice_type_matching() {
        let validator = AnalyzerFieldValidator {
            schema_manager: Arc::new(
                // This would normally be created properly, but for testing we'll use a placeholder
                unsafe { std::mem::zeroed() },
            ),
            model_provider: Arc::new(
                octofhir_fhirpath_model::mock_provider::MockModelProvider::new(),
            ),
            similarity_matcher: SimilarityMatcher::new(),
        };

        // Test choice type matching
        assert!(validator.matches_choice_type("value[x]", "valueQuantity"));
        assert!(validator.matches_choice_type("value[x]", "valueString"));
        assert!(!validator.matches_choice_type("value[x]", "name"));
        assert!(!validator.matches_choice_type("name", "valueQuantity"));
    }
}
