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

//! Integration tests with real FHIR resources and complex scenarios

use super::{
    TestUtils, as_collection_strings, as_single_boolean, as_single_decimal, as_single_integer,
    as_single_string, count,
};
use serde_json::json;

#[tokio::test]
async fn test_patient_resource_navigation() {
    let engine = TestUtils::create_test_engine().await.unwrap();
    let patient = TestUtils::sample_patient();

    // Test all common Patient navigation patterns
    let test_cases = vec![
        ("name.family", vec!["Doe".to_string()]),
        (
            "name.given",
            vec![
                "John".to_string(),
                "Robert".to_string(),
                "Johnny".to_string(),
            ],
        ),
        (
            "telecom.where(system='phone').value",
            vec!["555-1234".to_string()],
        ),
        (
            "telecom.where(system='email').value",
            vec!["john.doe@example.com".to_string()],
        ),
        ("address.line", vec!["123 Main St".to_string()]),
        ("address.city", vec!["Anytown".to_string()]),
        ("gender", vec!["male".to_string()]),
        ("birthDate", vec!["1974-12-25".to_string()]),
    ];

    for (expression, expected) in test_cases {
        let result = engine.evaluate(expression, patient.clone()).await.unwrap();
        let actual = as_collection_strings(&result).unwrap_or_default();

        // Check if we got the expected values (order might vary)
        for expected_val in expected {
            assert!(
                actual.contains(&expected_val),
                "Expected '{expected_val}' in result for expression '{expression}', got: {actual:?}"
            );
        }
    }
}

#[tokio::test]
async fn test_bundle_resource_navigation() {
    let engine = TestUtils::create_test_engine().await.unwrap();
    let bundle = TestUtils::sample_bundle();

    // Test Bundle structure navigation
    let result = engine.evaluate("type", bundle.clone()).await.unwrap();
    assert_eq!(as_single_string(&result), Some("collection".to_string()));

    let result = engine
        .evaluate("entry.count()", bundle.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(2));

    // Test filtering resources by type
    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Patient')",
            bundle.clone(),
        )
        .await
        .unwrap();
    assert_eq!(count(&result), 1);

    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Observation')",
            bundle.clone(),
        )
        .await
        .unwrap();
    assert_eq!(count(&result), 1);

    // Test accessing nested resource properties
    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Patient').name.given",
            bundle.clone(),
        )
        .await
        .unwrap();
    assert!(count(&result) >= 2); // Should have "John" and "Robert" at minimum

    // Test cross-resource references
    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Observation').subject.reference",
            bundle,
        )
        .await
        .unwrap();
    assert_eq!(
        as_single_string(&result),
        Some("Patient/example".to_string())
    );
}

#[tokio::test]
async fn test_complex_fhir_expressions() {
    let engine = TestUtils::create_test_engine().await.unwrap();
    let patient = TestUtils::sample_patient();

    // Test complex real-world FHIRPath expressions
    let expressions = vec![
        // Get primary phone number
        "telecom.where(system='phone' and use='home').value.first()",
        // Check if patient has any work contact
        "telecom.where(use='work').exists()",
        // Get full official name (simplified concatenation)
        "name.where(use='official').given.first()",
        // Check if birth date exists
        "birthDate.exists()",
        // Address validation - check if home address has required fields
        "address.where(use='home').line.exists() and address.where(use='home').city.exists()",
        // Check if patient has complete name
        "name.where(family.exists() and given.exists()).exists()",
        // Validate telecom has system and value
        "telecom.all(system.exists() and value.exists())",
    ];

    for expression in expressions {
        let result = engine.evaluate(expression, patient.clone()).await;
        assert!(result.is_ok(), "Failed to evaluate: {expression}");

        // Log results for inspection
        if let Ok(value) = result {
            println!("Expression '{expression}' -> {value:?}");
        }
    }
}

#[tokio::test]
async fn test_fhir_resource_validation_patterns() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test common validation patterns
    let valid_patient = TestUtils::sample_patient();

    // Patient must have name
    let result = engine
        .evaluate("name.exists()", valid_patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    // Patient must have gender
    let result = engine
        .evaluate("gender.exists()", valid_patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    // Check if patient has valid resource type
    let result = engine
        .evaluate("resourceType = 'Patient'", valid_patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    // Check if patient has identifier (our sample doesn't, so should be false)
    let result = engine
        .evaluate("identifier.exists()", valid_patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));

    // Test with invalid/incomplete resource
    let invalid_patient = json!({
        "resourceType": "Patient",
        "id": "invalid"
        // Missing required fields
    });

    let result = engine
        .evaluate("name.exists()", invalid_patient.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));

    let result = engine
        .evaluate("gender.exists()", invalid_patient)
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(false));
}

