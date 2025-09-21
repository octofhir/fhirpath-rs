use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::json;

fn patient_resource() -> FhirPathValue {
    let patient_json = json!({
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John", "James"],
                "family": "Doe"
            },
            {
                "given": ["Alice"],
                "family": "Smith"
            }
        ]
    });

    FhirPathValue::resource(patient_json)
}

fn boolean_from_result(result: &octofhir_fhirpath::EvaluationResult) -> bool {
    match result.value.first() {
        Some(FhirPathValue::Boolean(value, _, _)) => *value,
        other => panic!("expected boolean result, got {other:?}"),
    }
}

async fn evaluate_against_patient(expression: &str) -> bool {
    let patient_value = patient_resource();
    let input_collection = Collection::single(patient_value.clone());

    let model_provider = Arc::new(EmptyModelProvider);
    let context =
        EvaluationContext::new(input_collection, model_provider.clone(), None, None, None).await;

    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model_provider)
        .await
        .expect("engine creation");

    let result = engine
        .evaluate(expression, &context)
        .await
        .expect("expression evaluation");

    boolean_from_result(&result)
}

#[tokio::test]
#[ignore = "subsetOf function not yet implemented"]
async fn subset_of_argument_preserves_this_variable() {
    assert!(evaluate_against_patient("Patient.name.first().subsetOf($this.name)").await);
}

#[tokio::test]
#[ignore = "supersetOf function not yet implemented"]
async fn superset_of_argument_preserves_this_variable() {
    assert!(evaluate_against_patient("Patient.name.supersetOf($this.name.first())").await);
}
