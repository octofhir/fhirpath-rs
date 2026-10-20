//! A Quantity's `code` and `system` elements. With a model provider, a
//! Quantity element of a resource (`Observation.valueQuantity`) becomes a
//! `FhirPathValue::Quantity`; navigating it returned the unit for `code` and
//! nothing for `system`.

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};

async fn evaluate(expression: &str) -> Vec<Value> {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(Collection::empty(), model.clone(), None, None, None);
    // As read from `{"value": 120, "unit": "mmHg", "system":
    // "http://unitsofmeasure.org", "code": "mm[Hg]"}`.
    context.set_variable(
        "q".to_string(),
        FhirPathValue::quantity_with_components(
            Decimal::from(120),
            Some("mmHg".to_string()),
            Some("mm[Hg]".to_string()),
            Some("http://unitsofmeasure.org".to_string()),
        ),
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
async fn a_quantity_has_its_code_and_system() {
    assert_eq!(evaluate("%q.unit").await, [json!("mmHg")]);
    assert_eq!(evaluate("%q.code").await, [json!("mm[Hg]")]);
    assert_eq!(
        evaluate("%q.system").await,
        [json!("http://unitsofmeasure.org")]
    );
    assert_eq!(evaluate("%q.value").await, [json!(120.0)]);
}

#[tokio::test]
async fn a_quantity_literal_has_its_unit_as_its_code() {
    assert_eq!(evaluate("(5 'mg').code").await, [json!("mg")]);
    assert_eq!(evaluate("(1 year).code").await, [json!("year")]);
}
