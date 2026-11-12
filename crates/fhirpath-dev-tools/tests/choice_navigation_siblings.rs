//! An absent element is empty. It does not resolve to a sibling whose name only starts the
//! same way: `subscriberId` is not a choice of `subscriber`, and `boundaryType` is not a
//! choice of `boundary`. A choice (`value` for `valueQuantity`) still resolves.

use std::sync::Arc;

use octofhir_fhirpath::{
    Collection, EmptyModelProvider, EvaluationContext, FhirPathEngine, create_function_registry,
};
use octofhir_fhirschema::{EmbeddedSchemaProvider, ModelFhirVersion, ModelProvider};
use serde_json::{Value, json};

async fn evaluate(
    model: Arc<dyn ModelProvider + Send + Sync>,
    input: Value,
    expression: &str,
) -> Vec<Value> {
    let input = Collection::from_json_resource(input, Some(model.clone()))
        .await
        .expect("input");
    let context = EvaluationContext::new(input, model.clone(), None, None, None);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation")
        .value
        .into_vec()
        .iter()
        .map(|value| value.to_json_value())
        .collect()
}

fn r4() -> Arc<dyn ModelProvider + Send + Sync> {
    Arc::new(EmbeddedSchemaProvider::new(ModelFhirVersion::R4))
}

fn empty() -> Arc<dyn ModelProvider + Send + Sync> {
    Arc::new(EmptyModelProvider)
}

fn coverage() -> Value {
    json!({
        "resourceType": "Coverage",
        "status": "active",
        "subscriberId": "A-1",
        "beneficiary": { "reference": "Patient/1" },
        "payor": [{ "reference": "Organization/1" }]
    })
}

fn observation() -> Value {
    json!({
        "resourceType": "Observation",
        "status": "final",
        "code": { "text": "weight" },
        "valueQuantity": { "value": 72, "unit": "kg" }
    })
}

#[tokio::test]
async fn an_absent_element_is_not_its_longer_named_sibling() {
    assert_eq!(
        evaluate(r4(), coverage(), "Coverage.subscriber").await,
        Vec::<Value>::new()
    );
    assert_eq!(
        evaluate(r4(), coverage(), "Coverage.subscriber.exists()").await,
        vec![json!(false)]
    );
    assert_eq!(
        evaluate(r4(), coverage(), "Coverage.subscriberId").await,
        vec![json!("A-1")]
    );
}

#[tokio::test]
async fn a_sibling_whose_suffix_is_not_a_type_is_not_a_choice() {
    // No model for this JSON: the name alone has to tell.
    let node = json!({ "code": "NG", "boundaryType": "Country" });
    assert_eq!(
        evaluate(empty(), node.clone(), "boundary").await,
        Vec::<Value>::new()
    );
    assert_eq!(
        evaluate(empty(), node, "boundaryType").await,
        vec![json!("Country")]
    );
}

#[tokio::test]
async fn a_choice_still_resolves_by_its_base_name() {
    for model in [r4(), empty()] {
        assert_eq!(
            evaluate(model.clone(), observation(), "Observation.value.unit").await,
            vec![json!("kg")]
        );
        assert_eq!(
            evaluate(model, observation(), "Observation.value.exists()").await,
            vec![json!(true)]
        );
    }
}
