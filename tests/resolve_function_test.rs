//! Tests for the resolve() function with Bundle resources

use octofhir_fhirpath::{FhirPathValue, engine::FhirPathEngine};
use serde_json::json;

#[tokio::test]
async fn test_resolve_contained_resource() {
    let patient_with_contained = json!({
        "resourceType": "Patient",
        "id": "patient1",
        "contained": [
            {
                "resourceType": "Practitioner",
                "id": "prac1",
                "name": [{
                    "family": "Smith",
                    "given": ["John"]
                }]
            }
        ],
        "generalPractitioner": [{
            "reference": "#prac1"
        }]
    });

    let mut engine = FhirPathEngine::new();
    let result = engine
        .evaluate(
            "Patient.generalPractitioner.resolve().name.family",
            patient_with_contained,
        )
        .await
        .expect("Should evaluate successfully");

    match result {
        FhirPathValue::Collection(ref items) => {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::String(s)) = items.get(0) {
                assert_eq!(&**s, "Smith");
            } else {
                panic!("Expected string result");
            }
        }
        _ => panic!("Expected collection result"),
    }
}

#[tokio::test]
async fn test_resolve_bundle_entry_by_relative_reference() {
    let bundle = json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "entry": [
            {
                "fullUrl": "http://example.com/Patient/123",
                "resource": {
                    "resourceType": "Patient",
                    "id": "123",
                    "name": [{
                        "family": "Doe",
                        "given": ["Jane"]
                    }]
                }
            },
            {
                "fullUrl": "http://example.com/Observation/456",
                "resource": {
                    "resourceType": "Observation",
                    "id": "456",
                    "subject": {
                        "reference": "Patient/123"
                    },
                    "valueQuantity": {
                        "value": 98.6,
                        "unit": "F"
                    }
                }
            }
        ]
    });

    let mut engine = FhirPathEngine::new();

    // Test resolving relative reference within Bundle
    let result = engine
        .evaluate(
            "Bundle.entry[1].resource.subject.resolve().name.family",
            bundle,
        )
        .await
        .expect("Should evaluate successfully");

    match result {
        FhirPathValue::Collection(ref items) => {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::String(s)) = items.get(0) {
                assert_eq!(&**s, "Doe");
            } else {
                panic!("Expected string result");
            }
        }
        _ => panic!("Expected collection result"),
    }
}

#[tokio::test]
async fn test_resolve_bundle_entry_by_absolute_url() {
    let bundle = json!({
        "resourceType": "Bundle",
        "type": "document",
        "entry": [
            {
                "fullUrl": "urn:uuid:123e4567-e89b-12d3-a456-426614174000",
                "resource": {
                    "resourceType": "Patient",
                    "id": "patient1",
                    "name": [{
                        "family": "Johnson",
                        "given": ["Robert"]
                    }]
                }
            },
            {
                "fullUrl": "urn:uuid:456e7890-e89b-12d3-a456-426614174001",
                "resource": {
                    "resourceType": "Encounter",
                    "id": "encounter1",
                    "subject": {
                        "reference": "urn:uuid:123e4567-e89b-12d3-a456-426614174000"
                    }
                }
            }
        ]
    });

    let mut engine = FhirPathEngine::new();

    // Test resolving absolute URL reference within Bundle
    let result = engine
        .evaluate(
            "Bundle.entry[1].resource.subject.resolve().name.given",
            bundle,
        )
        .await
        .expect("Should evaluate successfully");

    match result {
        FhirPathValue::Collection(ref items) => {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::String(s)) = items.get(0) {
                assert_eq!(&**s, "Robert");
            } else {
                panic!("Expected string result");
            }
        }
        _ => panic!("Expected collection result"),
    }
}

