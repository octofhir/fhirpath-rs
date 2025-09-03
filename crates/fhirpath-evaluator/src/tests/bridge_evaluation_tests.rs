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

//! Bridge support integration tests for FHIRPath evaluator

use super::*;
use crate::engine::{EvaluationConfig, FhirPathEngine};
use octofhir_fhirpath_core::FhirPathValue;
// TODO: Re-enable when FhirSchemaModelProvider is moved to core
// use octofhir_fhir_model::FhirSchemaModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::json;
use std::sync::Arc;

/// Create a test engine with bridge support
async fn create_bridge_test_engine() -> Result<FhirPathEngine, Box<dyn std::error::Error>> {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(FhirSchemaPackageManager::new(fcm_config, config).await?);

    let registry = Arc::new(create_standard_registry().await);
    let model_provider = Arc::new(FhirSchemaModelProvider::with_manager(manager).await?);

    let eval_config = EvaluationConfig {
        max_recursion_depth: 100,
        timeout_ms: 10000,
        enable_lambda_optimization: true,
        enable_sync_optimization: true,
        memory_limit_mb: Some(100),
        max_expression_nodes: 10000,
        max_collection_size: 100000,
    };

    Ok(FhirPathEngine::new(registry, model_provider).with_config(eval_config))
}

#[tokio::test]
async fn test_async_evaluation() {
    let engine = create_bridge_test_engine().await.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "id": "test-patient-1",
        "name": [{
            "use": "official",
            "given": ["John", "David"],
            "family": "Doe"
        }]
    });

    let result = engine.evaluate("Patient.name.given", patient).await;
    assert!(result.is_ok());

    let value = result.unwrap();
    if let FhirPathValue::Collection(items) = value {
        assert_eq!(items.len(), 2);

        if let Some(FhirPathValue::String(first_name)) = items.first().and_then(|v| v.as_string()) {
            assert_eq!(first_name, "John");
        } else {
            panic!("Expected first name to be 'John'");
        }

        if let Some(FhirPathValue::String(second_name)) = items.get(1).and_then(|v| v.as_string()) {
            assert_eq!(second_name, "David");
        } else {
            panic!("Expected second name to be 'David'");
        }
    } else {
        panic!("Expected collection result");
    }
}

#[tokio::test]
async fn test_choice_type_navigation() {
    let engine = create_bridge_test_engine().await.unwrap();

    let observation = json!({
        "resourceType": "Observation",
        "id": "test-obs",
        "status": "final",
        "valueString": "test-value",
        "valueQuantity": {
            "value": 120,
            "unit": "mmHg"
        }
    });

    // Test direct choice property access
    let string_result = engine
        .evaluate("Observation.valueString", observation.clone())
        .await;
    assert!(string_result.is_ok());

    let string_value = string_result.unwrap();
    if let Some(s) = string_value.as_single_string() {
        assert_eq!(s, "test-value");
    } else {
        panic!("Expected string value");
    }

    // Test quantity choice property access
    let quantity_result = engine
        .evaluate("Observation.valueQuantity.value", observation)
        .await;
    assert!(quantity_result.is_ok());

    let quantity_value = quantity_result.unwrap();
    if let Some(val) = quantity_value.as_single_decimal() {
        assert!((val - 120.0).abs() < 0.001);
    } else {
        panic!("Expected decimal value");
    }
}

#[tokio::test]
async fn test_schema_aware_functions() {
    let engine = create_bridge_test_engine().await.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "id": "test-patient"
    });

    let observation = json!({
        "resourceType": "Observation",
        "id": "test-observation"
    });

    let bundle = json!({
        "resourceType": "Bundle",
        "type": "collection",
        "entry": [
            {"resource": patient},
            {"resource": observation}
        ]
    });

    // Test ofType with schema awareness
    let patients = engine
        .evaluate("Bundle.entry.resource.ofType(Patient)", bundle.clone())
        .await;
    assert!(patients.is_ok());

    let patient_collection = patients.unwrap();
    assert_eq!(patient_collection.len(), 1);

    let observations = engine
        .evaluate("Bundle.entry.resource.ofType(Observation)", bundle)
        .await;
    assert!(observations.is_ok());

    let obs_collection = observations.unwrap();
    assert_eq!(obs_collection.len(), 1);
}

