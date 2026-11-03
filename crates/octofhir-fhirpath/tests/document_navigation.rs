//! Document references follow fullUrl and section identity, not JSON value equality.
use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::{Value, json};
use std::sync::Arc;

async fn eval(resource: Value, expression: &str) -> Vec<Value> {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(resource)),
        model.clone(),
        None,
        None,
        None,
    );
    FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .unwrap()
        .evaluate(expression, &context)
        .await
        .unwrap()
        .value
        .iter()
        .map(FhirPathValue::to_json_value)
        .collect()
}
fn condition(id: &str, code: &str) -> Value {
    json!({"resourceType":"Condition","id":id,"code":{"coding":[{"system":"s","code":code}]}})
}
fn section(reference: &str) -> Value {
    json!({"code":{"coding":[{"system":"s","code":"section"}]},"entry":[{"reference":reference}]})
}
fn bundle(reference: &str, entries: Vec<Value>) -> Value {
    let mut all = vec![
        json!({"fullUrl":"https://example.org/fhir/Composition/c","resource":{"resourceType":"Composition","id":"c","section":[section(reference)]}}),
    ];
    all.extend(entries);
    json!({"resourceType":"Bundle","type":"document","entry":all})
}
const ENTRIES: &str = "Bundle.section('s','section').entry('s','x').id";

#[tokio::test]
async fn relative_and_versioned_references_use_the_restful_composition_base() {
    let mut resource = condition("o", "x");
    resource["meta"] = json!({"versionId":"2"});
    let entry = json!({"fullUrl":"https://example.org/fhir/Condition/o","resource":resource});
    for reference in [
        "Condition/o",
        "https://example.org/fhir/Condition/o#fragment",
        "Condition/o/_history/2",
    ] {
        assert_eq!(
            eval(bundle(reference, vec![entry.clone()]), ENTRIES).await,
            [json!("o")]
        );
    }
    for reference in [
        "https://wrong.example/Condition/o",
        "Condition/o/_history/1",
    ] {
        assert!(
            eval(bundle(reference, vec![entry.clone()]), ENTRIES)
                .await
                .is_empty()
        );
    }
}

#[tokio::test]
async fn ambiguous_full_urls_do_not_resolve() {
    let entry =
        json!({"fullUrl":"https://example.org/fhir/Condition/o","resource":condition("o","x")});
    assert!(
        eval(bundle("Condition/o", vec![entry.clone(), entry]), ENTRIES)
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn identical_sections_keep_their_composition_identity() {
    let shared_section = section("#local");
    let document = json!({"resourceType":"Bundle","type":"document","entry":[
        {"resource":{"resourceType":"Composition","contained":[condition("local","a")],"section":[shared_section.clone()]}},
        {"resource":{"resourceType":"Composition","contained":[condition("local","b")],"section":[shared_section]}}
    ]});
    let expression = "entry.resource.where(resourceType = 'Composition').section('s','section').entry('s',{}).code.coding.code";
    assert_eq!(eval(document, expression).await, [json!("a"), json!("b")]);
}

#[tokio::test]
async fn standalone_compositions_resolve_their_own_contained_resources() {
    let composition = json!({"resourceType":"Composition","contained":[condition("local","x")],"section":[section("#local")]});
    assert_eq!(
        eval(
            composition,
            "Composition.section('s','section').entry('s','x').id"
        )
        .await,
        [json!("local")]
    );
}

#[tokio::test]
async fn nested_sections_and_default_bounds_are_applied_per_input_composition() {
    let mut outer = section("#one");
    outer["section"] = json!([section("#two")]);
    let resource = json!({"resourceType":"Composition","section":[outer]});
    assert_eq!(
        eval(
            resource.clone(),
            "Composition.section('s','section').count()"
        )
        .await,
        [json!(0)]
    );
    assert_eq!(
        eval(resource, "Composition.section('s','section',0,'*').count()").await,
        [json!(2)]
    );
}

#[tokio::test]
async fn strict_casts_are_available_without_breaking_legacy_fhir_invariants() {
    let model = Arc::new(EmptyModelProvider);
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model.clone())
        .await
        .unwrap();
    let context = EvaluationContext::new(Collection::empty(), model, None, None, None);
    assert_eq!(
        engine
            .evaluate("(1 | 2).as(Integer)", &context)
            .await
            .unwrap()
            .value
            .len(),
        2
    );
    context.enable_strict_type_casts();
    assert!(
        engine
            .evaluate("(1 | 2).as(Integer)", &context)
            .await
            .is_err()
    );
    assert_eq!(
        engine
            .evaluate("(1 | 2).ofType(Integer)", &context)
            .await
            .unwrap()
            .value
            .len(),
        2
    );
}
