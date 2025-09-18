//! Integration tests for FHIR environment variables
//!
//! Tests the complete integration of environment variables with the FHIRPath evaluator
//! according to FHIR specification section 2.1.9.1.7 Environment variables.

use octofhir_fhirpath::{
    Collection, FhirPathEngine, FhirPathValue, EnvironmentVariables, EnvironmentVariablesBuilder,
    EvaluationContext, create_function_registry,
};
use octofhir_fhir_model::EmptyModelProvider;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::test]
async fn test_default_environment_variables() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);
    let engine = FhirPathEngine::new(registry, model_provider.clone()).await.unwrap();

    // Create a simple patient resource
    let patient = serde_json::json!({
        "resourceType": "Patient",
        "id": "test-patient",
        "name": [{
            "family": "Doe",
            "given": ["John"]
        }]
    });

    let input = Collection::single(FhirPathValue::resource(patient));
    let context = EvaluationContext::new(
        input,
        model_provider,
        None,
        None
    ).await;

    // Test %sct variable
    let result = engine.evaluate("%sct", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://snomed.info/sct");
    } else {
        panic!("Expected string value for %sct");
    }

    // Test %loinc variable
    let result = engine.evaluate("%loinc", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://loinc.org");
    } else {
        panic!("Expected string value for %loinc");
    }
}

#[tokio::test]
async fn test_custom_environment_variables() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);

    // Create custom environment variables
    let env_vars = EnvironmentVariablesBuilder::new()
        .sct_url("http://custom.snomed.org")
        .loinc_url("http://custom.loinc.org")
        .value_set("observation-vitalsignresult", "http://hl7.org/fhir/ValueSet/observation-vitalsignresult")
        .extension("patient-birthPlace", "http://hl7.org/fhir/StructureDefinition/patient-birthPlace")
        .custom_variable("%us-zip", FhirPathValue::string("[0-9]{5}(-[0-9]{4}){0,1}".to_string()))
        .build();

    let patient = serde_json::json!({
        "resourceType": "Patient",
        "id": "test-patient"
    });

    let input = Collection::single(FhirPathValue::resource(patient));
    let context = EvaluationContext::new_with_environment(
        input,
        model_provider.clone(),
        None,
        None,
        Arc::new(env_vars)
    ).await;

    let engine = FhirPathEngine::new(registry, model_provider).await.unwrap();

    // Test custom %sct
    let result = engine.evaluate("%sct", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://custom.snomed.org");
    } else {
        panic!("Expected custom string value for %sct");
    }

    // Test custom %loinc
    let result = engine.evaluate("%loinc", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://custom.loinc.org");
    } else {
        panic!("Expected custom string value for %loinc");
    }

    // Test value set variable
    let result = engine.evaluate("%vs-observation-vitalsignresult", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://hl7.org/fhir/ValueSet/observation-vitalsignresult");
    } else {
        panic!("Expected string value for value set variable");
    }

    // Test extension variable
    let result = engine.evaluate("%ext-patient-birthPlace", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://hl7.org/fhir/StructureDefinition/patient-birthPlace");
    } else {
        panic!("Expected string value for extension variable");
    }

    // Test custom variable
    let result = engine.evaluate("%us-zip", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(pattern, _, _)) = result.value.iter().next() {
        assert_eq!(pattern, "[0-9]{5}(-[0-9]{4}){0,1}");
    } else {
        panic!("Expected string value for custom variable");
    }
}

#[tokio::test]
async fn test_nonexistent_environment_variables() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);
    let engine = FhirPathEngine::new(registry, model_provider.clone()).await.unwrap();

    let patient = serde_json::json!({
        "resourceType": "Patient",
        "id": "test-patient"
    });

    let input = Collection::single(FhirPathValue::resource(patient));
    let context = EvaluationContext::new(
        input,
        model_provider,
        None,
        None
    ).await;

    // Test nonexistent value set variable
    let result = engine.evaluate("%vs-nonexistent", &context).await.unwrap();
    assert!(result.value.is_empty(), "Nonexistent value set variable should return empty");

    // Test nonexistent extension variable
    let result = engine.evaluate("%ext-nonexistent", &context).await.unwrap();
    assert!(result.value.is_empty(), "Nonexistent extension variable should return empty");

    // Test nonexistent custom variable
    let result = engine.evaluate("%nonexistent", &context).await.unwrap();
    assert!(result.value.is_empty(), "Nonexistent custom variable should return empty");
}

#[tokio::test]
async fn test_environment_variables_in_expressions() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);

    // Create environment variables with a value set
    let env_vars = EnvironmentVariablesBuilder::new()
        .value_set("observation-vitalsignresult", "http://hl7.org/fhir/ValueSet/observation-vitalsignresult")
        .build();

    let observation = serde_json::json!({
        "resourceType": "Observation",
        "code": {
            "coding": [{
                "system": "http://loinc.org",
                "code": "8867-4",
                "display": "Heart rate"
            }]
        },
        "component": [
            {
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "8867-4"
                    }]
                }
            },
            {
                "code": {
                    "coding": [{
                        "system": "http://snomed.info/sct",
                        "code": "364075005"
                    }]
                }
            }
        ]
    });

    let input = Collection::single(FhirPathValue::resource(observation));
    let context = EvaluationContext::new_with_environment(
        input,
        model_provider.clone(),
        None,
        None,
        Arc::new(env_vars)
    ).await;

    let engine = FhirPathEngine::new(registry, model_provider).await.unwrap();

    // Test using environment variable in string concatenation
    let result = engine.evaluate("'ValueSet URL: ' + %vs-observation-vitalsignresult", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(concat_result, _, _)) = result.value.iter().next() {
        assert_eq!(concat_result, "ValueSet URL: http://hl7.org/fhir/ValueSet/observation-vitalsignresult");
    } else {
        panic!("Expected concatenated string result");
    }

    // Test using terminology URLs in expressions
    let result = engine.evaluate("%sct", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(sct_url, _, _)) = result.value.iter().next() {
        assert_eq!(sct_url, "http://snomed.info/sct");
    } else {
        panic!("Expected SNOMED CT URL");
    }

    let result = engine.evaluate("%loinc", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(loinc_url, _, _)) = result.value.iter().next() {
        assert_eq!(loinc_url, "http://loinc.org");
    } else {
        panic!("Expected LOINC URL");
    }
}

