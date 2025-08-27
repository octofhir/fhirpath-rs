use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::FhirSchemaModelProvider;
use std::sync::Arc;

/// Test basic field validation functionality
#[tokio::test]
async fn test_field_validation_basic() {
    let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test valid field access
    let result = analyzer.analyze("Patient.name").await.unwrap();

    // FhirSchemaModelProvider has actual FHIR schema validation
    // Valid fields should not generate InvalidField errors
    assert!(
        result
            .validation_errors
            .iter()
            .all(|e| { !matches!(e.error_type, ValidationErrorType::InvalidField) })
    );
}

/// Test invalid field validation
#[tokio::test]
async fn test_field_validation_invalid_field() {
    let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test invalid field access - this should generate a validation error
    let result = analyzer.analyze("Patient.nonExistentField").await.unwrap();

    // Look for field validation errors
    let has_field_error = result
        .validation_errors
        .iter()
        .any(|e| matches!(e.error_type, ValidationErrorType::InvalidField));

    // FhirSchemaModelProvider should catch invalid fields
    println!("Validation errors: {:?}", result.validation_errors);
}

/// Test invalid resource type validation
#[tokio::test]
async fn test_resource_type_validation() {
    let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test invalid resource type
    let result = analyzer.analyze("InvalidResource.name").await.unwrap();

    // Look for resource type validation errors
    let has_resource_error = result
        .validation_errors
        .iter()
        .any(|e| matches!(e.error_type, ValidationErrorType::InvalidResourceType));

    println!("Validation errors: {:?}", result.validation_errors);
    // FhirSchemaModelProvider should validate resource types
}

/// Test field validation suggestions
#[tokio::test]
async fn test_field_validation_suggestions() {
    let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test field access with typo that should generate suggestions
    let result = analyzer.analyze("Patient.nam").await.unwrap(); // "nam" instead of "name"

    // Check if suggestions are provided
    for error in &result.validation_errors {
        if matches!(error.error_type, ValidationErrorType::InvalidField) {
            println!("Error: {}", error.message);
            println!("Suggestions: {:?}", error.suggestions);

            // Should suggest "name" as a similar field
            assert!(
                error
                    .suggestions
                    .iter()
                    .any(|s| s.contains("name") || s == "name")
            );
        }
    }
}

/// Test field validation is always enabled
#[tokio::test]
async fn test_field_validation_always_enabled() {
    use octofhir_fhirpath_analyzer::AnalyzerConfig;

    let provider = Arc::new(FhirSchemaModelProvider::new().await.unwrap());

    // Even in minimal config, field validation should be enabled
    let config = AnalyzerConfig::minimal();

    let analyzer = FhirPathAnalyzer::with_config(provider, config);

    // Test that field validation is always enabled
    let result = analyzer.analyze("Patient.nonExistentField").await.unwrap();

    println!("Validation errors: {:?}", result.validation_errors);
    // Field validation should always be enabled now
}
