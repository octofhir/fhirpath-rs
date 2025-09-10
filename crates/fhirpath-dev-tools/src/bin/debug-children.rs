use octofhir_fhirpath::{FhirPathEngine, MockModelProvider, FhirPathValue};
use std::sync::Arc;
use serde_json;

#[tokio::main]
async fn main() {
    let provider = Arc::new(MockModelProvider::new());
    let engine = FhirPathEngine::new(provider);
    
    // Load test data
    let data = serde_json::json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [{"given": ["John"]}]
    });
    
    // Test children() ordering
    println!("=== Testing Patient.children() ===");
    let result = engine.evaluate("Patient.children()", &data, None).await.unwrap();
    match &result {
        FhirPathValue::Collection(collection) => {
            println!("Collection is_ordered: {}", collection.is_ordered());
            println!("Collection size: {}", collection.len());
        },
        _ => println!("Not a collection: {:?}", result)
    }
    
    // Test the problematic skip expression
    println!("\n=== Testing Patient.children().skip(1) ===");
    let result2 = engine.evaluate("Patient.children().skip(1)", &data, None).await;
    match result2 {
        Ok(value) => {
            println!("SUCCESS (but should fail!): {:?}", value);
        },
        Err(error) => {
            println!("ERROR (expected): {}", error);
        }
    }
}