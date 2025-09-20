//! Diagnostic template registry for standardized error messages
//!
//! This module provides a comprehensive template system for diagnostic messages,
//! enabling consistent and helpful error reporting across all analyzers.

use std::collections::HashMap;

use crate::diagnostics::DiagnosticSeverity;

/// Template for diagnostic messages with placeholders for context-specific information
#[derive(Debug, Clone)]
pub struct DiagnosticTemplate {
    /// Error code (e.g., "FP0301")
    pub code: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Main message template with placeholders
    pub message_template: String,
    /// Optional help text template
    pub help_template: Option<String>,
    /// Optional note template for additional context
    pub note_template: Option<String>,
}

impl DiagnosticTemplate {
    /// Create a new diagnostic template
    pub fn new(code: String, severity: DiagnosticSeverity, message_template: String) -> Self {
        Self {
            code,
            severity,
            message_template,
            help_template: None,
            note_template: None,
        }
    }

    /// Add help text template
    pub fn with_help(mut self, help: String) -> Self {
        self.help_template = Some(help);
        self
    }

    /// Add note template
    pub fn with_note(mut self, note: String) -> Self {
        self.note_template = Some(note);
        self
    }
}

/// Registry for diagnostic message templates
#[derive(Debug)]
pub struct DiagnosticTemplateRegistry {
    templates: HashMap<String, DiagnosticTemplate>,
}

impl DiagnosticTemplateRegistry {
    /// Create a new registry with predefined templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Property validation templates (FP0201-FP0220)
        templates.insert(
            "PROPERTY_NOT_FOUND".to_string(),
            DiagnosticTemplate::new(
                "PROPERTY_NOT_FOUND".to_string(),
                DiagnosticSeverity::Error,
                "Property '{property_name}' not found on type '{type_name}'".to_string(),
            )
            .with_help("Available properties: {available_properties}".to_string())
            .with_note("Did you mean one of: {suggestions}?".to_string()),
        );

        templates.insert(
            "FP0201".to_string(),
            DiagnosticTemplate::new(
                "FP0201".to_string(),
                DiagnosticSeverity::Error,
                "Unknown ResourceType '{resource_type}'".to_string(),
            )
            .with_help("Available resource types: {available_types}".to_string())
            .with_note("Did you mean '{suggestion}'?".to_string()),
        );

        templates.insert("FP0202".to_string(), DiagnosticTemplate::new(
            "FP0202".to_string(),
            DiagnosticSeverity::Warning,
            "Ambiguous choice property '{property_name}' - specify type with '{property_name}[x]'".to_string(),
        ).with_help("Available choice types: {choice_types}".to_string())
         .with_note("Use '{specific_property}' to access specific type".to_string()));

        templates.insert(
            "FP0203".to_string(),
            DiagnosticTemplate::new(
                "FP0203".to_string(),
                DiagnosticSeverity::Error,
                "Invalid choice property '{choice_property}' for type '{type_name}'".to_string(),
            )
            .with_help("Valid choice properties for this type: {valid_choices}".to_string()),
        );

        // Union type validation templates (FP0301-FP0320)
        templates.insert(
            "FP0301".to_string(),
            DiagnosticTemplate::new(
                "FP0301".to_string(),
                DiagnosticSeverity::Warning,
                "ofType({target_type}) will always be empty".to_string(),
            )
            .with_help("Available types in this context: {available_types}".to_string())
            .with_note("Input type '{input_type}' cannot contain '{target_type}'".to_string()),
        );

        templates.insert(
            "FP0302".to_string(),
            DiagnosticTemplate::new(
                "FP0302".to_string(),
                DiagnosticSeverity::Error,
                "Function '{function_name}' requires collection input".to_string(),
            )
            .with_help(
                "'{function_name}' can only be applied to collections, not single values"
                    .to_string(),
            ),
        );

        templates.insert(
            "FP0303".to_string(),
            DiagnosticTemplate::new(
                "FP0303".to_string(),
                DiagnosticSeverity::Error,
                "Invalid type '{target_type}' for {operation} operation".to_string(),
            )
            .with_help("Valid types for this operation: {valid_types}".to_string()),
        );

        templates.insert(
            "FP0304".to_string(),
            DiagnosticTemplate::new(
                "FP0304".to_string(),
                DiagnosticSeverity::Error,
                "Unknown function '{function_name}'".to_string(),
            )
            .with_help("Available functions: {available_functions}".to_string())
            .with_note("Did you mean '{suggestion}'?".to_string()),
        );

        templates.insert("FP0305".to_string(), DiagnosticTemplate::new(
            "FP0305".to_string(),
            DiagnosticSeverity::Error,
            "Function '{function_name}' requires at least {required_args} arguments, got {actual_args}".to_string(),
        ).with_help("Function signature: {signature}".to_string()));

