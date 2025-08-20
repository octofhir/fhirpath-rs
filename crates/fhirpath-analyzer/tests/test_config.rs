//! Configuration tests for FHIRPath analyzer

use octofhir_fhirpath_analyzer::{
    AnalysisContext, AnalysisSettings, AnalyzerConfig, FhirPathAnalyzer,
};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_analyzer_configuration() {
    let provider = Arc::new(MockModelProvider::new());

    // Test with type inference disabled
    let config_no_inference = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: false,
            enable_function_validation: true,
            enable_union_analysis: true,
            max_analysis_depth: 50,
        },
        cache_size: 1000,
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider.clone(), config_no_inference);
    let result = analyzer.analyze("'hello world'").await.unwrap();

    // With type inference disabled, should still have basic analysis
    assert!(
        !result.type_annotations.is_empty(),
        "Should have some analysis even with type inference disabled"
    );

    // Test with function validation disabled
    let config_no_functions = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: false,
            enable_union_analysis: true,
            max_analysis_depth: 50,
        },
        cache_size: 1000,
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider, config_no_functions);
    let result = analyzer.analyze("count()").await.unwrap();

    // Should have analysis results
    assert!(
        !result.type_annotations.is_empty(),
        "Should have type analysis for function expression"
    );

    // Test completed successfully
    assert!(true);
}

#[tokio::test]
async fn test_cache_size_configuration() {
    let provider = Arc::new(MockModelProvider::new());

    // Test with small cache size
    let small_cache_config = AnalyzerConfig {
        settings: AnalysisSettings::default(),
        cache_size: 10, // Very small cache
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider.clone(), small_cache_config);

    // Perform more analyses than cache size to test cache eviction
    let expressions = vec![
        "'test1'", "'test2'", "'test3'", "'test4'", "'test5'", "'test6'", "'test7'", "'test8'",
        "'test9'", "'test10'", "'test11'", "'test12'", "'test13'", "'test14'", "'test15'",
    ];

    for expression in expressions {
        let result = analyzer.analyze(expression).await;
        assert!(
            result.is_ok(),
            "Analysis failed for expression: {expression}"
        );
    }

    // Test with large cache size
    let large_cache_config = AnalyzerConfig {
        settings: AnalysisSettings::default(),
        cache_size: 10000,
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider, large_cache_config);

    // Should work normally with large cache
    let result = analyzer.analyze("Patient.name").await.unwrap();
    assert!(!result.type_annotations.is_empty());
}

#[tokio::test]
async fn test_analysis_depth_configuration() {
    let provider = Arc::new(MockModelProvider::new());

    // Test with limited analysis depth
    let shallow_config = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: true,
            enable_union_analysis: true,
            max_analysis_depth: 3, // Very shallow
        },
        cache_size: 1000,
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider.clone(), shallow_config);

    // Test with a reasonably deep expression
    let deep_expression = "Patient.name.family";
    let result = analyzer.analyze(deep_expression).await;

    assert!(
        result.is_ok(),
        "Deep expression analysis failed with shallow config: {:?}",
        result.err()
    );

    // Test with unlimited depth
    let deep_config = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: true,
            enable_union_analysis: true,
            max_analysis_depth: 1000, // Very deep
        },
        cache_size: 1000,
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider, deep_config);
    let result = analyzer.analyze(deep_expression).await;

    assert!(
        result.is_ok(),
        "Deep expression analysis failed with deep config: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_profiling_configuration() {
    let provider = Arc::new(MockModelProvider::new());

    // Test with profiling enabled
    let profiling_config = AnalyzerConfig {
        settings: AnalysisSettings::default(),
        cache_size: 1000,
        enable_profiling: true,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider.clone(), profiling_config);

    // Perform analysis with profiling enabled
    let result = analyzer.analyze("Patient.name.given").await;
    assert!(result.is_ok(), "Analysis failed with profiling enabled");

    // Test with profiling disabled
    let no_profiling_config = AnalyzerConfig {
        settings: AnalysisSettings::default(),
        cache_size: 1000,
        enable_profiling: false,
    };

    let analyzer = FhirPathAnalyzer::with_config(provider, no_profiling_config);
    let result = analyzer.analyze("Patient.name.given").await;
    assert!(result.is_ok(), "Analysis failed with profiling disabled");
}

#[tokio::test]
async fn test_analysis_context_configuration() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test with different analysis contexts
    let context = AnalysisContext {
        root_type: Some("Patient".to_string()),
        variables: std::collections::HashMap::new(),
        environment: std::collections::HashMap::new(),
        settings: AnalysisSettings::default(),
    };

    // Analyze with specific context
    let result = analyzer.analyze_with_context("given", &context).await;
    assert!(
        result.is_ok(),
        "Context-specific analysis failed: {:?}",
        result.err()
    );

    let result = result.unwrap();
    assert!(
        !result.type_annotations.is_empty(),
        "No type annotations with specific context"
    );
}

#[tokio::test]
async fn test_settings_combinations() {
    let provider = Arc::new(MockModelProvider::new());

    // Test various combinations of settings
    let configs = vec![
        AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: true,
            enable_union_analysis: true,
            max_analysis_depth: 100,
        },
        AnalysisSettings {
            enable_type_inference: false,
            enable_function_validation: true,
            enable_union_analysis: false,
            max_analysis_depth: 50,
        },
        AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: false,
            enable_union_analysis: true,
            max_analysis_depth: 25,
        },
    ];

    for (i, settings) in configs.into_iter().enumerate() {
        let config = AnalyzerConfig {
            settings,
            cache_size: 1000,
            enable_profiling: false,
        };

        let analyzer = FhirPathAnalyzer::with_config(provider.clone(), config);

        // Test with a variety of expressions
        let test_expressions = vec!["'string literal'", "42", "Patient.name", "count()"];

        for expression in test_expressions {
            let result = analyzer.analyze(expression).await;

            assert!(
                result.is_ok(),
                "Config {} failed for expression '{}': {:?}",
                i,
                expression,
                result.err()
            );
        }
    }
}
