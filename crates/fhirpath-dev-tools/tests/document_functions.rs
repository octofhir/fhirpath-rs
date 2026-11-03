//! Replay the unmodified document extension cases with deterministic terminology.
use async_trait::async_trait;
use fhirpath_dev_tools::{TestSuite, verify_output_types};
use octofhir_fhir_model::terminology::{
    ConnectionStatus, ExpansionParameters, LookupResult, SubsumptionOutcome, SubsumptionResult,
    TerminologyProvider, TranslationResult, ValidationResult, ValueSetExpansion,
};
use octofhir_fhirpath::{
    Collection, EvaluationContext, FhirPathEngine, FhirPathValue, create_function_registry,
};
use octofhir_fhirschema::{EmbeddedSchemaProvider, ModelFhirVersion, ModelProvider};
use serde_json::Value;
use std::{path::PathBuf, sync::Arc};
#[derive(Debug)]
struct DocumentTerminology;

#[async_trait]
impl TerminologyProvider for DocumentTerminology {
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
        if valueset != "http://hl7.org/fhir/ValueSet/data-absent-reason" {
            return Err(octofhir_fhir_model::ModelError::validation_error(
                "Unknown value set",
            ));
        }
        let member = system == Some("http://terminology.hl7.org/CodeSystem/data-absent-reason")
            && matches!(
                code,
                "unknown" | "not-applicable" | "unsupported" | "not-asked"
            );
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

#[tokio::test]
async fn official_document_cases_r5_and_r6() {
    let mut failures = vec![];
    for (directory, version) in [("r5", ModelFhirVersion::R5), ("r6", ModelFhirVersion::R6)] {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/official")
            .join(directory);
        let suite: TestSuite =
            serde_json::from_slice(&std::fs::read(base.join("documentFunctions.json")).unwrap())
                .unwrap();
        let model = Arc::new(EmbeddedSchemaProvider::new(version));
        let engine = FhirPathEngine::new(Arc::new(create_function_registry()), model.clone())
            .await
            .unwrap();
        for case in suite.tests {
            let resource: Value = serde_json::from_slice(
                &std::fs::read(base.join("input").join(case.inputfile.as_ref().unwrap())).unwrap(),
            )
            .unwrap();
            let context = EvaluationContext::new(
                Collection::single(FhirPathValue::resource(resource)),
                model.clone(),
                Some(Arc::new(DocumentTerminology)),
                None,
                None,
            );
            if case.invalid_kind.as_deref() == Some("semantic") {
                let root = model.get_type("Bundle").await.unwrap();
                let result = octofhir_fhirpath::parser::parse_with_semantic_analysis(
                    &case.expression,
                    model.clone(),
                    root,
                )
                .await;
                if result.analysis.success {
                    failures.push(format!("{directory} {}: missing semantic error", case.name));
                }
                continue;
            }
            let result = engine.evaluate(&case.expression, &context).await;
            if case.expect_error == Some(true) {
                if result.is_ok() {
                    failures.push(format!("{directory} {}: missing error", case.name));
                }
                continue;
            }
            match result {
                Err(error) => failures.push(format!("{directory} {}: {error}", case.name)),
                Ok(result) => {
                    let values: Vec<Value> = result
                        .value
                        .iter()
                        .map(FhirPathValue::to_json_value)
                        .collect();
                    if Value::Array(values.clone()) != case.expected {
                        failures.push(format!(
                            "{directory} {}: got {values:?}, expected {}",
                            case.name, case.expected
                        ));
                    } else if let Err(error) =
                        verify_output_types(&case.output_types, &result.value)
                    {
                        failures.push(format!(
                            "{directory} {}: type mismatch: {:?} != {:?}",
                            case.name, error.actual, error.expected
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
async fn document_results_can_be_analyzed_and_navigated() {
    for version in [ModelFhirVersion::R5, ModelFhirVersion::R6] {
        let model = Arc::new(EmbeddedSchemaProvider::new(version));
        let focus = model.get_type("Bundle").await.unwrap();
        for expression in [
            "Bundle.section('s', 'c').entry('s', 'c').id",
            "Bundle.resources('s', 'c', 'Observation').value",
            "Bundle.resources('s', 'c').category.coding.code",
            "Bundle.resources('s', 'c').value.toString()",
            "Bundle.resources('s', 'c').code.hasCoding('s', 'c')",
        ] {
            let result = octofhir_fhirpath::parser::parse_with_semantic_analysis(
                expression,
                model.clone(),
                focus.clone(),
            )
            .await;
            assert!(
                result.analysis.success,
                "{expression}: {:?}",
                result.analysis.diagnostics
            );
        }
    }
}
