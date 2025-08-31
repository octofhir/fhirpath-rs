//! Comprehensive integration tests for bridge-enabled FHIRPath analyzer
//!
//! Tests the complete analysis system with bridge support API integration
//! including field validation, path navigation, error reporting, and constraint analysis.

use octofhir_canonical_manager::FcmConfig;
use octofhir_fhirpath_analyzer::{
    AnalysisContext, AnalyzerErrorReporter, AnalyzerFieldValidator, AnalyzerPathNavigator,
    ConstraintAnalyzer, ErrorCategory, FhirPathError,
};
use octofhir_fhirpath_model::FhirSchemaModelProvider;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use std::sync::Arc;

/// Create test schema manager for integration tests
async fn create_test_schema_manager()
-> Result<Arc<FhirSchemaPackageManager>, Box<dyn std::error::Error>> {
    let fcm_config = FcmConfig::default();
    let config = PackageManagerConfig::default();
    let schema_manager = Arc::new(FhirSchemaPackageManager::new(fcm_config, config).await?);
    Ok(schema_manager)
}

/// Create test field validator
async fn create_test_field_validator() -> Result<AnalyzerFieldValidator, Box<dyn std::error::Error>>
{
    let schema_manager = create_test_schema_manager().await?;
    Ok(AnalyzerFieldValidator::new(schema_manager).await?)
}

/// Create test path navigator
async fn create_test_path_navigator() -> Result<AnalyzerPathNavigator, Box<dyn std::error::Error>> {
    let schema_manager = create_test_schema_manager().await?;
    Ok(AnalyzerPathNavigator::new(schema_manager).await?)
}

/// Create test error reporter
async fn create_test_error_reporter() -> Result<AnalyzerErrorReporter, Box<dyn std::error::Error>> {
    let schema_manager = create_test_schema_manager().await?;
    Ok(AnalyzerErrorReporter::new(schema_manager).await?)
}

/// Create test constraint analyzer
async fn create_test_constraint_analyzer() -> Result<ConstraintAnalyzer, Box<dyn std::error::Error>>
{
    let schema_manager = create_test_schema_manager().await?;
    Ok(ConstraintAnalyzer::new(schema_manager).await?)
}

#[tokio::test]
async fn test_field_validation_with_bridge_support() -> Result<(), Box<dyn std::error::Error>> {
    let validator = create_test_field_validator().await?;

    // Test valid field
    let result = validator.validate_field("Patient", "name").await?;
    assert!(result.is_valid);
    assert!(result.suggestions.is_empty());
    assert!(result.property_info.is_some());

    // Test invalid field with suggestions
    let result = validator.validate_field("Patient", "invalidField").await?;
    assert!(!result.is_valid);
    assert!(!result.suggestions.is_empty());
    assert!(result.context_info.is_some());

    Ok(())
}

#[tokio::test]
async fn test_field_validation_similarity_suggestions() -> Result<(), Box<dyn std::error::Error>> {
    let validator = create_test_field_validator().await?;

    // Test typo in field name should provide suggestions
    let result = validator.validate_field("Patient", "nam").await?; // "name" with typo
    assert!(!result.is_valid);
    assert!(!result.suggestions.is_empty());

    // Should suggest "name" as a close match
    assert!(result.suggestions.iter().any(|s| s.contains("name")));

    Ok(())
}

#[tokio::test]
async fn test_path_navigation_with_suggestions() -> Result<(), Box<dyn std::error::Error>> {
    let navigator = create_test_path_navigator().await?;

    // Test path suggestions for partial input
    let suggestions = navigator
        .generate_path_suggestions("Patient", "nam")
        .await?;

    assert!(!suggestions.is_empty());

    // Should find "name" as a suggestion
    let name_suggestion = suggestions.iter().find(|s| s.property_name == "name");
    assert!(name_suggestion.is_some());

    let name_suggestion = name_suggestion.unwrap();
    assert!(name_suggestion.confidence > 0.5);
    assert_eq!(name_suggestion.full_path, "Patient.name");

    Ok(())
}

#[tokio::test]
async fn test_path_navigation_prefix_matching() -> Result<(), Box<dyn std::error::Error>> {
    let navigator = create_test_path_navigator().await?;

    // Test prefix matching should have high confidence
    let suggestions = navigator
        .generate_path_suggestions("Patient", "ide")
        .await?;

    assert!(!suggestions.is_empty());

    // Should find "identifier" with high confidence for prefix match
    let identifier_suggestion = suggestions.iter().find(|s| s.property_name == "identifier");
    assert!(identifier_suggestion.is_some());

    let identifier_suggestion = identifier_suggestion.unwrap();
    assert!(identifier_suggestion.confidence > 0.8); // High confidence for prefix match

    Ok(())
}

