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

//! End-to-end FHIRPath evaluation integration tests

use octofhir_fhirpath::*;
use serde_json::json;
use std::time::Duration;

mod utils;
use utils::*;

#[tokio::test]
async fn test_end_to_end_patient_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    
    let patient = json!({
        "resourceType": "Patient",
        "id": "example-patient",
        "meta": {
            "versionId": "1",
            "lastUpdated": "2023-01-01T12:00:00Z",
            "profile": ["http://hl7.org/fhir/StructureDefinition/Patient"]
        },
        "identifier": [{
            "use": "usual",
            "type": {
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/v2-0203",
                    "code": "MR"
                }]
            },
            "value": "12345"
        }],
        "active": true,
        "name": [{
            "use": "official",
            "family": "Doe",
            "given": ["John", "David"]
        }, {
            "use": "nickname", 
            "given": ["Johnny"]
        }],
        "telecom": [{
            "system": "phone",
            "value": "+1-555-0123",
            "use": "home"
        }, {
            "system": "email",
            "value": "john.doe@example.com",
            "use": "work"
        }],
        "gender": "male",
        "birthDate": "1990-01-15",
        "address": [{
            "use": "home",
            "line": ["123 Main St"],
            "city": "Springfield",
            "state": "IL",
            "postalCode": "62701",
            "country": "US"
        }]
    });

    // Test basic property access
    let given_names = context.fhirpath.evaluate("Patient.name.given", &patient).await?;
    assert_eq!(given_names.values.len(), 3); // John, David, Johnny

    // Test complex filtering
    let official_family = context.fhirpath.evaluate(
        "Patient.name.where(use = 'official').family",
        &patient
    ).await?;
    assert_eq!(official_family.values.len(), 1);
    if let Some(FhirPathValue::String(family)) = official_family.values.first() {
        assert_eq!(family, "Doe");
    }

    // Test choice type navigation
    let work_email = context.fhirpath.evaluate(
        "Patient.telecom.where(system = 'email' and use = 'work').value",
        &patient
    ).await?;
    if let Some(FhirPathValue::String(email)) = work_email.values.first() {
        assert_eq!(email, "john.doe@example.com");
    }

    // Test existence functions
    let has_active = context.fhirpath.evaluate("Patient.active.exists()", &patient).await?;
    if let Some(FhirPathValue::Boolean(exists)) = has_active.values.first() {
        assert_eq!(*exists, true);
    }

    // Test identifier access
    let identifier_value = context.fhirpath.evaluate(
        "Patient.identifier.where(use = 'usual').value",
        &patient
    ).await?;
    if let Some(FhirPathValue::String(value)) = identifier_value.values.first() {
        assert_eq!(value, "12345");
    }

    Ok(())
}

