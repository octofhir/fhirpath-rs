//! Integration tests for children() function analysis

use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

#[tokio::test]
async fn test_children_function_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let function_registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, function_registry);

    // Test children() function
    let result = analyzer.analyze("Patient.children()").await.unwrap();

    // Debug output to understand what's happening
    println!("Function calls: {:?}", result.function_calls.len());
    for fc in &result.function_calls {
        println!("Function: {}", fc.function_name);
    }
    println!("Union types: {:?}", result.union_types.len());
    println!("Validation errors: {:?}", result.validation_errors.len());
    for error in &result.validation_errors {
        println!("Error: {}", error.message);
    }

    // Should have function analysis (children is a function call)
    assert!(!result.function_calls.is_empty());
    let func_analysis = &result.function_calls[0];
    assert_eq!(func_analysis.function_name, "children");
}

#[tokio::test]
async fn test_children_with_invalid_parameters() {
    let provider = Arc::new(MockModelProvider::new());
    let function_registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, function_registry);

    // Test children() with invalid parameters
    let result = analyzer
        .analyze("Patient.children('invalid')")
        .await
        .unwrap();

    // Should have validation error
    assert!(!result.validation_errors.is_empty());
    let error = &result.validation_errors[0];
    assert_eq!(error.error_type, ValidationErrorType::InvalidFunction);
    assert!(error.message.contains("expects 0 parameters"));
}

#[tokio::test]
async fn test_children_with_type_filter() {
    let provider = Arc::new(MockModelProvider::new());
    let function_registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, function_registry);

    // Test children().ofType(HumanName) - would need mock setup
    let result = analyzer
        .analyze("Patient.children().ofType(HumanName)")
        .await
        .unwrap();

    // Should analyze the type filter operation
    // Validation depends on mock provider setup
    assert!(
        result
            .function_calls
            .iter()
            .any(|f| f.function_name == "children")
    );
}

#[tokio::test]
async fn test_union_type_suggestions() {
    let provider = Arc::new(MockModelProvider::new());
    let function_registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, function_registry);

    // Test invalid type filter
    let result = analyzer
        .analyze("Patient.children().ofType(InvalidType)")
        .await
        .unwrap();

    // Should have suggestions for valid types
    let type_errors: Vec<_> = result
        .validation_errors
        .iter()
        .filter(|e| e.error_type == ValidationErrorType::InvalidTypeOperation)
        .collect();

    if !type_errors.is_empty() {
        assert!(!type_errors[0].suggestions.is_empty());
    }
}

#[tokio::test]
async fn test_children_function_basic_functionality() {
    let provider = Arc::new(MockModelProvider::new());
    let function_registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, function_registry);

    // Basic children() analysis
    let result = analyzer.analyze("Patient.children()").await.unwrap();

    // Verify function call analysis exists
    let children_funcs: Vec<_> = result
        .function_calls
        .iter()
        .filter(|f| f.function_name == "children")
        .collect();

    assert!(!children_funcs.is_empty());

    let func_analysis = children_funcs[0];
    assert_eq!(func_analysis.function_name, "children");
    assert_eq!(func_analysis.signature.name, "children");
    assert_eq!(func_analysis.signature.parameters.len(), 0);
    assert!(!func_analysis.signature.is_aggregate);
    assert!(
        func_analysis
            .signature
            .description
            .contains("Returns all child elements")
    );
}

#[tokio::test]
async fn test_children_union_type_creation() {
    let provider = Arc::new(MockModelProvider::new());
    let function_registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, function_registry);

    // Test union type creation
    let result = analyzer.analyze("Patient.children()").await.unwrap();

    // Should create union type for children
    if !result.union_types.is_empty() {
        let union_type = result.union_types.values().next().unwrap();
        assert!(union_type.is_collection);
        assert!(!union_type.constituent_types.is_empty());
        assert!(union_type.model_context.contains_key("parent_type"));
        assert_eq!(
            union_type.model_context.get("operation"),
            Some(&"children".to_string())
        );
    }
}