#[tokio::test]
async fn test_path_navigation_caching() -> Result<(), Box<dyn std::error::Error>> {
    let navigator = create_test_path_navigator().await?;

    // First call should populate cache
    let _suggestions1 = navigator
        .generate_path_suggestions("Patient", "nam")
        .await?;

    let stats_before = navigator.get_cache_stats();
    assert_eq!(stats_before.total_entries, 1);

    // Second call should use cache
    let _suggestions2 = navigator
        .generate_path_suggestions("Patient", "nam")
        .await?;

    let stats_after = navigator.get_cache_stats();
    assert_eq!(stats_after.total_entries, 1); // Same entry

    Ok(())
}

#[tokio::test]
async fn test_error_reporting_property_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let reporter = create_test_error_reporter().await?;
    let context = AnalysisContext::default();

    let error = FhirPathError::PropertyNotFound {
        type_name: "Patient".to_string(),
        property: "invalidProperty".to_string(),
        source: None,
    };

    let report = reporter
        .analyze_fhirpath_error("Patient.invalidProperty", &error, &context)
        .await?;

    assert_eq!(report.error_category, ErrorCategory::FieldAccess);
    assert!(!report.suggestions.is_empty());
    assert!(report.context_info.is_some());
    assert!(!report.related_documentation.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_error_reporting_function_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let reporter = create_test_error_reporter().await?;
    let context = AnalysisContext::default();

    let error = FhirPathError::FunctionNotFound("coutn".to_string()); // "count" with typo

    let report = reporter
        .analyze_fhirpath_error("Patient.name.coutn()", &error, &context)
        .await?;

    assert_eq!(report.error_category, ErrorCategory::FunctionCall);
    assert!(!report.suggestions.is_empty());

    // Should suggest "count" as similar function
    assert!(
        report
            .suggestions
            .iter()
            .any(|s| s.suggestion.contains("count"))
    );

    Ok(())
}

#[tokio::test]
async fn test_error_reporting_type_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let reporter = create_test_error_reporter().await?;
    let context = AnalysisContext::default();

    let error = FhirPathError::TypeMismatch {
        expected: "string".to_string(),
        actual: "integer".to_string(),
    };

    let report = reporter
        .analyze_fhirpath_error("Patient.birthDate.year = 'invalid'", &error, &context)
        .await?;

    assert_eq!(report.error_category, ErrorCategory::TypeSystem);
    assert!(!report.suggestions.is_empty());
    assert!(!report.fixes.is_empty());

    // Should suggest type conversion
    assert!(
        report
            .suggestions
            .iter()
            .any(|s| s.suggestion.contains("toString"))
    );

    Ok(())
}

#[tokio::test]
async fn test_constraint_validation_valid_constraint() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = create_test_constraint_analyzer().await?;

    let result = analyzer
        .validate_constraint("Patient.name.exists()", "Patient")
        .await?;

    assert!(result.is_valid);
    assert!(result.violations.is_empty());
    assert!(result.confidence > 0.8);
    assert!(result.performance_metrics.complexity_score < 0.5); // Simple constraint

    Ok(())
}

#[tokio::test]
async fn test_constraint_validation_invalid_property() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = create_test_constraint_analyzer().await?;

    let result = analyzer
        .validate_constraint("Patient.invalidField.exists()", "Patient")
        .await?;

    assert!(!result.is_valid);
    assert!(!result.violations.is_empty());
    assert!(!result.suggestions.is_empty());

    // Should have property not found violation
    assert!(result.violations.iter().any(|v| matches!(
        v,
        octofhir_fhirpath_analyzer::ConstraintViolation::PropertyNotFound { .. }
    )));

    Ok(())
}

#[tokio::test]
async fn test_constraint_validation_performance_metrics() -> Result<(), Box<dyn std::error::Error>>
{
    let analyzer = create_test_constraint_analyzer().await?;

    // Simple constraint should have low complexity
    let simple_result = analyzer
        .validate_constraint("Patient.id.exists()", "Patient")
        .await?;

    // Complex constraint should have higher complexity
    let complex_result = analyzer
        .validate_constraint(
            "Patient.name.where(use='official').family.exists()",
            "Patient",
        )
        .await?;

    assert!(simple_result.performance_metrics.complexity_score < 0.5);
    assert!(
        complex_result.performance_metrics.complexity_score
            > simple_result.performance_metrics.complexity_score
    );
    assert!(
        !complex_result
            .performance_metrics
            .recommendations
            .is_empty()
    );

    Ok(())
}

