//! ModelProvider re-exports and utilities
//!
//! This module re-exports the ModelProvider trait from octofhir-fhir-model
//! and provides utility functions for working with ModelProviders.
//!
//! All concrete ModelProvider implementations are now in fhir-model-rs to maintain
//! clean dependency separation and avoid circular dependencies.

use serde_json::Value as JsonValue;

use super::error::{FhirPathError, Result};
use super::error_code::FP0151;

// Re-export ModelProvider trait and types from octofhir-fhir-model
pub use octofhir_fhir_model::{
    error::ModelError,
    provider::{
        ChoiceTypeInfo, ElementInfo, EmptyModelProvider, FhirVersion, ModelProvider, TypeInfo,
    },
};

/// Utility functions for working with ModelProviders
pub mod utils {
    use super::{FP0151, FhirPathError, JsonValue, ModelProvider, Result, TypeInfo};

    /// Return the unqualified part of a FHIRPath type name.
    ///
    /// Examples: `FHIR.Patient` -> `Patient`, `System.String` -> `String`.
    pub fn base_type_name(type_name: &str) -> &str {
        type_name
            .rsplit_once('.')
            .map(|(_, base)| base)
            .unwrap_or(type_name)
    }

    fn contains_type_name(candidates: &[String], type_name: &str, case_insensitive: bool) -> bool {
        let base = base_type_name(type_name);
        candidates.iter().any(|candidate| {
            if case_insensitive {
                candidate.eq_ignore_ascii_case(type_name) || candidate.eq_ignore_ascii_case(base)
            } else {
                candidate == type_name || candidate == base
            }
        })
    }

    fn is_builtin_primitive_type(type_name: &str) -> bool {
        let base = base_type_name(type_name);
        matches!(
            base,
            "Boolean"
                | "String"
                | "Integer"
                | "Long"
                | "Decimal"
                | "Date"
                | "DateTime"
                | "Time"
                | "Quantity"
                | "boolean"
                | "string"
                | "integer"
                | "long"
                | "decimal"
                | "date"
                | "dateTime"
                | "time"
                | "uri"
                | "url"
                | "canonical"
                | "code"
                | "id"
                | "markdown"
                | "oid"
                | "uuid"
                | "instant"
                | "unsignedInt"
                | "positiveInt"
                | "base64Binary"
                | "xhtml"
        )
    }

    /// Check whether a type is known to the active model provider.
    ///
    /// The provider remains authoritative for FHIR resource and complex types. Built-in
    /// FHIRPath/System primitive aliases are accepted so namespace-qualified primitive
    /// expressions such as `System.String` and `FHIR.string` work with lightweight test
    /// providers too.
    pub async fn type_exists(
        model_provider: &(dyn ModelProvider + Send + Sync),
        type_name: &str,
    ) -> bool {
        let base = base_type_name(type_name);

        if let Ok(Some(_)) = model_provider.get_type(type_name).await {
            return true;
        }
        if base != type_name
            && let Ok(Some(_)) = model_provider.get_type(base).await
        {
            return true;
        }

        if let Ok(primitive_types) = model_provider.get_primitive_types().await
            && contains_type_name(&primitive_types, type_name, true)
        {
            return true;
        }

        if let Ok(complex_types) = model_provider.get_complex_types().await
            && contains_type_name(&complex_types, type_name, false)
        {
            return true;
        }

        if let Ok(resource_types) = model_provider.get_resource_types().await
            && contains_type_name(&resource_types, type_name, false)
        {
            return true;
        }

        is_builtin_primitive_type(type_name)
    }

    /// Check whether a property exists on a type using the active model provider.
    pub async fn property_exists(
        model_provider: &(dyn ModelProvider + Send + Sync),
        parent_type: &TypeInfo,
        property_name: &str,
    ) -> bool {
        if let Ok(Some(_)) = model_provider
            .get_element_type(parent_type, property_name)
            .await
        {
            return true;
        }

        let element_names = model_provider.get_element_names(parent_type);
        if !element_names.is_empty() {
            return element_names.iter().any(|name| name == property_name);
        }

        let type_name = parent_type
            .name
            .as_deref()
            .unwrap_or(&parent_type.type_name);
        if let Ok(elements) = model_provider.get_elements(type_name).await
            && !elements.is_empty()
        {
            return elements.iter().any(|element| element.name == property_name);
        }

        true
    }

    /// Return property name suggestions from the active model provider.
    pub async fn property_suggestions(
        model_provider: &(dyn ModelProvider + Send + Sync),
        parent_type: &TypeInfo,
    ) -> Vec<String> {
        let element_names = model_provider.get_element_names(parent_type);
        if !element_names.is_empty() {
            return element_names;
        }

        let type_name = parent_type
            .name
            .as_deref()
            .unwrap_or(&parent_type.type_name);
        model_provider
            .get_elements(type_name)
            .await
            .map(|elements| elements.into_iter().map(|element| element.name).collect())
            .unwrap_or_default()
    }

    /// Extract resource type from a JsonValue safely
    pub fn extract_resource_type(resource: &JsonValue) -> Option<String> {
        resource
            .get("resourceType")
            .and_then(|rt| rt.as_str())
            .map(|s| s.to_string())
    }

    /// Check if a JsonValue represents a FHIR resource
    pub fn is_fhir_resource(value: &JsonValue) -> bool {
        value.is_object() && value.get("resourceType").is_some()
    }

    /// Extract reference target from a Reference object
    pub fn extract_reference_target(reference: &JsonValue) -> Option<String> {
        reference
            .get("reference")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string())
    }

    /// Parse a reference string into its components (resource_type/id)
    pub fn parse_reference(reference: &str) -> Result<(String, String)> {
        if let Some(slash_pos) = reference.find('/') {
            let resource_type = reference[..slash_pos].to_string();
            let id = reference[slash_pos + 1..].to_string();
            Ok((resource_type, id))
        } else {
            Err(FhirPathError::model_error(
                FP0151,
                format!("Invalid reference format: {reference}"),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_parsing() {
        let (resource_type, id) = utils::parse_reference("Patient/123").unwrap();
        assert_eq!(resource_type, "Patient");
        assert_eq!(id, "123");

        assert!(utils::parse_reference("invalid-ref").is_err());
    }

    #[test]
    fn test_resource_type_extraction() {
        let patient = serde_json::json!({
            "resourceType": "Patient",
            "id": "123"
        });

        assert_eq!(
            utils::extract_resource_type(&patient),
            Some("Patient".to_string())
        );
        assert!(utils::is_fhir_resource(&patient));
    }

    #[tokio::test]
    async fn test_empty_provider_basic() {
        let provider = EmptyModelProvider;

        // Test that EmptyModelProvider returns a generic "Any" type for all lookups
        let patient_type = provider.get_type("Patient").await.unwrap();
        assert!(patient_type.is_some());

        let type_info = patient_type.unwrap();
        assert_eq!(type_info.type_name, "Any");
        assert_eq!(type_info.name, Some("Patient".to_string()));
    }

    #[tokio::test]
    async fn test_type_exists_uses_provider_and_builtin_primitives() {
        let provider = EmptyModelProvider;

        assert!(utils::type_exists(&provider, "Patient").await);
        assert!(utils::type_exists(&provider, "System.String").await);
        assert!(utils::type_exists(&provider, "FHIR.string").await);
        assert!(!utils::type_exists(&provider, "NotAResource").await);
    }
}