        templates.insert("FP0306".to_string(), DiagnosticTemplate::new(
            "FP0306".to_string(),
            DiagnosticSeverity::Error,
            "Function '{function_name}' accepts at most {max_args} arguments, got {actual_args}".to_string(),
        ).with_help("Function signature: {signature}".to_string()));

        templates.insert("FP0307".to_string(), DiagnosticTemplate::new(
            "FP0307".to_string(),
            DiagnosticSeverity::Error,
            "Function '{function_name}' argument {arg_index} expects {expected_type}, got {actual_type}".to_string(),
        ).with_help("Function signature: {signature}".to_string()));

        templates.insert(
            "FP0308".to_string(),
            DiagnosticTemplate::new(
                "FP0308".to_string(),
                DiagnosticSeverity::Error,
                "Function '{function_name}' requires singleton input".to_string(),
            )
            .with_help(
                "'{function_name}' can only be applied to single values, not collections"
                    .to_string(),
            ),
        );

        templates.insert(
            "FP0309".to_string(),
            DiagnosticTemplate::new(
                "FP0309".to_string(),
                DiagnosticSeverity::Warning,
                "Function '{function_name}' on empty collection will always return empty"
                    .to_string(),
            )
            .with_help("Consider checking if collection is non-empty first".to_string()),
        );

        templates.insert(
            "FP0310".to_string(),
            DiagnosticTemplate::new(
                "FP0310".to_string(),
                DiagnosticSeverity::Error,
                "Context analysis error: {error_message}".to_string(),
            )
            .with_help("Check expression context and variable scoping".to_string()),
        );

        templates.insert(
            "FP0311".to_string(),
            DiagnosticTemplate::new(
                "FP0311".to_string(),
                DiagnosticSeverity::Warning,
                "Function '{function_name}' on singleton value is redundant".to_string(),
            )
            .with_help("Consider removing redundant function call".to_string()),
        );

        templates.insert(
            "FP0312".to_string(),
            DiagnosticTemplate::new(
                "FP0312".to_string(),
                DiagnosticSeverity::Error,
                "Operation '{operation}' requires collection input".to_string(),
            )
            .with_help("'{operation}' can only be applied to collections".to_string()),
        );

        templates.insert(
            "FP0313".to_string(),
            DiagnosticTemplate::new(
                "FP0313".to_string(),
                DiagnosticSeverity::Error,
                "Operation '{operation}' requires singleton input".to_string(),
            )
            .with_help("'{operation}' can only be applied to single values".to_string()),
        );

        // Cardinality validation templates (FP0314-FP0320)
        templates.insert(
            "FP0314".to_string(),
            DiagnosticTemplate::new(
                "FP0314".to_string(),
                DiagnosticSeverity::Error,
                "Expected singleton value, got collection".to_string(),
            )
            .with_help(
                "Use '.first()' or similar to get a single value from collection".to_string(),
            ),
        );

        templates.insert(
            "FP0315".to_string(),
            DiagnosticTemplate::new(
                "FP0315".to_string(),
                DiagnosticSeverity::Warning,
                "Expected collection, got singleton value".to_string(),
            )
            .with_help(
                "Singleton values are automatically treated as single-item collections".to_string(),
            ),
        );

        templates.insert(
            "FP0316".to_string(),
            DiagnosticTemplate::new(
                "FP0316".to_string(),
                DiagnosticSeverity::Warning,
                "Expected non-empty collection, got singleton value".to_string(),
            )
            .with_help("Singleton values count as non-empty collections".to_string()),
        );

        templates.insert(
            "FP0317".to_string(),
            DiagnosticTemplate::new(
                "FP0317".to_string(),
                DiagnosticSeverity::Warning,
                "Expected non-empty collection, but collection may be empty".to_string(),
            )
            .with_help("Consider checking if collection is non-empty before use".to_string()),
        );

        // Type analysis templates (FP0318-FP0320)
        templates.insert(
            "FP0318".to_string(),
            DiagnosticTemplate::new(
                "FP0318".to_string(),
                DiagnosticSeverity::Error,
                "Type '{type_name}' cannot be used in this context".to_string(),
            )
            .with_help("Expected context type: {expected_context}".to_string()),
        );

        templates.insert(
            "FP0319".to_string(),
            DiagnosticTemplate::new(
                "FP0319".to_string(),
                DiagnosticSeverity::Warning,
                "Type cast from '{from_type}' to '{to_type}' may fail at runtime".to_string(),
            )
            .with_help("Consider using type checking before casting".to_string()),
        );

        templates.insert(
            "FP0320".to_string(),
            DiagnosticTemplate::new(
                "FP0320".to_string(),
                DiagnosticSeverity::Error,
                "Invalid type inheritance: '{child_type}' is not a subtype of '{parent_type}'"
                    .to_string(),
            )
            .with_help("Valid parent types for '{child_type}': {valid_parents}".to_string()),
        );

