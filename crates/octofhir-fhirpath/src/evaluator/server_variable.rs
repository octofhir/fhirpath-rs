//! %server system variable implementation
//!
//! This module implements the %server system variable that provides access to
//! FHIR server operations in FHIRPath expressions.
//!
//! Usage examples:
//! - %server.read('Patient', '123')
//! - %server.search(false, 'name=Smith')
//! - %server.capabilities()
//! - %server.at('http://other-server/fhir').read('Patient', '456')

use std::sync::Arc;

use crate::core::FhirPathValue;
use crate::evaluator::EvaluationContext;
use octofhir_fhir_model::ServerProvider;

/// Represents the %server system variable
#[derive(Debug, Clone)]
pub struct ServerVariable {
    server_provider: Arc<dyn ServerProvider>,
}

impl ServerVariable {
    /// Create a new server variable with the given server provider
    pub fn new(server_provider: Arc<dyn ServerProvider>) -> Self {
        Self { server_provider }
    }

    /// Convert to FhirPathValue for use in expressions
    pub fn to_fhir_path_value(&self) -> FhirPathValue {
        let mut server_object = serde_json::Map::new();

        server_object.insert(
            "resourceType".to_string(),
            serde_json::Value::String("ServerVariable".to_string()),
        );

        server_object.insert(
            "_serverProvider".to_string(),
            serde_json::Value::String("internal".to_string()),
        );

        let base_url = self.server_provider.base_url();
        if !base_url.is_empty() {
            server_object.insert(
                "_baseUrl".to_string(),
                serde_json::Value::String(base_url.to_string()),
            );
        }

        FhirPathValue::resource(serde_json::Value::Object(server_object))
    }

    /// Get the underlying server provider
    pub fn server_provider(&self) -> &Arc<dyn ServerProvider> {
        &self.server_provider
    }
}

/// Helper function to check if a FhirPathValue represents the %server variable
pub fn is_server_variable(value: &FhirPathValue) -> bool {
    match value {
        FhirPathValue::Resource(resource, _, _) => resource
            .get("resourceType")
            .and_then(|rt| rt.as_str())
            .map(|rt| rt == "ServerVariable")
            .unwrap_or(false),
        _ => false,
    }
}

/// Extract server provider from a %server variable value.
/// If at() set a custom `_baseUrl`, resolves the provider from the server registry.
/// Falls back to the default provider from the context.
pub fn extract_server_provider(
    value: &FhirPathValue,
    context: &EvaluationContext,
) -> Option<Arc<dyn ServerProvider>> {
    if !is_server_variable(value) {
        return None;
    }
    // Check if at() set a custom _baseUrl
    if let FhirPathValue::Resource(json, _, _) = value
        && let Some(custom_url) = json.get("_baseUrl").and_then(|v| v.as_str())
    {
        // Try to get/create provider for this URL from registry
        if let Some(provider) = context.get_or_register_server(custom_url) {
            return Some(provider);
        }
    }
    // Fallback: default provider from context
    context.server_provider().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::NoOpServerProvider;

    #[test]
    fn test_server_variable_creation() {
        let provider = Arc::new(NoOpServerProvider::default());
        let server_var = ServerVariable::new(provider);
        let fhir_path_value = server_var.to_fhir_path_value();
        assert!(is_server_variable(&fhir_path_value));
    }

    #[test]
    fn test_server_variable_detection() {
        let provider = Arc::new(NoOpServerProvider::default());
        let server_var = ServerVariable::new(provider);
        let fhir_path_value = server_var.to_fhir_path_value();
        assert!(is_server_variable(&fhir_path_value));

        let regular_resource = FhirPathValue::resource(serde_json::json!({
            "resourceType": "Patient",
            "id": "example"
        }));
        assert!(!is_server_variable(&regular_resource));
    }
}
