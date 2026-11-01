//! Functions whose result on an empty input collection the FHIRPath
//! specification defines, rather than propagating the empty collection:
//! `all()`, `allTrue()`, `allFalse()`, `isDistinct()` and `subsetOf()` are
//! `true`; `anyTrue()` and `anyFalse()` are `false`; `union()` and
//! `combine()` return the other collection; `aggregate()` returns `init`;
//! `supersetOf({})` is `true`.

use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::json;

async fn evaluate(expression: &str) -> Vec<String> {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(
            json!({"resourceType": "Patient", "id": "p"}),
        )),
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
        .unwrap_or_else(|error| panic!("{expression}: {error}"))
        .value
        .iter()
        .map(|value| match value {
            FhirPathValue::Boolean(b, _, _) => b.to_string(),
            FhirPathValue::Integer(i, _, _) => i.to_string(),
            other => format!("{other:?}"),
        })
        .collect()
}

#[tokio::test]
async fn empty_input_results_are_the_ones_the_specification_defines() {
    for (expression, expected) in [
        ("{}.all(true)", vec!["true"]),
        ("telecom.all(value.exists())", vec!["true"]),
        ("{}.allTrue()", vec!["true"]),
        ("{}.anyTrue()", vec!["false"]),
        ("{}.allFalse()", vec!["true"]),
        ("{}.anyFalse()", vec!["false"]),
        ("{}.isDistinct()", vec!["true"]),
        ("{}.subsetOf(1 | 2)", vec!["true"]),
        ("(1 | 2).supersetOf({})", vec!["true"]),
        ("{}.supersetOf(1)", vec!["false"]),
        ("{}.union(1 | 2).count()", vec!["2"]),
        ("{}.combine(1 | 2).count()", vec!["2"]),
        ("{}.aggregate($total + $this, 0)", vec!["0"]),
        ("coalesce({}, 1)", vec!["1"]),
    ] {
        assert_eq!(evaluate(expression).await, expected, "{expression}");
    }
}

#[tokio::test]
async fn functions_without_a_defined_empty_result_still_propagate_it() {
    for expression in ["{}.distinct()", "{}.take(1)", "{}.tail()", "{}.first()"] {
        assert!(evaluate(expression).await.is_empty(), "{expression}");
    }
}
