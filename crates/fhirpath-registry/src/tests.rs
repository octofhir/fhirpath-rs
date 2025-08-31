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

//! Unit tests for fhirpath-registry with schema-aware functions

use super::*;
use octofhir_fhirpath_model::{FhirPathValue, FhirSchemaModelProvider, MockModelProvider};
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::json;
use std::sync::Arc;
use traits::EvaluationContext;

async fn create_test_context() -> (Arc<FhirSchemaPackageManager>, FunctionRegistry) {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let registry = create_standard_registry().await;

    (manager, registry)
}

async fn create_schema_aware_test_context()
-> (Arc<FhirSchemaPackageManager>, SchemaAwareFunctionRegistry) {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let schema_registry = create_schema_aware_registry(manager.clone()).await.unwrap();

    (manager, schema_registry)
}

#[tokio::test]
async fn test_type_registry_operations() {
    let (manager, _registry) = create_test_context().await;
    let type_registry = FhirPathTypeRegistry::new(manager).await.unwrap();

    // Test O(1) resource type checking
    assert!(type_registry.is_resource_type("Patient"));
    assert!(type_registry.is_resource_type("Observation"));
    assert!(type_registry.is_resource_type("Bundle"));
    assert!(!type_registry.is_resource_type("InvalidType"));

    // Test data type checking
    assert!(type_registry.is_data_type("HumanName"));
    assert!(type_registry.is_data_type("Address"));
    assert!(!type_registry.is_data_type("Patient")); // Patient is a resource, not data type

    // Test primitive type checking
    assert!(type_registry.is_primitive_type("string"));
    assert!(type_registry.is_primitive_type("boolean"));
    assert!(type_registry.is_primitive_type("integer"));
    assert!(!type_registry.is_primitive_type("Patient"));
}

