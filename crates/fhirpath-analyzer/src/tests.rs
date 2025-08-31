// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unit tests for fhirpath-analyzer with enhanced bridge support

use super::*;
use octofhir_fhirpath_model::{FhirSchemaModelProvider, MockModelProvider};
use octofhir_fhirpath_registry::create_standard_registry;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use std::sync::Arc;

async fn create_test_context() -> (Arc<FhirSchemaPackageManager>, FhirPathAnalyzer) {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );

    let provider = Arc::new(
        FhirSchemaModelProvider::with_manager(manager.clone())
            .await
            .unwrap(),
    );
    let registry = Arc::new(create_standard_registry().await);

    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    (manager, analyzer)
}

#[tokio::test]
async fn test_field_validation_with_suggestions() {
    let (_manager, analyzer) = create_test_context().await;

    // Test valid field
    let valid_result = analyzer.analyze("Patient.name").await;
    assert!(valid_result.is_ok());

    let analysis = valid_result.unwrap();
    assert!(analysis.validation_errors.is_empty());

    // Test invalid field with suggestions
    let invalid_result = analyzer.analyze("Patient.nam").await; // Typo
    assert!(invalid_result.is_ok()); // Should still return analysis with errors

    let analysis = invalid_result.unwrap();
    let has_property_error = analysis
        .validation_errors
        .iter()
        .any(|e| matches!(e.error_type, ValidationErrorType::InvalidProperty));

    if has_property_error {
        // Should have suggestions for the typo
        let error_with_suggestions = analysis
            .validation_errors
            .iter()
            .find(|e| !e.suggestions.is_empty());
        assert!(error_with_suggestions.is_some());
    }
}

#[tokio::test]
async fn test_function_signature_validation() {
    let (_manager, analyzer) = create_test_context().await;

    // Test valid function call
    let valid_result = analyzer.analyze("Patient.name.count()").await;
    assert!(valid_result.is_ok());

    let analysis = valid_result.unwrap();
    assert!(analysis.validation_errors.is_empty());

    // Test invalid function
    let invalid_result = analyzer.analyze("Patient.invalidFunction()").await;
    assert!(invalid_result.is_ok());

    let analysis = invalid_result.unwrap();
    let has_function_error = analysis
        .validation_errors
        .iter()
        .any(|e| matches!(e.error_type, ValidationErrorType::InvalidFunction));
    assert!(has_function_error);
}

#[tokio::test]
async fn test_path_navigation_suggestions() {
    let (manager, _analyzer) = create_test_context().await;
    let navigator = AnalyzerPathNavigator::new(manager).await.unwrap();

    // Test path suggestions for partial input
    let suggestions = navigator.generate_path_suggestions("Patient", "nam").await;

    assert!(suggestions.is_ok());
    let suggestion_list = suggestions.unwrap();
    assert!(!suggestion_list.is_empty());

    let name_suggestion = suggestion_list
        .iter()
        .find(|s| s.base_suggestion.property_name.contains("name"));
    assert!(name_suggestion.is_some());

    if let Some(suggestion) = name_suggestion {
        assert!(suggestion.base_suggestion.confidence > 0.0);
    }
}

#[tokio::test]
async fn test_constraint_validation() {
    let (manager, _analyzer) = create_test_context().await;
    let constraint_analyzer = ConstraintAnalyzer::new(manager).await.unwrap();

    // Test valid constraint
    let valid_constraint = "Patient.name.exists()";
    let valid_result = constraint_analyzer
        .validate_constraint(valid_constraint, "Patient")
        .await;

    assert!(valid_result.is_ok());
    let validation = valid_result.unwrap();
    assert!(validation.is_valid);
    assert!(validation.violations.is_empty());

    // Test invalid constraint
    let invalid_constraint = "Patient.invalidField.exists()";
    let invalid_result = constraint_analyzer
        .validate_constraint(invalid_constraint, "Patient")
        .await;

    // Should still return a result, but with violations
    assert!(invalid_result.is_ok());
    let validation = invalid_result.unwrap();
    assert!(!validation.is_valid);
    assert!(!validation.violations.is_empty());
}

#[tokio::test]
async fn test_error_analysis_and_reporting() {
    let (manager, _analyzer) = create_test_context().await;
    let error_reporter = AnalyzerErrorReporter::new(manager).await.unwrap();

    let error = FhirPathError::PropertyNotFound {
        type_name: "Patient".to_string(),
        property: "invalidProperty".to_string(),
        source: None,
    };

    let analysis_context = AnalysisContext::default();
    let report = error_reporter
        .analyze_fhirpath_error("Patient.invalidProperty", &error, &analysis_context)
        .await;

    assert!(report.is_ok());
    let error_report = report.unwrap();

    assert!(!error_report.suggestions.is_empty());
    assert!(matches!(
        error_report.error_category,
        ErrorCategory::PropertyNotFound
    ));
    assert!(!error_report.related_documentation.is_empty());
}

