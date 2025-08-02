//! Basic FHIRPath Usage Examples
//!
//! This example demonstrates the most common use cases for the FHIRPath library,
//! including parsing expressions, evaluating them against FHIR resources, and
//! handling results.

use octofhir_fhirpath::FhirPathValue;
use octofhir_fhirpath::engine::FhirPathEngine;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 FHIRPath Basic Usage Examples");
    println!("================================\n");

    // Create a FHIRPath engine
    let mut engine = FhirPathEngine::new();

    // Example FHIR Patient resource
    let patient_json = json!({
        "resourceType": "Patient",
        "id": "example-patient",
        "name": [
            {
                "use": "official",
                "family": "Smith",
                "given": ["John", "William"]
            },
            {
                "use": "nickname",
                "given": ["Johnny"]
            }
        ],
        "gender": "male",
        "birthDate": "1990-05-15",
        "active": true,
        "contact": [
            {
                "telecom": [
                    {
                        "system": "phone",
                        "value": "+1-555-123-4567",
                        "use": "home"
                    },
                    {
                        "system": "email",
                        "value": "john.smith@example.com",
                        "use": "work"
                    }
                ]
            }
        ]
    });

    println!("📋 Patient Resource:");
    println!("{}\n", serde_json::to_string_pretty(&patient_json)?);

    // Example 1: Simple property access
    println!("1️⃣  Simple Property Access");
    println!("----------------------------");

    let result = engine.evaluate("Patient.id", patient_json.clone())?;
    print_result("Patient.id", &result);

    // Example 2: Array access and filtering
    println!("2️⃣  Array Access and Filtering");
    println!("--------------------------------");

    let result = engine.evaluate("Patient.name.where(use = 'official')", patient_json.clone())?;
    print_result("Patient.name.where(use = 'official')", &result);

    // Example 3: Nested property access
    println!("3️⃣  Nested Property Access");
    println!("----------------------------");

    let result = engine.evaluate(
        "Patient.name.where(use = 'official').family",
        patient_json.clone(),
    )?;
    print_result("Patient.name.where(use = 'official').family", &result);

    // Example 4: Collection operations
    println!("4️⃣  Collection Operations");
    println!("---------------------------");

    let result = engine.evaluate("Patient.name.given.count()", patient_json.clone())?;
    print_result("Patient.name.given.count()", &result);

    // Example 5: Boolean expressions
    println!("5️⃣  Boolean Expressions");
    println!("-------------------------");

    let result = engine.evaluate("Patient.active = true", patient_json.clone())?;
    print_result("Patient.active = true", &result);

    // Example 6: String operations
    println!("6️⃣  String Operations");
    println!("----------------------");

    let result = engine.evaluate("Patient.name.where(use = 'official').given.first() + ' ' + Patient.name.where(use = 'official').family", patient_json.clone())?;
    print_result(
        "Patient.name.where(use = 'official').given.first() + ' ' + Patient.name.where(use = 'official').family",
        &result,
    );

    // Example 7: Type checking
    println!("7️⃣  Type Checking");
    println!("-------------------");

    let result = engine.evaluate("Patient.birthDate is System.String", patient_json.clone())?;
    print_result("Patient.birthDate is System.String", &result);

    // Example 8: Mathematical operations
    println!("8️⃣  Mathematical Operations");
    println!("----------------------------");

    let result = engine.evaluate("Patient.name.count() * 2", patient_json.clone())?;
    print_result("Patient.name.count() * 2", &result);

    // Example 9: Existence checking
    println!("9️⃣  Existence Checking");
    println!("-----------------------");

    let result = engine.evaluate(
        "Patient.contact.telecom.where(system = 'email').exists()",
        patient_json.clone(),
    )?;
    print_result(
        "Patient.contact.telecom.where(system = 'email').exists()",
        &result,
    );

    // Example 10: Complex expressions with select
    println!("🔟 Complex Expressions with Select");
    println!("------------------------------------");

    let result = engine.evaluate(
        "Patient.name.select(given.first() + ' ' + family)",
        patient_json.clone(),
    )?;
    print_result("Patient.name.select(given.first() + ' ' + family)", &result);

    Ok(())
}

/// Helper function to print FhirPathValue in a readable format
fn print_result(expression: &str, result: &FhirPathValue) {
    println!("Expression: {expression}");
    match result {
        FhirPathValue::Empty => println!("Result: (empty)"),
        FhirPathValue::Collection(items) if items.is_empty() => {
            println!("Result: (empty collection)")
        }
        FhirPathValue::Collection(items) if items.len() == 1 => {
            println!("Result: {:?}", items.get(0).unwrap());
        }
        _ => println!("Result: {result:?}"),
    }
    println!();
}
