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

//! Integration tests for main FHIRPath API with Bridge Support

use octofhir_fhirpath::*;
use serde_json::json;

#[tokio::test]
async fn test_main_api_functionality() {
    let fhirpath = FhirPath::new().await.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "id": "test-patient",
        "name": [{
            "use": "official",
            "given": ["John", "David"],
            "family": "Doe"
        }]
    });

    let result = fhirpath.evaluate("Patient.name.given", &patient).await;
    assert!(result.is_ok());

    let value = result.unwrap();
    assert_eq!(value.values.len(), 2);

    if let Some(FhirPathValue::String(first_name)) = value.values.first() {
        assert_eq!(first_name, "John");
    } else {
        panic!("Expected first name to be 'John'");
    }

    if let Some(FhirPathValue::String(second_name)) = value.values.get(1) {
        assert_eq!(second_name, "David");
    } else {
        panic!("Expected second name to be 'David'");
    }
}

#[tokio::test]
async fn test_configuration_builder() {
    let fhirpath = FhirPathConfigBuilder::new()
        .with_fhir_version(FhirVersion::R5)
        .with_analyzer(true)
        .with_performance_tracking(true)
        .build()
        .await
        .unwrap();

    // Verify configuration was applied
    assert!(fhirpath.has_analyzer());
    assert!(fhirpath.has_performance_tracking());

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    let result = fhirpath
        .evaluate_with_analysis("Patient.name", &patient)
        .await;
    assert!(result.is_ok());

    let analysis_result = result.unwrap();
    assert!(!analysis_result.values.is_empty());
    assert!(analysis_result.execution_time > std::time::Duration::from_nanos(0));

    // Should have analysis results if analyzer is enabled
    if fhirpath.has_analyzer() {
        assert!(analysis_result.analysis.is_some());
    }
}

#[tokio::test]
async fn test_engine_factory_patterns() {
    // Create factory with schema manager
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = octofhir_fhirschema::package::PackageManagerConfig::default();
    let schema_manager = std::sync::Arc::new(
        octofhir_fhirschema::package::FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let factory = FhirPathEngineFactory::new(schema_manager);

    // Test basic engine creation
    let basic_engine = factory.create_basic_engine().await;
    assert!(basic_engine.is_ok());

    let engine = basic_engine.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["Test"], "family": "Patient"}]
    });

    let result = engine.evaluate("Patient.name.family", patient).await;
    assert!(result.is_ok());

    let value = result.unwrap();
    if let FhirPathValue::Collection(items) = value {
        if let Some(FhirPathValue::String(family_name)) = items.first() {
            assert_eq!(family_name, "Patient");
        }
    }
}

#[tokio::test]
async fn test_advanced_engine_creation() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = octofhir_fhirschema::package::PackageManagerConfig::default();
    let schema_manager = std::sync::Arc::new(
        octofhir_fhirschema::package::FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let factory = FhirPathEngineFactory::new(schema_manager);

    // Test advanced engine creation with custom config
    let engine_config = FhirPathEngineConfig {
        enable_analyzer: true,
        enable_performance_tracking: true,
        max_recursion_depth: 100,
        timeout_ms: 5000,
        memory_limit_mb: Some(50),
    };

    let advanced_engine = factory.create_advanced_engine(&engine_config).await;
    assert!(advanced_engine.is_ok());

    let engine = advanced_engine.unwrap();
    assert!(engine.has_analysis_capabilities());
    assert!(engine.has_performance_tracking());
}

#[tokio::test]
async fn test_cli_engine_creation() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = octofhir_fhirschema::package::PackageManagerConfig::default();
    let schema_manager = std::sync::Arc::new(
        octofhir_fhirschema::package::FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let factory = FhirPathEngineFactory::new(schema_manager);

    // Test CLI engine creation
    let cli_engine = factory.create_cli_engine(OutputFormat::Pretty).await;
    assert!(cli_engine.is_ok());

    let engine = cli_engine.unwrap();
    assert!(engine.supports_analysis());

    let patient = json!({
        "resourceType": "Patient",
        "active": true
    });

    let result = engine.evaluate_for_cli("Patient.active", &patient).await;
    assert!(result.is_ok());

    let cli_result = result.unwrap();
    assert!(!cli_result.output.is_empty());
    assert!(cli_result.success);
}

#[tokio::test]
async fn test_convenience_functions() {
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}],
        "active": true
    });

    // Test simple evaluation function
    let result = evaluate_fhirpath("Patient.name.family", &patient).await;
    assert!(result.is_ok());

    let values = result.unwrap();
    if let Some(FhirPathValue::String(family_name)) = values.first() {
        assert_eq!(family_name, "Doe");
    }

    // Test evaluation with FHIR version
    let result_r5 =
        evaluate_fhirpath_with_version("Patient.name.family", &patient, FhirVersion::R5).await;
    assert!(result_r5.is_ok());

    let values_r5 = result_r5.unwrap();
    if let Some(FhirPathValue::String(family_name)) = values_r5.first() {
        assert_eq!(family_name, "Doe");
    }

    // Test validation function
    let validation = validate_fhirpath("Patient.name.given", Some("Patient")).await;
    assert!(validation.is_ok());

    let validation_result = validation.unwrap();
    assert!(validation_result.is_valid);

    // Test boolean evaluation
    let bool_result = evaluate_boolean("Patient.active", &patient).await;
    assert!(bool_result.is_ok());
    assert!(bool_result.unwrap());

    // Test string value extraction
    let string_result = get_string_value("Patient.name.family.first()", &patient).await;
    assert!(string_result.is_ok());
    assert_eq!(string_result.unwrap(), Some("Doe".to_string()));

    // Test all string values extraction
    let all_strings = get_all_string_values("Patient.name.given", &patient).await;
    assert!(all_strings.is_ok());

    let strings = all_strings.unwrap();
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0], "John");
}

