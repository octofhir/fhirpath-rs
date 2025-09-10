//! Concrete TerminologyService implementation using TerminologyProvider
//!
//! This module bridges the TerminologyProvider trait with the TerminologyService trait
//! to provide FHIRPath %terminologies built-in variable support with real HTTP requests
//! to terminology servers like tx.fhir.org.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

use super::terminology_provider::{DefaultTerminologyProvider, TerminologyProvider};
use super::terminology_utils::{Coding, TerminologyUtils};
use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::context::TerminologyService;

/// Concrete TerminologyService implementation that uses a TerminologyProvider for actual operations
///
/// This service integrates with the FHIRPath %terminologies built-in variable and provides
/// real terminology server functionality using HTTP requests to servers like tx.fhir.org.
pub struct ConcreteTerminologyService {
    provider: Arc<dyn TerminologyProvider>,
}

impl std::fmt::Debug for ConcreteTerminologyService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcreteTerminologyService")
            .field("provider", &"<TerminologyProvider>")
            .finish()
    }
}

impl ConcreteTerminologyService {
    /// Create a new service using the default terminology provider (tx.fhir.org/r4)
    pub fn new() -> Self {
        Self::with_fhir_version("r4")
    }

    /// Create a new service with specific FHIR version
    ///
    /// # Arguments
    /// * `fhir_version` - FHIR version ("r4", "r4b", "r5", etc.). Defaults to "r4" if not specified.
    pub fn with_fhir_version(fhir_version: &str) -> Self {
        Self {
            provider: Arc::new(DefaultTerminologyProvider::with_fhir_version(fhir_version)),
        }
    }

    /// Create a new service with a custom terminology provider
    pub fn with_provider(provider: Arc<dyn TerminologyProvider>) -> Self {
        Self { provider }
    }

    /// Create a new service with custom server URL
    pub fn with_server_url(server_url: impl Into<String>) -> Self {
        Self {
            provider: Arc::new(DefaultTerminologyProvider::with_server_url(server_url)),
        }
    }

    /// Helper to extract coding from FhirPathValue
    fn extract_coding_from_value(&self, value: &FhirPathValue) -> Result<Coding> {
        TerminologyUtils::extract_coding(value)
    }

    /// Convert Parameters resource to FhirPathValue collection
    fn parameters_to_collection(&self, params: &Value) -> Collection {
        match params.get("parameter") {
            Some(Value::Array(parameters)) => {
                let values: Vec<FhirPathValue> = parameters
                    .iter()
                    .map(|param| FhirPathValue::Resource(Arc::new(param.clone())))
                    .collect();
                Collection::from_values(values)
            }
            _ => Collection::empty(),
        }
    }

    /// Create a FHIR Parameters resource for a single result
    fn create_result_parameters(&self, result: bool) -> Value {
        json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "result",
                    "value": result
                }
            ]
        })
    }

    /// Create a FHIR Parameters resource for expansion results
    fn create_expansion_parameters(&self, codings: &[Coding]) -> Value {
        let contains: Vec<Value> = codings
            .iter()
            .map(|coding| {
                let mut obj = json!({
                    "system": coding.system,
                    "code": coding.code
                });
                if let Some(ref display) = coding.display {
                    obj["display"] = json!(display);
                }
                obj
            })
            .collect();

        json!({
            "resourceType": "ValueSet",
            "expansion": {
                "contains": contains
            }
        })
    }

    /// Create a FHIR Parameters resource for translation results
    fn create_translation_parameters(&self, translations: &[FhirPathValue]) -> Value {
        let mut parameters = Vec::new();

        for translation in translations {
            // Each translation contains equivalence and concept data
            if let FhirPathValue::Resource(resource) = translation {
                // Create the parameter structure expected by the test
                // .parameter.where(name = 'match').part.where(name = 'concept').value.code
                let mut parts = Vec::new();
                
                // Extract concept from the translation
                if let Some(concept) = resource.get("concept") {
                    parts.push(json!({
                        "name": "concept",
                        "value": concept
                    }));
                }
                
                // Add equivalence if present
                if let Some(equivalence) = resource.get("equivalence") {
                    parts.push(json!({
                        "name": "equivalence",
                        "value": equivalence
                    }));
                }
                
                parameters.push(json!({
                    "name": "match",
                    "part": parts
                }));
            }
        }

        json!({
            "resourceType": "Parameters",
            "parameter": parameters
        })
    }
}

impl Default for ConcreteTerminologyService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TerminologyService for ConcreteTerminologyService {
    /// Expand value set and return all codes
    async fn expand(
        &self,
        value_set: &str,
        _params: Option<HashMap<String, String>>,
    ) -> Result<Collection> {
        let codings = self.provider.expand_valueset(value_set).await?;
        let expansion = self.create_expansion_parameters(&codings);
        Ok(Collection::from_values(vec![FhirPathValue::Resource(
            Arc::new(expansion),
        )]))
    }

