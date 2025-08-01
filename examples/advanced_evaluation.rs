//! Advanced FHIRPath Evaluation Examples
//! 
//! This example demonstrates advanced FHIRPath features including:
//! - Lambda expressions and higher-order functions
//! - Complex filtering and transformations  
//! - Variable definitions and scoping
//! - Type operations and conversions
//! - Mathematical and date operations

use octofhir_fhirpath::engine::FhirPathEngine;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FHIRPath Advanced Evaluation Examples");
    println!("=========================================\n");

    let mut engine = FhirPathEngine::new();

    // Example: Bundle with multiple Patient resources
    let bundle = json!({
        "resourceType": "Bundle",
        "id": "patient-bundle",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "patient-1",
                    "name": [
                        {
                            "use": "official",
                            "family": "Smith",
                            "given": ["John", "William"]
                        }
                    ],
                    "gender": "male",
                    "birthDate": "1990-05-15",
                    "active": true,
                    "maritalStatus": {
                        "coding": [
                            {
                                "system": "http://terminology.hl7.org/CodeSystem/v3-MaritalStatus",
                                "code": "M",
                                "display": "Married"
                            }
                        ]
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "Patient", 
                    "id": "patient-2",
                    "name": [
                        {
                            "use": "official",
                            "family": "Johnson",
                            "given": ["Alice", "Marie"]
                        }
                    ],
                    "gender": "female",
                    "birthDate": "1985-12-03",
                    "active": false,
                    "maritalStatus": {
                        "coding": [
                            {
                                "system": "http://terminology.hl7.org/CodeSystem/v3-MaritalStatus",
                                "code": "S",
                                "display": "Single"
                            }
                        ]
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "obs-1",
                    "status": "final",
                    "code": {
                        "coding": [
                            {
                                "system": "http://loinc.org",
                                "code": "29463-7",
                                "display": "Body Weight"
                            }
                        ]
                    },
                    "subject": {
                        "reference": "Patient/patient-1"
                    },
                    "valueQuantity": {
                        "value": 75.5,
                        "unit": "kg",
                        "system": "http://unitsofmeasure.org",
                        "code": "kg"
                    }
                }
            }
        ]
    });

    println!("📦 Sample Bundle Resource");
    println!("=========================\n");

    // Example 1: Lambda expressions with select
    println!("1️⃣  Lambda Expressions with Select");
    println!("-----------------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').select(name.family + ', ' + name.given.first())",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').select(name.family + ', ' + name.given.first())", &result);

    // Example 2: Complex filtering with all() and exists()
    println!("2️⃣  Complex Filtering");
    println!("----------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient' and active = true)",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient' and active = true)", &result);

    // Example 3: Aggregation operations
    println!("3️⃣  Aggregation Operations");
    println!("---------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').count()",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').count()", &result);

    // Example 4: Type checking and casting
    println!("4️⃣  Type Operations");
    println!("-------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Observation').valueQuantity.value is System.Decimal",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Observation').valueQuantity.value is System.Decimal", &result);

    // Example 5: Mathematical operations
    println!("5️⃣  Mathematical Operations");
    println!("----------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Observation').valueQuantity.value * 2.2", // kg to lbs approximation
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Observation').valueQuantity.value * 2.2", &result);

    // Example 6: String manipulation
    println!("6️⃣  String Manipulation");
    println!("------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').name.family.upper()",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').name.family.upper()", &result);

    // Example 7: Date operations (if supported)
    println!("7️⃣  Date Operations");
    println!("-------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').birthDate",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').birthDate", &result);

    // Example 8: Complex path navigation
    println!("8️⃣  Complex Path Navigation");
    println!("----------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').maritalStatus.coding.where(system = 'http://terminology.hl7.org/CodeSystem/v3-MaritalStatus').code",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').maritalStatus.coding.where(system = 'http://terminology.hl7.org/CodeSystem/v3-MaritalStatus').code", &result);

    // Example 9: Union operations
    println!("9️⃣  Union Operations");
    println!("--------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').name.given | Bundle.entry.resource.where(resourceType = 'Patient').name.family",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').name.given | Bundle.entry.resource.where(resourceType = 'Patient').name.family", &result);

    // Example 10: Conditional expressions (iif)
    println!("🔟 Conditional Expressions");
    println!("---------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').select(iif(active, 'Active Patient', 'Inactive Patient'))",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').select(iif(active, 'Active Patient', 'Inactive Patient'))", &result);

    // Example 11: Complex aggregation with grouping-like behavior
    println!("1️⃣1️⃣ Advanced Aggregation");
    println!("---------------------------");
    
    let result = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient').name.given.distinct().count()",
        bundle.clone()
    )?;
    print_result("Bundle.entry.resource.where(resourceType = 'Patient').name.given.distinct().count()", &result);

    Ok(())
}

/// Helper function to print results in a readable format
fn print_result(expression: &str, result: &octofhir_fhirpath::FhirPathValue) {
    println!("Expression: {}", expression);
    match result {
        octofhir_fhirpath::FhirPathValue::Empty => println!("Result: (empty)"),
        octofhir_fhirpath::FhirPathValue::Collection(items) if items.is_empty() => println!("Result: (empty collection)"),
        octofhir_fhirpath::FhirPathValue::Collection(items) if items.len() == 1 => {
            println!("Result: {:?}", items.get(0).unwrap());
        }
        _ => println!("Result: {:?}", result),
    }
    println!();
}