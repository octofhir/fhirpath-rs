//! lowBoundary()/highBoundary() with no precision use the greatest precision of
//! the input's type (FHIRPath spec, Utility functions).

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::{Value, json};

async fn evaluate(expression: &str) -> Value {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(
            json!({ "resourceType": "Patient" }),
        )),
        model.clone(),
        None,
        None,
        None,
    );
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    let result = engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation");
    result.value.first().expect("one value").to_json_value()
}

#[tokio::test]
async fn dates_widen_to_the_day() {
    assert_eq!(
        evaluate("@1970-06.lowBoundary()").await,
        json!("1970-06-01")
    );
    assert_eq!(
        evaluate("@1970-06.highBoundary()").await,
        json!("1970-06-30")
    );
    assert_eq!(evaluate("@2014.lowBoundary()").await, json!("2014-01-01"));
}

#[tokio::test]
async fn datetimes_widen_to_the_millisecond() {
    assert_eq!(
        evaluate("@2010-10-10T.lowBoundary()").await,
        json!("2010-10-10T00:00:00.000+14:00")
    );
    assert_eq!(
        evaluate("@2010-10-10T.highBoundary()").await,
        json!("2010-10-10T23:59:59.999-12:00")
    );
}

#[tokio::test]
async fn times_widen_to_the_millisecond() {
    assert_eq!(
        evaluate("@T12:34.lowBoundary()").await,
        json!("T12:34:00.000")
    );
    assert_eq!(
        evaluate("@T12:34.highBoundary()").await,
        json!("T12:34:59.999")
    );
}

#[tokio::test]
async fn an_explicit_precision_still_applies() {
    assert_eq!(evaluate("@2014.lowBoundary(6)").await, json!("2014-01"));
    assert_eq!(
        evaluate("@T10:30.highBoundary(9)").await,
        json!("T10:30:59.999")
    );
}
