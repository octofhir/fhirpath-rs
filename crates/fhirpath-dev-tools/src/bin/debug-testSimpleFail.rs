use std::sync::Arc;
use octofhir_fhirpath::{FhirPathEngine, EvaluationContext, FhirPathValue, FunctionRegistry};
use fhirpath_dev_tools::EmbeddedModelProvider;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_provider = Arc::new(EmbeddedModelProvider::r4().await?);
    let registry = Arc::new(FunctionRegistry::new());
    let mut engine = FhirPathEngine::new(registry, model_provider).await?;
    
    // Load patient-example.json (simplified version)
    let patient = json!({
        "resourceType": "Patient",
        "name": [{
            "given": ["Peter", "James"]
        }, {
            "given": ["Jim"]
        }]
    });
    
    let context = EvaluationContext::from_value(FhirPathValue::Resource(Arc::new(patient)));
    
    println!("Context created");
    
    println!("Testing expression: name.given1");
    
    // Test the failing expression
    match engine.evaluate("name.given1", &context).await {
        Ok(result) => {
            println!("Result: {:?}", result);
            println!("Expected: ERROR but got result");
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("This is what was expected");
        }
    }
    
    // Also test a valid expression for comparison
    println!("\nTesting valid expression: name.given");
    match engine.evaluate("name.given", &context).await {
        Ok(result) => {
            println!("Result: {:?}", result);
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
    
    Ok(())
}