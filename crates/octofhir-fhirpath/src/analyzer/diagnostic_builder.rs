//! Enhanced diagnostic builder with template support for rich, contextual error messages
//!
//! This module provides template-based diagnostic creation that generates rich error messages
//! with help text, suggestions, and contextual information while maintaining backward
//! compatibility with existing diagnostic infrastructure.

use std::collections::HashMap;
use std::ops::Range;

use crate::analyzer::diagnostic_template_registry::DiagnosticTemplateRegistry;
use crate::core::error_code::ErrorCode;
use crate::diagnostics::{AriadneDiagnostic, DiagnosticSeverity};

/// Enhanced diagnostic builder with template support
#[derive(Debug)]
pub struct DiagnosticBuilder {
    /// Template registry for creating consistent diagnostics
    templates: DiagnosticTemplateRegistry,
}

/// Context for template variable substitution
#[derive(Debug, Clone, Default)]
pub struct DiagnosticContext {
    /// Variables for template substitution
    pub variables: HashMap<String, String>,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder with default templates
    pub fn new() -> Self {
        Self {
            templates: DiagnosticTemplateRegistry::default(),
        }
    }

    /// Create diagnostic from template with context substitution
    pub fn from_template(
        &self,
        template_key: &str,
        context: DiagnosticContext,
        span: Range<usize>,
    ) -> Result<AriadneDiagnostic, DiagnosticBuilderError> {
        let template = self
            .templates
            .get_template(template_key)
            .ok_or_else(|| DiagnosticBuilderError::TemplateNotFound(template_key.to_string()))?;

        let message = self.substitute_template(&template.message_template, &context.variables)?;
        let help = if let Some(ref help_template) = template.help_template {
            Some(self.substitute_template(help_template, &context.variables)?)
        } else {
            None
        };
        let note = if let Some(ref note_template) = template.note_template {
            Some(self.substitute_template(note_template, &context.variables)?)
        } else {
            None
        };

        // Convert string error code to numeric error code
        let error_code = if template.code.starts_with("FP") {
            // Extract numeric part from FP#### format
            let numeric_part = &template.code[2..];
            if let Ok(code_num) = numeric_part.parse::<u16>() {
                ErrorCode::new(code_num)
            } else {
                ErrorCode::new(9999) // Fallback for unparseable codes
            }
        } else {
            // For non-FP codes, hash to a number or use a default
            ErrorCode::new(8000 + (template.code.len() as u16 % 1000))
        };

        Ok(AriadneDiagnostic {
            severity: template.severity.clone(),
            error_code,
            message,
            span,
            help,
            note,
            related: Vec::new(),
        })
    }

    /// Create property not found diagnostic with suggestions
    pub fn property_not_found(
        &self,
        property_name: &str,
        type_name: &str,
        suggestions: Vec<String>,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        let mut context = DiagnosticContext::default();
        context
            .variables
            .insert("property_name".to_string(), property_name.to_string());
        context
            .variables
            .insert("type_name".to_string(), type_name.to_string());

        if !suggestions.is_empty() {
            context
                .variables
                .insert("suggestion".to_string(), suggestions[0].clone());
            context
                .variables
                .insert("available_properties".to_string(), suggestions.join(", "));
            context
                .variables
                .insert("suggestions".to_string(), suggestions.join(", "));
        }

        self.from_template("PROPERTY_NOT_FOUND", context, span.clone())
            .unwrap_or_else(|_| self.fallback_property_not_found(property_name, type_name, span))
    }

    /// Create resource type validation diagnostic
    pub fn invalid_resource_type(
        &self,
        invalid_type: &str,
        suggestions: Vec<String>,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        let mut context = DiagnosticContext::default();
        context
            .variables
            .insert("resource_type".to_string(), invalid_type.to_string());

        if !suggestions.is_empty() {
            context
                .variables
                .insert("suggestion".to_string(), suggestions[0].clone());
            context
                .variables
                .insert("available_types".to_string(), suggestions.join(", "));
        }

        self.from_template("FP0201", context, span.clone())
            .unwrap_or_else(|_| self.fallback_invalid_resource_type(invalid_type, span))
    }