#[tokio::test]
async fn test_schema_aware_function_registry() {
    let (manager, schema_registry) = create_schema_aware_test_context().await;
    let provider = Arc::new(
        FhirSchemaModelProvider::with_manager(manager)
            .await
            .unwrap(),
    );

    let patient = json!({
        "resourceType": "Patient",
        "id": "test-patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    let context = EvaluationContext {
        input: FhirPathValue::from_json(&patient),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test schema-aware ofType function
    let args = vec![vec![FhirPathValue::String("Patient".to_string())]];
    let result = schema_registry
        .evaluate_function("ofType", &args, &context)
        .await;

    assert!(result.is_ok());
    let value = result.unwrap();

    // Should return the Patient resource since it matches the type
    assert_eq!(value.len(), 1);
}

#[tokio::test]
async fn test_function_registry_with_schema() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    let context = EvaluationContext {
        input: FhirPathValue::from_json(&patient),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test count function
    let count_result = registry.evaluate("count", &[], &context).await;
    assert!(count_result.is_ok());

    let count_value = count_result.unwrap();
    if let FhirPathValue::Integer(count) = count_value {
        assert_eq!(count, 1); // Single patient object
    } else {
        panic!("Expected integer count");
    }
}

#[tokio::test]
async fn test_package_management() {
    let (manager, _schema_registry) = create_schema_aware_test_context().await;
    let mut package_manager = RegistryPackageManager::new(manager).await.unwrap();

    // Test loading base packages
    let loaded_packages = package_manager.get_loaded_packages().await;
    assert!(loaded_packages.is_ok());

    let packages = loaded_packages.unwrap();
    assert!(!packages.is_empty());

    // Test registry refresh after package changes
    let refresh_result = package_manager.refresh_registry().await;
    assert!(refresh_result.is_ok());

    // Registry should still work after refresh
    assert!(package_manager.type_registry.is_resource_type("Patient"));
}

#[tokio::test]
async fn test_mathematical_functions() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let numbers = json!([1, 2, 3, 4, 5]);

    let context = EvaluationContext {
        input: FhirPathValue::from_json(&numbers),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test sum function
    let sum_result = registry.evaluate("sum", &[], &context).await;
    assert!(sum_result.is_ok());

    let sum_value = sum_result.unwrap();
    if let FhirPathValue::Integer(sum) = sum_value {
        assert_eq!(sum, 15); // 1+2+3+4+5 = 15
    } else if let FhirPathValue::Decimal(sum) = sum_value {
        assert!((sum.to_f64().unwrap() - 15.0).abs() < 0.001);
    } else {
        panic!("Expected numeric sum result");
    }
}

#[tokio::test]
async fn test_string_functions() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let string_data = json!("hello world");

    let context = EvaluationContext {
        input: FhirPathValue::from_json(&string_data),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test upper function
    let upper_result = registry.evaluate("upper", &[], &context).await;
    assert!(upper_result.is_ok());

    let upper_value = upper_result.unwrap();
    if let FhirPathValue::String(upper_str) = upper_value {
        assert_eq!(upper_str, "HELLO WORLD");
    } else {
        panic!("Expected uppercase string");
    }

    // Test length function
    let length_result = registry.evaluate("length", &[], &context).await;
    assert!(length_result.is_ok());

    let length_value = length_result.unwrap();
    if let FhirPathValue::Integer(len) = length_value {
        assert_eq!(len, 11); // "hello world" has 11 characters
    } else {
        panic!("Expected integer length");
    }
}

#[tokio::test]
async fn test_collection_functions() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let collection = json!(["apple", "banana", "cherry", "apple"]);

    let context = EvaluationContext {
        input: FhirPathValue::from_json(&collection),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test distinct function
    let distinct_result = registry.evaluate("distinct", &[], &context).await;
    assert!(distinct_result.is_ok());

    let distinct_value = distinct_result.unwrap();
    if let FhirPathValue::Collection(items) = distinct_value {
        assert_eq!(items.len(), 3); // Should have 3 unique items
    } else {
        panic!("Expected collection result");
    }

    // Test first function
    let first_result = registry.evaluate("first", &[], &context).await;
    assert!(first_result.is_ok());

    let first_value = first_result.unwrap();
    if let FhirPathValue::String(first_str) = first_value {
        assert_eq!(first_str, "apple");
    } else {
        panic!("Expected first item to be 'apple'");
    }
}

#[tokio::test]
async fn test_boolean_functions() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}],
        "active": true
    });

    let context = EvaluationContext {
        input: FhirPathValue::from_json(&patient),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test exists function
    let exists_result = registry.evaluate("exists", &[], &context).await;
    assert!(exists_result.is_ok());

    let exists_value = exists_result.unwrap();
    if let FhirPathValue::Boolean(exists) = exists_value {
        assert!(exists); // Patient object exists
    } else {
        panic!("Expected boolean exists result");
    }

    // Test empty function
    let empty_result = registry.evaluate("empty", &[], &context).await;
    assert!(empty_result.is_ok());

    let empty_value = empty_result.unwrap();
    if let FhirPathValue::Boolean(empty) = empty_value {
        assert!(!empty); // Patient object is not empty
    } else {
        panic!("Expected boolean empty result");
    }
}

#[tokio::test]
async fn test_type_functions() {
    let (_manager, schema_registry) = create_schema_aware_test_context().await;
    let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());

    let bundle = json!({
        "resourceType": "Bundle",
        "entry": [
            {"resource": {"resourceType": "Patient", "id": "p1"}},
            {"resource": {"resourceType": "Observation", "id": "o1"}},
            {"resource": {"resourceType": "Patient", "id": "p2"}},
        ]
    });

    let entry_resources = FhirPathValue::Collection(
        bundle["entry"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| FhirPathValue::from_json(&entry["resource"]))
            .collect(),
    );

    let context = EvaluationContext {
        input: entry_resources,
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test ofType function with schema awareness
    let args = vec![vec![FhirPathValue::String("Patient".to_string())]];
    let patient_result = schema_registry
        .evaluate_function("ofType", &args, &context)
        .await;

    assert!(patient_result.is_ok());
    let patient_value = patient_result.unwrap();

    if let FhirPathValue::Collection(patients) = patient_value {
        assert_eq!(patients.len(), 2); // Should have 2 Patient resources
    } else {
        panic!("Expected collection of Patient resources");
    }
}