    /// Look up properties of a coded value
    async fn lookup(
        &self,
        coded: &FhirPathValue,
        params: Option<HashMap<String, String>>,
    ) -> Result<Collection> {
        let coding = self.extract_coding_from_value(coded)?;

        if let Some(params) = params {
            // Handle specific parameter requests
            if let Some(property_name) = params.get("property") {
                let properties = self
                    .provider
                    .get_concept_properties(&coding, property_name)
                    .await?;
                return Ok(Collection::from_values(properties));
            }

            if let Some(language) = params.get("language") {
                let use_code = params.get("use").map(|s| s.as_str());
                let designations = self
                    .provider
                    .get_designations(&coding, Some(language), use_code)
                    .await?;
                return Ok(Collection::from_values(designations));
            }
        }

        // Default lookup - get concept details
        match self.provider.lookup_concept(&coding).await? {
            Some(details) => {
                let lookup_result = json!({
                    "resourceType": "Parameters",
                    "parameter": [
                        {
                            "name": "name",
                            "valueString": details.name.unwrap_or_default()
                        },
                        {
                            "name": "version",
                            "valueString": details.version.unwrap_or_default()
                        },
                        {
                            "name": "display",
                            "valueString": details.display
                        }
                    ]
                });
                Ok(Collection::from_values(vec![FhirPathValue::Resource(
                    Arc::new(lookup_result),
                )]))
            }
            None => Ok(Collection::empty()),
        }
    }

    /// Validate code against value set
    async fn validate_vs(
        &self,
        value_set: &str,
        coded: &FhirPathValue,
        _params: Option<HashMap<String, String>>,
    ) -> Result<Collection> {
        let coding = self.extract_coding_from_value(coded)?;
        let is_member = self
            .provider
            .check_valueset_membership(&coding, value_set)
            .await?;
        let result = self.create_result_parameters(is_member);
        Ok(Collection::from_values(vec![FhirPathValue::Resource(
            Arc::new(result),
        )]))
    }

    /// Check subsumption relationship
    async fn subsumes(
        &self,
        _system: &str,
        coded1: &FhirPathValue,
        coded2: &FhirPathValue,
        _params: Option<HashMap<String, String>>,
    ) -> Result<Collection> {
        let coding1 = self.extract_coding_from_value(coded1)?;
        let coding2 = self.extract_coding_from_value(coded2)?;
        let subsumes = self.provider.check_subsumption(&coding1, &coding2).await?;

        let result = json!({
            "resourceType": "Parameters",
            "parameter": [
                {
                    "name": "outcome",
                    "valueCode": if subsumes { "subsumes" } else { "not-subsumed" }
                }
            ]
        });
        Ok(Collection::from_values(vec![FhirPathValue::Resource(
            Arc::new(result),
        )]))
    }

    /// Translate using concept map
    async fn translate(
        &self,
        concept_map: &str,
        coded: &FhirPathValue,
        params: Option<HashMap<String, String>>,
    ) -> Result<Collection> {
        let coding = self.extract_coding_from_value(coded)?;
        let reverse = params
            .as_ref()
            .and_then(|p| p.get("reverse"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(false);

        let translations = self
            .provider
            .translate_concept(&coding, concept_map, reverse)
            .await?;
        let result = self.create_translation_parameters(&translations);
        Ok(Collection::from_values(vec![FhirPathValue::Resource(
            Arc::new(result),
        )]))
    }

    /// Get terminology server base URL
    fn get_server_url(&self) -> &str {
        // This is a bit tricky since we need to call an async method
        // For now, return a default - this could be improved with async trait methods
        "https://tx.fhir.org/r4"
    }

    /// Set authentication credentials
    async fn set_credentials(&mut self, _credentials: HashMap<String, String>) -> Result<()> {
        // For now, DefaultTerminologyProvider doesn't support authentication
        // This could be enhanced in the future
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::terminology_provider::MockTerminologyProvider;

    #[tokio::test]
    async fn test_concrete_terminology_service_creation() {
        let service = ConcreteTerminologyService::new();
        assert_eq!(service.get_server_url(), "https://tx.fhir.org/r4");
    }

    #[tokio::test]
    async fn test_with_mock_provider() {
        let mock_provider = Arc::new(MockTerminologyProvider);
        let service = ConcreteTerminologyService::with_provider(mock_provider);

        // Test with mock data
        let coding_value = FhirPathValue::Resource(Arc::new(json!({
            "system": "http://hl7.org/fhir/administrative-gender",
            "code": "male",
            "display": "Male"
        })));

        let result = service
            .validate_vs(
                "http://hl7.org/fhir/ValueSet/administrative-gender",
                &coding_value,
                None,
            )
            .await;

        assert!(result.is_ok());
        let collection = result.unwrap();
        assert!(!collection.is_empty());
    }
}