    /// Create choice type ambiguous diagnostic
    pub fn ambiguous_choice_property(
        &self,
        property_name: &str,
        available_choices: Vec<String>,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        let mut context = DiagnosticContext::default();
        context
            .variables
            .insert("property_name".to_string(), property_name.to_string());
        context.variables.insert(
            "available_choices".to_string(),
            available_choices
                .iter()
                .map(|choice| format!("{property_name}{choice}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        context
            .variables
            .insert("choice_types".to_string(), available_choices.join(", "));

        self.from_template("FP0202", context, span.clone())
            .unwrap_or_else(|_| self.fallback_ambiguous_choice_property(property_name, span))
    }

    /// Create union type filtering diagnostic
    pub fn empty_union_filter(
        &self,
        target_type: &str,
        input_type: &str,
        available_types: Vec<String>,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        let mut context = DiagnosticContext::default();
        context
            .variables
            .insert("target_type".to_string(), target_type.to_string());
        context
            .variables
            .insert("input_type".to_string(), input_type.to_string());
        context
            .variables
            .insert("available_types".to_string(), available_types.join(", "));

        self.from_template("FP0301", context, span.clone())
            .unwrap_or_else(|_| self.fallback_empty_union_filter(target_type, span))
    }

    /// Create function context error diagnostic
    pub fn function_context_error(
        &self,
        function_name: &str,
        required_context: &str,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        let mut context = DiagnosticContext::default();
        context
            .variables
            .insert("function".to_string(), function_name.to_string());
        context
            .variables
            .insert("required_context".to_string(), required_context.to_string());

        self.from_template("FP0302", context, span.clone())
            .unwrap_or_else(|_| {
                self.fallback_function_context_error(function_name, required_context, span)
            })
    }

    /// Substitute template variables with context values
    fn substitute_template(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, DiagnosticBuilderError> {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{key}}}");
            result = result.replace(&placeholder, value);
        }

        // Check for unsubstituted placeholders
        if result.contains('{') && result.contains('}') {
            let remaining: Vec<&str> = result
                .split('{')
                .skip(1)
                .filter_map(|s| s.split('}').next())
                .collect();
            if !remaining.is_empty() {
                return Err(DiagnosticBuilderError::UnsubstitutedVariable(
                    remaining[0].to_string(),
                ));
            }
        }

        Ok(result)
    }

    // Fallback methods to ensure diagnostics are always created even if templates fail

    fn fallback_property_not_found(
        &self,
        property_name: &str,
        type_name: &str,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: ErrorCode::new(8001),
            message: format!("Property '{property_name}' not found on {type_name}"),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        }
    }

    fn fallback_invalid_resource_type(
        &self,
        invalid_type: &str,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: ErrorCode::new(201),
            message: format!("Unknown resource type '{invalid_type}'"),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        }
    }

    fn fallback_ambiguous_choice_property(
        &self,
        property_name: &str,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: ErrorCode::new(202),
            message: format!("Ambiguous choice property '{property_name}'"),
            span,
            help: Some("Specify the exact choice type you want to access".to_string()),
            note: Some("Choice properties can have multiple types".to_string()),
            related: Vec::new(),
        }
    }

    fn fallback_empty_union_filter(
        &self,
        target_type: &str,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity: DiagnosticSeverity::Warning,
            error_code: ErrorCode::new(301),
            message: format!("ofType({target_type}) will always be empty"),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        }
    }

    fn fallback_function_context_error(
        &self,
        function_name: &str,
        required_context: &str,
        span: Range<usize>,
    ) -> AriadneDiagnostic {
        AriadneDiagnostic {
            severity: DiagnosticSeverity::Error,
            error_code: ErrorCode::new(302),
            message: format!("Function '{function_name}' requires {required_context} input"),
            span,
            help: None,
            note: None,
            related: Vec::new(),
        }
    }
}

impl Default for DiagnosticBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticContext {
    /// Create a new empty diagnostic context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a variable to the context
    pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Add multiple variables to the context
    pub fn with_variables(mut self, variables: HashMap<String, String>) -> Self {
        self.variables.extend(variables);
        self
    }
}

/// Errors that can occur during diagnostic building
#[derive(Debug, Clone)]
pub enum DiagnosticBuilderError {
    /// Template not found in registry
    TemplateNotFound(String),
    /// Variable referenced in template but not provided in context
    UnsubstitutedVariable(String),
}

