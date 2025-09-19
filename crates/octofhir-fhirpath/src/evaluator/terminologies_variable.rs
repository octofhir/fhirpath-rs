//! %terminologies system variable implementation
//!
//! This module implements the %terminologies system variable that provides access to
//! terminology operations in FHIRPath 3.0.0-ballot specification.
//!
//! Usage examples:
//! - %terminologies.expand('http://example.org/ValueSet/example')
//! - %terminologies.lookup('http://loinc.org', '29463-7')
//! - %terminologies.validateVS('http://example.org/ValueSet/example', 'code', 'system')

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationContext;
use octofhir_fhir_model::TerminologyProvider;

/// Represents the %terminologies system variable
///
/// This is a special resource-like object that provides access to terminology functions
/// through method-like syntax in FHIRPath expressions.
#[derive(Debug, Clone)]
pub struct TerminologiesVariable {
    /// Reference to the terminology provider
    terminology_provider: Arc<dyn TerminologyProvider>,
}

impl TerminologiesVariable {
    /// Create a new terminologies variable with the given terminology provider
    pub fn new(terminology_provider: Arc<dyn TerminologyProvider>) -> Self {
        Self {
            terminology_provider,
        }
    }

    /// Convert to FhirPathValue for use in expressions
    ///
    /// The %terminologies variable appears as a special Resource-like object
    /// that exposes terminology operations as "properties" that can be called.
    pub fn to_fhir_path_value(&self) -> FhirPathValue {
        // Create a pseudo-resource representation of the terminologies variable
        let mut terminologies_object = serde_json::Map::new();

        // Add resource type to identify this as the terminologies variable
        terminologies_object.insert(
            "resourceType".to_string(),
            serde_json::Value::String("TerminologiesVariable".to_string()),
        );

        // Add metadata to indicate this supports terminology operations
        terminologies_object.insert(
            "supportedOperations".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("expand".to_string()),
                serde_json::Value::String("lookup".to_string()),
                serde_json::Value::String("validateVS".to_string()),
                serde_json::Value::String("validateCS".to_string()),
                serde_json::Value::String("subsumes".to_string()),
                serde_json::Value::String("translate".to_string()),
            ]),
        );

        // Store a reference to self in the resource for later access
        // Note: This is a conceptual representation - the actual function invocation
        // will be handled by the function evaluators when they detect calls on %terminologies
        terminologies_object.insert(
            "_terminologyProvider".to_string(),
            serde_json::Value::String("internal".to_string()),
        );

        FhirPathValue::resource(serde_json::Value::Object(terminologies_object))
    }

    /// Get the underlying terminology provider
    pub fn terminology_provider(&self) -> &Arc<dyn TerminologyProvider> {
        &self.terminology_provider
    }
}

/// Helper function to check if a FhirPathValue represents the %terminologies variable
pub fn is_terminologies_variable(value: &FhirPathValue) -> bool {
    match value {
        FhirPathValue::Resource(resource, _, _) => resource
            .get("resourceType")
            .and_then(|rt| rt.as_str())
            .map(|rt| rt == "TerminologiesVariable")
            .unwrap_or(false),
        _ => false,
    }
}

/// Extract terminology provider from a %terminologies variable value
///
/// This is used by terminology function evaluators to detect when they are being
/// called on the %terminologies variable and extract the provider.
pub fn extract_terminology_provider_from_terminologies_variable(
    value: &FhirPathValue,
    context: &EvaluationContext,
) -> Option<Arc<dyn TerminologyProvider>> {
    if is_terminologies_variable(value) {
        // Return the terminology provider from the context
        context.terminology_provider().cloned()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::terminology::NoOpTerminologyProvider;

    #[test]
    fn test_terminologies_variable_creation() {
        let provider = Arc::new(NoOpTerminologyProvider::default());
        let terminologies_var = TerminologiesVariable::new(provider);
        let fhir_path_value = terminologies_var.to_fhir_path_value();

        assert!(is_terminologies_variable(&fhir_path_value));
    }

    #[test]
    fn test_terminologies_variable_detection() {
        let provider = Arc::new(NoOpTerminologyProvider::default());
        let terminologies_var = TerminologiesVariable::new(provider);
        let fhir_path_value = terminologies_var.to_fhir_path_value();

        assert!(is_terminologies_variable(&fhir_path_value));

        // Test with regular resource
        let regular_resource = FhirPathValue::resource(serde_json::json!({
            "resourceType": "Patient",
            "id": "example"
        }));
        assert!(!is_terminologies_variable(&regular_resource));
    }
}