#[tokio::test]
async fn test_bridge_field_validator() {
    let (manager, _analyzer) = create_test_context().await;
    let validator = AnalyzerFieldValidator::new(manager).await.unwrap();

    // Test valid field
    let valid_result = validator.validate_field("Patient", "name").await;
    assert!(valid_result.is_ok());

    let validation = valid_result.unwrap();
    assert!(validation.is_valid);
    assert!(validation.suggestions.is_empty());

    // Test invalid field with suggestions
    let invalid_result = validator.validate_field("Patient", "nam").await; // Typo
    assert!(invalid_result.is_ok());

    let validation = invalid_result.unwrap();
    assert!(!validation.is_valid);
    assert!(!validation.suggestions.is_empty());

    // Check that "name" is suggested for "nam"
    let name_suggested = validation.suggestions.iter().any(|s| s.contains("name"));
    assert!(name_suggested);
}

#[tokio::test]
async fn test_children_function_analysis() {
    let (_manager, analyzer) = create_test_context().await;

    // Test children() function analysis
    let result = analyzer.analyze("Patient.children()").await;
    assert!(result.is_ok());

    let analysis = result.unwrap();

    // Should have semantic information about union types from children()
    if !analysis.union_types.is_empty() {
        // Verify union type information is present
        let union_type = analysis.union_types.values().next().unwrap();
        assert!(!union_type.member_types.is_empty());
    }

    // Test children() with ofType filter
    let filtered_result = analyzer
        .analyze("Patient.children().ofType(HumanName)")
        .await;
    assert!(filtered_result.is_ok());

    let filtered_analysis = filtered_result.unwrap();
    // Should have type annotations
    assert!(!filtered_analysis.type_annotations.is_empty());
}

#[tokio::test]
async fn test_type_annotation_analysis() {
    let (_manager, analyzer) = create_test_context().await;

    // Test basic type annotation
    let result = analyzer.analyze("Patient.name.given.first()").await;
    assert!(result.is_ok());

    let analysis = result.unwrap();
    assert!(!analysis.type_annotations.is_empty());

    // The type annotations should include string type for given names
    let has_string_annotation = analysis
        .type_annotations
        .values()
        .any(|info| info.type_name.contains("string") || info.type_name.contains("String"));
    assert!(has_string_annotation);
}

#[tokio::test]
async fn test_semantic_info_generation() {
    let (_manager, analyzer) = create_test_context().await;

    // Test semantic info for complex expression
    let result = analyzer
        .analyze("Bundle.entry.resource.ofType(Patient).name")
        .await;
    assert!(result.is_ok());

    let analysis = result.unwrap();

    // Should have semantic information
    assert!(!analysis.type_annotations.is_empty());

    // Should understand Bundle navigation
    let has_bundle_semantics = analysis.type_annotations.values().any(|info| {
        info.semantic_context.contains("Bundle") || info.semantic_context.contains("navigation")
    });
    assert!(has_bundle_semantics);
}

#[tokio::test]
async fn test_performance_analysis() {
    let (_manager, analyzer) = create_test_context().await;

    // Test performance analysis for complex expressions
    let complex_expr =
        "Bundle.entry.resource.ofType(Patient).name.where(use='official').given.first()";

    let start = std::time::Instant::now();
    let result = analyzer.analyze(complex_expr).await;
    let analysis_time = start.elapsed();

    assert!(result.is_ok());

    // Analysis should complete quickly (within reasonable time for tests)
    assert!(
        analysis_time.as_millis() < 1000,
        "Analysis took too long: {}ms",
        analysis_time.as_millis()
    );

    let analysis = result.unwrap();
    assert!(!analysis.type_annotations.is_empty());
}

#[tokio::test]
async fn test_caching_performance() {
    let (_manager, analyzer) = create_test_context().await;

    let expression = "Patient.name.given.first()";

    // First analysis (cache miss)
    let start1 = std::time::Instant::now();
    let result1 = analyzer.analyze(expression).await;
    let time1 = start1.elapsed();

    assert!(result1.is_ok());

    // Second analysis (cache hit)
    let start2 = std::time::Instant::now();
    let result2 = analyzer.analyze(expression).await;
    let time2 = start2.elapsed();

    assert!(result2.is_ok());

    // Second analysis should be faster or at least not significantly slower
    // Allow for timing variance in test environments
    assert!(time2 <= time1 + std::time::Duration::from_millis(10));

    // Results should be equivalent
    let analysis1 = result1.unwrap();
    let analysis2 = result2.unwrap();

    assert_eq!(
        analysis1.type_annotations.len(),
        analysis2.type_annotations.len()
    );
    assert_eq!(
        analysis1.validation_errors.len(),
        analysis2.validation_errors.len()
    );
}

