//! Integration tests for the main FHIRPath API with Bridge Support

use octofhir_fhirpath::{
    FhirPath, FhirPathConfigBuilder, FhirVersion, OutputFormat, evaluate_fhirpath,
    parse_expression, validate_fhirpath,
};
use serde_json::json;

// Helper function to create a test patient resource
fn create_test_patient() -> serde_json::Value {
    json!({
        "resourceType": "Patient",
        "id": "example-1",
        "active": true,
        "name": [
            {
                "use": "official",
                "family": "Doe",
                "given": ["John", "James"]
            },
            {
                "use": "nickname",
                "given": ["Johnny"]
            }
        ],
        "telecom": [
            {
                "system": "email",
                "value": "john.doe@example.com",
                "use": "home"
            },
            {
                "system": "phone",
                "value": "+1-555-123-4567",
                "use": "work"
            }
        ],
        "gender": "male",
        "birthDate": "1990-01-15"
    })
}

// Helper function to create a test bundle
fn create_test_bundle() -> serde_json::Value {
    json!({
        "resourceType": "Bundle",
        "id": "example-bundle",
        "type": "collection",
        "entry": [
            {
                "resource": create_test_patient()
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "example-observation",
                    "status": "final",
                    "code": {
                        "coding": [
                            {
                                "system": "http://loinc.org",
                                "code": "29463-7",
                                "display": "Body weight"
                            }
                        ]
                    },
                    "subject": {
                        "reference": "Patient/example-1"
                    },
                    "valueQuantity": {
                        "value": 75.5,
                        "unit": "kg",
                        "system": "http://unitsofmeasure.org"
                    }
                }
            }
        ]
    })
}

#[tokio::test]
async fn test_parse_expression_syntax_validation() {
    // Valid expressions
    let valid_expressions = [
        "Patient.name",
        "Patient.name.given",
        "Patient.name.given.first()",
        "Patient.telecom.where(system = 'email').value",
        "Bundle.entry.resource.ofType(Patient)",
        "Patient.name.where(use = 'official').family",
    ];

    for expression in &valid_expressions {
        let result = parse_expression(expression);
        assert!(result.is_ok(), "Expression should be valid: {}", expression);
    }

    // Invalid expressions
    let invalid_expressions = [
        "Patient.name.",
        "Patient..name",
        "Patient.name.(",
        ".Patient.name",
        "Patient.name where",
    ];

    for expression in &invalid_expressions {
        let result = parse_expression(expression);
        assert!(
            result.is_err(),
            "Expression should be invalid: {}",
            expression
        );
    }
}

#[tokio::test]
async fn test_convenience_functions() {
    let patient = create_test_patient();

    // Test parse_expression (synchronous)
    let parse_result = parse_expression("Patient.name.family");
    assert!(parse_result.is_ok());

    // Note: The following tests would require proper schema manager setup
    // They are included to demonstrate the intended API usage

    /*
    // Test basic evaluation
    let family_names = evaluate_fhirpath("Patient.name.family", &patient).await;
    // Would assert family names contain "Doe"

    // Test validation
    let validation = validate_fhirpath("Patient.name.invalidField", Some("Patient")).await;
    // Would assert validation fails with helpful message

    // Test version-specific evaluation
    let r5_result = evaluate_fhirpath_with_version(
        "Patient.name.family",
        &patient,
        FhirVersion::R5
    ).await;
    // Would assert successful evaluation with R5 semantics
    */
}

#[tokio::test]
async fn test_configuration_builder() {
    // Test configuration building
    let config = FhirPathConfigBuilder::new()
        .with_fhir_version(FhirVersion::R5)
        .with_analyzer(true)
        .with_performance_tracking(true)
        .with_output_format(OutputFormat::Json)
        .add_package("hl7.fhir.us.core")
        .with_caching(true)
        .with_strict_mode(false)
        .with_timeout(5000)
        .with_max_depth(50)
        .build();

    // Verify configuration
    assert_eq!(config.schema_config.fhir_version, FhirVersion::R5);
    assert!(config.analyzer_enabled);
    assert!(config.performance_tracking);
    assert_eq!(config.engine_config.output_format, OutputFormat::Json);
    assert!(
        config
            .schema_config
            .packages
            .contains(&"hl7.fhir.us.core".to_string())
    );
    assert!(config.caching_enabled);
    assert!(!config.engine_config.strict_mode);
    assert_eq!(config.engine_config.evaluation_timeout_ms, 5000);
    assert_eq!(config.engine_config.max_evaluation_depth, 50);
}

#[tokio::test]
async fn test_fhir_version_parsing() {
    // Test string parsing
    assert_eq!("r4".parse::<FhirVersion>().unwrap(), FhirVersion::R4);
    assert_eq!("R4B".parse::<FhirVersion>().unwrap(), FhirVersion::R4B);
    assert_eq!("r5".parse::<FhirVersion>().unwrap(), FhirVersion::R5);

    // Test invalid version
    assert!("invalid".parse::<FhirVersion>().is_err());
}

#[tokio::test]
async fn test_output_format_defaults() {
    let config = FhirPathConfigBuilder::new().build();
    assert_eq!(config.engine_config.output_format, OutputFormat::Raw);
}

