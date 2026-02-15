//! %factory system variable implementation
//!
//! This module implements the %factory system variable that provides access to
//! type factory operations in FHIRPath specification.
//!
//! Usage examples:
//! - %factory.Extension('http://example.org/ext', 'value')
//! - %factory.Coding('http://loinc.org', '29463-7', 'Body Weight')
//! - %factory.create('Patient')
//! - %factory.withProperty(instance, 'name', value)

use crate::core::FhirPathValue;

/// Represents the %factory system variable
///
/// This is a special resource-like object that provides access to type factory functions
/// through method-like syntax in FHIRPath expressions.
#[derive(Debug, Clone)]
pub struct FactoryVariable;

impl FactoryVariable {
    /// Convert to FhirPathValue for use in expressions
    ///
    /// The %factory variable appears as a special Resource-like object
    /// that exposes factory operations as "properties" that can be called.
    pub fn to_fhir_path_value() -> FhirPathValue {
        let mut factory_object = serde_json::Map::new();

        factory_object.insert(
            "resourceType".to_string(),
            serde_json::Value::String("FactoryVariable".to_string()),
        );

        factory_object.insert(
            "supportedOperations".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::String("Extension".to_string()),
                serde_json::Value::String("Identifier".to_string()),
                serde_json::Value::String("HumanName".to_string()),
                serde_json::Value::String("ContactPoint".to_string()),
                serde_json::Value::String("Address".to_string()),
                serde_json::Value::String("Quantity".to_string()),
                serde_json::Value::String("Coding".to_string()),
                serde_json::Value::String("CodeableConcept".to_string()),
                serde_json::Value::String("create".to_string()),
                serde_json::Value::String("withExtension".to_string()),
                serde_json::Value::String("withProperty".to_string()),
            ]),
        );

        FhirPathValue::resource(serde_json::Value::Object(factory_object))
    }
}

/// Helper function to check if a FhirPathValue represents the %factory variable
pub fn is_factory_variable(value: &FhirPathValue) -> bool {
    match value {
        FhirPathValue::Resource(resource, _, _) => resource
            .get("resourceType")
            .and_then(|rt| rt.as_str())
            .map(|rt| rt == "FactoryVariable")
            .unwrap_or(false),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_variable_creation() {
        let fhir_path_value = FactoryVariable::to_fhir_path_value();
        assert!(is_factory_variable(&fhir_path_value));
    }

    #[test]
    fn test_factory_variable_detection() {
        let fhir_path_value = FactoryVariable::to_fhir_path_value();
        assert!(is_factory_variable(&fhir_path_value));

        // Test with regular resource
        let regular_resource = FhirPathValue::resource(serde_json::json!({
            "resourceType": "Patient",
            "id": "example"
        }));
        assert!(!is_factory_variable(&regular_resource));
    }
}
