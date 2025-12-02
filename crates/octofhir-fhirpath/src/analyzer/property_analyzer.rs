use std::sync::Arc;

use crate::ast::analysis::AnalysisMetadata;
use crate::core::{FhirPathError, ModelProvider, SourceLocation};
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use octofhir_fhir_model::TypeInfo;

/// Result type for property analysis operations
pub type AnalysisResult = Result<AnalysisMetadata, FhirPathError>;

/// Property analyzer for enhanced validation with rich diagnostics
pub struct PropertyAnalyzer {
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
}

/// Property suggestion with confidence scoring
pub struct PropertySuggestion {
    pub property_name: String,
    pub confidence: f32,
    pub description: Option<String>,
}

impl PropertyAnalyzer {
    /// Create a new PropertyAnalyzer with the given ModelProvider
    pub fn new(model_provider: Arc<dyn ModelProvider + Send + Sync>) -> Self {
        Self { model_provider }
    }

    /// Enhanced property validation with rich diagnostics
    pub async fn validate_property_access(
        &self,
        parent_type: &TypeInfo,
        property_name: &str,
        location: Option<SourceLocation>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Special case: handle resourceType property on Reference types
        if property_name == "resourceType" && self.is_reference_type(parent_type) {
            // resourceType is a built-in property on Reference types
            metadata.type_info = Some(TypeInfo {
                type_name: "String".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("String".to_string()),
            });
            return Ok(metadata);
        }

        // Try to get element type for the property
        match self
            .model_provider
            .get_element_type(parent_type, property_name)
            .await
        {
            Ok(Some(element_type)) => {
                // Property found successfully
                metadata.type_info = Some(element_type);
                Ok(metadata)
            }
            Ok(None) | Err(_) => {
                // Property not found - generate enhanced diagnostic
                self.generate_property_not_found_diagnostic(
                    parent_type,
                    property_name,
                    location,
                    &mut metadata,
                )
                .await;
                Ok(metadata)
            }
        }
    }

    /// Choice type validation (valueX patterns)
    pub async fn validate_choice_property(
        &self,
        parent_type: &TypeInfo,
        property_name: &str,
        location: Option<SourceLocation>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Check if this is a choice property pattern
        if let Some(choice_info) = self
            .analyze_choice_property(parent_type, property_name)
            .await?
        {
            match choice_info {
                ChoicePropertyResult::Valid(type_info) => {
                    metadata.type_info = Some(type_info);
                }
                ChoicePropertyResult::Ambiguous(base_name, available_choices) => {
                    let diagnostic = Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0202".to_string(),
                            namespace: None,
                        },
                        message: format!("Ambiguous choice property '{property_name}'"),
                        location,
                        related: vec![],
                    };

                    // Add help text with specific suggestions
                    let help_text = if available_choices.is_empty() {
                        format!("Use specific choice type for '{base_name}' property")
                    } else {
                        let choices = available_choices.join(", ");
                        format!("Use specific choice type: {choices}")
                    };

                    // Create enhanced diagnostic with help
                    let enhanced_diagnostic = self.create_enhanced_diagnostic(
                        diagnostic,
                        Some(help_text),
                        Some(format!(
                            "Available choices for '{base_name}': {}",
                            available_choices.join(", ")
                        )),
                    );

                    metadata.add_diagnostic(enhanced_diagnostic);
                }
                ChoicePropertyResult::InvalidSuffix(
                    base_name,
                    attempted_suffix,
                    valid_suffixes,
                ) => {
                    let diagnostic = Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: DiagnosticCode {
                            code: "FP0203".to_string(),
                            namespace: None,
                        },
                        message: format!(
                            "Invalid choice property suffix '{attempted_suffix}' for '{base_name}'"
                        ),
                        location,
                        related: vec![],
                    };

                    let help_text = if !valid_suffixes.is_empty() {
                        let suggestions = valid_suffixes
                            .iter()
                            .map(|suffix| format!("{base_name}{suffix}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("Valid options: {suggestions}")
                    } else {
                        format!("Property '{base_name}' is not a choice type")
                    };

                    let enhanced_diagnostic =
                        self.create_enhanced_diagnostic(diagnostic, Some(help_text), None);

                    metadata.add_diagnostic(enhanced_diagnostic);
                }
            }
        } else {
            // Not a choice property, use regular property validation
            return self
                .validate_property_access(parent_type, property_name, location)
                .await;
        }

