#![cfg(test)]

use super::models::{
    ContextEvaluationInfo,
    ContextItem,
    EvaluationResultItem,
    EvaluationResultSet,
    EvaluationTiming,
    OperationOutcome,
    ParsedServerRequest,
    PathSegment,
    ParametersResource,
};
use super::response::{build_success_response, ParseDebugInfo, ResponseMetadata};
use super::version::ServerFhirVersion;
use super::{handlers, registry};
use octofhir_fhirpath::FhirPathValue;
use octofhir_fhirpath::parser::parse_with_semantic_analysis;
use serde_json::json;
use std::time::Duration;

#[test]
fn operation_outcome_serializes() {
    let outcome = OperationOutcome::error("invalid", "Bad request", Some("details".to_string()));
    let value = serde_json::to_value(outcome).expect("serializes");
    assert_eq!(value["resourceType"], "OperationOutcome");
}

#[tokio::test]
async fn semantic_analysis_succeeds_for_patient_chain() {
    let registry = registry::ServerRegistry::new()
        .await
        .expect("registry");

    let engine = registry
        .get_evaluation_engine(ServerFhirVersion::R4)
        .expect("engine");

    let engine_guard = engine.lock_owned().await;
    let model_provider = engine_guard.get_model_provider();

    let patient_type = model_provider
        .get_type("Patient")
        .await
        .expect("get type")
        .expect("patient type available");

    let result = parse_with_semantic_analysis(
        "Patient.name.given",
        model_provider,
        Some(patient_type),
    )
    .await;

    assert!(result.analysis.success, "semantic analysis should succeed");
}

#[tokio::test]
async fn semantic_analysis_allows_of_type_navigation() {
    let registry = registry::ServerRegistry::new()
        .await
        .expect("registry");

    let engine = registry
        .get_evaluation_engine(ServerFhirVersion::R4)
        .expect("engine");

    let engine_guard = engine.lock_owned().await;
    let model_provider = engine_guard.get_model_provider();

    let result = parse_with_semantic_analysis(
        "Patient.name.ofType(HumanName).use",
        model_provider,
        None,
    )
    .await;

    assert!(
        result.analysis.success,
        "expected success got diagnostics: {:?}",
        result.analysis.diagnostics
    );
}

#[tokio::test]
async fn handle_request_accepts_context_navigation() {
    let registry = registry::ServerRegistry::new()
        .await
        .expect("registry");

    let request_json = json!({
        "resourceType": "Parameters",
        "parameter": [
            { "name": "expression", "valueString": "given" },
            { "name": "context", "valueString": "name" },
            { "name": "validate", "valueBoolean": true },
            { "name": "variables" },
            {
                "name": "resource",
                "resource": {
                    "resourceType": "Patient",
                    "id": "example",
                    "name": [
                        {
                            "use": "official",
                            "family": "Chalmers",
                            "given": ["Peter", "James"]
                        },
                        {
                            "use": "usual",
                            "given": ["Jim"]
                        }
                    ]
                }
            }
        ]
    });

    let parameters: ParametersResource = serde_json::from_value(request_json).expect("parameters");
    let response = handlers::handle_request(&registry, ServerFhirVersion::R4, parameters)
        .await
        .expect("handler response");

    let payload = response.0;
    assert_eq!(payload["resourceType"], "Parameters");
    assert!(payload["parameter"].as_array().unwrap().iter().any(|entry| entry["name"] == "result"));
}

#[tokio::test]
async fn handle_request_accepts_of_type_navigation() {
    let registry = registry::ServerRegistry::new()
        .await
        .expect("registry");

    let request_json = json!({
        "resourceType": "Parameters",
        "parameter": [
            { "name": "expression", "valueString": "Patient.name.ofType(HumanName).use" },
            { "name": "validate", "valueBoolean": true },
            {
                "name": "resource",
                "resource": {
                    "resourceType": "Patient",
                    "id": "example",
                    "name": [
                        { "use": "official", "family": "Chalmers", "given": ["Peter", "James"] }
                    ]
                }
            }
        ]
    });

    let parameters: ParametersResource = serde_json::from_value(request_json).expect("parameters");
    let response = handlers::handle_request(&registry, ServerFhirVersion::R4, parameters)
        .await
        .expect("handler response");

    let payload = response.0;
    assert_eq!(payload["resourceType"], "Parameters");
    assert!(payload["parameter"].as_array().unwrap().iter().any(|entry| entry["name"] == "result"));
}