#[tokio::test]
async fn test_error_scenarios() {
    let (_manager, analyzer) = create_test_context().await;

    // Test malformed expression
    let malformed_result = analyzer.analyze("Patient.name.").await;
    // Should return analysis with errors rather than failing completely
    assert!(malformed_result.is_ok());

    let analysis = malformed_result.unwrap();
    assert!(!analysis.validation_errors.is_empty());

    // Test deeply nested invalid expression
    let deep_invalid = analyzer.analyze("Patient.invalidA.invalidB.invalidC").await;
    assert!(deep_invalid.is_ok());

    let deep_analysis = deep_invalid.unwrap();
    assert!(!deep_analysis.validation_errors.is_empty());

    // Should have suggestions for the first invalid property
    let has_suggestions = deep_analysis
        .validation_errors
        .iter()
        .any(|e| !e.suggestions.is_empty());
    assert!(has_suggestions);
}

#[tokio::test]
async fn test_concurrent_analysis() {
    let (_manager, analyzer) = create_test_context().await;
    let analyzer = Arc::new(analyzer);

    let expressions = vec![
        "Patient.name.given",
        "Patient.address.city",
        "Patient.telecom.value",
        "Patient.active",
        "Patient.birthDate",
    ];

    // Create concurrent analysis tasks
    let mut tasks = Vec::new();
    for (i, expr) in expressions.into_iter().enumerate() {
        let analyzer_clone = analyzer.clone();
        let task = tokio::spawn(async move {
            let result = analyzer_clone.analyze(expr).await;
            (i, result)
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // All tasks should complete successfully
    for result in results {
        let (i, analysis_result) = result.unwrap();
        assert!(analysis_result.is_ok(), "Task {} failed", i);

        let analysis = analysis_result.unwrap();
        // Should have meaningful analysis results
        assert!(!analysis.type_annotations.is_empty() || !analysis.validation_errors.is_empty());
    }
}

#[tokio::test]
async fn test_mock_provider_fallback() {
    // Test with mock provider for comparison
    let mock_provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await);
    let mock_analyzer = FhirPathAnalyzer::with_function_registry(mock_provider, registry);

    let result = mock_analyzer.analyze("Patient.name").await;
    assert!(result.is_ok());

    let analysis = result.unwrap();
    // Mock provider should still provide basic analysis
    assert!(!analysis.type_annotations.is_empty());
}

#[tokio::test]
async fn test_complex_choice_type_analysis() {
    let (_manager, analyzer) = create_test_context().await;

    // Test choice type analysis
    let choice_result = analyzer.analyze("Observation.value[x]").await;
    assert!(choice_result.is_ok());

    let analysis = choice_result.unwrap();

    // Should have type information about choice types
    let has_choice_info = analysis
        .type_annotations
        .values()
        .any(|info| info.semantic_context.contains("choice") || info.type_name.contains("[x]"));
    assert!(has_choice_info);

    // Test specific choice type
    let specific_choice = analyzer.analyze("Observation.valueString").await;
    assert!(specific_choice.is_ok());

    let specific_analysis = specific_choice.unwrap();
    assert!(!specific_analysis.type_annotations.is_empty());
}

#[tokio::test]
async fn test_mathematical_expression_analysis() {
    let (_manager, analyzer) = create_test_context().await;

    // Test mathematical operations analysis
    let math_result = analyzer
        .analyze("Bundle.entry.resource.ofType(Observation).valueQuantity.value.sum()")
        .await;
    assert!(math_result.is_ok());

    let analysis = math_result.unwrap();
    assert!(!analysis.type_annotations.is_empty());

    // Should understand that sum() returns a decimal
    let has_numeric_info = analysis.type_annotations.values().any(|info| {
        info.type_name.contains("decimal")
            || info.type_name.contains("Decimal")
            || info.type_name.contains("number")
    });
    assert!(has_numeric_info);
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_analysis_pipeline() {
        let (manager, analyzer) = create_test_context().await;

        // Test complete analysis pipeline with all components
        let complex_expression = "Bundle.entry.resource.ofType(Patient).name.where(use='official').given.first().upper()";

        let result = analyzer.analyze(complex_expression).await;
        assert!(result.is_ok());

        let analysis = result.unwrap();

        // Should have comprehensive analysis results
        assert!(!analysis.type_annotations.is_empty());

        // Test individual components work together
        let field_validator = AnalyzerFieldValidator::new(manager.clone()).await.unwrap();
        let path_navigator = AnalyzerPathNavigator::new(manager.clone()).await.unwrap();
        let error_reporter = AnalyzerErrorReporter::new(manager).await.unwrap();

        // All components should be functional
        let field_result = field_validator.validate_field("Patient", "name").await;
        assert!(field_result.is_ok());

        let path_result = path_navigator
            .generate_path_suggestions("Patient", "nam")
            .await;
        assert!(path_result.is_ok());

        // Test error reporting with mock error
        let mock_error = FhirPathError::PropertyNotFound {
            type_name: "Patient".to_string(),
            property: "invalid".to_string(),
            source: None,
        };

        let analysis_context = AnalysisContext::default();
        let error_result = error_reporter
            .analyze_fhirpath_error("Patient.invalid", &mock_error, &analysis_context)
            .await;
        assert!(error_result.is_ok());
    }
}