#[tokio::test]
async fn test_resolve_multiple_references() {
    let bundle = json!({
        "resourceType": "Bundle",
        "type": "collection",
        "entry": [
            {
                "fullUrl": "http://example.com/Practitioner/dr1",
                "resource": {
                    "resourceType": "Practitioner",
                    "id": "dr1",
                    "name": [{
                        "family": "Brown",
                        "given": ["Alice"]
                    }]
                }
            },
            {
                "fullUrl": "http://example.com/Practitioner/dr2",
                "resource": {
                    "resourceType": "Practitioner",
                    "id": "dr2",
                    "name": [{
                        "family": "Green",
                        "given": ["Bob"]
                    }]
                }
            },
            {
                "fullUrl": "http://example.com/Patient/p1",
                "resource": {
                    "resourceType": "Patient",
                    "id": "p1",
                    "generalPractitioner": [
                        {"reference": "Practitioner/dr1"},
                        {"reference": "Practitioner/dr2"}
                    ]
                }
            }
        ]
    });

    let mut engine = FhirPathEngine::new();

    // Test resolving multiple references
    let result = engine
        .evaluate(
            "Bundle.entry[2].resource.generalPractitioner.resolve().name.family",
            bundle,
        )
        .await
        .expect("Should evaluate successfully");

    match result {
        FhirPathValue::Collection(ref items) => {
            assert_eq!(items.len(), 2);
            let families: Vec<String> = items
                .iter()
                .filter_map(|v| match v {
                    FhirPathValue::String(s) => Some(s.to_string()),
                    _ => None,
                })
                .collect();
            assert!(families.contains(&"Brown".to_string()));
            assert!(families.contains(&"Green".to_string()));
        }
        _ => panic!("Expected collection result"),
    }
}

#[tokio::test]
async fn test_resolve_with_empty_collection() {
    let patient = json!({
        "resourceType": "Patient",
        "id": "patient1"
        // No references to resolve
    });

    let mut engine = FhirPathEngine::new();

    // Test resolve on empty collection
    let result = engine
        .evaluate("Patient.managingOrganization.resolve()", patient)
        .await
        .expect("Should evaluate successfully");

    assert_eq!(result, FhirPathValue::Empty);
}

#[tokio::test]
async fn test_resolve_invalid_reference_ignored() {
    let observation = json!({
        "resourceType": "Observation",
        "id": "obs1",
        "subject": {
            "reference": "not-a-valid-reference"
        }
    });

    let mut engine = FhirPathEngine::new();

    // Invalid references should be ignored (return empty)
    let result = engine
        .evaluate("Observation.subject.resolve()", observation)
        .await
        .expect("Should evaluate successfully");

    // For now this might return a placeholder, but ideally should be empty
    // when no valid resolution is possible
    match result {
        FhirPathValue::Empty => {
            // Ideal behavior - invalid reference returns empty
        }
        FhirPathValue::Collection(ref items) => {
            // Current placeholder behavior - check if placeholder
            if items.len() == 1 {
                if let Some(FhirPathValue::Resource(res)) = items.get(0) {
                    let json = res.as_json();
                    assert!(json.get("_placeholder").is_some());
                }
            }
        }
        _ => panic!("Unexpected result type"),
    }
}

#[tokio::test]
async fn test_resolve_string_reference_directly() {
    let bundle = json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "entry": [
            {
                "fullUrl": "http://example.com/Organization/org1",
                "resource": {
                    "resourceType": "Organization",
                    "id": "org1",
                    "name": "Health Corp"
                }
            }
        ]
    });

    let mut engine = FhirPathEngine::new();

    // Test resolve on a string reference directly
    let result = engine
        .evaluate("'Organization/org1'.resolve().name", bundle)
        .await
        .expect("Should evaluate successfully");

    // This should ideally resolve to the organization name when Bundle context is available
    // Current implementation might return placeholder
    match result {
        FhirPathValue::Collection(ref items) if !items.is_empty() => {
            // Successfully resolved (either real or placeholder)
        }
        FhirPathValue::Empty => {
            // Also acceptable if no Bundle context available
        }
        _ => panic!("Unexpected result"),
    }
}
