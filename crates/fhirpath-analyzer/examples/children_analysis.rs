//! Children function and union type analysis example

use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("👶 FHIRPath Analyzer - Children Function Analysis Example");
    println!("════════════════════════════════════════════════════════");

    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Basic children() analysis
    println!("\n🔍 Basic children() Analysis:");
    println!("{}", "─".repeat(35));

    let children_expressions = vec![
        "Patient.children()",
        "Observation.children()",
        "Bundle.children()",
    ];

    for expression in children_expressions {
        println!("📊 Analyzing: {expression}");
        match analyzer.analyze(expression).await {
            Ok(result) => {
                // Check for function analysis
                if let Some(children_func) = result
                    .function_calls
                    .iter()
                    .find(|f| f.function_name == "children")
                {
                    println!("  ✅ children() function recognized");
                    println!("  📝 Description: {}", children_func.signature.description);
                }

                // Check for union type information
                if !result.union_types.is_empty() {
                    println!("  🔗 Union types found: {}", result.union_types.len());
                    for (node_id, union_type) in result.union_types {
                        println!(
                            "    Node {}: {} constituent types",
                            node_id,
                            union_type.constituent_types.len()
                        );
                        if union_type.is_collection {
                            println!("    📦 Returns collection");
                        }

                        // Show model context if available
                        if !union_type.model_context.is_empty() {
                            println!("    📄 Model context: {:?}", union_type.model_context);
                        }
                    }
                }

                // Check validation
                if result.validation_errors.is_empty() {
                    println!("  ✅ No validation errors");
                } else {
                    for error in result.validation_errors {
                        println!("  ⚠️  {}", error.message);
                    }
                }
            }
            Err(e) => println!("  ❌ Analysis failed: {e}"),
        }
        println!();
    }

    // Type filtering on children
    println!("🎯 Type Filtering Analysis:");
    println!("{}", "─".repeat(30));

    let filtering_expressions = vec![
        ("Patient.children().ofType(HumanName)", "Valid type filter"),
        (
            "Patient.children().ofType(InvalidType)",
            "Invalid type filter",
        ),
        ("Observation.children() is Quantity", "Type check operation"),
        ("Bundle.children() as Patient", "Type cast operation"),
    ];

    for (expression, description) in filtering_expressions {
        println!("🔍 Testing: {expression} ({description})");
        match analyzer.analyze(expression).await {
            Ok(result) => {
                // Check for type operation errors
                let type_op_errors: Vec<_> = result
                    .validation_errors
                    .iter()
                    .filter(|e| matches!(e.error_type, ValidationErrorType::InvalidTypeOperation))
                    .collect();

                if type_op_errors.is_empty() {
                    println!("  ✅ Type filtering validation passed");
                } else {
                    for error in type_op_errors {
                        println!("  🎯 Type operation error: {}", error.message);
                        if !error.suggestions.is_empty() {
                            println!("    💡 Valid types: {}", error.suggestions.join(", "));
                        }
                    }
                }

                // Show union type analysis
                if !result.union_types.is_empty() {
                    println!("  🔗 Union type analysis available");
                }

                // Show function calls
                if !result.function_calls.is_empty() {
                    println!("  🔧 Function calls: {}", result.function_calls.len());
                }
            }
            Err(e) => println!("  ❌ Analysis error: {e}"),
        }
        println!();
    }

    // Invalid children() usage
    println!("❌ Invalid children() Usage:");
    println!("{}", "─".repeat(30));

    let invalid_cases = vec![
        "Patient.children('invalid')",
        "Patient.children(1, 2)",
        "children()", // Without base object
    ];

    for expression in invalid_cases {
        println!("🔍 Testing invalid case: {expression}");
        match analyzer.analyze(expression).await {
            Ok(result) => {
                let has_errors = !result.validation_errors.is_empty();
                if has_errors {
                    for error in result.validation_errors {
                        let icon = match error.error_type {
                            ValidationErrorType::InvalidFunction => "🚫",
                            ValidationErrorType::TypeMismatch => "🔧",
                            ValidationErrorType::InvalidTypeOperation => "📝",
                            _ => "⚠️ ",
                        };
                        println!("  {} {}", icon, error.message);
                        if !error.suggestions.is_empty() {
                            println!("    💡 Suggestions: {}", error.suggestions.join(", "));
                        }
                    }
                } else {
                    println!("  ⚠️  No validation errors detected (may need improvement)");
                }
            }
            Err(e) => println!("  ❌ Analysis error: {e}"),
        }
        println!();
    }

    // Complex children analysis
    println!("🔬 Complex Children Analysis:");
    println!("{}", "─".repeat(35));

    let complex_cases = vec![
        "Patient.children().where($this is HumanName)",
        "Patient.children().select($this as HumanName)",
        "Patient.children().count()",
        "Patient.children().first()",
    ];

    for expression in complex_cases {
        println!("🔬 Analyzing: {expression}");
        match analyzer.analyze(expression).await {
            Ok(result) => {
                println!("  📊 Type annotations: {}", result.type_annotations.len());
                println!("  🔧 Function calls: {}", result.function_calls.len());
                println!("  🔗 Union types: {}", result.union_types.len());
                println!(
                    "  ⚠️  Validation errors: {}",
                    result.validation_errors.len()
                );

                if !result.validation_errors.is_empty() {
                    for error in &result.validation_errors {
                        println!("    - {}", error.message);
                    }
                }
            }
            Err(e) => println!("  ❌ Analysis failed: {e}"),
        }
        println!();
    }

    println!("✅ Children function analysis example completed!");
    Ok(())
}