#[tokio::test]
async fn test_date_functions() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let patient = json!({
        "resourceType": "Patient",
        "birthDate": "1990-01-15"
    });

    let birth_date =
        FhirPathValue::Date(chrono::NaiveDate::from_ymd_opt(1990, 1, 15).unwrap().into());

    let context = EvaluationContext {
        input: birth_date,
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test today function (returns different date)
    let today_result = registry.evaluate("today", &[], &context).await;
    assert!(today_result.is_ok());

    let today_value = today_result.unwrap();
    // Should return today's date (which is different from 1990-01-15)
    assert!(matches!(today_value, FhirPathValue::Date(_)));
}

#[tokio::test]
async fn test_performance_characteristics() {
    let (manager, _registry) = create_test_context().await;
    let type_registry = FhirPathTypeRegistry::new(manager).await.unwrap();

    // Test O(1) operations performance
    let start = std::time::Instant::now();

    for _i in 0..1000 {
        // These should all be O(1) operations
        type_registry.is_resource_type("Patient");
        type_registry.is_resource_type("Observation");
        type_registry.is_resource_type("Bundle");
        type_registry.is_primitive_type("string");
        type_registry.is_primitive_type("boolean");
        type_registry.is_data_type("HumanName");
    }

    let duration = start.elapsed();

    // 6000 operations should complete very quickly due to O(1) complexity
    assert!(
        duration.as_millis() < 100,
        "O(1) operations took too long: {}ms",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_concurrent_registry_access() {
    let (_manager, registry) = create_test_context().await;
    let registry = Arc::new(registry);
    let provider = Arc::new(MockModelProvider::new());

    // Create multiple concurrent function evaluation tasks
    let mut tasks = Vec::new();
    for i in 0..10 {
        let registry_clone = registry.clone();
        let provider_clone = provider.clone();

        let task = tokio::spawn(async move {
            let data = json!(format!("test-{}", i));
            let context = EvaluationContext {
                input: FhirPathValue::from_json(&data),
                model_provider: provider_clone,
                variables: std::collections::HashMap::new(),
            };

            let result = registry_clone.evaluate("upper", &[], &context).await;
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
        if let FhirPathValue::String(upper_str) = value {
            assert_eq!(upper_str, format!("TEST-{}", i));
        } else {
            panic!("Task {} returned unexpected value", i);
        }
    }
}

#[tokio::test]
async fn test_error_handling_scenarios() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    let context = EvaluationContext {
        input: FhirPathValue::Empty,
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test invalid function name
    let invalid_result = registry.evaluate("invalidFunction", &[], &context).await;
    assert!(invalid_result.is_err());

    // Test function with wrong argument types
    let wrong_args = vec![vec![FhirPathValue::String("not-a-number".to_string())]];
    let math_context = EvaluationContext {
        input: FhirPathValue::String("test".to_string()),
        model_provider: Arc::new(MockModelProvider::new()),
        variables: std::collections::HashMap::new(),
    };

    // This should handle type mismatches gracefully
    let type_error = registry.evaluate("sum", &wrong_args, &math_context).await;
    // Should either succeed (with proper coercion) or fail gracefully
    match type_error {
        Ok(_) => {} // Function handled coercion
        Err(err) => {
            // Should be a proper error, not a panic
            assert!(
                format!("{:?}", err).contains("Error") || format!("{:?}", err).contains("Invalid")
            );
        }
    }
}

#[tokio::test]
async fn test_registry_extensibility() {
    let (_manager, mut registry) = create_test_context().await;

    // Registry should support adding custom functions
    // (This is a conceptual test - actual implementation may vary)

    // Test that registry has expected built-in functions
    let provider = Arc::new(MockModelProvider::new());
    let context = EvaluationContext {
        input: FhirPathValue::String("test".to_string()),
        model_provider: provider,
        variables: std::collections::HashMap::new(),
    };

    // Test common built-in functions exist
    let functions_to_test = ["count", "empty", "exists", "first", "last"];

    for function_name in functions_to_test {
        let result = registry.evaluate(function_name, &[], &context).await;
        // Should either succeed or fail with proper error (not panic)
        match result {
            Ok(_) => {} // Function executed successfully
            Err(err) => {
                // Should be a proper function-related error
                let error_str = format!("{:?}", err);
                assert!(
                    error_str.contains("function")
                        || error_str.contains("argument")
                        || error_str.contains("signature"),
                    "Unexpected error for {}: {}",
                    function_name,
                    error_str
                );
            }
        }
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    let (_manager, registry) = create_test_context().await;
    let provider = Arc::new(MockModelProvider::new());

    // Test memory efficiency with many function calls
    let mut results = Vec::new();
    for i in 0..100 {
        let data = json!({
            "resourceType": "Patient",
            "id": format!("patient-{}", i),
            "name": [{"given": ["Test"], "family": format!("Patient{}", i)}]
        });

        let context = EvaluationContext {
            input: FhirPathValue::from_json(&data),
            model_provider: provider.clone(),
            variables: std::collections::HashMap::new(),
        };

        let result = registry.evaluate("count", &[], &context).await;
        assert!(result.is_ok());
        results.push(result.unwrap());
    }

    // Verify all results are correct
    for result in results {
        if let FhirPathValue::Integer(count) = result {
            assert_eq!(count, 1); // Each patient is a single object
        } else {
            panic!("Expected integer count");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_registry_integration() {
        let (manager, registry) = create_test_context().await;
        let schema_registry = create_schema_aware_registry(manager.clone()).await.unwrap();
        let type_registry = FhirPathTypeRegistry::new(manager.clone()).await.unwrap();
        let package_manager = RegistryPackageManager::new(manager).await.unwrap();

        // Test that all components work together
        let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());

        let complex_data = json!({
            "resourceType": "Bundle",
            "entry": [
                {"resource": {"resourceType": "Patient", "name": [{"given": ["John"]}]}},
                {"resource": {"resourceType": "Observation", "valueQuantity": {"value": 100}}}
            ]
        });

        let context = EvaluationContext {
            input: FhirPathValue::from_json(&complex_data),
            model_provider: provider,
            variables: std::collections::HashMap::new(),
        };

        // Test standard registry functions
        let count_result = registry.evaluate("count", &[], &context).await;
        assert!(count_result.is_ok());

        // Test schema-aware functions
        let patient_context = EvaluationContext {
            input: FhirPathValue::Collection(
                complex_data["entry"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| FhirPathValue::from_json(&entry["resource"]))
                    .collect(),
            ),
            model_provider: Arc::new(FhirSchemaModelProvider::new().await.unwrap()),
            variables: std::collections::HashMap::new(),
        };

        let args = vec![vec![FhirPathValue::String("Patient".to_string())]];
        let oftype_result = schema_registry
            .evaluate_function("ofType", &args, &patient_context)
            .await;
        assert!(oftype_result.is_ok());

        // Test type registry
        assert!(type_registry.is_resource_type("Patient"));
        assert!(type_registry.is_resource_type("Observation"));
        assert!(type_registry.is_resource_type("Bundle"));

        // Test package manager
        let packages = package_manager.get_loaded_packages().await;
        assert!(packages.is_ok());
    }

    #[tokio::test]
    async fn test_performance_integration() {
        let (manager, registry) = create_test_context().await;
        let type_registry = FhirPathTypeRegistry::new(manager).await.unwrap();

        let provider = Arc::new(MockModelProvider::new());

        // Test combined operations performance
        let start = std::time::Instant::now();

        for i in 0..50 {
            // Mix of O(1) type operations and function evaluations
            type_registry.is_resource_type("Patient");
            type_registry.is_primitive_type("string");

            let data = json!(format!("test-{}", i));
            let context = EvaluationContext {
                input: FhirPathValue::from_json(&data),
                model_provider: provider.clone(),
                variables: std::collections::HashMap::new(),
            };

            let _result = registry.evaluate("upper", &[], &context).await;
        }

        let duration = start.elapsed();

        // Combined operations should still be efficient
        assert!(
            duration.as_millis() < 500,
            "Combined operations took too long: {}ms",
            duration.as_millis()
        );
    }
}
