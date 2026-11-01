//! `memberOf()` (FHIRPath terminology functions): the value set, system and
//! code reach the terminology provider in the trait's order, a CodeableConcept
//! is a member when any of its codings is, and an empty input propagates.

use std::sync::Arc;

use async_trait::async_trait;
use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhir_model::terminology::{
    ConnectionStatus, ExpansionParameters, LookupResult, SubsumptionOutcome, SubsumptionResult,
    TerminologyProvider, TranslationResult, ValidationResult, ValueSetExpansion,
};
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use serde_json::{Value, json};

const VITALS: &str = "http://example.org/ValueSet/vitals";
const LOINC: &str = "http://loinc.org";

/// Knows one value set with one member, LOINC 8867-4. Answers by matching the
/// arguments exactly, so swapped arguments are a non-member, not a pass.
#[derive(Debug)]
struct OneMemberValueSet;

#[async_trait]
impl TerminologyProvider for OneMemberValueSet {
    async fn validate_code(
        &self,
        _code: &str,
        _system: &str,
        _version: Option<&str>,
    ) -> octofhir_fhir_model::Result<bool> {
        Ok(false)
    }

    async fn expand_valueset(
        &self,
        _valueset_url: &str,
        _parameters: Option<&ExpansionParameters>,
    ) -> octofhir_fhir_model::Result<ValueSetExpansion> {
        Ok(ValueSetExpansion {
            contains: Vec::new(),
            total: Some(0),
            parameters: Vec::new(),
            timestamp: None,
        })
    }

    async fn translate_code(
        &self,
        _source_code: &str,
        _target_system: &str,
        _concept_map_url: Option<&str>,
    ) -> octofhir_fhir_model::Result<TranslationResult> {
        Ok(TranslationResult {
            success: false,
            targets: Vec::new(),
            message: None,
        })
    }

    async fn lookup_code(
        &self,
        _system: &str,
        _code: &str,
        _version: Option<&str>,
        _properties: Option<Vec<&str>>,
    ) -> octofhir_fhir_model::Result<LookupResult> {
        Ok(LookupResult {
            display: None,
            definition: None,
            properties: Vec::new(),
        })
    }

    async fn validate_code_vs(
        &self,
        valueset: &str,
        system: Option<&str>,
        code: &str,
        _display: Option<&str>,
    ) -> octofhir_fhir_model::Result<ValidationResult> {
        let member = valueset == VITALS && system.is_none_or(|s| s == LOINC) && code == "8867-4";
        Ok(ValidationResult {
            result: member,
            display: None,
            message: None,
        })
    }

    async fn subsumes(
        &self,
        _system: &str,
        _parent: &str,
        _child: &str,
    ) -> octofhir_fhir_model::Result<SubsumptionResult> {
        Ok(SubsumptionResult {
            outcome: SubsumptionOutcome::NotSubsumed,
        })
    }

    async fn test_connection(&self) -> octofhir_fhir_model::Result<ConnectionStatus> {
        Ok(ConnectionStatus {
            connected: true,
            response_time_ms: None,
            server_version: None,
            error: None,
        })
    }
}

async fn evaluate(resource: Value, expression: &str) -> Vec<FhirPathValue> {
    let model = Arc::new(EmptyModelProvider);
    let context = EvaluationContext::new(
        Collection::single(FhirPathValue::resource(resource)),
        model.clone(),
        Some(Arc::new(OneMemberValueSet)),
        None,
        None,
    );
    let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model)
        .await
        .expect("engine creation");
    engine
        .evaluate(expression, &context)
        .await
        .expect("evaluation")
        .value
        .iter()
        .cloned()
        .collect()
}

fn observation(codings: Value) -> Value {
    json!({ "resourceType": "Observation", "status": "final", "code": { "coding": codings } })
}

fn as_bools(values: &[FhirPathValue]) -> Vec<bool> {
    values
        .iter()
        .map(|v| match v {
            FhirPathValue::Boolean(b, _, _) => *b,
            other => panic!("expected a boolean, got {other:?}"),
        })
        .collect()
}

#[tokio::test]
async fn a_member_coding_is_a_member() {
    let obs = observation(json!([{ "system": LOINC, "code": "8867-4" }]));
    let result = evaluate(obs, &format!("code.memberOf('{VITALS}')")).await;
    assert_eq!(as_bools(&result), vec![true]);
}

#[tokio::test]
async fn a_concept_is_one_answer_true_when_any_coding_is_a_member() {
    let obs = observation(json!([
        { "system": "http://snomed.info/sct", "code": "364075005" },
        { "system": LOINC, "code": "8867-4" }
    ]));
    let result = evaluate(obs, &format!("code.memberOf('{VITALS}')")).await;
    assert_eq!(as_bools(&result), vec![true]);
}

#[tokio::test]
async fn a_concept_with_no_member_coding_is_not_a_member() {
    let obs = observation(json!([{ "system": LOINC, "code": "29463-7" }]));
    let result = evaluate(obs, &format!("code.memberOf('{VITALS}')")).await;
    assert_eq!(as_bools(&result), vec![false]);
}

#[tokio::test]
async fn a_coding_input_is_tested_directly() {
    let obs = observation(json!([{ "system": LOINC, "code": "8867-4" }]));
    let result = evaluate(obs, &format!("code.coding.first().memberOf('{VITALS}')")).await;
    assert_eq!(as_bools(&result), vec![true]);
}

#[tokio::test]
async fn an_empty_input_is_empty() {
    let obs = json!({ "resourceType": "Observation", "status": "final" });
    let result = evaluate(obs, &format!("code.memberOf('{VITALS}')")).await;
    assert!(result.is_empty(), "expected empty, got {result:?}");
}
