use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{Collection, EvaluationContext, FhirPathEngine, create_function_registry};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn type_function_name_returns_system_string() {
    let registry = Arc::new(create_function_registry());
    let model_provider = Arc::new(EmptyModelProvider);
    let engine = FhirPathEngine::new(registry, model_provider.clone())
        .await
        .expect("engine creation");

    let patient_value = octofhir_fhirpath::core::value::utils::json_to_fhirpath_value(json!({
        "resourceType": "Patient",
        "id": "example",
    }));

    let context = EvaluationContext::new(
        Collection::single(patient_value),
        model_provider,
        None,
        None,
        None,
    );

    let evaluation = engine
        .evaluate("Patient.ofType(FHIR.`Patient`).type().name", &context)
        .await
        .expect("evaluation succeeded");

    assert_eq!(evaluation.value.len(), 1);
    let name_value = evaluation.value.first().expect("single result");

    assert_eq!(name_value.as_string(), Some("Patient"));
    let type_info = name_value.type_info();
    assert_eq!(type_info.namespace.as_deref(), Some("System"));
    assert_eq!(type_info.type_name, "String");
}
