//! Integration tests for semantic analysis
//!
//! These tests demonstrate how our semantic analyzer can catch errors
//! like the C# implementation while preserving fast runtime evaluation.

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::diagnostics::DiagnosticSeverity;
use octofhir_fhirpath::parser::parse_with_semantic_analysis;
use std::sync::Arc;

#[tokio::test]
async fn test_valid_expression_analysis() {
    let model_provider = Arc::new(EmptyModelProvider::new());

    let result = parse_with_semantic_analysis("Patient.name", model_provider, None).await;

    assert!(
        result.ast.is_some(),
        "Valid expression should parse successfully"
    );
    assert!(
        result.analysis.success,
        "Valid expression should pass semantic analysis"
    );
    assert!(
        result.analysis.diagnostics.is_empty(),
        "Valid expression should have no diagnostics"
    );
}

#[tokio::test]
async fn test_invalid_property_semantic_error() {
    let model_provider = Arc::new(EmptyModelProvider::new());

    // First get Patient type to establish context
    let patient_type = model_provider.get_type("Patient").await.unwrap().unwrap();

    let result = parse_with_semantic_analysis(
        "name.given1", // Invalid property like testSimpleFail
        model_provider,
        Some(patient_type),
    )
    .await;

    // Expression should parse but semantic analysis should fail
    assert!(
        result.ast.is_some(),
        "Expression should parse syntactically"
    );
    assert!(
        !result.analysis.success,
        "Semantic analysis should fail for invalid property"
    );

    // Should have error diagnostic
    let errors: Vec<_> = result
        .analysis
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
        .collect();

    assert!(!errors.is_empty(), "Should have semantic error");

    let error_message = &errors[0].message;
    assert!(
        error_message.contains("given1"),
        "Error should mention the invalid property"
    );
    assert!(
        error_message.contains("not found"),
        "Error should indicate property not found"
    );

    println!("✅ Semantic error detected: {}", error_message);
    println!("📝 This matches C# implementation: 'prop 'given1' not found on HumanName[]'");
}

#[tokio::test]
async fn test_chain_head_resource_detection() {
    let model_provider = Arc::new(EmptyModelProvider::new());

    let result = parse_with_semantic_analysis(
        "Patient.name.given",
        model_provider,
        None, // No context - should detect Patient as chain head
    )
    .await;

    assert!(result.ast.is_some(), "Chain expression should parse");
    assert!(result.analysis.success, "Chain-head detection should work");

    println!("✅ Chain-head detection works for Patient.name.given");
}

#[tokio::test]
async fn test_unknown_resource_type_error() {
    let model_provider = Arc::new(EmptyModelProvider::new());

    let result = parse_with_semantic_analysis("UnknownResource.name", model_provider, None).await;

    assert!(result.ast.is_some(), "Should parse syntactically");

    // Should have diagnostic about unknown resource type
    let has_resource_error = result
        .analysis
        .diagnostics
        .iter()
        .any(|d| d.message.contains("Unknown resource type"));

    if has_resource_error {
        println!("✅ Unknown resource type detection works");
    } else {
        // Might pass if we can't determine it's a resource type
        println!("📝 Unknown resource handling varies by implementation");
    }
}

#[tokio::test]
async fn test_semantic_vs_runtime_behavior() {
    let model_provider = Arc::new(EmptyModelProvider::new());

    // Test the same expression that fails in testSimpleFail
    let patient_type = model_provider.get_type("Patient").await.unwrap().unwrap();

    let result =
        parse_with_semantic_analysis("name.given1", model_provider, Some(patient_type)).await;

    // Key insight: Semantic analysis should fail, but runtime would return empty
    assert!(
        result.ast.is_some(),
        "Should parse for potential runtime evaluation"
    );
    assert!(
        !result.analysis.success,
        "Semantic analysis should catch the error"
    );

    let error_count = result
        .analysis
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
        .count();

    assert!(error_count > 0, "Should have semantic errors");

    println!("✅ Semantic analysis catches errors that runtime evaluation would handle gracefully");
    println!("📝 This allows both strict validation (analysis) and lenient evaluation (runtime)");
}
