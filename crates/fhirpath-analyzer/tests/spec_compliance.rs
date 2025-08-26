//! Specification compliance tests for FHIRPath analyzer

use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

/// Analyzer test suite for spec compliance
struct AnalyzerTestSuite {
    analyzer: FhirPathAnalyzer,
}

impl AnalyzerTestSuite {
    async fn new() -> Self {
        let provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(create_standard_registry());
        let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

        Self { analyzer }
    }
}

/// Tests against FHIRPath specification requirements
/// Based on official test suites that should pass analysis

#[tokio::test]
async fn test_spec_compliant_expressions() {
    let suite = AnalyzerTestSuite::new().await;

    // Sample from official FHIRPath tests that should pass analysis
    let spec_expressions = vec![
        // From literals.json
        "true",
        "false",
        "'test string'",
        "1",
        "1.0",
        // From basics.json
        "Patient",
        "Patient.name",
        "Patient.name.given",
        // From functions that should be recognized
        "empty()",
        "count()",
        "first()",
        "last()",
        "tail()",
        "skip(1)",
        "take(1)",
        "single()",
        "distinct()",
    ];

    for expression in spec_expressions {
        let result = suite.analyzer.analyze(expression).await;

        assert!(
            result.is_ok(),
            "Spec-compliant expression failed analysis: {} - {:?}",
            expression,
            result.err()
        );

        let result = result.unwrap();

        // Should not have critical validation errors for spec-compliant expressions
        let critical_errors: Vec<_> = result
            .validation_errors
            .iter()
            .filter(|e| {
                matches!(
                    e.error_type,
                    ValidationErrorType::InvalidFunction | ValidationErrorType::TypeMismatch
                )
            })
            .collect();

        assert!(
            critical_errors.is_empty(),
            "Spec-compliant expression has critical errors: {expression} - {critical_errors:?}"
        );
    }
}

#[tokio::test]
async fn test_maintains_existing_coverage() {
    // This test ensures that analyzer doesn't break existing evaluation
    // by testing expressions from TEST_COVERAGE.md passing suites

    let suite = AnalyzerTestSuite::new().await;

    // Sample expressions from suites with high pass rates
    let passing_expressions = vec![
        // From aggregate.json (4/4 passing)
        "Patient.name.count()",
        // From boolean-logic-and.json (9/9 passing)
        "true and true",
        "false and true",
        // From contains-string.json (11/11 passing)
        "'hello'.contains('ell')",
        "'world'.contains('foo')",
        // From count.json (4/4 passing)
        "Patient.name.count()",
        // From literals.json - if it exists and passes
        "1",
        "'test'",
        "true",
    ];

    for expression in passing_expressions {
        let analysis_result = suite.analyzer.analyze(expression).await;

        // Analysis should succeed
        assert!(
            analysis_result.is_ok(),
            "Analysis failed for expression from passing test suite: {} - {:?}",
            expression,
            analysis_result.err()
        );

        // Should provide useful analysis
        let result = analysis_result.unwrap();
        assert!(
            !result.type_annotations.is_empty() || !result.function_calls.is_empty(),
            "No analysis information for passing expression: {expression}"
        );
    }
}

#[tokio::test]
async fn test_literal_expressions() {
    let suite = AnalyzerTestSuite::new().await;

    // Test various literal types from FHIRPath specification
    let literals = vec![
        ("'string literal'", "String"),
        ("42", "Integer"),
        ("3.14159", "Decimal"),
        ("true", "Boolean"),
        ("false", "Boolean"),
        ("@2024-01-01", "Date"),
        ("@2024-01-01T12:00:00", "DateTime"),
        ("@T12:00:00", "Time"),
    ];

    for (expression, expected_type) in literals {
        let result = suite.analyzer.analyze(expression).await.unwrap();

        assert!(
            !result.type_annotations.is_empty(),
            "No type annotation for literal: {expression}"
        );

        let semantic_info = result.type_annotations.values().next().unwrap();
        assert_eq!(
            semantic_info.fhir_path_type.as_ref().unwrap(),
            expected_type,
            "Wrong type for literal: {expression}"
        );
    }
}

#[tokio::test]
async fn test_basic_navigation() {
    let suite = AnalyzerTestSuite::new().await;

    // Test basic path navigation expressions
    let navigation_expressions = vec![
        "Patient",
        "Patient.name",
        "Patient.name.family",
        "Patient.name.given",
        "Patient.telecom",
        "Patient.telecom.value",
    ];

    for expression in navigation_expressions {
        let result = suite.analyzer.analyze(expression).await;

        assert!(
            result.is_ok(),
            "Basic navigation failed: {} - {:?}",
            expression,
            result.err()
        );

        let result = result.unwrap();
        assert!(
            !result.type_annotations.is_empty(),
            "No type analysis for navigation: {expression}"
        );
    }
}

#[tokio::test]
async fn test_collection_functions() {
    let suite = AnalyzerTestSuite::new().await;

    // Test FHIRPath collection functions
    let collection_functions = vec![
        "empty()",
        "count()",
        "first()",
        "last()",
        "tail()",
        "distinct()",
        "single()",
    ];

    for expression in collection_functions {
        let result = suite.analyzer.analyze(expression).await;

        assert!(
            result.is_ok(),
            "Collection function analysis failed: {} - {:?}",
            expression,
            result.err()
        );

        let result = result.unwrap();

        // Should have function call analysis
        assert!(
            !result.function_calls.is_empty(),
            "No function analysis for: {expression}"
        );

        let func_analysis = &result.function_calls[0];
        assert!(
            func_analysis.validation_errors.is_empty(),
            "Validation errors for valid function: {} - {:?}",
            expression,
            func_analysis.validation_errors
        );
    }
}

#[tokio::test]
async fn test_boolean_logic() {
    let suite = AnalyzerTestSuite::new().await;

    // Test boolean logic expressions
    let boolean_expressions = vec![
        "true and true",
        "true or false",
        "not true",
        "true implies false",
        "true xor false",
    ];

    for expression in boolean_expressions {
        let result = suite.analyzer.analyze(expression).await;

        assert!(
            result.is_ok(),
            "Boolean logic analysis failed: {} - {:?}",
            expression,
            result.err()
        );

        // Should have type annotations for the operands
        let result = result.unwrap();
        assert!(
            !result.type_annotations.is_empty(),
            "No type analysis for boolean expression: {expression}"
        );
    }
}
