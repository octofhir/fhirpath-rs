println!("Testing basic integer literal evaluation...");
use octofhir_fhirpath::{model::MockModelProvider, engine::IntegratedFhirPathEngine};
use std::sync::Arc;
let provider = Arc::new(MockModelProvider::new());
let mut engine = IntegratedFhirPathEngine::new(provider);
let result = engine.evaluate("42", serde_json::json!({})).await;
println!("Result: {:?}", result);
