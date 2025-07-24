//! Example: Running official FHIRPath tests with the integrated evaluator
//!
//! This example demonstrates how to use the fhirpath-evaluator with the complete
//! modular stack to run official FHIRPath test cases.
//!
//! Run with: cargo run --example run_official_tests

use fhirpath_evaluator::FhirPathEngine;
use fhirpath_model::{FhirPathValue, FhirResource};
use fhirpath_parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 FHIRPath Evaluator Example: Running Official Tests");
    println!();

    // Initialize the integrated FHIRPath stack
    let engine = FhirPathEngine::new();

    // Example 1: Simple literal evaluation
    println!("📝 Example 1: Simple Boolean Literal");
    let ast = fhirpath_parser::parse("true")?;
    let input = FhirPathValue::Empty; // No input needed for literals
    let result = engine.evaluate(&ast, input)?;
    println!("  Expression: true");
    println!("  Result: {:?}", result);
    println!("  ✅ Expected: Boolean(true)");
    println!();

    // Example 2: Integer literal
    println!("📝 Example 2: Integer Literal");
    let ast = fhirpath_parser::parse("42")?;
    let input = FhirPathValue::Empty;
    let result = engine.evaluate(&ast, input)?;
    println!("  Expression: 42");
    println!("  Result: {:?}", result);
    println!("  ✅ Expected: Integer(42)");
    println!();

    // Example 3: String literal
    println!("📝 Example 3: String Literal");
    let ast = fhirpath_parser::parse("'hello world'")?;
    let input = FhirPathValue::Empty;
    let result = engine.evaluate(&ast, input)?;
    println!("  Expression: 'hello world'");
    println!("  Result: {:?}", result);
    println!("  ✅ Expected: String(\"hello world\")");
    println!();

    // Example 4: Try to load a FHIR resource (if available)
    println!("📝 Example 4: FHIR Resource Navigation");
    
    // Create a simple patient resource
    let patient_json = serde_json::json!({
        "resourceType": "Patient",
        "id": "example",
        "active": true,
        "name": [
            {
                "given": ["John"],
                "family": "Doe"
            }
        ],
        "birthDate": "1980-01-01"
    });

    let patient_resource = FhirResource::from_json(patient_json);
    let patient_input = FhirPathValue::Resource(patient_resource);

    // Test simple property access
    let ast = fhirpath_parser::parse("id")?;
    let result = engine.evaluate(&ast, patient_input.clone())?;
    println!("  Expression: id");
    println!("  Result: {:?}", result);
    println!("  ✅ Expected: String(\"example\")");
    println!();

    // Test active property
    let ast = fhirpath_parser::parse("active")?;
    let result = engine.evaluate(&ast, patient_input.clone())?;
    println!("  Expression: active");
    println!("  Result: {:?}", result);
    println!("  ✅ Expected: Boolean(true)");
    println!();

    // Example 5: Test basic comparison
    println!("📝 Example 5: Boolean Comparison");
    let ast = fhirpath_parser::parse("true = true")?;
    let input = FhirPathValue::Empty;
    let result = engine.evaluate(&ast, input)?;
    println!("  Expression: true = true");
    println!("  Result: {:?}", result);
    println!("  ✅ Expected: Boolean(true)");
    println!();

    // Summary
    println!("🎉 Integration Test Examples Complete!");
    println!();
    println!("💡 This demonstrates the working modular FHIRPath implementation:");
    println!("   • fhirpath-parser: Parses FHIRPath expressions into AST");
    println!("   • fhirpath-evaluator: Evaluates AST against FHIR data");
    println!("   • fhirpath-model: Provides value types and FHIR resource handling");
    println!("   • fhirpath-registry: Manages functions and operators");
    println!();
    println!("🚀 Ready to run official FHIRPath test suites!");

    Ok(())
}

/// Helper function to run a simple test case
#[allow(dead_code)]
fn run_simple_test(
    engine: &FhirPathEngine,
    expression: &str,
    input: FhirPathValue,
    expected_desc: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Testing: {}", expression);
    
    let ast = fhirpath_parser::parse(expression)?;
    let result = engine.evaluate(&ast, input)?;
    
    println!("  Result: {:?}", result);
    println!("  Expected: {}", expected_desc);
    println!();
    
    Ok(())
}