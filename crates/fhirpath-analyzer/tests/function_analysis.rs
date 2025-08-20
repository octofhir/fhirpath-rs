//! Integration tests for function analysis

use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

#[tokio::test]
async fn test_function_signature_validation() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);

    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Test valid function call
    let result = analyzer.analyze("count()").await?;
    assert!(
        result.validation_errors.is_empty(),
        "Valid function should not have validation errors"
    );
    assert!(
        !result.function_calls.is_empty(),
        "Should have function call analysis"
    );

    Ok(())
}

#[tokio::test]
async fn test_unknown_function() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);

    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Test invalid function call
    let result = analyzer.analyze("unknownFunction()").await?;

    // Should have validation errors for unknown function
    let function_errors: Vec<_> = result
        .validation_errors
        .iter()
        .filter(|e| matches!(e.error_type, ValidationErrorType::InvalidFunction))
        .collect();

    assert!(
        !function_errors.is_empty(),
        "Should have function validation errors"
    );

    let error = function_errors[0];
    assert!(
        error.message.contains("unknownFunction") || error.message.contains("Function not found"),
        "Error should mention function name or not found: {}",
        error.message
    );

    Ok(())
}

#[tokio::test]
async fn test_function_with_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);

    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Test function call with parameters
    let result = analyzer.analyze("substring('hello', 1, 3)").await?;

    // Should analyze the function call successfully
    assert!(
        !result.function_calls.is_empty(),
        "Should have function call analysis for substring"
    );

    let func_call = &result.function_calls[0];
    assert_eq!(func_call.function_name, "substring");
    assert_eq!(
        func_call.parameter_types.len(),
        3,
        "Should have 3 parameters"
    );

    Ok(())
}

#[tokio::test]
async fn test_analyzer_without_function_registry() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());

    // Create analyzer without function registry
    let analyzer = FhirPathAnalyzer::new(provider);

    // Should still work but without function analysis
    let result = analyzer.analyze("count()").await?;

    // Should not have function analysis
    assert!(
        result.function_calls.is_empty(),
        "Should not have function analysis without registry"
    );

    Ok(())
}

#[tokio::test]
async fn test_nested_function_calls() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);

    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Test nested function calls
    let result = analyzer.analyze("length(substring('hello', 1, 3))").await?;

    // Should analyze both function calls
    assert_eq!(
        result.function_calls.len(),
        2,
        "Should analyze both nested functions"
    );

    let function_names: Vec<String> = result
        .function_calls
        .iter()
        .map(|f| f.function_name.clone())
        .collect();

    assert!(
        function_names.contains(&"length".to_string()),
        "Should analyze length function"
    );
    assert!(
        function_names.contains(&"substring".to_string()),
        "Should analyze substring function"
    );

    Ok(())
}

#[tokio::test]
async fn test_function_validation_errors() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);

    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Test validation-only method
    let errors = analyzer.validate("invalidFunction()").await?;
    assert!(
        !errors.is_empty(),
        "Should have validation errors for invalid function"
    );

    // Check that error mentions function issue
    let has_function_error = errors.iter().any(|e| {
        matches!(e.error_type, ValidationErrorType::InvalidFunction)
            || e.message.to_lowercase().contains("function")
    });
    assert!(
        has_function_error,
        "Should have function-related validation error"
    );

    Ok(())
}