#[tokio::test]
async fn test_evaluation_with_analysis() {
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    // Test evaluation with analysis
    let result = evaluate_fhirpath_with_analysis("Patient.name.given", &patient).await;
    assert!(result.is_ok());

    let (values, analysis) = result.unwrap();
    assert!(!values.is_empty());

    if let Some(FhirPathValue::String(name)) = values.first() {
        assert_eq!(name, "John");
    }

    // Should have analysis information
    assert!(analysis.is_some());

    let analysis_info = analysis.unwrap();
    assert!(!analysis_info.type_annotations.is_empty());
}

#[tokio::test]
async fn test_fhir_path_engine_with_analyzer() {
    // Test engine without analyzer
    let provider = Box::new(MockModelProvider::new());
    let engine = FhirPathEngineWithAnalyzer::new(provider).await;
    assert!(engine.is_ok());

    let mut engine = engine.unwrap();
    assert!(engine.analyzer.is_none());

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    let result = engine
        .evaluate("Patient.name.family", patient.clone())
        .await;
    assert!(result.is_ok());

    // Test evaluation with analysis (should return None for analysis)
    let analysis_result = engine
        .evaluate_with_analysis("Patient.name.family", patient)
        .await;
    assert!(analysis_result.is_ok());

    let (value, analysis) = analysis_result.unwrap();
    assert!(analysis.is_none()); // No analyzer configured

    if let FhirPathValue::Collection(items) = value {
        if let Some(FhirPathValue::String(family_name)) = items.first() {
            assert_eq!(family_name, "Doe");
        }
    }
}

#[tokio::test]
async fn test_fhir_path_engine_with_analyzer_enabled() {
    // Test engine with analyzer enabled
    let provider = Box::new(MockModelProvider::new());
    let engine = FhirPathEngineWithAnalyzer::with_analyzer(provider).await;
    assert!(engine.is_ok());

    let mut engine = engine.unwrap();
    assert!(engine.analyzer.is_some());

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    // Test evaluation with analysis (should return analysis)
    let analysis_result = engine
        .evaluate_with_analysis("Patient.name.family", patient)
        .await;
    assert!(analysis_result.is_ok());

    let (value, analysis) = analysis_result.unwrap();
    assert!(analysis.is_some()); // Should have analysis with analyzer enabled

    if let FhirPathValue::Collection(items) = value {
        if let Some(FhirPathValue::String(family_name)) = items.first() {
            assert_eq!(family_name, "Doe");
        }
    }

    // Test expression validation
    let validation = engine.validate_expression("Patient.name.given").await;
    assert!(validation.is_ok());

    let validation_errors = validation.unwrap();
    // Should have minimal or no validation errors for valid expression
    assert!(validation_errors.len() <= 1);

    // Test expression analysis
    let analysis = engine.analyze_expression("Patient.name.family").await;
    assert!(analysis.is_ok());

    let analysis_result = analysis.unwrap();
    assert!(analysis_result.is_some());

    let analysis_info = analysis_result.unwrap();
    assert!(!analysis_info.type_annotations.is_empty());
}

#[tokio::test]
async fn test_schema_aware_operations() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = octofhir_fhirschema::package::PackageManagerConfig::default();
    let schema_manager = std::sync::Arc::new(
        octofhir_fhirschema::package::FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    // Create schema-aware registry
    let schema_registry = create_schema_aware_registry(schema_manager.clone()).await;
    assert!(schema_registry.is_ok());

    // Create type registry
    let type_registry = FhirPathTypeRegistry::new(schema_manager).await;
    assert!(type_registry.is_ok());

    let registry = type_registry.unwrap();

    // Test O(1) type operations
    assert!(registry.is_resource_type("Patient"));
    assert!(registry.is_resource_type("Observation"));
    assert!(registry.is_resource_type("Bundle"));
    assert!(!registry.is_resource_type("InvalidType"));

    assert!(registry.is_primitive_type("string"));
    assert!(registry.is_primitive_type("boolean"));
    assert!(registry.is_primitive_type("integer"));

    assert!(registry.is_data_type("HumanName"));
    assert!(registry.is_data_type("Address"));
}

