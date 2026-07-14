//! Constraint evaluation against a context node that is not a whole resource.
//!
//! Invariants declared on an element (`Observation.effective[x]`, say) are
//! evaluated with that element as the focus. The element's JSON alone is not
//! enough to model it — `"2017-03-15T20:23:41+00:00"` could be any string — so
//! the caller passes the declared FHIR type alongside it.

use std::collections::HashMap;
use std::sync::Arc;

use octofhir_fhir_model::{EmptyModelProvider, FhirPathEvaluator};
use octofhir_fhirpath::{FhirPathEngine, create_function_registry};
use serde_json::{Value as JsonValue, json};

async fn engine() -> FhirPathEngine {
    FhirPathEngine::new(
        Arc::new(create_function_registry()),
        Arc::new(EmptyModelProvider),
    )
    .await
    .expect("engine creation")
}

/// Evaluate one constraint against `context`, typed as `context_type`.
async fn eval(context: JsonValue, context_type: Option<&str>, expression: &str) -> bool {
    let engine = engine().await;
    let variables: HashMap<String, Arc<JsonValue>> = HashMap::new();

    let mut results = engine
        .evaluate_constraints_shared_context_typed(
            Arc::new(context),
            context_type,
            &variables,
            &[expression],
        )
        .await
        .expect("shared context builds");

    results
        .remove(0)
        .unwrap_or_else(|e| panic!("`{expression}` failed to evaluate: {e}"))
}

#[tokio::test]
async fn typed_primitive_context_is_modelled_as_its_fhir_type() {
    let value = json!("2017-03-15T20:23:41+00:00");

    // The focus is a FHIR dateTime, and says so.
    assert!(eval(value.clone(), Some("dateTime"), "$this is dateTime").await);
    assert!(eval(value.clone(), Some("dateTime"), "$this is FHIR.dateTime").await);

    // ...and can be operated on as one, rather than as an opaque resource.
    assert!(
        eval(
            value.clone(),
            Some("dateTime"),
            "$this.toString().length() = 25"
        )
        .await
    );

    // A FHIR primitive is not the System type of the same name — see the
    // official `testType12`/`testType14` (`Patient.active.is(Boolean).not()`).
    assert!(eval(value, Some("dateTime"), "($this is DateTime).not()").await);
}

#[tokio::test]
async fn untyped_primitive_context_still_evaluates() {
    // Without a declared type the node is modelled from its JSON shape alone,
    // which is the long-standing behavior for callers that have no type to give.
    assert!(eval(json!("2017-03-15T20:23:41+00:00"), None, "$this.exists()").await);
}

#[tokio::test]
async fn the_untyped_entry_point_still_works() {
    // Callers written against the pre-0.1.16 trait keep calling this method;
    // it must behave exactly as it did, i.e. as the typed one given no type.
    let engine = engine().await;
    let variables: HashMap<String, Arc<JsonValue>> = HashMap::new();

    let mut results = engine
        .evaluate_constraints_shared_context(
            Arc::new(json!({"resourceType": "Patient", "id": "example"})),
            &variables,
            &["$this is Patient"],
        )
        .await
        .expect("shared context builds");

    assert!(results.remove(0).expect("evaluates"));
}

#[tokio::test]
async fn resource_context_ignores_the_type_hint() {
    // `resourceType` is more specific than anything the caller can name.
    let patient = json!({"resourceType": "Patient", "id": "example"});
    assert!(eval(patient.clone(), None, "$this is Patient").await);
    assert!(eval(patient, Some("dateTime"), "$this is Patient").await);
}

#[tokio::test]
async fn implies_does_not_evaluate_its_right_side_when_the_left_is_false() {
    // `us-core-1` on `Observation.effective[x]`: the right side is only well
    // defined for a DateTime focus, so a Period focus must never reach it.
    // Before short-circuiting, `$this.toString()` raised here instead.
    let period = json!({"start": "2017-03-15", "end": "2017-03-16"});
    assert!(
        eval(
            period,
            Some("Period"),
            "$this is DateTime implies $this.toString().length() >= 10",
        )
        .await
    );
}
