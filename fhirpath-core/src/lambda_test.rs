//! Test lambda functions like all(), any(), exists()

use crate::FhirPathEngine;
use crate::value_ext::FhirPathValue;
use serde_json::json;

pub fn test_lambda_functions() {
    let mut engine = FhirPathEngine::new();
    
    // Test data - a Patient with multiple names and telecom entries  
    let patient_data = json!({
        "resourceType": "Patient",
        "name": [
            {
                "use": "official",
                "given": ["John"],
                "family": "Doe"
            },
            {
                "use": "nickname", 
                "given": ["Johnny"]
            }
        ],
        "telecom": [
            {
                "system": "phone",
                "value": "+1-555-123-4567"
            },
            {
                "system": "email",
                "value": "john.doe@example.com"
            }
        ]
    });
    
    // Test cases from the task specification
    
    // Test 1: Patient.name.all(given.count() > 0) - all names have given
    println!("Testing: Patient.name.all(given.count() > 0)");
    match engine.evaluate("Patient.name.all(given.count() > 0)", patient_data.clone()) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 2: Patient.telecom.any(system = 'phone') - any phone contact
    println!("\nTesting: Patient.telecom.any(system = 'phone')");
    match engine.evaluate("Patient.telecom.any(system = 'phone')", patient_data.clone()) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 3: Patient.identifier.exists(type.coding.system = 'ssn') - has SSN (should be false)
    println!("\nTesting: Patient.identifier.exists(type.coding.system = 'ssn')");
    match engine.evaluate("Patient.identifier.exists(type.coding.system = 'ssn')", patient_data.clone()) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }

    // Test simple cases without lambdas
    println!("\nTesting: Patient.name.all() - basic all() without criteria");
    match engine.evaluate("Patient.name.all()", patient_data.clone()) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nTesting: Patient.name.any() - basic any() without criteria");
    match engine.evaluate("Patient.name.any()", patient_data.clone()) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lambda_functions() {
        test_lambda_functions();
    }
}