//! `hasValue()`: true only for a single FHIR primitive that has a value
//! (testHasValue in the official FHIRPath test suite).

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::json;

async fn evaluate(expression: &str) -> Option<bool> {
    let patient = FhirPathValue::resource(json!({
        "resourceType": "Patient",
        "name": [{ "family": "Chalmers", "given": [null, "Peter"], "_given": [
            { "extension": [{ "url": "http://example.org/x", "valueString": "x" }] }, null
        ] }],
        "contact": [{ "telecom": [{ "value": "1" }] }]
    }));
    let model = Arc::new(EmptyModelProvider);
    let context =
        EvaluationContext::new(Collection::single(patient), model.clone(), None, None, None);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    let result = engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation");
    match result.value.first() {
        Some(FhirPathValue::Boolean(b, _, _)) => Some(*b),
        None => None,
        other => panic!("expected a boolean, got {other:?}"),
    }
}

#[tokio::test]
async fn a_primitive_with_a_value_has_a_value() {
    assert_eq!(evaluate("name.family.hasValue()").await, Some(true));
    assert_eq!(evaluate("name.given.last().hasValue()").await, Some(true));
}

#[tokio::test]
async fn a_complex_type_never_has_a_value() {
    assert_eq!(evaluate("name.first().hasValue()").await, Some(false));
    assert_eq!(
        evaluate("contact.telecom.first().hasValue()").await,
        Some(false)
    );
}

#[tokio::test]
async fn several_items_do_not_have_a_value() {
    assert_eq!(evaluate("(1 | 2).hasValue()").await, Some(false));
}

#[tokio::test]
async fn a_primitive_with_only_extensions_has_no_value() {
    assert_eq!(evaluate("name.given.first().hasValue()").await, Some(false));
}
