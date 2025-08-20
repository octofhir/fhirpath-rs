//! Basic analysis example showing type inference and validation

use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, types::ConfidenceLevel};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 FHIRPath Analyzer - Basic Analysis Example");
    println!("═════════════════════════════════════════════");

    // Create analyzer
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);

    // Test cases demonstrating different analysis capabilities
    let test_cases = vec![
        ("'hello world'", "String literal analysis"),
        ("42", "Integer literal analysis"),
        ("3.14", "Decimal literal analysis"),
        ("true", "Boolean literal analysis"),
        ("Patient", "Identifier analysis"),
        ("Patient.name", "Path navigation analysis"),
        ("Patient.name.given", "Complex path analysis"),
        ("not true", "Boolean not operator"),
        ("true and false", "Boolean and operator"),
    ];

    for (expression, description) in test_cases {
        println!("\n📊 {description}: {expression}");
        println!("{}", "─".repeat(50));

        match analyzer.analyze(expression).await {
            Ok(result) => {
                // Show type annotations
                if !result.type_annotations.is_empty() {
                    println!("🔍 Type Annotations:");
                    for (node_id, semantic_info) in result.type_annotations {
                        println!("  Node {node_id}: ");
                        if let Some(fhir_type) = semantic_info.fhir_path_type {
                            println!("    FHIRPath Type: {fhir_type}");
                        }
                        if let Some(model_type) = semantic_info.model_type {
                            println!("    Model Type: {model_type}");
                        }
                        println!("    Cardinality: {:?}", semantic_info.cardinality);

                        let confidence_icon = match semantic_info.confidence {
                            ConfidenceLevel::High => "🟢",
                            ConfidenceLevel::Medium => "🟡",
                            ConfidenceLevel::Low => "🔴",
                        };
                        println!(
                            "    Confidence: {} {:?}",
                            confidence_icon, semantic_info.confidence
                        );
                    }
                }

                // Show validation results
                if result.validation_errors.is_empty() {
                    println!("✅ No validation errors");
                } else {
                    println!("❌ Validation Errors:");
                    for error in result.validation_errors {
                        println!("  - {}", error.message);
                        if !error.suggestions.is_empty() {
                            println!("    Suggestions: {}", error.suggestions.join(", "));
                        }
                    }
                }

                // Show function call analysis
                if !result.function_calls.is_empty() {
                    println!("🔧 Function Calls:");
                    for func_analysis in result.function_calls {
                        println!(
                            "  - {} -> {:?}",
                            func_analysis.function_name, func_analysis.return_type
                        );
                    }
                }
            }
            Err(e) => {
                println!("❌ Analysis failed: {e}");
            }
        }
    }

    println!("\n✅ Basic analysis example completed!");
    Ok(())
}