        Ok(metadata)
    }

    /// Enhanced resource type validation
    pub async fn validate_resource_type(
        &self,
        resource_type: &str,
        location: Option<SourceLocation>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // First check: Try to get the type directly from ModelProvider
        match self.model_provider.get_type(resource_type).await {
            Ok(Some(type_info)) => {
                // Valid resource type
                metadata.type_info = Some(type_info);
                return Ok(metadata);
            }
            Ok(None) | Err(_) => {
                // Continue to second check...
            }
        }

        // Second check: Check against resource types list from ModelProvider
        match self
            .model_provider
            .resource_type_exists(resource_type)
            .await
        {
            Ok(true) => {
                // Valid resource type from resource list - create basic TypeInfo
                metadata.type_info = Some(TypeInfo {
                    type_name: resource_type.to_string(),
                    singleton: Some(true),
                    is_empty: Some(false),
                    namespace: Some("FHIR".to_string()),
                    name: Some(resource_type.to_string()),
                });
                return Ok(metadata);
            }
            Ok(false) | Err(_) => {
                // Invalid resource type - provide suggestions
                let suggestions = self.suggest_resource_types(resource_type).await;

                let diagnostic = Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "FP0201".to_string(),
                        namespace: None,
                    },
                    message: format!("Unknown resource type '{resource_type}'"),
                    location,
                    related: vec![],
                };

                // Create enhanced diagnostic with suggestions
                let help_text = if !suggestions.is_empty() {
                    Some(format!("Did you mean '{}'?", suggestions[0].property_name))
                } else {
                    None
                };

                let note_text = if suggestions.len() > 1 {
                    let available = suggestions
                        .iter()
                        .take(5)
                        .map(|s| s.property_name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    Some(format!("Available resource types: {available}"))
                } else {
                    None
                };

                let enhanced_diagnostic =
                    self.create_enhanced_diagnostic(diagnostic, help_text, note_text);
                metadata.add_diagnostic(enhanced_diagnostic);
            }
        }

        Ok(metadata)
    }

    /// Validate resourceType property on Reference types
    pub async fn validate_reference_resource_type(
        &self,
        parent_type: &TypeInfo,
        resource_type_value: &str,
        location: Option<SourceLocation>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Check if parent type is a Reference or has Reference-like properties
        if !self.is_reference_type(parent_type) {
            // Not a reference type, just validate as regular resource type
            return self
                .validate_resource_type(resource_type_value, location)
                .await;
        }

        // For Reference types, validate that the resourceType value is a valid resource
        match self
            .model_provider
            .resource_type_exists(resource_type_value)
            .await
        {
            Ok(true) => {
                // Valid resource type for Reference
                metadata.type_info = Some(TypeInfo {
                    type_name: "String".to_string(), // resourceType property is always String
                    singleton: Some(true),
                    is_empty: Some(false),
                    namespace: Some("System".to_string()),
                    name: Some("String".to_string()),
                });
            }
            Ok(false) | Err(_) => {
                // Invalid resource type for Reference
                let suggestions = self.suggest_resource_types(resource_type_value).await;

                let diagnostic = Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    code: DiagnosticCode {
                        code: "FP0202".to_string(),
                        namespace: None,
                    },
                    message: format!("Invalid resourceType '{resource_type_value}' for Reference"),
                    location,
                    related: vec![],
                };

                let help_text = if !suggestions.is_empty() {
                    Some(format!("Did you mean '{}'?", suggestions[0].property_name))
                } else {
                    None
                };

                let note_text = if suggestions.len() > 1 {
                    let available = suggestions
                        .iter()
                        .take(5)
                        .map(|s| s.property_name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    Some(format!("Valid resource types for Reference: {available}"))
                } else {
                    None
                };

                let enhanced_diagnostic =
                    self.create_enhanced_diagnostic(diagnostic, help_text, note_text);
                metadata.add_diagnostic(enhanced_diagnostic);
            }
        }

        Ok(metadata)
    }

    /// Property suggestions for typos
    pub async fn suggest_properties(
        &self,
        parent_type: &TypeInfo,
        attempted_property: &str,
    ) -> Vec<PropertySuggestion> {
        let element_names = self.model_provider.get_element_names(parent_type);
        let mut suggestions = Vec::new();

        for property in element_names {
            let distance = self.levenshtein_distance(attempted_property, &property);
            let max_len = attempted_property.len().max(property.len());

            if max_len == 0 {
                continue;
            }

            let confidence = 1.0 - (distance as f32 / max_len as f32);

            // Only include suggestions with reasonable confidence
            if confidence >= 0.4 {
                suggestions.push(PropertySuggestion {
                    property_name: property,
                    confidence,
                    description: None,
                });
            }
        }

        // Sort by confidence (highest first)
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 5 suggestions
        suggestions.truncate(5);

        suggestions
    }

    /// Generate enhanced diagnostic for property not found
    async fn generate_property_not_found_diagnostic(
        &self,
        parent_type: &TypeInfo,
        property_name: &str,
        location: Option<SourceLocation>,
        metadata: &mut AnalysisMetadata,
    ) {
        let type_display = if parent_type.singleton.unwrap_or(true) {
            parent_type.type_name.clone()
        } else {
            format!("{}[]", parent_type.type_name)
        };

        let suggestions = self.suggest_properties(parent_type, property_name).await;

        let diagnostic = Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "PROPERTY_NOT_FOUND".to_string(),
                namespace: Some("fhirpath".to_string()),
            },
            message: format!("prop '{property_name}' not found on {type_display}"),
            location,
            related: vec![],
        };

        // Create enhanced diagnostic with suggestions
        let help_text = if !suggestions.is_empty() {
            Some(format!("Did you mean '{}'?", suggestions[0].property_name))
        } else {
            None
        };

        let note_text = if suggestions.len() > 1 {
            let available = suggestions
                .iter()
                .take(5)
                .map(|s| s.property_name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("Similar properties: {available}"))
        } else {
            None
        };

        let enhanced_diagnostic = self.create_enhanced_diagnostic(diagnostic, help_text, note_text);
        metadata.add_diagnostic(enhanced_diagnostic);

        // Set type to Any for recovery
        metadata.type_info = Some(TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        });
    }

    /// Analyze choice property patterns
    pub async fn analyze_choice_property(
        &self,
        parent_type: &TypeInfo,
        property_name: &str,
    ) -> Result<Option<ChoicePropertyResult>, FhirPathError> {
        // Check for common choice element patterns
        let choice_bases = [
            "value",
            "onset",
            "effective",
            "occurs",
            "multipleBirth",
            "deceased",
        ];

        for base in &choice_bases {
            if property_name == *base {
                // Accessing the base property directly - this is ambiguous
                let available_choices = self.get_choice_types_for_base(parent_type, base).await;
                return Ok(Some(ChoicePropertyResult::Ambiguous(
                    base.to_string(),
                    available_choices,
                )));
            } else if property_name.starts_with(base) && property_name.len() > base.len() {
                // Accessing a specific choice type
                let suffix = &property_name[base.len()..];
                let valid_suffixes = self.get_choice_types_for_base(parent_type, base).await;

                if valid_suffixes.contains(&suffix.to_string()) {
                    // Valid choice type - get the actual type
                    if let Ok(Some(element_type)) = self
                        .model_provider
                        .get_element_type(parent_type, property_name)
                        .await
                    {
                        return Ok(Some(ChoicePropertyResult::Valid(element_type)));
                    }
                } else if !valid_suffixes.is_empty() {
                    // Invalid suffix for a known choice base
                    return Ok(Some(ChoicePropertyResult::InvalidSuffix(
                        base.to_string(),
                        suffix.to_string(),
                        valid_suffixes,
                    )));
                }
            }
        }

        Ok(None)
    }

    /// Get available choice types for a base property
    async fn get_choice_types_for_base(&self, _parent_type: &TypeInfo, base: &str) -> Vec<String> {
        // For now, return common FHIR choice types
        // This should be enhanced to use ModelProvider's choice type information
        match base {
            "value" => vec![
                "String".to_string(),
                "Quantity".to_string(),
                "Boolean".to_string(),
                "CodeableConcept".to_string(),
                "Integer".to_string(),
                "Decimal".to_string(),
                "Date".to_string(),
                "DateTime".to_string(),
                "Time".to_string(),
            ],
            "onset" => vec![
                "DateTime".to_string(),
                "Age".to_string(),
                "Period".to_string(),
                "Range".to_string(),
                "String".to_string(),
            ],
            "effective" => vec![
                "DateTime".to_string(),
                "Period".to_string(),
                "Timing".to_string(),
                "Instant".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Suggest resource types using fuzzy matching
    pub async fn suggest_resource_types(&self, attempted_type: &str) -> Vec<PropertySuggestion> {
        // Get available resource types from model provider
        let resource_types = match self.model_provider.get_resource_types().await {
            Ok(types) => types,
            Err(_) => return vec![], // Return empty suggestions on error
        };
        let mut suggestions = Vec::new();

        for resource_type in &resource_types {
            let distance = self.levenshtein_distance(attempted_type, resource_type);
            let max_len = attempted_type.len().max(resource_type.len());

            if max_len == 0 {
                continue;
            }

            let confidence = 1.0 - (distance as f32 / max_len as f32);

            // Only include suggestions with reasonable confidence
            if confidence >= 0.3 {
                suggestions.push(PropertySuggestion {
                    property_name: resource_type.clone(),
                    confidence,
                    description: None,
                });
            }
        }

        // Sort by confidence (highest first)
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 5 suggestions
        suggestions.truncate(5);

        suggestions
    }

    /// Create enhanced diagnostic with help and note text
    fn create_enhanced_diagnostic(
        &self,
        mut diagnostic: Diagnostic,
        help: Option<String>,
        note: Option<String>,
    ) -> Diagnostic {
        // For now, append help and note to the message
        // This can be enhanced when we have full Ariadne integration

        if let Some(help_text) = help {
            diagnostic.message = format!("{}\n  = help: {}", diagnostic.message, help_text);
        }

        if let Some(note_text) = note {
            diagnostic.message = format!("{}\n  = note: {}", diagnostic.message, note_text);
        }

        diagnostic
    }

    /// Check if a type is a Reference type or contains Reference-like elements
    fn is_reference_type(&self, type_info: &TypeInfo) -> bool {
        // Check if the type is specifically a Reference
        type_info.type_name == "Reference" ||
        type_info.type_name == "ResourceReference" ||
        // Check if it's a collection of References
        type_info.type_name.ends_with("Reference[]") ||
        // Check for common reference-like patterns
        type_info.type_name.contains("Reference")
    }

    /// Calculate Levenshtein distance between two strings
    pub fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        if a_chars.is_empty() {
            return b_chars.len();
        }
        if b_chars.is_empty() {
            return a_chars.len();
        }

        let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

        #[allow(clippy::needless_range_loop)]
        for i in 0..=a_chars.len() {
            matrix[i][0] = i;
        }
        #[allow(clippy::needless_range_loop)]
        for j in 0..=b_chars.len() {
            matrix[0][j] = j;
        }

        #[allow(clippy::needless_range_loop)]
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
}

/// Result of choice property analysis
pub enum ChoicePropertyResult {
    /// Valid choice property with resolved type
    Valid(TypeInfo),
    /// Ambiguous base property (e.g., accessing 'value' instead of 'valueString')
    Ambiguous(String, Vec<String>),
    /// Invalid suffix for choice property
    InvalidSuffix(String, String, Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    fn create_test_analyzer() -> PropertyAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        PropertyAnalyzer::new(provider)
    }

    fn create_test_type_info(type_name: &str) -> TypeInfo {
        TypeInfo {
            type_name: type_name.to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("FHIR".to_string()),
            name: Some(type_name.to_string()),
        }
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = create_test_analyzer();
        // Just verify it can be created without panic
        assert_eq!(
            std::mem::size_of_val(&analyzer),
            std::mem::size_of::<PropertyAnalyzer>()
        );
    }

    #[test]
    fn test_levenshtein_distance() {
        let analyzer = create_test_analyzer();

        assert_eq!(analyzer.levenshtein_distance("test", "test"), 0);
        assert_eq!(analyzer.levenshtein_distance("test", "best"), 1);
        assert_eq!(analyzer.levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(analyzer.levenshtein_distance("", "abc"), 3);
        assert_eq!(analyzer.levenshtein_distance("abc", ""), 3);
    }

    #[tokio::test]
    #[ignore = "experimental analyzer feature"]
    async fn test_property_suggestions() {
        let analyzer = create_test_analyzer();
        let type_info = create_test_type_info("Patient");

        // EmptyModelProvider will return empty element names
        let suggestions = analyzer.suggest_properties(&type_info, "nam").await;
        assert!(suggestions.is_empty()); // EmptyModelProvider has no properties
    }

    #[tokio::test]
    async fn test_resource_type_validation() {
        let analyzer = create_test_analyzer();

        // EmptyModelProvider will return None for all types
        let result = analyzer.validate_resource_type("Patients", None).await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert!(!metadata.diagnostics.is_empty()); // Should have error diagnostic
        assert!(
            metadata
                .diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error)
        );
    }

    #[tokio::test]
    async fn test_choice_property_validation() {
        let analyzer = create_test_analyzer();
        let type_info = create_test_type_info("Observation");

        // Test ambiguous choice property
        let result = analyzer
            .validate_choice_property(&type_info, "value", None)
            .await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // Should detect ambiguous choice property
        assert!(!metadata.diagnostics.is_empty());
    }

    #[tokio::test]
    async fn test_property_access_validation() {
        let analyzer = create_test_analyzer();
        let type_info = create_test_type_info("Patient");

        // EmptyModelProvider will not find any properties
        let result = analyzer
            .validate_property_access(&type_info, "invalidProperty", None)
            .await;
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert!(!metadata.diagnostics.is_empty()); // Should have error diagnostic
        assert!(metadata.type_info.is_some()); // Should have recovery type
    }
}
