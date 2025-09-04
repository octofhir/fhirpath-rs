#[cfg(test)]
mod tests {
    use super::super::{FunctionRegistry, FunctionCategory, FunctionContext};
    use super::*;
    use crate::registry::dispatcher::FunctionDispatcher;
    use crate::mock_provider::MockModelProvider;
    use crate::core::FhirPathValue;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_extension_and_children() {
        let registry = FunctionRegistry::default();
        let dispatcher = FunctionDispatcher::new(registry);

        let patient = serde_json::json!({
            "resourceType": "Patient",
            "id": "p1",
            "extension": [
                {
                    "url": "http://hl7.org/fhir/StructureDefinition/patient-nationality",
                    "valueCodeableConcept": {
                        "coding": [{"system":"urn:iso:std:iso:3166", "code":"US", "display":"United States"}]
                    }
                }
            ],
            "identifier": [
                {"system": "http://hl7.org/fhir/sid/us-ssn", "value": "123456789"}
            ],
            "name": [
                {"use": "official", "family": "Doe", "given": ["John", "James"]}
            ]
        });

        let input = vec![FhirPathValue::Resource(patient)];
        let args = vec![FhirPathValue::String(
            "http://hl7.org/fhir/StructureDefinition/patient-nationality".to_string(),
        )];
        let model_provider = MockModelProvider::default();
        let vars: HashMap<String, FhirPathValue> = HashMap::new();
        let ctx = FunctionContext { input: &input, arguments: &args, model_provider: &model_provider, variables: &vars, resource_context: None, terminology: None };

        // extension(url)
        let result = dispatcher.dispatch_sync("extension", &ctx).unwrap();
        assert_eq!(result.len(), 1);

        // children()
        let args2: Vec<FhirPathValue> = vec![];
        let ctx2 = FunctionContext { input: &input, arguments: &args2, model_provider: &model_provider, variables: &vars, resource_context: None, terminology: None };
        let result2 = dispatcher.dispatch_sync("children", &ctx2).unwrap();
        assert!(result2.len() >= 3);

        // identifier(system)
        let args3 = vec![FhirPathValue::String("http://hl7.org/fhir/sid/us-ssn".to_string())];
        let ctx3 = FunctionContext { input: &input, arguments: &args3, model_provider: &model_provider, variables: &vars, resource_context: None, terminology: None };
        let result3 = dispatcher.dispatch_sync("identifier", &ctx3).unwrap();
        assert_eq!(result3.len(), 1);
    }
}
