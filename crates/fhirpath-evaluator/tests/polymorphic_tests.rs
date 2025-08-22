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

//! Comprehensive test suite for polymorphic path resolution
//!
//! This module tests the polymorphic path resolution engine's ability to handle
//! FHIR choice types (value[x] patterns) correctly.

use octofhir_fhirpath_evaluator::FhirPathEngine;
use octofhir_fhirpath_model::FhirPathValue;
use sonic_rs::{JsonValueTrait, json};

/// Test basic observation value resolution
#[tokio::test]
async fn test_observation_value_quantity_resolution() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider()
        .await?
        .with_polymorphic_navigation();

    let observation = json!({
        "resourceType": "Observation",
        "id": "weight-example",
        "valueQuantity": {
            "value": 185,
            "unit": "lbs",
            "system": "http://unitsofmeasure.org",
            "code": "[lb_av]"
        }
    });

    // Test basic value resolution
    let result = engine
        .evaluate("Observation.value", observation.clone())
        .await?;

    match &result {
        FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 1);
            // Should resolve to the valueQuantity object
            if let Some(FhirPathValue::JsonValue(json_val)) = items.iter().next() {
                let sonic_val = json_val.as_sonic_value();
                assert_eq!(sonic_val.get("value").unwrap().as_i64().unwrap(), 185);
                assert_eq!(sonic_val.get("unit").unwrap().as_str().unwrap(), "lbs");
            } else {
                panic!("Expected JsonValue result");
            }
        }
        _ => panic!("Expected collection result"),
    }

    Ok(())
}

/// Test observation value unit access through polymorphic path
#[tokio::test]
async fn test_observation_value_unit_access() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider()
        .await?
        .with_polymorphic_navigation();

    let observation = json!({
        "resourceType": "Observation",
        "valueQuantity": {
            "value": 185,
            "unit": "lbs"
        }
    });

    // Test unit access through polymorphic path
    let result = engine
        .evaluate("Observation.value.unit", observation)
        .await?;

    match &result {
        FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 1);
            match items.iter().next() {
                Some(FhirPathValue::String(s)) => assert_eq!(s.as_ref(), "lbs"),
                _ => panic!("Expected string result for unit"),
            }
        }
        _ => panic!("Expected collection result"),
    }

    Ok(())
}

/// Test observation with string value
#[tokio::test]
async fn test_observation_value_string() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider()
        .await?
        .with_polymorphic_navigation();

    let observation = json!({
        "resourceType": "Observation",
        "valueString": "Normal"
    });

    // Test string value resolution
    let result = engine.evaluate("Observation.value", observation).await?;

    match &result {
        FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 1);
            match items.iter().next() {
                Some(FhirPathValue::String(s)) => assert_eq!(s.as_ref(), "Normal"),
                _ => panic!("Expected string result"),
            }
        }
        _ => panic!("Expected collection result"),
    }

    Ok(())
}

/// Test observation with boolean value
#[tokio::test]
async fn test_observation_value_boolean() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider()
        .await?
        .with_polymorphic_navigation();

    let observation = json!({
        "resourceType": "Observation",
        "valueBoolean": true
    });

    // Test boolean value resolution
    let result = engine.evaluate("Observation.value", observation).await?;

    match &result {
        FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 1);
            match items.iter().next() {
                Some(FhirPathValue::Boolean(b)) => assert!(*b),
                _ => panic!("Expected boolean result"),
            }
        }
        _ => panic!("Expected collection result"),
    }

    Ok(())
}

/// Test patient deceased choice type
#[tokio::test]
async fn test_patient_deceased_boolean() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider()
        .await?
        .with_polymorphic_navigation();

    let patient = json!({
        "resourceType": "Patient",
        "deceasedBoolean": true
    });

    // Test deceased boolean resolution
    let result = engine.evaluate("Patient.deceased", patient).await?;

    match &result {
        FhirPathValue::Collection(items) => {
            assert_eq!(items.len(), 1);
            match items.iter().next() {
                Some(FhirPathValue::Boolean(b)) => assert!(*b),
                _ => panic!("Expected boolean result"),
            }
        }
        _ => panic!("Expected collection result"),
    }

    Ok(())
}

/// Test polymorphic navigation with regular properties still works
#[tokio::test]
async fn test_regular_properties_still_work() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider()
        .await?
        .with_polymorphic_navigation();

    let patient = json!({
        "resourceType": "Patient",
        "name": [{
            "given": ["John"],
            "family": "Doe"
        }],
        "deceasedBoolean": false
    });

    // Test regular property access (should not be affected by polymorphic navigation)
    let name_result = engine.evaluate("Patient.name", patient.clone()).await?;
    let given_result = engine
        .evaluate("Patient.name.given", patient.clone())
        .await?;
    let family_result = engine
        .evaluate("Patient.name.family", patient.clone())
        .await?;

    // Also test choice type in same resource
    let deceased_result = engine.evaluate("Patient.deceased", patient).await?;

    // Verify all work correctly
    assert!(!matches!(name_result, FhirPathValue::Empty));
    assert!(!matches!(given_result, FhirPathValue::Empty));
    assert!(!matches!(family_result, FhirPathValue::Empty));
    assert!(!matches!(deceased_result, FhirPathValue::Empty));

    Ok(())
}