#[tokio::test]
async fn test_observation_resource_patterns() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Create a more complex Observation
    let observation = json!({
        "resourceType": "Observation",
        "id": "weight-example",
        "status": "final",
        "category": [{
            "coding": [{
                "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                "code": "vital-signs",
                "display": "Vital Signs"
            }]
        }],
        "code": {
            "coding": [{
                "system": "http://loinc.org",
                "code": "29463-7",
                "display": "Body Weight"
            }]
        },
        "subject": {
            "reference": "Patient/example"
        },
        "valueQuantity": {
            "value": 85.5,
            "unit": "kg",
            "system": "http://unitsofmeasure.org",
            "code": "kg"
        },
        "component": [
            {
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "8302-2",
                        "display": "Body height"
                    }]
                },
                "valueQuantity": {
                    "value": 175,
                    "unit": "cm",
                    "system": "http://unitsofmeasure.org",
                    "code": "cm"
                }
            }
        ]
    });

    // Test Observation navigation patterns
    let result = engine
        .evaluate("status", observation.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("final".to_string()));

    let result = engine
        .evaluate("code.coding.code", observation.clone())
        .await
        .unwrap();
    let codes = as_collection_strings(&result).unwrap_or_default();
    assert!(codes.contains(&"29463-7".to_string()));

    let result = engine
        .evaluate("valueQuantity.value", observation.clone())
        .await
        .unwrap();
    if let Some(value) = as_single_decimal(&result) {
        assert!((value.to_string().parse::<f64>().unwrap_or(0.0) - 85.5).abs() < 0.001);
    }

    // Test component navigation
    let result = engine
        .evaluate("component.count()", observation.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(1));

    let result = engine
        .evaluate("component.valueQuantity.value", observation)
        .await
        .unwrap();
    if let Some(height) = as_single_integer(&result) {
        assert_eq!(height, 175);
    }
}

#[tokio::test]
async fn test_medication_request_patterns() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    let medication_request = json!({
        "resourceType": "MedicationRequest",
        "id": "example",
        "status": "active",
        "intent": "order",
        "medicationCodeableConcept": {
            "coding": [{
                "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                "code": "1049502",
                "display": "Acetaminophen 325 MG Oral Tablet"
            }]
        },
        "subject": {
            "reference": "Patient/example"
        },
        "dosageInstruction": [{
            "text": "Take 1-2 tablets every 4-6 hours as needed for pain",
            "timing": {
                "repeat": {
                    "frequency": 1,
                    "period": 4,
                    "periodUnit": "h"
                }
            },
            "doseAndRate": [{
                "doseRange": {
                    "low": {
                        "value": 1,
                        "unit": "tablet"
                    },
                    "high": {
                        "value": 2,
                        "unit": "tablet"
                    }
                }
            }]
        }]
    });

    // Test MedicationRequest patterns
    let result = engine
        .evaluate("status", medication_request.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("active".to_string()));

    let result = engine
        .evaluate("intent", medication_request.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("order".to_string()));

    let result = engine
        .evaluate(
            "medicationCodeableConcept.coding.code",
            medication_request.clone(),
        )
        .await
        .unwrap();
    let codes = as_collection_strings(&result).unwrap_or_default();
    assert!(codes.contains(&"1049502".to_string()));

    // Test dosage instruction navigation
    let result = engine
        .evaluate("dosageInstruction.count()", medication_request.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(1));

    let result = engine
        .evaluate(
            "dosageInstruction.doseAndRate.doseRange.low.value",
            medication_request,
        )
        .await
        .unwrap();
    if let Some(dose) = as_single_integer(&result) {
        assert_eq!(dose, 1);
    }
}

#[tokio::test]
async fn test_practitioner_and_organization_patterns() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    let practitioner = json!({
        "resourceType": "Practitioner",
        "id": "example",
        "identifier": [{
            "system": "http://www.acme.org/practitioners",
            "value": "23"
        }],
        "name": [{
            "family": "Careful",
            "given": ["Adam", "Dr."],
            "prefix": ["Dr"]
        }],
        "telecom": [{
            "system": "email",
            "value": "dr.careful@example.org",
            "use": "work"
        }],
        "qualification": [{
            "code": {
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/v2-0360/2.7",
                    "code": "MD",
                    "display": "Doctor of Medicine"
                }]
            }
        }]
    });

    // Test Practitioner patterns
    let result = engine
        .evaluate("name.family", practitioner.clone())
        .await
        .unwrap();
    let families = as_collection_strings(&result).unwrap_or_default();
    assert!(families.contains(&"Careful".to_string()));

    let result = engine
        .evaluate("name.given", practitioner.clone())
        .await
        .unwrap();
    let given_names = as_collection_strings(&result).unwrap_or_default();
    assert!(given_names.contains(&"Adam".to_string()));
    assert!(given_names.contains(&"Dr.".to_string()));

    let result = engine
        .evaluate("qualification.code.coding.code", practitioner.clone())
        .await
        .unwrap();
    let codes = as_collection_strings(&result).unwrap_or_default();
    assert!(codes.contains(&"MD".to_string()));

    // Test identifier access
    let result = engine
        .evaluate(
            "identifier.where(system='http://www.acme.org/practitioners').value",
            practitioner,
        )
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("23".to_string()));
}

