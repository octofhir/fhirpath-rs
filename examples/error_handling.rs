//! Error Handling Examples
//!
//! This example demonstrates how to handle various types of errors that can occur
//! when working with FHIRPath expressions, including parse errors, evaluation errors,
//! and data type mismatches.

use octofhir_fhirpath::engine::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  FHIRPath Error Handling Examples");
    println!("=====================================\n");

    let mut engine = FhirPathEngine::with_mock_provider();

    // Example FHIR data
    let patient = json!({
        "resourceType": "Patient",
        "id": "example-patient",
        "name": [
            {
                "use": "official",
                "family": "Smith",
                "given": ["John", "William"]
            }
        ],
        "birthDate": "1990-05-15",
        "active": true
    });

    println!("1️⃣  Handling Parse Errors");
    println!("==========================");

    // Example 1: Syntax errors in expressions
    let invalid_expressions = [
        "Patient.name.where(",                 // Unclosed parenthesis
        "Patient.name..family",                // Double dot
        "Patient.name.where(use =)",           // Incomplete comparison
        "Patient.name.where(use = 'official'", // Missing closing parenthesis
        "Patient.[name",                       // Invalid bracket syntax
    ];

    for expr in &invalid_expressions {
        println!("Testing invalid expression: {expr}");
        match engine.evaluate(expr, patient.clone()).await {
            Ok(result) => {
                println!("  ✅ Parsed successfully (returned empty per FHIRPath spec): {result:?}");
            }
            Err(error) => {
                println!("  ❌ Parse error: {error}");
            }
        }
        println!();
    }

    println!("2️⃣  Handling Evaluation Errors");
    println!("===============================");

    // Example 2: Type mismatches and invalid operations
    let problematic_expressions = [
        "Patient.name + Patient.birthDate",  // String + Date
        "Patient.active * 5",                // Boolean * Number
        "Patient.name.unknownFunction()",    // Unknown function
        "Patient.nonExistentProperty.value", // Non-existent property
    ];

    for expr in &problematic_expressions {
        println!("Testing problematic expression: {expr}");
        match engine.evaluate(expr, patient.clone()).await {
            Ok(result) => {
                println!("  ✅ Evaluated successfully: {result:?}");
            }
            Err(error) => {
                println!("  ❌ Evaluation error: {error}");
            }
        }
        println!();
    }

    println!("3️⃣  Handling Data Type Issues");
    println!("==============================");

    // Example 3: Working with missing or null data
    let incomplete_patient = json!({
        "resourceType": "Patient",
        "id": "incomplete-patient"
        // Missing name, birthDate, etc.
    });

    let expressions_for_missing_data = [
        "Patient.name.family",                     // Missing name
        "Patient.birthDate",                       // Missing birthDate
        "Patient.telecom.where(system = 'email')", // Missing telecom
        "Patient.contact.name.family",             // Deeply nested missing data
    ];

    for expr in &expressions_for_missing_data {
        println!("Testing expression with missing data: {expr}");
        match engine.evaluate(expr, incomplete_patient.clone()).await {
            Ok(result) => {
                println!("  ✅ Handled gracefully: {result:?}");
            }
            Err(error) => {
                println!("  ❌ Error: {error}");
            }
        }
        println!();
    }

    println!("4️⃣  Best Practices for Error Handling");
    println!("======================================");

    // Example 4: Defensive programming with existence checks
    let defensive_expressions = [
        "Patient.name.exists() and Patient.name.family.exists()",
        "iif(Patient.birthDate.exists(), Patient.birthDate, 'Unknown')",
        "Patient.telecom.where(system = 'email').exists().not() or Patient.telecom.where(system = 'email').value",
    ];

    for expr in &defensive_expressions {
        println!("Defensive expression: {expr}");
        match engine.evaluate(expr, patient.clone()).await {
            Ok(result) => {
                println!("  ✅ Result: {result:?}");
            }
            Err(error) => {
                println!("  ❌ Error: {error}");
            }
        }
        println!();
    }

    println!("5️⃣  Error Recovery Strategies");
    println!("==============================");

    // Demonstrate different error recovery strategies
    let expressions_to_test = [
        ("Patient.name.family", "Basic property access"),
        ("Patient.invalidProperty", "Invalid property access"),
        ("Patient.name.invalidMethod()", "Invalid method call"),
    ];

    for (expr, description) in &expressions_to_test {
        println!("{description}: {expr}");

        // Strategy 1: Default to empty result
        let result = engine
            .evaluate(expr, patient.clone())
            .await
            .unwrap_or_else(|_| octofhir_fhirpath::FhirPathValue::collection(vec![]));
        println!("  Strategy 1 (default empty): {result:?}");

        // Strategy 2: Try alternative expression
        if expr.contains("invalidProperty") {
            let alternative = "Patient.id"; // Use a known property instead
            match engine.evaluate(alternative, patient.clone()).await {
                Ok(alt_result) => println!("  Strategy 2 (alternative): {alt_result:?}"),
                Err(_) => println!("  Strategy 2 (alternative): Failed"),
            }
        }

        // Strategy 3: Validate before evaluation
        if expr.len() > 100 {
            println!("  Strategy 3: Expression too long, skipping");
        } else if !expr.starts_with("Patient.") {
            println!("  Strategy 3: Expression doesn't start with 'Patient.', potentially unsafe");
        } else {
            println!("  Strategy 3: Expression appears valid for evaluation");
        }

        println!();
    }

    println!("✅ Error handling examples completed!");
    println!("Remember to always handle errors gracefully in production code.");

    Ok(())
}
