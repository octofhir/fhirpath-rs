use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use crate::ast::analysis::AnalysisMetadata;
use crate::core::model_provider::ChoiceTypeInfo;
use crate::core::{FP0151, FhirPathError, ModelProvider, SourceLocation};
use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use octofhir_fhir_model::TypeInfo;

/// Result type for choice type analysis operations
pub type AnalysisResult = Result<AnalysisMetadata, FhirPathError>;

/// Cache for choice type lookups to improve performance
pub struct ChoiceTypeCache {
    cache: std::collections::HashMap<(String, String), Option<Vec<ChoiceTypeInfo>>>,
    cache_stats: RefCell<CacheStatistics>,
}

#[derive(Clone)]
pub struct CacheStatistics {
    pub hits: usize,
    pub misses: usize,
}

impl Default for ChoiceTypeCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ChoiceTypeCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            cache_stats: RefCell::new(CacheStatistics { hits: 0, misses: 0 }),
        }
    }

    pub fn get(
        &self,
        parent_type: &str,
        property_name: &str,
    ) -> Option<&Option<Vec<ChoiceTypeInfo>>> {
        let key = (parent_type.to_string(), property_name.to_string());
        if let Some(cached) = self.cache.get(&key) {
            self.cache_stats.borrow_mut().hits += 1;
            Some(cached)
        } else {
            self.cache_stats.borrow_mut().misses += 1;
            None
        }
    }

    pub fn insert(
        &mut self,
        parent_type: &str,
        property_name: &str,
        value: Option<Vec<ChoiceTypeInfo>>,
    ) {
        let key = (parent_type.to_string(), property_name.to_string());
        self.cache.insert(key, value);
    }

    pub fn get_stats(&self) -> CacheStatistics {
        self.cache_stats.borrow().clone()
    }
}

/// Choice type analyzer for validating choice property usage with full ModelProvider capabilities
pub struct ChoiceTypeAnalyzer {
    model_provider: Arc<dyn ModelProvider>,
    cache: ChoiceTypeCache,
}