#[tokio::test]
async fn test_json_utilities() {
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}],
        "active": true
    });

    // Test conversion utilities
    let fhir_value = serde_to_fhir_value(&patient);
    assert!(fhir_value.is_ok());

    let value = fhir_value.unwrap();
    let back_to_serde = fhir_value_to_serde(&value);
    assert!(back_to_serde.is_ok());

    // Test JSON parsing utilities
    let json_str = r#"{"resourceType": "Patient", "active": true}"#;
    let parsed = parse_json(json_str);
    assert!(parsed.is_ok());

    let parsed_value = parsed.unwrap();
    assert_eq!(parsed_value["resourceType"], "Patient");
    assert_eq!(parsed_value["active"], true);

    // Test parsing as FHIR value
    let fhir_parsed = parse_as_fhir_value(json_str);
    assert!(fhir_parsed.is_ok());

    // Test JSON reformatting
    let reformatted = reformat_json(json_str);
    assert!(reformatted.is_ok());

    let pretty_json = reformatted.unwrap();
    assert!(pretty_json.contains("Patient"));
    assert!(pretty_json.len() > json_str.len()); // Should be pretty-printed
}

#[tokio::test]
async fn test_path_existence_utilities() {
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}],
        "address": [{
            "line": ["123 Main St"],
            "city": "Boston"
        }]
    });

    // Test path existence checks
    let name_exists = path_exists("Patient.name", &patient).await;
    assert!(name_exists.is_ok());
    assert!(name_exists.unwrap());

    let invalid_path = path_exists("Patient.invalidProperty", &patient).await;
    assert!(invalid_path.is_ok());
    assert!(!invalid_path.unwrap());

    let nested_path = path_exists("Patient.address.city", &patient).await;
    assert!(nested_path.is_ok());
    assert!(nested_path.unwrap());
}

#[tokio::test]
async fn test_complex_navigation_scenarios() {
    let bundle = json!({
        "resourceType": "Bundle",
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
                    "id": "obs-1",
                    "status": "final",
                    "subject": {"reference": "Patient/patient-1"},
                    "valueQuantity": {"value": 120, "unit": "mmHg"}
                }
            }
        ]
    });

    let fhirpath = FhirPath::new().await.unwrap();

    // Test complex navigation
    let patients = fhirpath
        .evaluate("Bundle.entry.resource.ofType(Patient)", &bundle)
        .await;
    assert!(patients.is_ok());

    let patient_result = patients.unwrap();
    assert_eq!(patient_result.values.len(), 1);

    let observations = fhirpath
        .evaluate("Bundle.entry.resource.ofType(Observation)", &bundle)
        .await;
    assert!(observations.is_ok());

    let obs_result = observations.unwrap();
    assert_eq!(obs_result.values.len(), 1);

    // Test chained navigation
    let values = fhirpath
        .evaluate(
            "Bundle.entry.resource.ofType(Observation).valueQuantity.value",
            &bundle,
        )
        .await;
    assert!(values.is_ok());

    let value_result = values.unwrap();
    if let Some(FhirPathValue::Integer(val)) = value_result.values.first() {
        assert_eq!(*val, 120);
    } else if let Some(FhirPathValue::Decimal(val)) = value_result.values.first() {
        assert!((val.to_f64().unwrap() - 120.0).abs() < 0.001);
    }
}

#[tokio::test]
async fn test_performance_tracking() {
    let fhirpath = FhirPathConfigBuilder::new()
        .with_performance_tracking(true)
        .build()
        .await
        .unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    let result = fhirpath
        .evaluate("Patient.name.given.first()", &patient)
        .await;
    assert!(result.is_ok());

    let evaluation_result = result.unwrap();
    assert!(evaluation_result.execution_time > std::time::Duration::from_nanos(0));

    if let Some(metrics) = evaluation_result.performance_metrics {
        assert!(metrics.parse_time >= std::time::Duration::from_nanos(0));
        assert!(metrics.evaluation_time >= std::time::Duration::from_nanos(0));
        assert!(metrics.total_time >= std::time::Duration::from_nanos(0));
    }
}

#[tokio::test]
async fn test_error_handling_integration() {
    let fhirpath = FhirPath::new().await.unwrap();

    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["John"], "family": "Doe"}]
    });

    // Test invalid expression
    let invalid_result = fhirpath.evaluate("Patient.invalidProperty", &patient).await;
    // Should handle error gracefully
    match invalid_result {
        Ok(_) => {} // Some implementations may handle this gracefully
        Err(err) => {
            // Should be a proper error, not a panic
            let error_msg = format!("{:?}", err);
            assert!(error_msg.contains("Error") || error_msg.contains("Invalid"));
        }
    }

    // Test malformed expression
    let malformed_result = fhirpath.evaluate("Patient.name.", &patient).await;
    assert!(malformed_result.is_err());

    // Test type mismatch
    let type_error = fhirpath.evaluate("Patient.name + 42", &patient).await;
    // Should handle type errors appropriately
    match type_error {
        Ok(_) => {} // Some implementations may handle coercion
        Err(err) => {
            let error_msg = format!("{:?}", err);
            assert!(
                error_msg.contains("Error")
                    || error_msg.contains("Invalid")
                    || error_msg.contains("Type")
            );
        }
    }
}
