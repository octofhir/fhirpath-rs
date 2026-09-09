use octofhir_fhir_model::EmptyModelProvider;
use octofhir_fhirpath::{FhirPathEngine, create_function_registry};
use octofhir_fhirschema::{FhirSchema, FhirValidator, InMemorySchemaProvider};
use serde_json::json;
use std::sync::Arc;

async fn validator() -> FhirValidator {
    let schema: FhirSchema = serde_json::from_value(json!({
        "name":"Patient","type":"Patient","url":"http://test/Patient","kind":"resource","class":"resource",
        "elements":{
            "id":{"type":"id"},
            "birthDate":{"type":"date","constraint":{
                "date-1":{"expression":"$this is date","human":"Typed date","severity":"error"}
            }},
            "name":{"type":"HumanName","array":true,"elements":{"family":{"type":"string"}},
                "constraint":{
                    "name-1":{"expression":"%context.family.exists() and %rootResource.id = 'good'","human":"Name and root","severity":"error"},
                    "name-2":{"expression":"%context.family.exists() and %rootResource.id = 'good'","human":"Duplicate invariant","severity":"error"},
                    "warning":{"expression":"doesNotExist()","human":"Must be skipped","severity":"warning"}
                }
            }
        }
    })).unwrap();
    let mut provider = InMemorySchemaProvider::new();
    provider.add_schema_owned("Patient", schema);
    let engine = Arc::new(
        FhirPathEngine::new(
            Arc::new(create_function_registry()),
            Arc::new(EmptyModelProvider),
        )
        .await
        .unwrap(),
    );
    FhirValidator::new_with_fhirpath(Arc::new(provider), engine)
}

#[tokio::test]
async fn prepared_validation_preserves_typed_nodes_roots_and_duplicate_diagnostics() {
    let validator = validator().await;
    let mut patient = json!({"resourceType":"Patient","id":"good","birthDate":"2000-01-02","name":[{"family":"Smith"},{"family":"Jones"}]});
    let result = validator
        .validate(&patient, vec!["Patient".into(), "Patient".into()])
        .await;
    assert!(result.valid, "{:?}", result.errors);
    patient["id"] = "bad".into();
    let result = validator
        .validate(&patient, vec!["Patient".into(), "Patient".into()])
        .await;
    assert!(!result.valid);
    assert_eq!(result.errors.len(), 8, "{:?}", result.errors);
    assert!(
        result
            .errors
            .iter()
            .all(|e| e.message.as_ref().unwrap().contains("failed:"))
    );
    patient["id"] = "good".into();
    let result = validator.validate(&patient, vec!["Patient".into()]).await;
    assert!(result.valid, "{:?}", result.errors);
}
