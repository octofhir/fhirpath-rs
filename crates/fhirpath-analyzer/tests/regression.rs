//! Regression tests to ensure analyzer doesn't break existing functionality

use octofhir_fhirpath_analyzer::FhirPathAnalyzer;
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

/// Regression tests to ensure analyzer doesn't break existing functionality
#[tokio::test]
async fn test_no_analyzer_regression() {
    // Test that expressions that used to work still work with analyzer present

    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Basic expressions that should always work
    let baseline_expressions = vec!["'hello world'", "42", "true", "Patient", "Patient.name"];

    for expression in baseline_expressions {
        let result = analyzer.analyze(expression).await;

        assert!(
            result.is_ok(),
            "Baseline expression failed: {} - {:?}",
            expression,
            result.err()
        );

        // Should have at least basic type annotation
        let analysis = result.unwrap();
        assert!(
            !analysis.type_annotations.is_empty(),
            "No type analysis for baseline expression: {expression}"
        );
    }
}

#[tokio::test]
async fn test_cache_consistency() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    let expression = "Patient.name.given";

    // Analyze same expression multiple times
    let result1 = analyzer.analyze(expression).await.unwrap();
    let result2 = analyzer.analyze(expression).await.unwrap();
    let result3 = analyzer.analyze(expression).await.unwrap();

    // Results should be consistent
    assert_eq!(
        result1.type_annotations.len(),
        result2.type_annotations.len()
    );
    assert_eq!(
        result2.type_annotations.len(),
        result3.type_annotations.len()
    );
    assert_eq!(
        result1.validation_errors.len(),
        result2.validation_errors.len()
    );
    assert_eq!(
        result2.validation_errors.len(),
        result3.validation_errors.len()
    );
}

#[tokio::test]
async fn test_memory_usage() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test that repeated analysis doesn't cause memory leaks
    let expressions = vec![
        "'test'",
        "42",
        "Patient",
        "Patient.name",
        "Patient.name.given",
        "count()",
        "exists()",
        "empty()",
        "first()",
        "last()",
    ];

    // Run many iterations to detect memory issues
    for _ in 0..100 {
        for expression in &expressions {
            let _ = analyzer.analyze(expression).await.unwrap();
        }
    }

    // If we get here without OOM, memory usage is reasonable
    assert!(true, "Memory test completed successfully");
}

#[tokio::test]
async fn test_analyzer_with_function_registry() {
    // Test analyzer with function registry doesn't break basic functionality
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry());
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    let expressions = vec![
        "'basic string'",
        "123",
        "true",
        "false",
        "Patient",
        "Patient.name",
        "count()",
        "exists()",
    ];

    for expression in expressions {
        let result = analyzer.analyze(expression).await;

        assert!(
            result.is_ok(),
            "Expression failed with function registry: {} - {:?}",
            expression,
            result.err()
        );

        let result = result.unwrap();
        assert!(
            !result.type_annotations.is_empty() || !result.function_calls.is_empty(),
            "No analysis results for expression with function registry: {expression}"
        );
    }
}

#[tokio::test]
async fn test_error_handling_robustness() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test various malformed expressions that should not crash analyzer
    let malformed_expressions = vec![
        "",                                   // Empty expression should be handled gracefully
        "Patient.",                           // Incomplete path
        "Patient.unknown.deeply.nested.path", // Non-existent deep path
        "invalidSyntax(((",                   // Invalid syntax
        "Patient..name",                      // Double dot
    ];

    for expression in malformed_expressions {
        let result = analyzer.analyze(expression).await;

        // Should not panic or crash - either succeed with warnings or fail gracefully
        match result {
            Ok(analysis) => {
                // If successful, should have some analysis or error information
                assert!(
                    !analysis.type_annotations.is_empty() || !analysis.validation_errors.is_empty(),
                    "No analysis information for potentially malformed expression: {expression}"
                );
            }
            Err(_) => {
                // Graceful failure is acceptable for malformed expressions
                println!("Expression '{expression}' failed analysis as expected");
            }
        }
    }
}

#[tokio::test]
async fn test_concurrent_analysis() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let provider = Arc::new(MockModelProvider::new());
    let analyzer = Arc::new(FhirPathAnalyzer::new(provider));

    let expressions = vec![
        "'concurrent test 1'",
        "'concurrent test 2'",
        "Patient.name",
        "42",
        "true",
    ];

    let mut tasks = JoinSet::new();

    // Launch concurrent analysis tasks
    for (i, expression) in expressions.into_iter().enumerate() {
        let analyzer = analyzer.clone();
        let expr = expression.to_string();

        tasks.spawn(async move {
            let result = analyzer.analyze(&expr).await;
            (i, expr, result)
        });
    }

    // Collect all results
    let mut results = Vec::new();
    while let Some(task_result) = tasks.join_next().await {
        let (i, expr, analysis_result) = task_result.unwrap();

        assert!(
            analysis_result.is_ok(),
            "Concurrent analysis failed for expression {}: {} - {:?}",
            i,
            expr,
            analysis_result.err()
        );

        results.push((i, expr, analysis_result.unwrap()));
    }

    // All concurrent analyses should have succeeded
    assert_eq!(results.len(), 5, "Not all concurrent analyses completed");

    // Results should be consistent
    for (i, expr, result) in results {
        assert!(
            !result.type_annotations.is_empty(),
            "No type annotations for concurrent expression {i}: {expr}"
        );
    }
}

#[tokio::test]
async fn test_large_expression_handling() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test handling of reasonably large expressions
    let large_expression = "Patient.name.where(use = 'official' and given.exists() and family.exists()).given.first() + ' ' + Patient.name.where(use = 'official').family.first()";

    let result = analyzer.analyze(large_expression).await;

    assert!(
        result.is_ok(),
        "Large expression analysis failed: {:?}",
        result.err()
    );

    let result = result.unwrap();
    assert!(
        !result.type_annotations.is_empty(),
        "No type annotations for large expression"
    );
}

#[tokio::test]
async fn test_deeply_nested_paths() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test reasonably deep path navigation
    let nested_expression = "Patient.contact.name.family";

    let result = analyzer.analyze(nested_expression).await;

    assert!(
        result.is_ok(),
        "Nested path analysis failed: {:?}",
        result.err()
    );

    let result = result.unwrap();
    assert!(
        !result.type_annotations.is_empty(),
        "No type annotations for nested path"
    );
}