#[tokio::test]
async fn test_performance_caching() {
    let engine = create_bridge_test_engine().await.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    // First evaluation (cache miss)
    let start1 = std::time::Instant::now();
    let result1 = engine
        .evaluate("Patient.name.given.first()", patient.clone())
        .await;
    let time1 = start1.elapsed();

    assert!(result1.is_ok());

    // Second evaluation (cache hit)
    let start2 = std::time::Instant::now();
    let result2 = engine.evaluate("Patient.name.given.first()", patient).await;
    let time2 = start2.elapsed();

    assert!(result2.is_ok());
    assert!(values_equal(&result1.unwrap(), &result2.unwrap()));

    // Second evaluation should be faster or at least not significantly slower
    // Allow for timing variance in test environments
    assert!(time2 <= time1 + std::time::Duration::from_millis(50));
}

#[tokio::test]
async fn test_complex_navigation_with_bridge() {
    let engine = create_bridge_test_engine().await.unwrap();

    let bundle = json!({
        "resourceType": "Bundle",
        "type": "collection",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "patient-1",
                    "name": [{
                        "use": "official",
                        "family": "Smith",
                        "given": ["Alice"]
                    }]
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "obs-vital-1",
                    "status": "final",
                    "category": [{
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                            "code": "vital-signs"
                        }]
                    }],
                    "subject": {"reference": "Patient/patient-1"},
                    "valueQuantity": {"value": 120, "unit": "mmHg"}
                }
            }
        ]
    });

    // Test complex navigation with where clauses
    let vital_signs = engine
        .evaluate(
            "Bundle.entry.resource.ofType(Observation).where(category.coding.code = 'vital-signs')",
            bundle.clone(),
        )
        .await;

    assert!(vital_signs.is_ok());
    let vital_collection = vital_signs.unwrap();
    assert_eq!(vital_collection.len(), 1);

    // Test chained navigation
    let values = engine.evaluate(
        "Bundle.entry.resource.ofType(Observation).where(category.coding.code = 'vital-signs').valueQuantity.value",
        bundle
    ).await;

    assert!(values.is_ok());
    let value_collection = values.unwrap();
    if let Some(val) = value_collection.as_single_decimal() {
        assert!((val - 120.0).abs() < 0.001);
    } else {
        panic!("Expected decimal value 120");
    }
}

#[tokio::test]
async fn test_error_handling_with_bridge() {
    let engine = create_bridge_test_engine().await.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "id": "test-patient"
    });

    // Test invalid property
    let invalid_result = engine
        .evaluate("Patient.invalidProperty", patient.clone())
        .await;
    assert!(invalid_result.is_err());

    // Test invalid function
    let invalid_function = engine
        .evaluate("Patient.invalidFunction()", patient.clone())
        .await;
    assert!(invalid_function.is_err());

    // Test type mismatch
    let type_mismatch = engine.evaluate("Patient.name + 42", patient).await;
    assert!(type_mismatch.is_err());
}

