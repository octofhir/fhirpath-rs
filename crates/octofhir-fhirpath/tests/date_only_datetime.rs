//! A dateTime element whose value is only a date (`2016-03-28`), which FHIR
//! allows, is still a dateTime (official isSemanticChecks tests on
//! observation-example).

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::{Value, json};

async fn evaluate(expression: &str) -> Vec<Value> {
    let observation = FhirPathValue::resource(json!({
        "resourceType": "Observation",
        "status": "final",
        "effectiveDateTime": "2016-03-28",
        "issued": "2016-03-28T10:10:10Z"
    }));
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(observation),
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
    result.value.iter().map(|v| v.to_json_value()).collect()
}

#[tokio::test]
async fn a_date_only_datetime_is_a_datetime() {
    assert_eq!(
        evaluate("Observation.effective is dateTime").await,
        vec![json!(true)]
    );
    assert_eq!(
        evaluate("Observation.effective.type().name").await,
        vec![json!("dateTime")]
    );
    assert_eq!(
        evaluate("Observation.effective.ofType(dateTime)").await,
        vec![json!("2016-03-28")]
    );
}

#[tokio::test]
async fn it_orders_against_other_datetimes() {
    assert_eq!(
        evaluate("Observation.effective < @2016-03-29T").await,
        vec![json!(true)]
    );
}

#[tokio::test]
async fn a_date_compares_with_a_datetime_of_the_same_precision() {
    // A Date converts to a DateTime, so a date literal still equals a
    // date-only dateTime (`@2016-03-28T`, as a model reads `2016-03-28` in a
    // dateTime element).
    assert_eq!(evaluate("@2016-03-28 = @2016-03-28T").await, [json!(true)]);
    assert_eq!(evaluate("@2016-03-28T = @2016-03-28").await, [json!(true)]);
    assert_eq!(evaluate("@2016-03-28 = @2016-03-29T").await, [json!(false)]);
    assert_eq!(evaluate("@2016-03-28 ~ @2016-03-28T").await, [json!(true)]);
    assert_eq!(evaluate("@2016-03 = @2016-03T").await, [json!(true)]);
    // At different precisions `=` is empty and `~` is false (testEquality19,
    // testEquivalent16).
    let empty: Vec<Value> = vec![];
    assert_eq!(evaluate("@2012-04-15 = @2012-04-15T10:00:00").await, empty);
    assert_eq!(
        evaluate("@2012-04-15 ~ @2012-04-15T10:00:00").await,
        [json!(false)]
    );
    assert_eq!(evaluate("@2012-04 = @2012-04-15T").await, empty);
}