impl ChoiceTypeAnalyzer {
    /// Create a new ChoiceTypeAnalyzer with the given ModelProvider
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self {
            model_provider,
            cache: ChoiceTypeCache::new(),
        }
    }

    /// Analyze choice type usage with full ModelProvider capabilities
    /// Leverages ModelProvider::get_choice_types() for comprehensive validation
    pub async fn analyze_choice_property(
        &mut self,
        parent_type: &str,
        property_name: &str,
        location: Range<usize>,
    ) -> AnalysisResult {
        // Check cache first
        let cached_result = self.cache.get(parent_type, property_name).cloned();
        if let Some(cached_choice_types) = cached_result {
            return self.process_choice_types(
                property_name,
                cached_choice_types.as_ref(),
                location,
            );
        }

        // Leverage ModelProvider::get_choice_types()
        let choice_types = self
            .model_provider
            .get_choice_types(parent_type, property_name)
            .await
            .map_err(|e| FhirPathError::model_error(FP0151, format!("ModelProvider error: {e}")))?;

        // Cache the result
        self.cache
            .insert(parent_type, property_name, choice_types.clone());

        if let Some(choices) = choice_types {
            self.validate_choice_usage(property_name, &choices, location)
        } else {
            // Not a choice type - return success
            Ok(AnalysisMetadata::new())
        }
    }

    /// Process cached choice types
    fn process_choice_types(
        &self,
        property_name: &str,
        choice_types: Option<&Vec<ChoiceTypeInfo>>,
        location: Range<usize>,
    ) -> AnalysisResult {
        if let Some(choices) = choice_types {
            self.validate_choice_usage(property_name, choices, location)
        } else {
            // Not a choice type - return success
            Ok(AnalysisMetadata::new())
        }
    }

    /// Validate choice type usage against available choice types
    fn validate_choice_usage(
        &self,
        property_name: &str,
        choices: &[ChoiceTypeInfo],
        location: Range<usize>,
    ) -> AnalysisResult {
        let mut metadata = AnalysisMetadata::new();

        // Check if this is a polymorphic property (valueX pattern)
        if self.is_polymorphic_property(property_name) {
            let (base_name, suffix) = self.parse_polymorphic_property(property_name);

            // Validate that the suffix corresponds to a valid choice type
            let valid_choice = choices
                .iter()
                .any(|choice| self.matches_choice_type(&suffix, choice));

            if !valid_choice {
                let diagnostic = self.create_invalid_choice_diagnostic(
                    property_name,
                    &base_name,
                    &suffix,
                    choices,
                    location,
                );
                metadata.add_diagnostic(diagnostic);
            } else {
                // Set the correct type for valid choice
                if let Some(choice) = choices
                    .iter()
                    .find(|c| self.matches_choice_type(&suffix, c))
                {
                    metadata.type_info = Some(TypeInfo {
                        type_name: choice.type_name.clone(),
                        singleton: Some(true),
                        is_empty: Some(false),
                        namespace: Some("FHIR".to_string()),
                        name: Some(choice.type_name.clone()),
                    });
                }
            }
        } else {
            // Non-polymorphic access to choice property - should suggest alternatives
            let diagnostic = self.create_choice_access_suggestion(property_name, choices, location);
            metadata.add_diagnostic(diagnostic);
        }

        Ok(metadata)
    }

    /// Check if a property follows the polymorphic naming pattern (e.g., valueString, valueInteger)
    fn is_polymorphic_property(&self, property_name: &str) -> bool {
        // Common FHIR choice property patterns
        let choice_prefixes = [
            "value",
            "onset",
            "deceased",
            "multipleBirth",
            "fixed",
            "pattern",
        ];

        choice_prefixes
            .iter()
            .any(|prefix| property_name.starts_with(prefix) && property_name.len() > prefix.len())
    }

    /// Parse polymorphic property into base name and type suffix
    fn parse_polymorphic_property(&self, property_name: &str) -> (String, String) {
        let choice_prefixes = [
            "value",
            "onset",
            "deceased",
            "multipleBirth",
            "fixed",
            "pattern",
        ];

        for prefix in &choice_prefixes {
            if property_name.starts_with(prefix) && property_name.len() > prefix.len() {
                let suffix = &property_name[prefix.len()..];
                return (prefix.to_string(), suffix.to_string());
            }
        }

        // Fallback - shouldn't happen if is_polymorphic_property returned true
        (property_name.to_string(), String::new())
    }

    /// Check if a type suffix matches a choice type
    fn matches_choice_type(&self, suffix: &str, choice: &ChoiceTypeInfo) -> bool {
        // Direct match with choice suffix
        if choice.suffix == suffix {
            return true;
        }

        // Handle common FHIR type name variations
        let normalized_suffix = self.normalize_type_name(suffix);
        let normalized_choice_suffix = self.normalize_type_name(&choice.suffix);

        normalized_suffix == normalized_choice_suffix
    }

    /// Normalize type names for comparison (handle casing variations)
    fn normalize_type_name(&self, type_name: &str) -> String {
        // Handle common FHIR type variations
        match type_name.to_lowercase().as_str() {
            "boolean" => "boolean".to_string(),
            "integer" => "integer".to_string(),
            "string" => "string".to_string(),
            "decimal" => "decimal".to_string(),
            "datetime" => "dateTime".to_string(),
            "date" => "date".to_string(),
            "time" => "time".to_string(),
            "instant" => "instant".to_string(),
            "uri" => "uri".to_string(),
            "url" => "url".to_string(),
            "canonical" => "canonical".to_string(),
            "oid" => "oid".to_string(),
            "uuid" => "uuid".to_string(),
            "base64binary" => "base64Binary".to_string(),
            "codeableconcept" => "CodeableConcept".to_string(),
            "coding" => "Coding".to_string(),
            "quantity" => "Quantity".to_string(),
            "range" => "Range".to_string(),
            "period" => "Period".to_string(),
            "reference" => "Reference".to_string(),
            "attachment" => "Attachment".to_string(),
            "identifier" => "Identifier".to_string(),
            "humanname" => "HumanName".to_string(),
            "address" => "Address".to_string(),
            "contactpoint" => "ContactPoint".to_string(),
            _ => type_name.to_string(),
        }
    }

    /// Create diagnostic for invalid choice type usage
    fn create_invalid_choice_diagnostic(
        &self,
        property_name: &str,
        base_name: &str,
        suffix: &str,
        choices: &[ChoiceTypeInfo],
        location: Range<usize>,
    ) -> Diagnostic {
        let valid_properties: Vec<String> = choices
            .iter()
            .map(|choice| format!("{}{}", base_name, choice.suffix))
            .collect();

        let suggestion = if valid_properties.len() == 1 {
            format!("Did you mean '{}'?", valid_properties[0])
        } else {
            format!("Valid options: {}", valid_properties.join(", "))
        };

        Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: DiagnosticCode {
                code: "FP0501".to_string(),
                namespace: None,
            },
            message: format!(
                "Invalid choice type '{suffix}' for property '{property_name}'\n  = help: {suggestion}\n  = note: '{suffix}' is not a valid type suffix for this choice property"
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        }
    }

    /// Create diagnostic suggesting proper choice property access
    fn create_choice_access_suggestion(
        &self,
        property_name: &str,
        choices: &[ChoiceTypeInfo],
        location: Range<usize>,
    ) -> Diagnostic {
        let choice_properties: Vec<String> = choices
            .iter()
            .map(|choice| format!("{}{}", property_name, choice.suffix))
            .collect();

        let suggestion = if choice_properties.len() == 1 {
            format!("Use '{}' to access the typed value", choice_properties[0])
        } else if choice_properties.len() <= 5 {
            format!("Use one of: {}", choice_properties.join(", "))
        } else {
            format!(
                "Use a typed variant like '{}' (and {} others)",
                choice_properties[0],
                choice_properties.len() - 1
            )
        };

        Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: DiagnosticCode {
                code: "FP0502".to_string(),
                namespace: None,
            },
            message: format!(
                "Accessing choice property '{property_name}' without type specification\n  = help: {suggestion}\n  = note: Choice properties require type-specific access"
            ),
            location: Some(SourceLocation::new(
                1,
                location.start + 1,
                location.start,
                location.len(),
            )),
            related: vec![],
        }
    }

    /// Get cache statistics for performance monitoring
    pub fn get_cache_stats(&self) -> CacheStatistics {
        self.cache.get_stats()
    }

    /// Clear the cache (useful for testing or memory management)
    pub fn clear_cache(&mut self) {
        self.cache = ChoiceTypeCache::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;
    use std::sync::Arc;

    fn create_test_analyzer() -> ChoiceTypeAnalyzer {
        let provider = Arc::new(EmptyModelProvider);
        ChoiceTypeAnalyzer::new(provider)
    }

    fn create_test_choice_types() -> Vec<ChoiceTypeInfo> {
        vec![
            ChoiceTypeInfo {
                suffix: "String".to_string(),
                type_name: "string".to_string(),
            },
            ChoiceTypeInfo {
                suffix: "Integer".to_string(),
                type_name: "integer".to_string(),
            },
            ChoiceTypeInfo {
                suffix: "Boolean".to_string(),
                type_name: "boolean".to_string(),
            },
            ChoiceTypeInfo {
                suffix: "CodeableConcept".to_string(),
                type_name: "CodeableConcept".to_string(),
            },
        ]
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = create_test_analyzer();
        assert_eq!(
            std::mem::size_of_val(&analyzer),
            std::mem::size_of::<ChoiceTypeAnalyzer>()
        );
    }

    #[test]
    fn test_is_polymorphic_property() {
        let analyzer = create_test_analyzer();

        assert!(analyzer.is_polymorphic_property("valueString"));
        assert!(analyzer.is_polymorphic_property("valueInteger"));
        assert!(analyzer.is_polymorphic_property("onsetDateTime"));
        assert!(analyzer.is_polymorphic_property("deceasedBoolean"));

        assert!(!analyzer.is_polymorphic_property("value"));
        assert!(!analyzer.is_polymorphic_property("name"));
        assert!(!analyzer.is_polymorphic_property("id"));
    }

    #[test]
    fn test_parse_polymorphic_property() {
        let analyzer = create_test_analyzer();

        let (base, suffix) = analyzer.parse_polymorphic_property("valueString");
        assert_eq!(base, "value");
        assert_eq!(suffix, "String");

        let (base, suffix) = analyzer.parse_polymorphic_property("onsetDateTime");
        assert_eq!(base, "onset");
        assert_eq!(suffix, "DateTime");
    }

    #[test]
    fn test_matches_choice_type() {
        let analyzer = create_test_analyzer();
        let choice_types = create_test_choice_types();

        assert!(analyzer.matches_choice_type("String", &choice_types[0]));
        assert!(analyzer.matches_choice_type("Integer", &choice_types[1]));
        assert!(analyzer.matches_choice_type("Boolean", &choice_types[2]));
        assert!(analyzer.matches_choice_type("CodeableConcept", &choice_types[3]));

        assert!(!analyzer.matches_choice_type("DateTime", &choice_types[0]));
        assert!(!analyzer.matches_choice_type("InvalidType", &choice_types[0]));
    }

    #[test]
    fn test_normalize_type_name() {
        let analyzer = create_test_analyzer();

        assert_eq!(analyzer.normalize_type_name("boolean"), "boolean");
        assert_eq!(analyzer.normalize_type_name("Boolean"), "boolean");
        assert_eq!(analyzer.normalize_type_name("datetime"), "dateTime");
        assert_eq!(analyzer.normalize_type_name("DateTime"), "dateTime");
        assert_eq!(analyzer.normalize_type_name("base64binary"), "base64Binary");
        assert_eq!(
            analyzer.normalize_type_name("codeableconcept"),
            "CodeableConcept"
        );
    }

    #[tokio::test]
    async fn test_choice_validation_valid() {
        let analyzer = create_test_analyzer();
        let choices = create_test_choice_types();
        let location = 0..11; // "valueString"

        let result = analyzer.validate_choice_usage("valueString", &choices, location);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // Should not have errors for valid choice
        let errors: Vec<_> = metadata
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(errors.is_empty());

        // Should have correct type info
        assert!(metadata.type_info.is_some());
        let type_info = metadata.type_info.unwrap();
        assert_eq!(type_info.type_name, "string");
    }

    #[tokio::test]
    async fn test_choice_validation_invalid() {
        let analyzer = create_test_analyzer();
        let choices = create_test_choice_types();
        let location = 0..13; // "valueDateTime"

        let result = analyzer.validate_choice_usage("valueDateTime", &choices, location);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        // Should have error for invalid choice
        let errors: Vec<_> = metadata
            .diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(!errors.is_empty());
        assert!(errors[0].code.code == "FP0501");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let mut analyzer = create_test_analyzer();

        // Initial stats should be zero
        let stats = analyzer.get_cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Access should result in cache miss (but won't actually call ModelProvider in EmptyModelProvider)
        let result = analyzer
            .analyze_choice_property("Observation", "value", 0..5)
            .await;
        assert!(result.is_ok());

        // Test cache clearing
        analyzer.clear_cache();
        let stats = analyzer.get_cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }
}