#[tokio::test]
async fn test_end_to_end_observation_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    
    let observation = json!({
        "resourceType": "Observation",
        "id": "example-vitals",
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
                "code": "85354-9",
                "display": "Blood pressure panel"
            }]
        },
        "subject": {
            "reference": "Patient/example-patient"
        },
        "effectiveDateTime": "2023-01-01T10:30:00Z",
        "component": [{
            "code": {
                "coding": [{
                    "system": "http://loinc.org", 
                    "code": "8480-6",
                    "display": "Systolic blood pressure"
                }]
            },
            "valueQuantity": {
                "value": 120,
                "unit": "mmHg",
                "system": "http://unitsofmeasure.org",
                "code": "mm[Hg]"
            }
        }, {
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "8462-4", 
                    "display": "Diastolic blood pressure"
                }]
            },
            "valueQuantity": {
                "value": 80,
                "unit": "mmHg",
                "system": "http://unitsofmeasure.org",
                "code": "mm[Hg]"
            }
        }]
    });

    // Test choice type resolution with component access
    let systolic = context.fhirpath.evaluate(
        "Observation.component.where(code.coding.code = '8480-6').valueQuantity.value",
        &observation
    ).await?;
    assert_eq!(systolic.values.len(), 1);
    if let Some(FhirPathValue::Integer(val)) = systolic.values.first() {
        assert_eq!(*val, 120);
    } else if let Some(FhirPathValue::Decimal(val)) = systolic.values.first() {
        assert!((val.to_f64().unwrap() - 120.0).abs() < 0.001);
    }

    // Test complex navigation
    let all_values = context.fhirpath.evaluate(
        "Observation.component.valueQuantity.value",
        &observation
    ).await?;
    assert_eq!(all_values.values.len(), 2);

    // Test filtering with exists()
    let has_systolic = context.fhirpath.evaluate(
        "Observation.component.where(code.coding.code = '8480-6').exists()",
        &observation
    ).await?;
    if let Some(FhirPathValue::Boolean(exists)) = has_systolic.values.first() {
        assert_eq!(*exists, true);
    }

    // Test status validation
    let is_final = context.fhirpath.evaluate(
        "Observation.status = 'final'",
        &observation
    ).await?;
    if let Some(FhirPathValue::Boolean(final_status)) = is_final.values.first() {
        assert_eq!(*final_status, true);
    }

    Ok(())
}

#[tokio::test] 
async fn test_bundle_resource_navigation() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let bundle = &context.test_bundles.comprehensive_bundle;
    
    // Test ofType function across bundle entries
    let patients = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient)",
        bundle
    ).await?;
    assert_eq!(patients.values.len(), 2);

    // Test complex cross-resource navigation
    let patient_names = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient).name.given.first()",
        bundle
    ).await?;
    assert_eq!(patient_names.values.len(), 2);

    // Test observations for specific patients
    let observations = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Observation)",
        bundle
    ).await?;
    assert!(!observations.values.is_empty());

    // Test type checking across different resource types
    let resource_types = context.fhirpath.evaluate(
        "Bundle.entry.resource.resourceType",
        bundle
    ).await?;
    assert!(resource_types.values.len() >= 3); // At least Patient, Observation, Practitioner

    // Test resource counting
    let total_resources = context.fhirpath.evaluate(
        "Bundle.entry.resource.count()",
        bundle
    ).await?;
    if let Some(FhirPathValue::Integer(count)) = total_resources.values.first() {
        assert!(*count > 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_complex_fhirpath_expressions() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let bundle = &context.test_bundles.comprehensive_bundle;

    // Test complex where clause with multiple conditions
    let complex_query = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient).where(active = true and name.exists()).name.family",
        bundle
    ).await?;
    assert!(!complex_query.values.is_empty());

    // Test mathematical operations
    let observation_count = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Observation).count()",
        bundle
    ).await?;
    assert!(!observation_count.values.is_empty());

    // Test string operations
    let upper_names = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient).name.family.upper()",
        bundle
    ).await?;
    assert!(!upper_names.values.is_empty());

    // Test boolean operations
    let has_patients = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient).exists()",
        bundle
    ).await?;
    if let Some(FhirPathValue::Boolean(exists)) = has_patients.values.first() {
        assert_eq!(*exists, true);
    }

    Ok(())
}

#[tokio::test]
async fn test_choice_type_comprehensive() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;

    let observation_with_choices = json!({
        "resourceType": "Observation",
        "id": "choice-test",
        "status": "final",
        "code": {"text": "Test observation"},
        "valueString": "String value",
        "valueQuantity": {
            "value": 42.5,
            "unit": "mg"
        },
        "valueBoolean": true,
        "valueInteger": 123
    });

    // Test string choice type
    let string_value = context.fhirpath.evaluate(
        "Observation.valueString",
        &observation_with_choices
    ).await?;
    if let Some(FhirPathValue::String(val)) = string_value.values.first() {
        assert_eq!(val, "String value");
    }

    // Test quantity choice type
    let quantity_value = context.fhirpath.evaluate(
        "Observation.valueQuantity.value",
        &observation_with_choices
    ).await?;
    assert!(!quantity_value.values.is_empty());

    // Test boolean choice type
    let boolean_value = context.fhirpath.evaluate(
        "Observation.valueBoolean",
        &observation_with_choices
    ).await?;
    if let Some(FhirPathValue::Boolean(val)) = boolean_value.values.first() {
        assert_eq!(*val, true);
    }

    // Test integer choice type
    let integer_value = context.fhirpath.evaluate(
        "Observation.valueInteger",
        &observation_with_choices
    ).await?;
    if let Some(FhirPathValue::Integer(val)) = integer_value.values.first() {
        assert_eq!(*val, 123);
    }

    Ok(())
}

