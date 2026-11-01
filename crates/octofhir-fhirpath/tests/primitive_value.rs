//! System values have no `value` element: only FHIR primitives do (see the
//! model-aware test in fhirpath-dev-tools).

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{Collection, EvaluationContext, FhirPathEngine, create_function_registry};
use serde_json::Value;

async fn evaluate(expression: &str) -> Vec<Value> {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(Collection::empty(), model.clone(), None, None, None);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    let result = engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation");
    result.value.iter().map(|v| v.to_json_value()).collect()
}

#[tokio::test]
async fn system_values_have_no_value_element() {
    assert_eq!(evaluate("'abc'.value").await, Vec::<Value>::new());
    assert_eq!(evaluate("(1).value").await, Vec::<Value>::new());
    assert_eq!(evaluate("@2024-01-01.value").await, Vec::<Value>::new());
}