#[tokio::test]
async fn test_integrated_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Test complete workflow: validation -> navigation -> error analysis -> constraint check

    let validator = create_test_field_validator().await?;
    let navigator = create_test_path_navigator().await?;
    let reporter = create_test_error_reporter().await?;
    let constraint_analyzer = create_test_constraint_analyzer().await?;

    // Step 1: Validate a field (should fail)
    let validation_result = validator.validate_field("Patient", "invalidField").await?;
    assert!(!validation_result.is_valid);

    // Step 2: Get path suggestions for the invalid field
    let path_suggestions = navigator
        .generate_path_suggestions("Patient", "invalidField")
        .await?;
    assert!(!path_suggestions.is_empty());

    // Step 3: Analyze the error
    let context = AnalysisContext::default();
    let error = FhirPathError::PropertyNotFound {
        type_name: "Patient".to_string(),
        property: "invalidField".to_string(),
        source: None,
    };

    let error_report = reporter
        .analyze_fhirpath_error("Patient.invalidField", &error, &context)
        .await?;
    assert!(!error_report.suggestions.is_empty());

    // Step 4: Validate constraint with the invalid field
    let constraint_result = constraint_analyzer
        .validate_constraint("Patient.invalidField.exists()", "Patient")
        .await?;
    assert!(!constraint_result.is_valid);
    assert!(!constraint_result.suggestions.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_bridge_api_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Test that all bridge components work together with schema manager

    let schema_manager = create_test_schema_manager().await?;

    // Create all bridge components with the same schema manager
    let validator = AnalyzerFieldValidator::new(schema_manager.clone()).await?;
    let navigator = AnalyzerPathNavigator::new(schema_manager.clone()).await?;
    let reporter = AnalyzerErrorReporter::new(schema_manager.clone()).await?;
    let constraint_analyzer = ConstraintAnalyzer::new(schema_manager.clone()).await?;

    // Verify all components have access to the same schema manager
    assert!(Arc::ptr_eq(
        validator.schema_manager(),
        navigator.schema_manager()
    ));
    assert!(Arc::ptr_eq(
        navigator.schema_manager(),
        reporter.schema_manager()
    ));
    assert!(Arc::ptr_eq(
        reporter.schema_manager(),
        constraint_analyzer.schema_manager()
    ));

    // Test that they can work with the same data
    let validation_result = validator.validate_field("Patient", "name").await?;
    assert!(validation_result.is_valid);

    let path_suggestions = navigator
        .generate_path_suggestions("Patient", "nam")
        .await?;
    assert!(!path_suggestions.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_similarity_matching_accuracy() -> Result<(), Box<dyn std::error::Error>> {
    use octofhir_fhirpath_analyzer::SimilarityMatcher;

    let matcher = SimilarityMatcher::new();

    // Test various similarity scenarios
    assert_eq!(matcher.calculate_similarity("name", "name"), 1.0);
    assert!(matcher.calculate_similarity("name", "nam") > 0.7);
    assert!(matcher.calculate_similarity("patient", "patien") > 0.8);
    assert!(matcher.calculate_similarity("identifier", "identifi") > 0.8);

    // Test dissimilar strings
    assert!(matcher.calculate_similarity("name", "xyz") < 0.5);
    assert!(matcher.calculate_similarity("patient", "observation") < 0.5);

    // Test that typos are detected as similar
    assert!(matcher.is_similar("count", "coutn"));
    assert!(matcher.is_similar("exists", "exist"));

    Ok(())
}

#[tokio::test]
async fn test_performance_characteristics() -> Result<(), Box<dyn std::error::Error>> {
    // Test performance characteristics of bridge components

    let validator = create_test_field_validator().await?;
    let navigator = create_test_path_navigator().await?;

    // Test that operations complete in reasonable time
    let start = std::time::Instant::now();

    for _ in 0..10 {
        let _result = validator.validate_field("Patient", "name").await?;
    }

    let validation_duration = start.elapsed();
    assert!(
        validation_duration.as_millis() < 1000,
        "Field validation should be fast"
    );

    // Test navigation caching improves performance
    let start = std::time::Instant::now();

    // First call populates cache
    let _suggestions1 = navigator
        .generate_path_suggestions("Patient", "nam")
        .await?;
    let first_call_duration = start.elapsed();

    let start = std::time::Instant::now();

    // Second call should be faster (cached)
    let _suggestions2 = navigator
        .generate_path_suggestions("Patient", "nam")
        .await?;
    let second_call_duration = start.elapsed();

    // Second call should be significantly faster or at least not slower
    assert!(second_call_duration <= first_call_duration + std::time::Duration::from_millis(10));

    Ok(())
}