#[test]
fn path_segments_formatting() {
    let segments = vec![
        PathSegment::Property("name".to_string()),
        PathSegment::Index(0),
        PathSegment::Property("given".to_string()),
    ];
    let path = super::models::path_segments_to_string("Patient", &segments);
    assert_eq!(path, "Patient.name[0].given");
}

#[test]
fn build_response_contains_metadata_and_results() {
    let request = ParsedServerRequest {
        expression: "name.given".to_string(),
        resource: json!({"resourceType": "Patient", "name": []}),
        context: None,
        validate: false,
        variables: Vec::new(),
        terminology_server: None,
    };

    let context_info = ContextEvaluationInfo {
        context_expression: None,
        context_item_count: 1,
        context_success: true,
    };

    let context_item = ContextItem {
        value: FhirPathValue::string("Patient"),
        path: Some("Patient".to_string()),
        path_segments: Vec::new(),
        index: 0,
    };

    let result_item = EvaluationResultItem {
        value: FhirPathValue::string("John"),
        datatype: "string".to_string(),
        path: Some("Patient".to_string()),
        path_segments: Vec::new(),
        index: 0,
    };

    let contextual_result = super::models::ContextualResult {
        context: context_item,
        results: vec![result_item],
        traces: Vec::new(),
    };

    let evaluation = EvaluationResultSet {
        context_info,
        contexts: vec![contextual_result],
        timing: EvaluationTiming {
            parse: Duration::from_millis(1),
            evaluation: Duration::from_millis(1),
            total: Duration::from_millis(2),
        },
    };

    let parse_debug = ParseDebugInfo {
        summary: "summary".to_string(),
        tree: "{}".to_string(),
    };

    let response = build_success_response(
        &request,
        &evaluation,
        ResponseMetadata {
            evaluator_label: "test-engine",
            expected_return_type: Some("string".to_string()),
            parse_debug: &parse_debug,
            semantic_diagnostics: &[],
        },
    );

    assert_eq!(response.resource_type, "Parameters");
    assert_eq!(response.parameter.len(), 2);
    assert_eq!(response.parameter[0].name, "parameters");
    assert_eq!(response.parameter[1].name, "result");
}

#[tokio::test]
async fn handle_request_returns_full_extension_body() {
    let registry = registry::ServerRegistry::new()
        .await
        .expect("registry");

    let request_json = json!({
        "resourceType": "Parameters",
        "parameter": [
            { "name": "expression", "valueString": "birthDate.extension" },
            {
                "name": "resource",
                "resource": {
                    "resourceType": "Patient",
                    "id": "example",
                    "birthDate": "1974-12-25",
                    "_birthDate": {
                        "extension": [
                            {
                                "url": "http://hl7.org/fhir/StructureDefinition/patient-birthTime",
                                "valueDateTime": "1974-12-25T14:35:45-05:00"
                            }
                        ]
                    }
                }
            }
        ]
    });

    let parameters: ParametersResource = serde_json::from_value(request_json).expect("parameters");
    let response = handlers::handle_request(&registry, ServerFhirVersion::R4, parameters)
        .await
        .expect("handler response");

    let payload = response.0;
    assert_eq!(payload["resourceType"], "Parameters");

    // Find the result parameter
    let result_param = payload["parameter"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "result")
        .expect("result parameter should exist");

    // Get the part array which contains the actual results
    let parts = result_param["part"].as_array().expect("result should have parts");

    // Should have exactly 1 result
    assert_eq!(parts.len(), 1, "Should return exactly 1 extension");

    // The first part should contain the extension resource
    let first_result = &parts[0];

    // Get the extension value from the resource field
    let extension_value = first_result
        .get("resource")
        .expect("result should have resource field");

    // The extension value should contain both url and valueDateTime
    assert_eq!(
        extension_value["url"],
        "http://hl7.org/fhir/StructureDefinition/patient-birthTime",
        "Extension should have the correct URL"
    );
    assert_eq!(
        extension_value["valueDateTime"],
        "1974-12-25T14:35:45-05:00",
        "Extension should have the full valueDateTime field"
    );
}
