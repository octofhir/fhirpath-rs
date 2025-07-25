use fhirpath_core::FhirPathEngine;

fn main() {
    let engine = FhirPathEngine::new();
    
    // Test basic string length
    let result = engine.evaluate("'123456'.length()", &serde_json::json!({}));
    println!("'123456'.length() result: {:?}", result);
    
    // Test comparison
    let result2 = engine.evaluate("'123456'.length() = 6", &serde_json::json!({}));
    println!("'123456'.length() = 6 result: {:?}", result2);
    
    // Test empty object length
    let result3 = engine.evaluate("{}.length()", &serde_json::json!({}));
    println!("{}.length() result: {:?}", result3);
    
    // Test empty object length().empty()
    let result4 = engine.evaluate("{}.length().empty()", &serde_json::json!({}));
    println!("{}.length().empty() result: {:?}", result4);
}