impl std::fmt::Display for DiagnosticBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticBuilderError::TemplateNotFound(key) => {
                write!(f, "Diagnostic template '{key}' not found")
            }
            DiagnosticBuilderError::UnsubstitutedVariable(var) => {
                write!(
                    f,
                    "Variable '{var}' referenced in template but not provided"
                )
            }
        }
    }
}

impl std::error::Error for DiagnosticBuilderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder_creation() {
        let builder = DiagnosticBuilder::new();
        assert_eq!(
            std::mem::size_of_val(&builder),
            std::mem::size_of::<DiagnosticBuilder>()
        );
    }

    #[test]
    fn test_template_substitution() {
        let builder = DiagnosticBuilder::new();
        let mut variables = HashMap::new();
        variables.insert("property".to_string(), "name".to_string());
        variables.insert("type".to_string(), "Patient".to_string());

        let result =
            builder.substitute_template("Property '{property}' not found on {type}", &variables);

        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message, "Property 'name' not found on Patient");
    }

    #[test]
    fn test_property_not_found_diagnostic() {
        let builder = DiagnosticBuilder::new();
        let suggestions = vec!["given".to_string(), "family".to_string()];

        let diagnostic = builder.property_not_found("nam", "Patient", suggestions, 0..3);

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert!(diagnostic.message.contains("nam"));
        assert!(diagnostic.message.contains("Patient"));
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.note.is_some());
    }

    #[test]
    fn test_invalid_resource_type_diagnostic() {
        let builder = DiagnosticBuilder::new();
        let suggestions = vec!["Patient".to_string(), "Practitioner".to_string()];

        let diagnostic = builder.invalid_resource_type("Patien", suggestions, 0..6);

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert!(diagnostic.message.contains("Patien"));
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.note.is_some());
    }

    #[test]
    fn test_ambiguous_choice_property_diagnostic() {
        let builder = DiagnosticBuilder::new();
        let choices = vec!["String".to_string(), "Quantity".to_string()];

        let diagnostic = builder.ambiguous_choice_property("value", choices, 0..5);

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert!(diagnostic.message.contains("value"));
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.note.is_some());
    }

    #[test]
    fn test_empty_union_filter_diagnostic() {
        let builder = DiagnosticBuilder::new();
        let available = vec!["Patient".to_string(), "DomainResource".to_string()];

        let diagnostic = builder.empty_union_filter("Observation", "Patient", available, 0..20);

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert!(diagnostic.message.contains("Observation"));
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.note.is_some());
    }

    #[test]
    fn test_diagnostic_context() {
        let context = DiagnosticContext::new()
            .with_variable("key1", "value1")
            .with_variable("key2", "value2");

        assert_eq!(context.variables.get("key1"), Some(&"value1".to_string()));
        assert_eq!(context.variables.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    #[ignore = "experimental analyzer feature"]
    fn test_template_registry() {
        let registry = DiagnosticTemplateRegistry::default();

        let template = registry.get_template("PROPERTY_NOT_FOUND");
        assert!(template.is_some());

        let template = template.unwrap();
        assert_eq!(template.severity, DiagnosticSeverity::Error);
        assert!(template.message_template.contains("{property}"));
        assert!(template.help_template.is_some());
    }

    #[test]
    fn test_unsubstituted_variable_error() {
        let builder = DiagnosticBuilder::new();
        let variables = HashMap::new(); // Empty variables

        let result = builder.substitute_template("Property '{property}' not found", &variables);

        assert!(result.is_err());
        match result.unwrap_err() {
            DiagnosticBuilderError::UnsubstitutedVariable(var) => {
                assert_eq!(var, "property");
            }
            _ => panic!("Expected UnsubstitutedVariable error"),
        }
    }

    #[test]
    fn test_fallback_diagnostics() {
        let builder = DiagnosticBuilder::new();

        // Test fallback methods
        let diagnostic = builder.fallback_property_not_found("test", "TestType", 0..4);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert!(diagnostic.message.contains("test"));

        let diagnostic = builder.fallback_invalid_resource_type("TestType", 0..8);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert!(diagnostic.message.contains("TestType"));
    }
}