        // Performance and optimization templates (FP0401-FP0410)
        templates.insert(
            "FP0401".to_string(),
            DiagnosticTemplate::new(
                "FP0401".to_string(),
                DiagnosticSeverity::Warning,
                "Complex expression may impact performance".to_string(),
            )
            .with_help("Consider simplifying or caching the result".to_string()),
        );

        templates.insert(
            "FP0402".to_string(),
            DiagnosticTemplate::new(
                "FP0402".to_string(),
                DiagnosticSeverity::Warning,
                "Redundant type check: expression already guarantees type '{type_name}'"
                    .to_string(),
            )
            .with_help("Consider removing redundant type check".to_string()),
        );

        templates.insert(
            "FP0403".to_string(),
            DiagnosticTemplate::new(
                "FP0403".to_string(),
                DiagnosticSeverity::Info,
                "Expression can be simplified".to_string(),
            )
            .with_help("Suggested simplification: {simplification}".to_string()),
        );

        Self { templates }
    }

    /// Get a template by key
    pub fn get_template(&self, key: &str) -> Option<&DiagnosticTemplate> {
        self.templates.get(key)
    }

    /// Register a new template
    pub fn register_template(&mut self, key: String, template: DiagnosticTemplate) {
        self.templates.insert(key, template);
    }

    /// Get all available template keys
    pub fn template_keys(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// Check if a template exists
    pub fn has_template(&self, key: &str) -> bool {
        self.templates.contains_key(key)
    }

    /// Get template count
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }
}

impl Default for DiagnosticTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = DiagnosticTemplateRegistry::new();
        assert!(registry.template_count() > 0);
        assert!(registry.has_template("FP0301"));
        assert!(registry.has_template("PROPERTY_NOT_FOUND"));
    }

    #[test]
    fn test_template_access() {
        let registry = DiagnosticTemplateRegistry::new();

        let template = registry.get_template("FP0301").unwrap();
        assert_eq!(template.code, "FP0301");
        assert_eq!(template.severity, DiagnosticSeverity::Warning);
        assert!(template.message_template.contains("ofType"));
        assert!(template.help_template.is_some());
        assert!(template.note_template.is_some());
    }

    #[test]
    fn test_template_registration() {
        let mut registry = DiagnosticTemplateRegistry::new();
        let initial_count = registry.template_count();

        let custom_template = DiagnosticTemplate::new(
            "CUSTOM_001".to_string(),
            DiagnosticSeverity::Info,
            "Custom diagnostic message".to_string(),
        );

        registry.register_template("CUSTOM_001".to_string(), custom_template);
        assert_eq!(registry.template_count(), initial_count + 1);
        assert!(registry.has_template("CUSTOM_001"));
    }

    #[test]
    fn test_template_keys() {
        let registry = DiagnosticTemplateRegistry::new();
        let keys = registry.template_keys();

        assert!(keys.contains(&&"FP0301".to_string()));
        assert!(keys.contains(&&"FP0302".to_string()));
        assert!(keys.contains(&&"PROPERTY_NOT_FOUND".to_string()));
    }

    #[test]
    fn test_property_validation_templates() {
        let registry = DiagnosticTemplateRegistry::new();

        // Test property not found template
        let template = registry.get_template("PROPERTY_NOT_FOUND").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("Property"));
        assert!(template.help_template.is_some());
        assert!(template.note_template.is_some());

        // Test resource type template
        let template = registry.get_template("FP0201").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("ResourceType"));
    }

    #[test]
    fn test_union_type_templates() {
        let registry = DiagnosticTemplateRegistry::new();

        // Test ofType warning template
        let template = registry.get_template("FP0301").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Warning);
        assert!(template.message_template.contains("ofType"));

        // Test function requires collection template
        let template = registry.get_template("FP0302").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("collection input"));
    }

    #[test]
    fn test_function_validation_templates() {
        let registry = DiagnosticTemplateRegistry::new();

        // Test unknown function template
        let template = registry.get_template("FP0304").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("Unknown function"));

        // Test argument count templates
        let template = registry.get_template("FP0305").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("requires at least"));

        let template = registry.get_template("FP0306").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("accepts at most"));
    }

    #[test]
    fn test_cardinality_templates() {
        let registry = DiagnosticTemplateRegistry::new();

        // Test singleton/collection mismatch templates
        let template = registry.get_template("FP0314").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("Expected singleton"));

        let template = registry.get_template("FP0315").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Warning);
        assert!(template.message_template.contains("Expected collection"));
    }

    #[test]
    fn test_performance_templates() {
        let registry = DiagnosticTemplateRegistry::new();

        // Test performance warning template
        let template = registry.get_template("FP0401").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Warning);
        assert!(template.message_template.contains("performance"));

        // Test simplification suggestion template
        let template = registry.get_template("FP0403").unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Info);
        assert!(template.message_template.contains("simplified"));
    }

    #[test]
    fn test_default_implementation() {
        let registry = DiagnosticTemplateRegistry::default();
        assert!(registry.template_count() > 0);
    }
}