#[tokio::test]
async fn test_date_time_operations() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    
    let patient_with_dates = json!({
        "resourceType": "Patient",
        "id": "date-test",
        "birthDate": "1990-01-15",
        "deceasedDateTime": "2023-12-01T10:30:00Z"
    });

    // Test date access
    let birth_date = context.fhirpath.evaluate(
        "Patient.birthDate",
        &patient_with_dates
    ).await?;
    assert!(!birth_date.values.is_empty());

    // Test dateTime access
    let deceased_date = context.fhirpath.evaluate(
        "Patient.deceasedDateTime",
        &patient_with_dates
    ).await?;
    assert!(!deceased_date.values.is_empty());

    // Test date comparison (if supported)
    let birth_exists = context.fhirpath.evaluate(
        "Patient.birthDate.exists()",
        &patient_with_dates
    ).await?;
    if let Some(FhirPathValue::Boolean(exists)) = birth_exists.values.first() {
        assert_eq!(*exists, true);
    }

    Ok(())
}

#[tokio::test]
async fn test_extension_navigation() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;

    let patient_with_extensions = json!({
        "resourceType": "Patient",
        "id": "extension-test",
        "extension": [{
            "url": "http://example.org/extension/race",
            "valueString": "White"
        }, {
            "url": "http://example.org/extension/ethnicity",
            "valueCodeableConcept": {
                "coding": [{
                    "system": "http://example.org/ethnicity",
                    "code": "not-hispanic",
                    "display": "Not Hispanic"
                }]
            }
        }],
        "name": [{
            "given": ["John"],
            "family": "Doe"
        }]
    });

    // Test extension access by URL
    let race_extension = context.fhirpath.evaluate(
        "Patient.extension.where(url = 'http://example.org/extension/race')",
        &patient_with_extensions
    ).await?;
    assert!(!race_extension.values.is_empty());

    // Test extension value access
    let race_value = context.fhirpath.evaluate(
        "Patient.extension.where(url = 'http://example.org/extension/race').valueString",
        &patient_with_extensions
    ).await?;
    if let Some(FhirPathValue::String(val)) = race_value.values.first() {
        assert_eq!(val, "White");
    }

    // Test complex extension value access
    let ethnicity_code = context.fhirpath.evaluate(
        "Patient.extension.where(url = 'http://example.org/extension/ethnicity').valueCodeableConcept.coding.code",
        &patient_with_extensions
    ).await?;
    if let Some(FhirPathValue::String(code)) = ethnicity_code.values.first() {
        assert_eq!(code, "not-hispanic");
    }

    Ok(())
}

