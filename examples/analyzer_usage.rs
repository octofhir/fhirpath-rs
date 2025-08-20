use octofhir_fhirpath::{
    AnalysisSettings, AnalyzerConfig, FhirPathEngineWithAnalyzer, MockModelProvider,
};
use sonic_rs::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic usage without analyzer (existing behavior)
    println!("=== Example 1: Standard Engine (No Analysis) ===");
    let model_provider = Box::new(MockModelProvider::new());
    let engine = FhirPathEngineWithAnalyzer::new(model_provider).await?;

    let patient = json!({"resourceType": "Patient", "name": [{"given": ["John"]}]});
    let result = engine.evaluate("Patient.name.given", patient).await?;
    println!("Result: {result:?}");

    // Example 2: Engine with analyzer enabled
    println!("\n=== Example 2: Engine with Analysis ===");
    let model_provider = Box::new(MockModelProvider::new());
    let engine = FhirPathEngineWithAnalyzer::with_analyzer(model_provider).await?;

    let patient = json!({"resourceType": "Patient", "name": [{"given": ["John"]}]});
    let (result, analysis) = engine
        .evaluate_with_analysis("Patient.name.given", patient)
        .await?;

    println!("Evaluation result: {result:?}");
    if let Some(analysis) = analysis {
        println!("Analysis result:");
        println!("  - Type annotations: {}", analysis.type_annotations.len());
        println!(
            "  - Validation errors: {}",
            analysis.validation_errors.len()
        );
        println!("  - Function calls: {}", analysis.function_calls.len());
    }

    // Example 3: Pre-validation without evaluation
    println!("\n=== Example 3: Expression Validation ===");
    let validation_errors = engine
        .validate_expression("Patient.invalidProperty")
        .await?;
    if !validation_errors.is_empty() {
        println!("Validation errors found:");
        for error in validation_errors {
            println!("  - {}", error.message);
        }
    } else {
        println!("Expression is valid");
    }

    // Example 4: Analysis-only (no evaluation)
    println!("\n=== Example 4: Analysis Only ===");
    if let Some(analysis) = engine.analyze_expression("'hello world'").await? {
        println!("Analysis for string literal:");
        for (node_id, semantic_info) in analysis.type_annotations {
            println!(
                "  Node {}: type={:?}, confidence={:?}",
                node_id, semantic_info.fhir_path_type, semantic_info.confidence
            );
        }
    }

    // Example 5: Custom analyzer configuration
    println!("\n=== Example 5: Custom Configuration ===");
    let custom_config = AnalyzerConfig {
        settings: AnalysisSettings {
            enable_type_inference: true,
            enable_function_validation: true,
            enable_union_analysis: false, // Disable for performance
            max_analysis_depth: 50,
        },
        cache_size: 5000,
        enable_profiling: true,
    };

    let model_provider = Box::new(MockModelProvider::new());
    let engine =
        FhirPathEngineWithAnalyzer::with_analyzer_config(model_provider, custom_config).await?;

    let (result, analysis) = engine.evaluate_with_analysis("42 + 58", json!({})).await?;
    println!("Result: {result:?}");
    if let Some(analysis) = analysis {
        println!(
            "Custom analysis completed with {} type annotations",
            analysis.type_annotations.len()
        );
    }

    Ok(())
}
