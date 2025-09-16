//! ModelProvider re-exports and utilities
//!
//! This module re-exports the ModelProvider trait from octofhir-fhir-model
//! and provides utility functions for working with ModelProviders.
//!
//! All concrete ModelProvider implementations are now in fhir-model-rs to maintain
//! clean dependency separation and avoid circular dependencies.

use serde_json::Value as JsonValue;

use super::error::{FhirPathError, Result};
use super::error_code::*;

// Re-export ModelProvider trait and types from octofhir-fhir-model
pub use octofhir_fhir_model::{
    error::ModelError,
    provider::{EmptyModelProvider, FhirVersion, ModelProvider, NavigationResult, TypeInfo},
};

/// Utility functions for working with ModelProviders
pub mod utils {
    use super::*;

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

        // Test that EmptyModelProvider returns None for type lookup
        let patient_type = provider.get_type("Patient").await.unwrap();
        assert!(patient_type.is_none());
    }
}
