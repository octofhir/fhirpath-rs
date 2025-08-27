use clap::{Arg, Command};
use octofhir_fhirpath::FhirPathEngineWithAnalyzer;
use octofhir_fhirpath_model::FhirSchemaModelProvider;
use octofhir_fhirpath_registry::create_standard_registry;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("fhirpath-analyze")
        .about("Analyze FHIRPath expressions with comprehensive FHIR field validation using FhirSchemaModelProvider")
        .arg(
            Arg::new("expression")
                .help("FHIRPath expression to analyze")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("validate-only")
                .long("validate-only")
                .help("Only validate, don't analyze types")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-inference")
                .long("no-inference")
                .help("Disable type inference")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let expression = matches.get_one::<String>("expression").unwrap();
    let validate_only = matches.get_flag("validate-only");
    let no_inference = matches.get_flag("no-inference");

    // Always use FhirSchemaModelProvider for comprehensive field validation
    println!("🔧 Initializing FhirSchemaModelProvider for comprehensive field validation...");

    let model_provider: Box<dyn octofhir_fhirpath_model::provider::ModelProvider> =
        match FhirSchemaModelProvider::new().await {
            Ok(provider) => {
                println!("✅ FhirSchemaModelProvider initialized successfully");
                Box::new(provider)
            }
            Err(e) => {
                eprintln!(
                    "❌ CRITICAL: Failed to create FhirSchemaModelProvider: {}",
                    e
                );
                eprintln!("💡 This is required for proper FHIR field validation.");
                eprintln!("🔧 Please ensure FHIR schema data is available and try again.");
                std::process::exit(1);
            }
        };

    let function_registry = Arc::new(create_standard_registry().await);
    let engine =
        FhirPathEngineWithAnalyzer::with_full_analysis(model_provider, function_registry).await?;

    if validate_only {
        // Validation only
        let errors = engine.validate_expression(expression).await?;
        if errors.is_empty() {
            println!("✅ Expression is valid");
        } else {
            println!("❌ Validation errors:");
            for error in errors {
                println!("  - {} (type: {:?})", error.message, error.error_type);
                if !error.suggestions.is_empty() {
                    println!("    Suggestions: {}", error.suggestions.join(", "));
                }
            }
        }
    } else {
        // Full analysis
        if let Some(analysis) = engine.analyze_expression(expression).await? {
            println!("📊 Analysis Results for: {expression}");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            if !analysis.validation_errors.is_empty() {
                println!("\n❌ Validation Errors:");
                for error in analysis.validation_errors {
                    let icon = match error.error_type {
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidField => "🔍",
                        octofhir_fhirpath_analyzer::ValidationErrorType::DeprecatedField => "⚠️",
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidResourceType => {
                            "🏥"
                        }
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidFunction => "🔧",
                        _ => "❗",
                    };
                    println!("  {} {}", icon, error.message);
                    if !error.suggestions.is_empty() {
                        println!("    💡 Suggestions: {}", error.suggestions.join(", "));
                    }
                }
            }

            if !no_inference && !analysis.type_annotations.is_empty() {
                println!("\n🔍 Type Annotations:");
                for (node_id, semantic_info) in analysis.type_annotations {
                    println!("  Node {node_id}: ");
                    if let Some(fhir_type) = semantic_info.fhir_path_type {
                        println!("    FHIRPath Type: {fhir_type}");
                    }
                    if let Some(model_type) = semantic_info.model_type {
                        println!("    FHIR Model Type: {model_type}");
                    }
                    println!("    Cardinality: {:?}", semantic_info.cardinality);
                    println!("    Confidence: {:?}", semantic_info.confidence);
                }
            }

            if !analysis.function_calls.is_empty() {
                println!("\n🔧 Function Calls:");
                for func_analysis in analysis.function_calls {
                    println!(
                        "  - {} ({})",
                        func_analysis.function_name, func_analysis.signature.description
                    );
                    if !func_analysis.validation_errors.is_empty() {
                        for error in func_analysis.validation_errors {
                            println!("    ⚠️  {}", error.message);
                        }
                    }
                }
            }

            println!("\n✅ Analysis complete");
        } else {
            println!("⚠️  No analyzer available");
        }
    }

    Ok(())
}
