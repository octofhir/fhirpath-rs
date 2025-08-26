//! Function signature validation example

use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 FHIRPath Analyzer - Function Validation Example");
    println!("═══════════════════════════════════════════════════");

    // Create analyzer with function registry
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Valid function calls
    println!("\n✅ Valid Function Calls:");
    println!("{}", "─".repeat(30));
    let valid_functions = vec![
        "empty()",
        "exists()",
        "count()",
        "first()",
        "last()",
        "single()",
        "distinct()",
    ];

    for func in valid_functions {
        match analyzer.analyze(func).await {
            Ok(result) => {
                if !result.function_calls.is_empty() {
                    let func_analysis = &result.function_calls[0];
                    println!("✅ {}: {}", func, func_analysis.signature.description);

                    if !result.validation_errors.is_empty() {
                        println!("  ⚠️  Warnings: {}", result.validation_errors.len());
                    }
                } else {
                    println!("✅ {func}: Parsed successfully");
                }
            }
            Err(e) => println!("❌ {func}: {e}"),
        }
    }

    // Invalid function calls
    println!("\n❌ Invalid Function Calls:");
    println!("{}", "─".repeat(30));
    let invalid_functions = vec![
        ("unknownFunction()", "Function not found"),
        ("count(42)", "Wrong parameter count"),
        ("substring('hello')", "Missing required parameters"),
        ("first(1, 2)", "Too many parameters"),
    ];

    for (func, expected_issue) in invalid_functions {
        match analyzer.analyze(func).await {
            Ok(result) => {
                println!("🔍 {func}: (Expected: {expected_issue})");

                for error in result.validation_errors {
                    let icon = match error.error_type {
                        ValidationErrorType::InvalidFunction => "🚫",
                        ValidationErrorType::TypeMismatch => "🔧",
                        _ => "⚠️ ",
                    };
                    println!("  {} {}", icon, error.message);

                    if !error.suggestions.is_empty() {
                        println!("    💡 Suggestions: {}", error.suggestions.join(", "));
                    }
                }
            }
            Err(e) => println!("❌ {func}: {e}"),
        }
    }

    // Parameter type validation examples
    println!("\n🔧 Parameter Type Examples:");
    println!("{}", "─".repeat(35));
    let type_validation_cases = vec![
        (
            "substring('hello', 1, 3)",
            "Valid string/integer parameters",
        ),
        ("contains('hello', 'world')", "Valid string parameters"),
        ("length('test')", "Valid string parameter"),
    ];

    for (func, description) in type_validation_cases {
        println!("🔍 Testing: {func} ({description})");
        match analyzer.analyze(func).await {
            Ok(result) => {
                let type_errors: Vec<_> = result
                    .validation_errors
                    .iter()
                    .filter(|e| matches!(e.error_type, ValidationErrorType::TypeMismatch))
                    .collect();

                if type_errors.is_empty() {
                    println!("  ✅ Type validation passed");
                } else {
                    for error in type_errors {
                        println!("  🔧 Type error: {}", error.message);
                    }
                }
            }
            Err(e) => println!("  ❌ Analysis error: {e}"),
        }
    }

    // Boolean operators
    println!("\n🔀 Boolean Operator Validation:");
    println!("{}", "─".repeat(35));
    let boolean_cases = vec![
        "true and false",
        "true or false",
        "not true",
        "true implies false",
        "true xor false",
    ];

    for expression in boolean_cases {
        match analyzer.analyze(expression).await {
            Ok(result) => {
                println!(
                    "✅ {}: {} type annotations",
                    expression,
                    result.type_annotations.len()
                );
                if !result.validation_errors.is_empty() {
                    for error in &result.validation_errors {
                        println!("  ⚠️  {}", error.message);
                    }
                }
            }
            Err(e) => println!("❌ {expression}: {e}"),
        }
    }

    println!("\n✅ Function validation example completed!");
    Ok(())
}
