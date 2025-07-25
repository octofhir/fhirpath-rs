//! Debug test for function calls
use crate::FhirPathEngine;
use fhirpath_model::FhirPathValue;
use serde_json::{json, Value};

pub fn test_union_operator() {
    let mut engine = FhirPathEngine::new();
    let input_data = json!({"resourceType": "Patient"});

    // Test union with duplicates
    let expression = "1 | 2 | 1 | 3";
    match engine.evaluate(expression, input_data.clone()) {
        Ok(result) => {
            println!("Union result: {:?}", result);
            // Should return [1, 2, 3] (no duplicates)
        }
        Err(e) => println!("Union error: {:?}", e),
    }

    // Test union with empty collection
    let expression = "{} | 1 | 2";
    match engine.evaluate(expression, input_data.clone()) {
        Ok(result) => {
            println!("Union with empty result: {:?}", result);
            // Should return [1, 2]
        }
        Err(e) => println!("Union with empty error: {:?}", e),
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn debug_function_calls() {
        use crate::FhirPathEngine;
        use fhirpath_model::FhirPathValue;
        use serde_json::json;
        
        println!("Testing FHIRPath function calls...");
        
        let mut engine = FhirPathEngine::new();
        
        // Create a simple test patient with names
        let patient_json = json!({
            "resourceType": "Patient",
            "name": [
                {"family": "Smith", "given": ["John"]},
                {"family": "Doe", "given": ["Jane"]}, 
                {"family": "Johnson", "given": ["Bob"]}
            ]
        });
        
        // Test count() function
        println!("\n=== Testing Patient.name.count() ===");
        let result1 = engine.evaluate("Patient.name.count()", patient_json.clone()).unwrap();
        println!("Result for Patient.name.count(): {:?}", result1);
        
        // Check if it's actually single or double wrapped
        match &result1 {
            FhirPathValue::Collection(outer_coll) => {
                println!("Outer collection length: {}", outer_coll.len());
                if let Some(first_item) = outer_coll.get(0) {
                    println!("First item type: {:?}", first_item);
                    match first_item {
                        FhirPathValue::Collection(inner_coll) => {
                            println!("Double wrapped! Inner collection length: {}", inner_coll.len());
                        },
                        FhirPathValue::Integer(i) => {
                            println!("Single wrapped integer: {}", i);
                        },
                        _ => println!("Other type in collection"),
                    }
                }
            },
            _ => println!("Not a collection result"),
        }

        // Test literal 3
        println!("\n=== Testing literal '3' ===");
        let result_literal = engine.evaluate("3", patient_json.clone()).unwrap();
        println!("Result for literal '3': {:?}", result_literal);

        // Test equality comparison
        println!("\n=== Testing Patient.name.count() = 3 ===");
        let result2 = engine.evaluate("Patient.name.count() = 3", patient_json.clone()).unwrap();
        println!("Result for Patient.name.count() = 3: {:?}", result2);
    }
}