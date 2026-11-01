//! A FHIR primitive's `value` element. FHIR defines each primitive type with a
//! `value` element of the matching System type (`string.value` is a
//! System.String), and StructureMaps written for the reference engine rely on
//! it (`item where linkId.value = 'patient.sex'`); it navigated to nothing.

use std::sync::Arc;

use octofhir_fhirpath::{Collection, EvaluationContext, FhirPathEngine, create_function_registry};
use octofhir_fhirschema::{EmbeddedSchemaProvider, ModelFhirVersion, ModelProvider};
use serde_json::{Value, json};

async fn evaluate(expression: &str) -> Vec<Value> {
    let model: Arc<dyn ModelProvider + Send + Sync> =
        Arc::new(EmbeddedSchemaProvider::new(ModelFhirVersion::R4));
    let resource = json!({
        "resourceType": "QuestionnaireResponse",
        "status": "completed",
        "authored": "2024-05-01T10:00:00Z",
        "item": [
            { "linkId": "patient.sex", "answer": [{ "valueBoolean": true }] },
            { "linkId": "patient.name" }
        ]
    });
    let input = Collection::from_json_resource(resource, Some(model.clone()))
        .await
        .expect("resource");
    let context = EvaluationContext::new(input, model.clone(), None, None, None);
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
async fn a_primitive_has_its_value() {
    assert_eq!(
        evaluate("item.linkId.value").await,
        [json!("patient.sex"), json!("patient.name")]
    );
    assert_eq!(
        evaluate("item.where(linkId.value = 'patient.sex').linkId").await,
        [json!("patient.sex")]
    );
    assert_eq!(
        evaluate("item.where(linkId.value in ('patient.name')).exists()").await,
        [json!(true)]
    );
    assert_eq!(evaluate("status.value = 'completed'").await, [json!(true)]);
    assert_eq!(evaluate("item.answer.value.value").await, [json!(true)]);
}

#[tokio::test]
async fn a_primitive_value_is_a_system_value() {
    assert_eq!(
        evaluate("item.linkId.value.all($this is System.String)").await,
        [json!(true)]
    );
    assert_eq!(
        evaluate("item.linkId.value.value").await,
        Vec::<Value>::new()
    );
}