#[tokio::test]
async fn test_complex_bundle_operations() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Create a larger bundle with multiple resource types
    let complex_bundle = json!({
        "resourceType": "Bundle",
        "id": "complex-example",
        "type": "searchset",
        "total": 3,
        "entry": [
            {
                "resource": TestUtils::sample_patient()
            },
            {
                "resource": {
                    "resourceType": "Practitioner",
                    "id": "prac1",
                    "name": [{"family": "Smith", "given": ["John"]}]
                }
            },
            {
                "resource": {
                    "resourceType": "Organization",
                    "id": "org1",
                    "name": "Test Hospital",
                    "type": [{
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/organization-type",
                            "code": "prov",
                            "display": "Healthcare Provider"
                        }]
                    }]
                }
            }
        ]
    });

    // Test bundle summary operations
    let result = engine
        .evaluate("entry.count()", complex_bundle.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(3));

    let result = engine
        .evaluate("total", complex_bundle.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(3));

    // Test filtering by resource type
    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Patient').count()",
            complex_bundle.clone(),
        )
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(1));

    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Practitioner').count()",
            complex_bundle.clone(),
        )
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(1));

    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Organization').count()",
            complex_bundle.clone(),
        )
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(1));

    // Test cross-resource queries
    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Practitioner').name.family",
            complex_bundle.clone(),
        )
        .await
        .unwrap();
    let families = as_collection_strings(&result).unwrap_or_default();
    assert!(families.contains(&"Smith".to_string()));

    let result = engine
        .evaluate(
            "entry.resource.where(resourceType='Organization').name",
            complex_bundle,
        )
        .await
        .unwrap();
    let org_names = as_collection_strings(&result).unwrap_or_default();
    assert!(org_names.contains(&"Test Hospital".to_string()));
}

#[tokio::test]
async fn test_fhir_data_type_handling() {
    let engine = TestUtils::create_test_engine().await.unwrap();

    // Test various FHIR data types
    let resource_with_types = json!({
        "resourceType": "TestResource",
        "booleanField": true,
        "integerField": 42,
        "decimalField": std::f64::consts::PI,
        "stringField": "hello world",
        "dateField": "2023-12-25",
        "dateTimeField": "2023-12-25T10:30:00Z",
        "codeField": "active",
        "codingField": {
            "system": "http://example.org/codes",
            "code": "test-code",
            "display": "Test Code"
        },
        "quantityField": {
            "value": 85.5,
            "unit": "kg",
            "system": "http://unitsofmeasure.org",
            "code": "kg"
        },
        "arrayField": ["item1", "item2", "item3"],
        "nestedObject": {
            "field1": "value1",
            "field2": 123
        }
    });

    // Test accessing different data types
    let result = engine
        .evaluate("booleanField", resource_with_types.clone())
        .await
        .unwrap();
    assert_eq!(as_single_boolean(&result), Some(true));

    let result = engine
        .evaluate("integerField", resource_with_types.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(42));

    let result = engine
        .evaluate("stringField", resource_with_types.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("hello world".to_string()));

    let result = engine
        .evaluate("arrayField.count()", resource_with_types.clone())
        .await
        .unwrap();
    assert_eq!(as_single_integer(&result), Some(3));

    let result = engine
        .evaluate("codingField.code", resource_with_types.clone())
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("test-code".to_string()));

    let result = engine
        .evaluate("quantityField.value", resource_with_types.clone())
        .await
        .unwrap();
    if let Some(value) = as_single_decimal(&result) {
        assert!((value.to_string().parse::<f64>().unwrap_or(0.0) - 85.5).abs() < 0.001);
    }

    let result = engine
        .evaluate("nestedObject.field1", resource_with_types)
        .await
        .unwrap();
    assert_eq!(as_single_string(&result), Some("value1".to_string()));
}
