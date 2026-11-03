//! Decimal input/output precision must not depend on serde_json/arbitrary_precision.
use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::core::node::FhirNode;
use octofhir_fhirpath::{Collection, EvaluationContext, FhirPathEngine, create_function_registry};
use std::sync::Arc;

async fn evaluate(expression: &str) -> Collection {
    let model = Arc::new(EmptyModelProvider);
    let input = Collection::from_json_str(r#"{"resourceType":"Observation","component":[{"valueDecimal":1.50},{"valueDecimal":0.10000000000000000001},{"valueDecimal":1.50e-3}],"valueQuantity":{"value":1.50,"system":"http://unitsofmeasure.org","code":"mg"}}"#, Some(model.clone())).await.unwrap();
    let context = EvaluationContext::new(input, model.clone(), None, None, None);
    FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .unwrap()
        .evaluate(expression, &context)
        .await
        .unwrap()
        .value
}

#[tokio::test]
async fn decimals_keep_scale_through_navigation_and_serialization() {
    let values = evaluate("component.value").await;
    assert_eq!(
        serde_json::to_string(&values).unwrap(),
        "[1.50,0.10000000000000000001,1.50e-3]"
    );
    assert_eq!(
        serde_json::to_string(&evaluate("component.value.select(precision())").await).unwrap(),
        "[2,20,5]"
    );
}

#[tokio::test]
async fn boundaries_and_quantity_values_use_the_source_precision() {
    assert_eq!(
        evaluate("component.first().value.lowBoundary() = 1.495")
            .await
            .first()
            .unwrap()
            .as_boolean(),
        Some(true)
    );
    assert_eq!(
        evaluate("valueQuantity.value.precision()")
            .await
            .first()
            .unwrap()
            .as_integer(),
        Some(2)
    );
    assert_eq!(
        serde_json::to_string(&evaluate("component.first().children()").await).unwrap(),
        "[1.50]"
    );
}

#[test]
fn input_nodes_roundtrip_and_reject_unrepresentable_decimals() {
    let node = FhirNode::parse_json(r#"{"n":1.50,"big":1234567890123456789012345678}"#).unwrap();
    assert_eq!(
        serde_json::to_string(&node).unwrap(),
        r#"{"n":1.50,"big":1234567890123456789012345678}"#
    );
    assert!(FhirNode::parse_json("0.1234567890123456789012345678901").is_err());
    assert!(FhirNode::parse_json("1e100").is_err());
    assert!(FhirNode::parse_json("1e-29").is_err());
    assert!(FhirNode::parse_json("0.1234567890123456789012345678901e-1").is_err());
    assert!(FhirNode::parse_json("[1,]").is_err());
}

#[test]
fn untagged_and_flattened_json_consumers_remain_compatible() {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Payload {
        Number(f64),
        Object {
            #[serde(flatten)]
            fields: std::collections::HashMap<String, serde_json::Value>,
        },
    }
    assert!(
        matches!(serde_json::from_str::<Payload>("1.50").unwrap(), Payload::Number(n) if n == 1.5)
    );
    assert!(
        matches!(serde_json::from_str::<Payload>(r#"{"n":1.50}"#).unwrap(), Payload::Object { fields } if fields["n"] == serde_json::json!(1.5))
    );
}

#[tokio::test]
async fn scientific_notation_keeps_negative_scale_and_resolves_without_rounding() {
    let model = Arc::new(EmptyModelProvider);
    let input = Collection::from_json_str(r#"{"resourceType":"Bundle","entry":[{"resource":{"resourceType":"Observation","id":"o","valueDecimal":9.5e2}},{"resource":{"resourceType":"DiagnosticReport","result":[{"reference":"Observation/o"}]}}]}"#, Some(model.clone())).await.unwrap();
    let context = EvaluationContext::new(input, model.clone(), None, None, None);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .unwrap();
    let expression = "entry.resource.result.resolve().value";
    for (suffix, expected) in [
        (".precision()", "[-1]"),
        (".lowBoundary() = 945", "[true]"),
        (".highBoundary() = 955", "[true]"),
    ] {
        let result = engine
            .evaluate(&format!("{expression}{suffix}"), &context)
            .await
            .unwrap();
        assert_eq!(serde_json::to_string(&result.value).unwrap(), expected);
    }
}

#[tokio::test]
async fn object_property_order_does_not_change_equality_or_distinctness() {
    let model = Arc::new(EmptyModelProvider);
    let input = Collection::from_json_str(r#"[{"a":1,"b":1.50},{"b":1.50,"a":1}]"#, None)
        .await
        .unwrap();
    assert_eq!(input[0], input[1]);
    let context = EvaluationContext::new(input, model.clone(), None, None, None);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .unwrap();
    assert_eq!(
        engine
            .evaluate("distinct().count()", &context)
            .await
            .unwrap()
            .value
            .first()
            .unwrap()
            .as_integer(),
        Some(1)
    );
}
