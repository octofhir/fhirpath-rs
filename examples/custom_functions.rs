//! Custom Function Examples
//!
//! This example demonstrates how to extend the FHIRPath engine with custom functions.
//! Note: This is a conceptual example showing how custom functions might be implemented.
//! The actual API may vary based on the current implementation.

use octofhir_fhirpath::engine::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 FHIRPath Custom Functions Examples");
    println!("======================================\n");

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
        "telecom": [
            {
                "system": "email",
                "value": "john.smith@example.com",
                "use": "work"
            },
            {
                "system": "phone",
                "value": "+1-555-123-4567",
                "use": "home"
            }
        ]
    });

    println!("📋 Working with Standard Functions");
    println!("===================================\n");

    // Example 1: Built-in string functions
    println!("1️⃣  String Functions");
    println!("--------------------");

    let result = engine
        .evaluate("Patient.name.family.upper()", patient.clone())
        .await?;
    print_result("Patient.name.family.upper()", &result);

    let result = engine
        .evaluate("Patient.name.family.lower()", patient.clone())
        .await?;
    print_result("Patient.name.family.lower()", &result);

    let result = engine
        .evaluate(
            "Patient.telecom.where(system = 'email').value.contains('@')",
            patient.clone(),
        )
        .await?;
    print_result(
        "Patient.telecom.where(system = 'email').value.contains('@')",
        &result,
    );

    // Example 2: Mathematical functions
    println!("2️⃣  Mathematical Functions");
    println!("---------------------------");

    let result = engine
        .evaluate("Patient.name.count() * 10", patient.clone())
        .await?;
    print_result("Patient.name.count() * 10", &result);

    // Example 3: Date functions
    println!("3️⃣  Date Functions");
    println!("------------------");

    let result = engine
        .evaluate("Patient.birthDate", patient.clone())
        .await?;
    print_result("Patient.birthDate", &result);

    // Example 4: Type conversion functions
    println!("4️⃣  Type Conversion Functions");
    println!("------------------------------");

    let result = engine
        .evaluate("Patient.name.count().toString()", patient.clone())
        .await?;
    print_result("Patient.name.count().toString()", &result);

    // Example 5: Aggregate functions
    println!("5️⃣  Aggregate Functions");
    println!("-----------------------");

    let bundle = json!({
        "resourceType": "Bundle",
        "entry": [
            {
                "resource": {
                    "resourceType": "Observation",
                    "valueQuantity": { "value": 75.5 }
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "valueQuantity": { "value": 80.2 }
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "valueQuantity": { "value": 78.9 }
                }
            }
        ]
    });

    let result = engine
        .evaluate(
            "Bundle.entry.resource.valueQuantity.value.sum()",
            bundle.clone(),
        )
        .await?;
    print_result("Bundle.entry.resource.valueQuantity.value.sum()", &result);

    let result = engine
        .evaluate(
            "Bundle.entry.resource.valueQuantity.value.avg()",
            bundle.clone(),
        )
        .await?;
    print_result("Bundle.entry.resource.valueQuantity.value.avg()", &result);

    // Example 6: Advanced functions
    println!("6️⃣  Advanced Functions");
    println!("-----------------------");

    let result = engine
        .evaluate(
            "Bundle.entry.resource.valueQuantity.value.distinct().count()",
            bundle.clone(),
        )
        .await?;
    print_result(
        "Bundle.entry.resource.valueQuantity.value.distinct().count()",
        &result,
    );

    println!("\n📝 Custom Function Implementation");
    println!("==================================");
    println!("Custom functions would require extending the FunctionRegistry.");
    println!("This involves implementing the Function trait and registering");
    println!("the function with the engine's registry.\n");

    println!("Example custom function structure:");
    println!("----------------------------------");
    println!(
        r#"
// Pseudo-code for custom function implementation:
struct FullNameFunction;

impl Function for FullNameFunction {{
    fn name(&self) -> &str {{ "fullName" }}
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {{
        // Implementation would extract given and family names
        // and concatenate them with appropriate spacing
        // ...
    }}
}}

// Usage in expressions:
// Patient.name.fullName() -> "John William Smith"
"#
    );

    Ok(())
}

/// Helper function to print results
fn print_result(expression: &str, result: &octofhir_fhirpath::FhirPathValue) {
    println!("Expression: {expression}");
    match result {
        octofhir_fhirpath::FhirPathValue::Empty => println!("Result: (empty)"),
        octofhir_fhirpath::FhirPathValue::Collection(items) if items.is_empty() => {
            println!("Result: (empty collection)")
        }
        octofhir_fhirpath::FhirPathValue::Collection(items) if items.len() == 1 => {
            println!("Result: {:?}", items.get(0).unwrap());
        }
        _ => println!("Result: {result:?}"),
    }
    println!();
}
