//! A decimal read from a resource keeps the digits its JSON number was
//! written with, and a decimal written back to JSON keeps them too. JSON
//! numbers reach the engine as `serde_json::Number`; converting them through
//! `f64` (`Decimal::from_f64_retain`) turns `0.1` into
//! `0.1000000000000000055511151231` and `1.0` into `1`.

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::{Value, json};

fn observation() -> Value {
    serde_json::from_str(
        r#"{
            "resourceType": "Observation",
            "valueQuantity": { "value": 0.1, "unit": "mg", "code": "mg", "system": "http://unitsofmeasure.org" },
            "component": [
                { "valueDecimal": 1.0 },
                { "valueDecimal": 42.3601 },
                { "valueDecimal": -71.0589123 },
                { "valueDecimal": 1e-7 }
            ]
        }"#,
    )
    .unwrap()
}

async fn evaluate(expression: &str) -> Collection {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(observation())),
        model.clone(),
        None,
        None,
        None,
    );
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation")
        .value
}

/// Each value's decimal text, and its JSON form.
async fn decimals(expression: &str) -> Vec<(String, String)> {
    evaluate(expression)
        .await
        .iter()
        .map(|v| match v {
            FhirPathValue::Decimal(d, _, _) => (d.to_string(), v.to_json_value().to_string()),
            other => panic!("expected a decimal, got {other:?}"),
        })
        .collect()
}

#[tokio::test]
async fn a_decimal_keeps_the_digits_it_was_written_with() {
    assert_eq!(
        decimals("component.value").await,
        [
            ("1.0".into(), "1.0".into()),
            ("42.3601".into(), "42.3601".into()),
            ("-71.0589123".into(), "-71.0589123".into()),
            ("0.0000001".into(), "1e-7".into()),
        ]
    );
}

#[tokio::test]
async fn a_decimal_written_with_a_zero_fraction_keeps_its_precision() {
    // 1.0 has one decimal place, so its low boundary is 0.95, not 0.5.
    let low = evaluate("component.value.first().lowBoundary()").await;
    assert_eq!(low.first().unwrap().to_json_value(), json!(0.95));
}

#[tokio::test]
async fn decimal_comparisons_use_the_written_value() {
    let result = evaluate("component.value.where($this = 42.3601).count()").await;
    assert_eq!(result.first().unwrap().to_json_value(), json!(1));
    let result = evaluate("component.value.where($this ~ 1).count()").await;
    assert_eq!(result.first().unwrap().to_json_value(), json!(1));
}

#[tokio::test]
async fn a_quantity_value_keeps_its_digits() {
    match evaluate("valueQuantity.value").await.first() {
        Some(FhirPathValue::Decimal(d, _, _)) => assert_eq!(d.to_string(), "0.1"),
        other => panic!("expected a decimal, got {other:?}"),
    }
    let result = evaluate("valueQuantity.value = 0.1").await;
    assert_eq!(result.first().unwrap().to_json_value(), json!(true));
}

#[tokio::test]
async fn to_quantity_keeps_the_digits_of_the_string() {
    let result = evaluate("'0.1 \\'mg\\''.toQuantity() = 0.1 'mg'").await;
    assert_eq!(result.first().unwrap().to_json_value(), json!(true));
    let result = evaluate("'1.0'.toQuantity().value.lowBoundary()").await;
    assert_eq!(result.first().unwrap().to_json_value(), json!(0.95));
}
