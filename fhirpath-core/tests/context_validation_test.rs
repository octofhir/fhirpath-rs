// Test to demonstrate context validation issue
// This test shows that FHIRPath expressions can be evaluated against
// inappropriate resource types without validation

use fhirpath_core::evaluate;
use serde_json::json;

#[test]
fn test_context_validation_issue() {
    // Load a Patient resource
    let patient_resource = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [
            {
                "use": "official",
                "family": "Smith",
                "given": ["John", "Jacob"]
            }
        ],
        "gender": "male",
        "birthDate": "1974-12-25"
    });

    println!("Testing context validation issues...\n");

    // Test 1: Valid Patient expression - should work
    println!("1. Valid Patient expression: 'name.given'");
    match evaluate("name.given", patient_resource.clone()) {
        Ok(result) => {
            println!("   Result: {}", result);
            // This should work and return the given names
            assert!(!result.is_null());
        },
        Err(e) => {
            println!("   Error: {:?}", e);
            panic!("Valid Patient expression should work");
        },
    }

    // Test 2: Encounter-specific expression on Patient - should fail with proper validation
    println!("\n2. Encounter expression on Patient: 'class.code'");
    match evaluate("class.code", patient_resource.clone()) {
        Ok(result) => {
            println!("   Result: {} (This should not succeed with proper validation!)", result);
            // Currently this probably returns null/empty, but with proper validation it should error
        },
        Err(e) => {
            println!("   Error: {:?}", e);
        },
    }

    // Test 3: Another Encounter-specific expression
    println!("\n3. Encounter expression on Patient: 'status'");
    match evaluate("status", patient_resource.clone()) {
        Ok(result) => {
            println!("   Result: {} (This should not succeed with proper validation!)", result);
        },
        Err(e) => {
            println!("   Error: {:?}", e);
        },
    }

    // Test 4: Observation-specific expression on Patient
    println!("\n4. Observation expression on Patient: 'valueQuantity.value'");
    match evaluate("valueQuantity.value", patient_resource.clone()) {
        Ok(result) => {
            println!("   Result: {} (This should not succeed with proper validation!)", result);
        },
        Err(e) => {
            println!("   Error: {:?}", e);
        },
    }

    // Test 5: Non-existent property
    println!("\n5. Non-existent property: 'nonExistentProperty'");
    match evaluate("nonExistentProperty", patient_resource.clone()) {
        Ok(result) => {
            println!("   Result: {}", result);
        },
        Err(e) => {
            println!("   Error: {:?}", e);
        },
    }

    println!("\n=== Analysis ===");
    println!("The issue is that expressions specific to other resource types");
    println!("(like Encounter.class.code or Observation.valueQuantity.value)");
    println!("can be evaluated against Patient resources without validation.");
    println!("This violates the FHIRPath specification's type safety requirements.");

    // This test demonstrates the issue - it should fail but currently passes
    // because there's no context validation
}
