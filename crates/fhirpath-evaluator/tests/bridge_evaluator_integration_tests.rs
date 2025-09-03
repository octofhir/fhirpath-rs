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

//! Integration tests for bridge-enabled FHIRPath evaluator
//!
//! Tests the complete evaluation system with bridge support API integration
//! including choice type resolution, property navigation, and type checking.

use octofhir_canonical_manager::FcmConfig;
use octofhir_fhirpath_evaluator::FhirPathEngine;
// TODO: Re-enable when FhirSchemaModelProvider is moved to core
// use octofhir_fhir_model::FhirSchemaModelProvider;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::json;
use std::sync::Arc;

async fn create_test_engine() -> FhirPathEngine {
    let fcm_config = FcmConfig::default();
    let config = PackageManagerConfig::default();
    let schema_manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .expect("Failed to create schema manager"),
    );
    let model_provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());

    FhirPathEngine::with_bridge_support(schema_manager, model_provider)
        .await
        .expect("Failed to create bridge-enabled engine")
}

#[tokio::test]
async fn test_bridge_property_navigation() {
    let engine = create_test_engine().await;

    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [{
            "given": ["John"],
            "family": "Doe"
        }],
        "gender": "male"
    });

    // Test simple property access
    let result = engine
        .evaluate("Patient.gender", patient.clone())
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert_eq!(items.len(), 1);
        if let Some(octofhir_fhir_model::FhirPathValue::String(gender)) = items.first() {
            assert_eq!(gender.as_ref(), "male");
        } else {
            panic!("Expected string value for gender");
        }
    } else {
        panic!("Expected collection result");
    }

    // Test complex property navigation
    let result = engine
        .evaluate("Patient.name.given", patient)
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert!(!items.is_empty());
        if let Some(octofhir_fhir_model::FhirPathValue::String(given)) = items.first() {
            assert_eq!(given.as_ref(), "John");
        } else {
            panic!("Expected string value for given name");
        }
    } else {
        panic!("Expected collection result");
    }
}

#[tokio::test]
async fn test_bridge_choice_type_resolution() {
    let engine = create_test_engine().await;

    let observation = json!({
        "resourceType": "Observation",
        "id": "example",
        "status": "final",
        "code": {
            "coding": [{
                "system": "http://loinc.org",
                "code": "15074-8",
                "display": "Glucose"
            }]
        },
        "valueQuantity": {
            "value": 6.3,
            "unit": "mmol/l",
            "system": "http://unitsofmeasure.org",
            "code": "mmol/L"
        }
    });

    // Test choice type resolution - valueQuantity should be accessible via value[x]
    let result = engine
        .evaluate("Observation.valueQuantity", observation.clone())
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert_eq!(items.len(), 1);
        // Should be a JSON object representing the Quantity
        if let Some(octofhir_fhir_model::FhirPathValue::JsonValue(quantity)) = items.first() {
            assert!(quantity.get_property("value").is_some());
            assert!(quantity.get_property("unit").is_some());
        } else {
            panic!("Expected JsonValue for quantity");
        }
    } else {
        panic!("Expected collection result");
    }

    // Test accessing quantity properties
    let result = engine
        .evaluate("Observation.valueQuantity.value", observation)
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert_eq!(items.len(), 1);
        // Value should be a number
        match items.first() {
            Some(octofhir_fhir_model::FhirPathValue::Decimal(_))
            | Some(octofhir_fhir_model::FhirPathValue::Integer(_)) => {
                // Success - got a numeric value
            }
            _ => panic!("Expected numeric value for quantity.value"),
        }
    } else {
        panic!("Expected collection result");
    }
}

#[tokio::test]
async fn test_bridge_type_checking() {
    let engine = create_test_engine().await;

    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "active": true
    });

    // Test basic type checking
    let result = engine
        .evaluate("Patient.active is boolean", patient.clone())
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert_eq!(items.len(), 1);
        if let Some(octofhir_fhir_model::FhirPathValue::Boolean(is_boolean)) = items.first() {
            assert!(*is_boolean);
        } else {
            panic!("Expected boolean result for type check");
        }
    } else {
        panic!("Expected collection result");
    }

    // Test resource type checking
    let result = engine
        .evaluate("Patient is Patient", patient)
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert_eq!(items.len(), 1);
        if let Some(octofhir_fhir_model::FhirPathValue::Boolean(is_patient)) = items.first() {
            assert!(*is_patient);
        } else {
            panic!("Expected boolean result for resource type check");
        }
    } else {
        panic!("Expected collection result");
    }
}

#[tokio::test]
async fn test_bridge_complex_navigation() {
    let engine = create_test_engine().await;

    let bundle = json!({
        "resourceType": "Bundle",
        "id": "example",
        "type": "collection",
        "entry": [{
            "resource": {
                "resourceType": "Patient",
                "id": "patient1",
                "name": [{
                    "given": ["Alice"],
                    "family": "Smith"
                }]
            }
        }, {
            "resource": {
                "resourceType": "Observation",
                "id": "obs1",
                "status": "final",
                "valueString": "Normal"
            }
        }]
    });

    // Test navigation through Bundle to Patient names
    let result = engine
        .evaluate("Bundle.entry.resource.ofType(Patient).name.given", bundle)
        .await
        .unwrap();
    if let octofhir_fhir_model::FhirPathValue::Collection(items) = result {
        assert!(!items.is_empty());
        if let Some(octofhir_fhir_model::FhirPathValue::String(given)) = items.first() {
            assert_eq!(given.as_ref(), "Alice");
        } else {
            panic!("Expected string value for given name");
        }
    } else {
        panic!("Expected collection result");
    }
}

#[tokio::test]
async fn test_bridge_performance_characteristics() {
    let engine = create_test_engine().await;

    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [{
            "given": ["John"],
            "family": "Doe"
        }]
    });

    // Test that multiple evaluations are fast (should hit cache)
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _result = engine
            .evaluate("Patient.name.given", patient.clone())
            .await
            .unwrap();
    }
    let duration = start.elapsed();

    // 100 evaluations should complete quickly with caching
    assert!(
        duration.as_millis() < 1000,
        "Bridge evaluation should be fast with caching"
    );
}

#[tokio::test]
async fn test_bridge_error_handling() {
    let engine = create_test_engine().await;

    let patient = json!({
        "resourceType": "Patient",
        "id": "example"
    });

    // Test accessing non-existent property - should return empty, not error
    let result = engine
        .evaluate("Patient.nonExistentProperty", patient)
        .await
        .unwrap();
    match result {
        octofhir_fhir_model::FhirPathValue::Empty => {
            // Correct - should return empty for non-existent properties
        }
        octofhir_fhir_model::FhirPathValue::Collection(items) => {
            assert!(
                items.is_empty(),
                "Should return empty collection for non-existent property"
            );
        }
        _ => panic!("Expected empty result for non-existent property"),
    }
}
