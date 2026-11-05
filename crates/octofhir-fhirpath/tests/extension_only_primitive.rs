//! A primitive element with only extensions (`"_issued": {...}` and no `"issued"`) is present:
//! FHIR JSON represents it so, and FHIRPath sees an element that exists, has no value, and
//! carries its extensions (US Core invariants such as us-core-9 rely on it with
//! data-absent-reason).

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::json;

const DATA_ABSENT: &str = "http://hl7.org/fhir/StructureDefinition/data-absent-reason";

async fn evaluate(expression: &str) -> Vec<FhirPathValue> {
    let report = FhirPathValue::resource(json!({
        "resourceType": "DiagnosticReport",
        "status": "final",
        "_issued": { "extension": [{ "url": DATA_ABSENT, "valueCode": "unknown" }] },
        "_conclusion": null,
        "_effectiveDateTime": { "extension": [{ "url": DATA_ABSENT, "valueCode": "unknown" }] }
    }));
    let model = Arc::new(EmptyModelProvider);
    let context =
        EvaluationContext::new(Collection::single(report), model.clone(), None, None, None);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation")
        .value
        .into_vec()
}

fn boolean(values: &[FhirPathValue]) -> Option<bool> {
    match values.first() {
        Some(FhirPathValue::Boolean(b, _, _)) => Some(*b),
        None => None,
        other => panic!("expected a boolean, got {other:?}"),
    }
}

#[tokio::test]
async fn it_exists_without_a_value() {
    assert_eq!(boolean(&evaluate("issued.exists()").await), Some(true));
    assert_eq!(boolean(&evaluate("issued.hasValue()").await), Some(false));
}

#[tokio::test]
async fn its_extensions_are_there() {
    let urls = evaluate("issued.extension.url").await;
    assert_eq!(urls.len(), 1);
    assert_eq!(
        boolean(&evaluate(&format!("issued.extension('{DATA_ABSENT}').exists()")).await),
        Some(true)
    );
}

#[tokio::test]
async fn an_invariant_that_needs_it_holds() {
    // us-core-9: issued SHALL be present if the status is final (and similar).
    let expression = "(status in ('partial' | 'preliminary' | 'final' | 'amended' | 'corrected' | 'appended')) implies issued.exists()";
    assert_eq!(boolean(&evaluate(expression).await), Some(true));
}

#[tokio::test]
async fn a_null_primitive_element_is_not_there() {
    assert_eq!(boolean(&evaluate("conclusion.exists()").await), Some(false));
    assert_eq!(
        boolean(&evaluate("presentedForm.exists()").await),
        Some(false)
    );
}

#[tokio::test]
async fn a_choice_with_only_extensions_is_there() {
    // us-core-8: effective[x] SHALL be present if the status is final (and similar).
    assert_eq!(boolean(&evaluate("effective.exists()").await), Some(true));
    assert_eq!(
        boolean(&evaluate("effective.hasValue()").await),
        Some(false)
    );
    assert_eq!(
        boolean(&evaluate(&format!("effective.extension('{DATA_ABSENT}').exists()")).await),
        Some(true)
    );
}