#[tokio::test]
async fn test_environment_variables_persistence_across_contexts() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);

    let env_vars = EnvironmentVariablesBuilder::new()
        .value_set("test-vs", "http://example.org/ValueSet/test")
        .build();

    let patient = serde_json::json!({
        "resourceType": "Patient",
        "id": "test-patient",
        "name": [{
            "family": "Doe"
        }]
    });

    let input = Collection::single(FhirPathValue::resource(patient));
    let context = EvaluationContext::new_with_environment(
        input,
        model_provider.clone(),
        None,
        None,
        Arc::new(env_vars)
    ).await;

    let engine = FhirPathEngine::new(registry, model_provider).await.unwrap();

    // Test that environment variables are available in the main context
    let result = engine.evaluate("%vs-test-vs", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(url, _, _)) = result.value.iter().next() {
        assert_eq!(url, "http://example.org/ValueSet/test");
    } else {
        panic!("Expected value set URL in main context");
    }

    // Test that environment variables are available in nested contexts (e.g., in where() function)
    let result = engine.evaluate("name.where(%vs-test-vs = 'http://example.org/ValueSet/test')", &context).await.unwrap();
    // This should not error and should evaluate the condition properly
    // The exact result depends on the where() function implementation
}

#[tokio::test]
async fn test_environment_variables_builder_methods() {
    let env_vars = EnvironmentVariablesBuilder::new()
        .sct_url("http://test.snomed.org")
        .loinc_url("http://test.loinc.org")
        .value_set("vs1", "http://example.org/vs1")
        .value_set("vs2", "http://example.org/vs2")
        .extension("ext1", "http://example.org/ext1")
        .extension("ext2", "http://example.org/ext2")
        .custom_variable("%custom1", FhirPathValue::integer(123))
        .custom_variable("%custom2", FhirPathValue::boolean(true))
        .build();

    // Test basic variables
    assert_eq!(
        env_vars.get_variable("%sct"),
        Some(FhirPathValue::string("http://test.snomed.org".to_string()))
    );
    assert_eq!(
        env_vars.get_variable("%loinc"),
        Some(FhirPathValue::string("http://test.loinc.org".to_string()))
    );

    // Test value sets
    assert_eq!(
        env_vars.get_variable("%vs-vs1"),
        Some(FhirPathValue::string("http://example.org/vs1".to_string()))
    );
    assert_eq!(
        env_vars.get_variable("%vs-vs2"),
        Some(FhirPathValue::string("http://example.org/vs2".to_string()))
    );

    // Test extensions
    assert_eq!(
        env_vars.get_variable("%ext-ext1"),
        Some(FhirPathValue::string("http://example.org/ext1".to_string()))
    );
    assert_eq!(
        env_vars.get_variable("%ext-ext2"),
        Some(FhirPathValue::string("http://example.org/ext2".to_string()))
    );

    // Test custom variables
    assert_eq!(
        env_vars.get_variable("%custom1"),
        Some(FhirPathValue::integer(123))
    );
    assert_eq!(
        env_vars.get_variable("%custom2"),
        Some(FhirPathValue::boolean(true))
    );

    // Test variable listing
    let vars = env_vars.list_variables();
    assert!(vars.contains(&"%sct".to_string()));
    assert!(vars.contains(&"%loinc".to_string()));
    assert!(vars.contains(&"%vs-vs1".to_string()));
    assert!(vars.contains(&"%vs-vs2".to_string()));
    assert!(vars.contains(&"%ext-ext1".to_string()));
    assert!(vars.contains(&"%ext-ext2".to_string()));
    assert!(vars.contains(&"%custom1".to_string()));
    assert!(vars.contains(&"%custom2".to_string()));
}

#[tokio::test]
async fn test_resource_environment_variable() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);
    let engine = FhirPathEngine::new(registry, model_provider.clone()).await.unwrap();

    let patient = serde_json::json!({
        "resourceType": "Patient",
        "id": "test-patient",
        "name": [{
            "family": "Doe",
            "given": ["John"]
        }]
    });

    let input = Collection::single(FhirPathValue::resource(patient.clone()));
    let context = EvaluationContext::new(
        input,
        model_provider,
        None,
        None
    ).await;

    // Test %resource variable - should return the original resource
    let result = engine.evaluate("%resource", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::Resource(resource_json, _, _)) = result.value.iter().next() {
        assert_eq!(resource_json.get("resourceType").unwrap().as_str().unwrap(), "Patient");
        assert_eq!(resource_json.get("id").unwrap().as_str().unwrap(), "test-patient");
    } else {
        panic!("Expected resource value for %resource");
    }

    // Test %resource.id to verify we can navigate from %resource
    let result = engine.evaluate("%resource.id", &context).await.unwrap();
    assert_eq!(result.value.len(), 1);
    if let Some(FhirPathValue::String(id, _, _)) = result.value.iter().next() {
        assert_eq!(id, "test-patient");
    } else {
        panic!("Expected string value for %resource.id");
    }
}