use octofhir_fhirpath_analyzer::{AnalysisContext, FhirPathAnalyzer};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_literal_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let result = analyzer.analyze("'hello world'").await.unwrap();

    assert!(!result.type_annotations.is_empty());
    // Should have one annotation for the string literal
    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.fhir_path_type, Some("String".to_string()));
    assert_eq!(
        semantic_info.confidence,
        octofhir_fhirpath_analyzer::types::ConfidenceLevel::High
    );
}

#[tokio::test]
async fn test_integer_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let result = analyzer.analyze("42").await.unwrap();

    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.fhir_path_type, Some("Integer".to_string()));
}

#[tokio::test]
async fn test_boolean_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let result = analyzer.analyze("true").await.unwrap();

    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.fhir_path_type, Some("Boolean".to_string()));
}

#[tokio::test]
async fn test_decimal_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let result = analyzer.analyze("3.14").await.unwrap();

    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.fhir_path_type, Some("Decimal".to_string()));
}

#[tokio::test]
async fn test_identifier_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let result = analyzer.analyze("Patient").await.unwrap();

    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.model_type, Some("Patient".to_string()));
    // Should resolve through MockModelProvider which has Patient type
    assert_eq!(semantic_info.fhir_path_type, Some("Resource".to_string()));
    assert_eq!(
        semantic_info.confidence,
        octofhir_fhirpath_analyzer::types::ConfidenceLevel::Medium
    );
}

#[tokio::test]
async fn test_unknown_identifier_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let result = analyzer.analyze("UnknownType").await.unwrap();

    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.fhir_path_type, None);
    assert_eq!(semantic_info.model_type, None);
    assert_eq!(
        semantic_info.confidence,
        octofhir_fhirpath_analyzer::types::ConfidenceLevel::Low
    );
}

#[tokio::test]
async fn test_complex_expression_analysis() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test a complex expression with multiple nodes
    let result = analyzer.analyze("Patient.name").await.unwrap();

    // Should have annotations for at least the Patient identifier
    assert!(!result.type_annotations.is_empty());

    // At least one annotation should be for an identifier
    let has_identifier = result
        .type_annotations
        .values()
        .any(|info| info.model_type == Some("Patient".to_string()));
    assert!(has_identifier, "Should have Patient identifier annotation");
}

#[tokio::test]
async fn test_caching_behavior() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // First analysis
    let result1 = analyzer.analyze("'test'").await.unwrap();

    // Second analysis (should be cached)
    let result2 = analyzer.analyze("'test'").await.unwrap();

    // Results should be identical
    assert_eq!(
        result1.type_annotations.len(),
        result2.type_annotations.len()
    );

    let info1 = result1.type_annotations.values().next().unwrap();
    let info2 = result2.type_annotations.values().next().unwrap();
    assert_eq!(info1.fhir_path_type, info2.fhir_path_type);
}

#[tokio::test]
async fn test_analyze_with_context() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let context = AnalysisContext {
        root_type: Some("Patient".to_string()),
        variables: std::collections::HashMap::new(),
        environment: std::collections::HashMap::new(),
        settings: octofhir_fhirpath_analyzer::AnalysisSettings::default(),
    };

    let result = analyzer
        .analyze_with_context("'test'", &context)
        .await
        .unwrap();

    let semantic_info = result.type_annotations.values().next().unwrap();
    assert_eq!(semantic_info.fhir_path_type, Some("String".to_string()));
}

#[tokio::test]
async fn test_validation_basic() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let validation_errors = analyzer.validate("'test'").await.unwrap();

    // Basic validation shouldn't produce errors for simple literals
    assert!(validation_errors.is_empty());
}

#[tokio::test]
async fn test_get_type_info() {
    use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};

    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let node = ExpressionNode::Literal(LiteralValue::String("test".to_string()));
    let context = AnalysisContext {
        root_type: None,
        variables: std::collections::HashMap::new(),
        environment: std::collections::HashMap::new(),
        settings: octofhir_fhirpath_analyzer::AnalysisSettings::default(),
    };

    let type_info = analyzer.get_type_info(&node, &context).await;

    assert!(type_info.is_some());
    let info = type_info.unwrap();
    assert_eq!(info.fhir_path_type, Some("String".to_string()));
    assert_eq!(
        info.confidence,
        octofhir_fhirpath_analyzer::types::ConfidenceLevel::High
    );
}