// Integration tests that would work with proper mock setup
#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_fhirpath_api_integration() -> Result<(), Box<dyn std::error::Error>> {
        let fhirpath = FhirPath::new().await?;
        let patient = create_test_patient();

        // Test basic evaluation
        let result = fhirpath.evaluate("Patient.name.given", &patient).await?;
        assert!(!result.is_empty());

        // Test evaluation with analysis
        let analysis_result = fhirpath
            .evaluate_with_analysis("Patient.name.where(use = 'official').family", &patient)
            .await?;

        assert!(!analysis_result.values.is_empty());
        assert!(analysis_result.execution_time.as_millis() >= 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_engine_factory_patterns() -> Result<(), Box<dyn std::error::Error>> {
        use octofhir_fhirpath::{FhirPathEngineFactory, FhirSchemaPackageManager};
        use std::sync::Arc;

        let schema_manager = Arc::new(FhirSchemaPackageManager::new().await?);
        let factory = FhirPathEngineFactory::new(schema_manager);

        // Test basic engine
        let _basic_engine = factory.create_basic_engine().await?;

        // Test advanced engine
        let config = octofhir_fhirpath::FhirPathEngineConfig::default();
        let advanced_engine = factory.create_advanced_engine(&config).await?;
        assert!(advanced_engine.has_analysis_capabilities());

        // Test CLI engine
        let _cli_engine = factory.create_cli_engine(OutputFormat::Pretty).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_convenience_function_integration() -> Result<(), Box<dyn std::error::Error>> {
        let patient = create_test_patient();

        // Test basic evaluation
        let names = evaluate_fhirpath("Patient.name.given", &patient).await?;
        assert!(!names.is_empty());

        // Test version-specific evaluation
        let family_names = octofhir_fhirpath::evaluate_fhirpath_with_version(
            "Patient.name.family",
            &patient,
            FhirVersion::R4,
        )
        .await?;
        assert!(!family_names.is_empty());

        // Test validation
        let validation = validate_fhirpath("Patient.name.given", Some("Patient")).await?;
        if let Some(result) = validation {
            assert!(result.is_valid);
        }

        // Test boolean evaluation
        let is_active = octofhir_fhirpath::evaluate_boolean("Patient.active", &patient).await?;
        assert!(is_active);

        // Test string value extraction
        let family_name = octofhir_fhirpath::get_string_value(
            "Patient.name.where(use = 'official').family.first()",
            &patient,
        )
        .await?;
        assert_eq!(family_name, Some("Doe".to_string()));

        // Test path existence
        let has_name = octofhir_fhirpath::path_exists("Patient.name", &patient).await?;
        assert!(has_name);

        let has_invalid = octofhir_fhirpath::path_exists("Patient.invalidField", &patient).await?;
        assert!(!has_invalid);

        Ok(())
    }

    #[tokio::test]
    async fn test_complex_expressions() -> Result<(), Box<dyn std::error::Error>> {
        let bundle = create_test_bundle();

        // Complex bundle navigation
        let patients = evaluate_fhirpath("Bundle.entry.resource.ofType(Patient)", &bundle).await?;
        assert!(!patients.is_empty());

        // Complex filtering
        let email_contacts = evaluate_fhirpath(
            "Bundle.entry.resource.ofType(Patient).telecom.where(system = 'email')",
            &bundle,
        )
        .await?;
        assert!(!email_contacts.is_empty());

        // Cross-resource references
        let observations = evaluate_fhirpath(
            "Bundle.entry.resource.ofType(Observation).where(subject.reference.startsWith('Patient/'))",
            &bundle
        ).await?;
        assert!(!observations.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_tracking() -> Result<(), Box<dyn std::error::Error>> {
        let config = FhirPathConfigBuilder::new()
            .with_performance_tracking(true)
            .create_fhirpath()
            .await?;

        let patient = create_test_patient();

        let result = config
            .evaluate_with_analysis("Patient.name.given.first()", &patient)
            .await?;

        // Verify performance metrics are available
        assert!(result.performance_metrics.is_some());

        if let Some(metrics) = result.performance_metrics {
            assert!(metrics.parse_time.as_nanos() > 0);
            assert!(metrics.evaluation_time.as_nanos() > 0);
        }

        Ok(())
    }
}

// Mock implementation tests (always run)
#[tokio::test]
async fn test_api_surface_availability() {
    // Test that all API components are available and can be imported
    let _ = FhirPath::new().await; // Will fail but should compile
    let _ = FhirPathConfigBuilder::new();
    let _ = parse_expression("Patient.name");

    // Test enum availability
    let _version = FhirVersion::R4;
    let _format = OutputFormat::Json;
}

#[tokio::test]
async fn test_error_handling() {
    // Test that invalid expressions produce appropriate errors
    let invalid_result = parse_expression("Invalid..expression");
    assert!(invalid_result.is_err());

    if let Err(error) = invalid_result {
        // Verify error contains useful information
        let error_message = error.to_string();
        assert!(error_message.contains("Parse error"));
    }
}

#[tokio::test]
async fn test_default_configurations() {
    // Test that defaults are reasonable
    let default_config = octofhir_fhirpath::FhirPathConfig::default();

    assert_eq!(default_config.schema_config.fhir_version, FhirVersion::R4);
    assert!(default_config.analyzer_enabled);
    assert!(!default_config.performance_tracking);
    assert!(default_config.caching_enabled);
    assert_eq!(
        default_config.engine_config.output_format,
        OutputFormat::Raw
    );
    assert!(!default_config.engine_config.strict_mode);
    assert_eq!(default_config.engine_config.max_evaluation_depth, 100);
    assert_eq!(default_config.engine_config.evaluation_timeout_ms, 5000);
}

#[test]
fn test_documentation_examples_compile() {
    // Ensure that examples in documentation comments compile
    // This is a compile-time test

    // From FhirPath::new example
    let _future = async {
        let _fhirpath = FhirPath::new().await;
    };

    // From convenience function examples
    let _future2 = async {
        let patient = json!({"resourceType": "Patient"});
        let _result = evaluate_fhirpath("Patient.name", &patient).await;
    };
}