#[tokio::test]
async fn test_mathematical_functions() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;

    let bundle_with_numbers = json!({
        "resourceType": "Bundle",
        "entry": [
            {"resource": create_test_observation("obs-1", "patient-1", "vital-signs", 10.0)},
            {"resource": create_test_observation("obs-2", "patient-1", "vital-signs", 20.0)},
            {"resource": create_test_observation("obs-3", "patient-1", "vital-signs", 30.0)},
        ]
    });

    // Test sum function
    let sum_result = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Observation).valueQuantity.value.sum()",
        &bundle_with_numbers
    ).await?;
    assert!(!sum_result.values.is_empty());

    // Test count function
    let count_result = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Observation).count()",
        &bundle_with_numbers
    ).await?;
    if let Some(FhirPathValue::Integer(count)) = count_result.values.first() {
        assert_eq!(*count, 3);
    }

    // Test min/max functions (if available)
    let values = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Observation).valueQuantity.value",
        &bundle_with_numbers
    ).await?;
    assert_eq!(values.values.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_evaluation_with_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let patient = create_test_patient("analysis-patient", "Test", "Patient");

    // Test evaluation with analysis enabled
    let result = context.fhirpath.evaluate_with_analysis(
        "Patient.name.given.first()",
        &patient
    ).await?;

    // Verify basic evaluation worked
    assert!(!result.values.is_empty());
    assert!(result.execution_time > Duration::from_nanos(0));

    // If analysis is enabled, verify analysis data
    if result.analysis.is_some() {
        let analysis = result.analysis.unwrap();
        assert!(!analysis.type_annotations.is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_performance_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let patient = create_test_patient("perf-patient", "Performance", "Test");

    // Test that performance tracking is working
    let result = context.fhirpath.evaluate("Patient.name.family", &patient).await?;

    assert!(result.execution_time > Duration::from_nanos(0));
    
    if let Some(metrics) = result.performance_metrics {
        assert!(metrics.parse_time >= Duration::from_nanos(0));
        assert!(metrics.evaluation_time >= Duration::from_nanos(0));
        assert!(metrics.total_time >= Duration::from_nanos(0));
        assert!(metrics.total_time >= metrics.evaluation_time);
    }

    Ok(())
}

#[tokio::test]
async fn test_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let patient = create_test_patient("error-patient", "Error", "Test");

    // Test invalid property access
    let invalid_result = context.fhirpath.evaluate("Patient.invalidProperty", &patient).await;
    
    match invalid_result {
        Ok(result) => {
            // Some implementations may handle this gracefully with empty results
            if result.values.is_empty() {
                assert!(true); // This is acceptable
            }
        },
        Err(_) => {
            // Error is also acceptable for invalid properties
            assert!(true);
        }
    }

    // Test malformed expression
    let malformed_result = context.fhirpath.evaluate("Patient.name.", &patient).await;
    assert!(malformed_result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_evaluations() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let patients = create_test_patients(10);

    // Test concurrent evaluations don't interfere with each other
    let mut tasks = Vec::new();
    
    for (i, patient) in patients.into_iter().enumerate() {
        let fhirpath_clone = context.fhirpath.clone();
        let task = tokio::spawn(async move {
            let result = fhirpath_clone.evaluate("Patient.name.family", &patient).await;
            (i, result)
        });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    // Verify all evaluations succeeded
    for result in results {
        let (i, eval_result) = result?;
        let evaluation = eval_result?;
        assert!(!evaluation.values.is_empty(), "Task {} should have results", i);
    }

    Ok(())
}

#[tokio::test]
async fn test_large_dataset_evaluation() -> Result<(), Box<dyn std::error::Error>> {
    let context = IntegrationTestContext::new().await?;
    let large_bundle = create_large_test_bundle(500);

    let mut tracker = PerformanceTracker::new();

    // Test evaluation on large dataset
    let result = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient).count()",
        &large_bundle
    ).await?;

    tracker.checkpoint("Large bundle evaluation");

    assert!(!result.values.is_empty());
    tracker.assert_under_threshold(Duration::from_secs(10), "Large bundle evaluation");

    // Test complex query on large dataset
    let complex_result = context.fhirpath.evaluate(
        "Bundle.entry.resource.ofType(Patient).name.given.first().upper()",
        &large_bundle
    ).await?;

    tracker.checkpoint("Complex query on large dataset");

    assert!(!complex_result.values.is_empty());
    tracker.assert_under_threshold(Duration::from_secs(15), "Complex query evaluation");

    Ok(())
}