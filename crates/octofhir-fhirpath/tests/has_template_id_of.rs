//! `hasTemplateIdOf(profileUrl)` of the CDA core specification ("FHIRPath Supplements"): true
//! when the element has a `templateId` corresponding to the identifier of the profile.

use std::collections::HashMap;
use std::sync::Arc;

use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::evaluator::functions::{HasTemplateIdOfFunctionEvaluator, TemplateId};
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::json;

const SECTION: &str = "http://hl7.org/cda/us/custom/StructureDefinition/ExampleSection";

async fn evaluate(expression: &str, resolve: bool) -> Vec<FhirPathValue> {
    let document = FhirPathValue::resource(json!({
        "component": { "structuredBody": { "component": [
            { "section": { "templateId": [
                { "root": "2.16.840.1.113883.10.20.22.99.999" },
                { "root": "2.16.840.1.113883.10.20.22.99.999", "extension": "2024-05-01" }
            ] } },
            { "section": { "templateId": [{ "root": "2.16.840.1.113883.10.20.22.2.6.1", "extension": "2015-08-01" }] } }
        ] } }
    }));
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(document),
        model.clone(),
        None,
        None,
        None,
    );
    let mut registry = create_function_registry();
    if resolve {
        let mut profiles: HashMap<String, Vec<TemplateId>> = HashMap::new();
        profiles.insert(
            SECTION.to_owned(),
            vec![(
                "2.16.840.1.113883.10.20.22.99.999".to_owned(),
                Some("2024-05-01".to_owned()),
            )],
        );
        registry.register_pure_function(HasTemplateIdOfFunctionEvaluator::with_resolver(Arc::new(
            profiles,
        )));
    }
    let engine = FhirPathEngine::new(Arc::new(registry), model)
        .await
        .expect("engine creation");
    engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation")
        .value
        .into_vec()
}

fn count(values: &[FhirPathValue]) -> usize {
    values.len()
}

#[tokio::test]
async fn the_specification_example() {
    // `%resource.component.structuredBody.component.where(section.hasTemplateIdOf('…/ExampleSection'))`
    let found = evaluate(
        &format!("component.structuredBody.component.where(section.hasTemplateIdOf('{SECTION}'))"),
        true,
    )
    .await;
    assert_eq!(count(&found), 1);
    // The same as the expression it stands for.
    let longhand = evaluate(
        "component.structuredBody.component.where(section.templateId.where(root = '2.16.840.1.113883.10.20.22.99.999' and extension = '2024-05-01').exists())",
        false,
    )
    .await;
    assert_eq!(count(&longhand), 1);
}

#[tokio::test]
async fn a_profile_url_without_a_resolver_matches_nothing() {
    let found = evaluate(
        &format!("component.structuredBody.component.where(section.hasTemplateIdOf('{SECTION}'))"),
        false,
    )
    .await;
    assert_eq!(count(&found), 0);
}

#[tokio::test]
async fn template_identifiers_work_without_a_resolver() {
    let versioned = evaluate(
        "component.structuredBody.component.where(section.hasTemplateIdOf('urn:hl7ii:2.16.840.1.113883.10.20.22.2.6.1:2015-08-01'))",
        false,
    )
    .await;
    assert_eq!(count(&versioned), 1);
    let other_version = evaluate(
        "component.structuredBody.component.where(section.hasTemplateIdOf('urn:hl7ii:2.16.840.1.113883.10.20.22.2.6.1:2014-06-09'))",
        false,
    )
    .await;
    assert_eq!(count(&other_version), 0);
    // A template without an extension matches its root.
    let any_version = evaluate(
        "component.structuredBody.component.where(section.hasTemplateIdOf('urn:oid:2.16.840.1.113883.10.20.22.2.6.1'))",
        false,
    )
    .await;
    assert_eq!(count(&any_version), 1);
}