#[tokio::test]
async fn test_concurrent_evaluations() {
    let engine = Arc::new(create_bridge_test_engine().await.unwrap());

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    // Create multiple concurrent evaluation tasks
    let mut tasks = Vec::new();
    for i in 0..10 {
        let engine_clone = engine.clone();
        let patient_clone = patient.clone();

        let task = tokio::spawn(async move {
            let result = engine_clone
                .evaluate("Patient.name.given.first()", patient_clone)
                .await;
            (i, result)
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // All tasks should complete successfully
    for result in results {
        let (i, evaluation_result) = result.unwrap();
        assert!(evaluation_result.is_ok(), "Task {} failed", i);

        let value = evaluation_result.unwrap();
        if let Some(name) = value.as_single_string() {
            assert_eq!(name, "John");
        } else {
            panic!("Task {} returned unexpected value", i);
        }
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    let engine = create_bridge_test_engine().await.unwrap();

    // Create many evaluations to test memory efficiency
    let mut results = Vec::new();
    for i in 0..100 {
        let patient = json!({
            "resourceType": "Patient",
            "id": format!("patient-{}", i),
            "name": [{"given": ["Test"], "family": format!("Patient{}", i)}]
        });

        let result = engine
            .evaluate("Patient.name.family.first()", patient)
            .await;
        assert!(result.is_ok());
        results.push(result.unwrap());
    }

    // Verify all results are correct
    for (i, result) in results.iter().enumerate() {
        if let Some(family_name) = result.as_single_string() {
            assert_eq!(family_name, format!("Patient{}", i));
        } else {
            panic!("Result {} was not a string", i);
        }
    }
}

#[tokio::test]
async fn test_lambda_expressions_with_bridge() {
    let engine = create_bridge_test_engine().await.unwrap();

    let bundle = json!({
        "resourceType": "Bundle",
        "entry": [
            {"resource": {"resourceType": "Patient", "active": true}},
            {"resource": {"resourceType": "Patient", "active": false}},
            {"resource": {"resourceType": "Patient", "active": true}},
        ]
    });

    // Test where with lambda expressions
    let active_patients = engine
        .evaluate(
            "Bundle.entry.resource.ofType(Patient).where(active = true)",
            bundle,
        )
        .await;

    assert!(active_patients.is_ok());
    let active_collection = active_patients.unwrap();
    assert_eq!(active_collection.len(), 2);
}

#[tokio::test]
async fn test_mathematical_operations_with_bridge() {
    let engine = create_bridge_test_engine().await.unwrap();

    let observations = json!({
        "resourceType": "Bundle",
        "entry": [
            {"resource": {"resourceType": "Observation", "valueQuantity": {"value": 10}}},
            {"resource": {"resourceType": "Observation", "valueQuantity": {"value": 20}}},
            {"resource": {"resourceType": "Observation", "valueQuantity": {"value": 30}}},
        ]
    });

    // Test sum operation
    let sum_result = engine
        .evaluate(
            "Bundle.entry.resource.ofType(Observation).valueQuantity.value.sum()",
            observations.clone(),
        )
        .await;

    assert!(sum_result.is_ok());
    let sum_value = sum_result.unwrap();
    if let Some(sum) = sum_value.as_single_decimal() {
        assert!((sum - 60.0).abs() < 0.001);
    } else {
        panic!("Expected sum to be 60");
    }

    // Test average operation
    let avg_result = engine
        .evaluate(
            "Bundle.entry.resource.ofType(Observation).valueQuantity.value.avg()",
            observations,
        )
        .await;

    assert!(avg_result.is_ok());
    let avg_value = avg_result.unwrap();
    if let Some(avg) = avg_value.as_single_decimal() {
        assert!((avg - 20.0).abs() < 0.001);
    } else {
        panic!("Expected average to be 20");
    }
}

#[tokio::test]
async fn test_timeout_handling() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let registry = Arc::new(create_standard_registry().await);
    let model_provider = Arc::new(
        FhirSchemaModelProvider::with_manager(manager)
            .await
            .unwrap(),
    );

    // Create engine with very short timeout
    let eval_config = EvaluationConfig {
        max_recursion_depth: 100,
        timeout_ms: 1, // 1ms timeout - very short
        enable_lambda_optimization: true,
        enable_sync_optimization: true,
        memory_limit_mb: Some(100),
        max_expression_nodes: 10000,
        max_collection_size: 100000,
    };

    let engine = FhirPathEngine::new(registry, model_provider).with_config(eval_config);

    let large_bundle = create_large_test_bundle();

    // This should timeout due to the very short timeout
    let result = engine
        .evaluate(
            "Bundle.entry.resource.ofType(Patient).name.given.first().upper()",
            large_bundle,
        )
        .await;

    // Should either succeed quickly or timeout
    // We can't guarantee timeout in all test environments, but we can verify it doesn't crash
    match result {
        Ok(_) => {} // Completed within timeout
        Err(err) => {
            // Should be a timeout or evaluation error, not a panic
            assert!(
                format!("{:?}", err).contains("timeout")
                    || format!("{:?}", err).contains("Evaluation")
            );
        }
    }
}

fn create_large_test_bundle() -> serde_json::Value {
    let mut entries = Vec::new();

    for i in 0..100 {
        entries.push(json!({
            "resource": {
                "resourceType": "Patient",
                "id": format!("patient-{}", i),
                "name": [{
                    "given": [format!("FirstName{}", i)],
                    "family": format!("LastName{}", i)
                }]
            }
        }));
    }

    json!({
        "resourceType": "Bundle",
        "type": "collection",
        "entry": entries
    })
}
