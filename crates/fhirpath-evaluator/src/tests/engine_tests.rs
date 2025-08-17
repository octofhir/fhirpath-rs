// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::super::engine::{EvaluationConfig, FhirPathEngine};
// Basic engine test placeholder - currently unused

#[tokio::test]
async fn test_engine_creation() {
    let _engine = FhirPathEngine::with_mock_provider();

    // Verify engine can be created
    // Basic functionality will be tested in Task 2
}

#[tokio::test]
async fn test_engine_with_custom_config() {
    let config = EvaluationConfig {
        max_recursion_depth: 500,
        timeout_ms: 15000,
        enable_lambda_optimization: false,
        enable_sync_optimization: true,
        memory_limit_mb: Some(100),
        max_expression_nodes: 10000,
        max_collection_size: 100000,
    };

    // Create engine with mock provider first, then apply config
    use octofhir_fhirpath_model::MockModelProvider;
    use octofhir_fhirpath_registry::create_standard_registry;
    use std::sync::Arc;

    let registry = Arc::new(create_standard_registry().await.unwrap());
    let model_provider = Arc::new(MockModelProvider::empty());

    let engine = FhirPathEngine::new(registry, model_provider).with_config(config.clone());

    assert_eq!(engine.config().max_recursion_depth, 500);
    assert_eq!(engine.config().timeout_ms, 15000);
    assert!(!engine.config().enable_lambda_optimization);
    assert_eq!(engine.config().memory_limit_mb, Some(100));
}

#[tokio::test]
async fn test_thread_safety() {
    let engine = FhirPathEngine::with_mock_provider();

    // Test that engine can be shared between threads
    let engine_clone = engine;
    let handle = tokio::spawn(async move {
        // This should compile and run without issues
        let _ = engine_clone;
    });

    handle.await.unwrap();
}

// FHIR Choice Type Polymorphic Access Tests

#[tokio::test]
async fn test_fhir_choice_type_value_access() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test Observation with valueString
    let observation_string = serde_json::json!({
        "resourceType": "Observation",
        "id": "obs1",
        "valueString": "test result"
    });
    
    let result = engine.evaluate("Observation.value", observation_string.clone()).await.unwrap();
    assert_eq!(result.to_string_value(), Some("test result".to_string()));
    
    // Direct access should also work
    let result_direct = engine.evaluate("Observation.valueString", observation_string).await.unwrap();
    assert_eq!(result_direct.to_string_value(), Some("test result".to_string()));
}

#[tokio::test]
async fn test_fhir_choice_type_value_quantity() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test Observation with valueQuantity
    let observation_quantity = serde_json::json!({
        "resourceType": "Observation",
        "id": "obs2",
        "valueQuantity": {
            "value": 120,
            "unit": "mmHg",
            "system": "http://unitsofmeasure.org",
            "code": "mm[Hg]"
        }
    });
    
    let result = engine.evaluate("Observation.value.value", observation_quantity.clone()).await.unwrap();
    assert_eq!(result.as_integer(), Some(120));
    
    // Direct access should also work
    let result_direct = engine.evaluate("Observation.valueQuantity.value", observation_quantity).await.unwrap();
    assert_eq!(result_direct.as_integer(), Some(120));
}

#[tokio::test]
async fn test_fhir_choice_type_value_boolean() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test with valueBoolean
    let observation_boolean = serde_json::json!({
        "resourceType": "Observation",
        "id": "obs3",
        "valueBoolean": true
    });
    
    let result = engine.evaluate("Observation.value", observation_boolean.clone()).await.unwrap();
    assert_eq!(result.as_boolean(), Some(true));
    
    // Direct access should also work
    let result_direct = engine.evaluate("Observation.valueBoolean", observation_boolean).await.unwrap();
    assert_eq!(result_direct.as_boolean(), Some(true));
}

#[tokio::test]
async fn test_fhir_choice_type_case_sensitivity() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test that lowercase properties after the base name don't match
    let test_resource = serde_json::json!({
        "resourceType": "TestResource",
        "value": "direct_value",
        "valueString": "polymorphic_value",
        "valuetype": "should_not_match",
        "valueLowerCase": "should_not_match_either"
    });
    
    // Should get polymorphic access to valueString, not the lowercase variants
    let result = engine.evaluate("TestResource.value", test_resource.clone()).await.unwrap();
    assert_eq!(result.to_string_value(), Some("direct_value".to_string()));
    
    // Test case where there's no direct property but polymorphic exists
    let test_resource_no_direct = serde_json::json!({
        "resourceType": "TestResource",
        "valueString": "polymorphic_only",
        "valuetype": "should_not_match"
    });
    
    let result = engine.evaluate("TestResource.value", test_resource_no_direct).await.unwrap();
    assert_eq!(result.to_string_value(), Some("polymorphic_only".to_string()));
}

#[tokio::test]
async fn test_fhir_choice_type_other_properties() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test other common FHIR choice types beyond just "value"
    let medication_request = serde_json::json!({
        "resourceType": "MedicationRequest",
        "id": "med1",
        "medicationCodeableConcept": {
            "coding": [{
                "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                "code": "582620",
                "display": "Nizatidine 15 MG/ML Oral Solution"
            }]
        }
    });
    
    // Should be able to access medication property via polymorphic access
    let result = engine.evaluate("MedicationRequest.medication.coding.display", medication_request.clone()).await.unwrap();
    assert_eq!(result.to_string_value(), Some("Nizatidine 15 MG/ML Oral Solution".to_string()));
    
    // Direct access should also work
    let result_direct = engine.evaluate("MedicationRequest.medicationCodeableConcept.coding.display", medication_request).await.unwrap();
    assert_eq!(result_direct.to_string_value(), Some("Nizatidine 15 MG/ML Oral Solution".to_string()));
}

#[tokio::test]
async fn test_fhir_choice_type_empty_when_not_found() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test that accessing a choice type property that doesn't exist returns empty
    let observation_no_value = serde_json::json!({
        "resourceType": "Observation",
        "id": "obs4",
        "status": "final"
    });
    
    let result = engine.evaluate("Observation.value", observation_no_value).await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_fhir_choice_type_collection_handling() {
    let engine = FhirPathEngine::with_mock_provider().await.unwrap();
    
    // Test choice type access on collections
    let bundle = serde_json::json!({
        "resourceType": "Bundle",
        "entry": [
            {
                "resource": {
                    "resourceType": "Observation",
                    "valueString": "first result"
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "valueQuantity": {
                        "value": 42
                    }
                }
            }
        ]
    });
    
    let result = engine.evaluate("Bundle.entry.resource.value", bundle).await.unwrap();
    // Should get both values in the collection
    match result {
        octofhir_fhirpath_model::FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 2);
            // First item should be the string
            assert_eq!(items.first().unwrap().to_string_value(), Some("first result".to_string()));
        }
        single => {
            // If it's a single value, that's also acceptable depending on implementation
            panic!("Expected collection but got: {:?}", single);
        }
    }
}
