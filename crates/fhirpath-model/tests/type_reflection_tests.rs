// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive tests for the FHIRPath type reflection system

use octofhir_fhirpath_model::{FhirPathTypeObject, FhirPathValue, JsonValue, ValueTypeAnalyzer};
use serde_json::{ json};
use std::sync::Arc;

#[tokio::test]
async fn test_system_type_reflection_boolean() {
    let bool_val = FhirPathValue::Boolean(true);
    let type_obj = ValueTypeAnalyzer::get_type_object(&bool_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "System");
    assert_eq!(type_obj.name, "Boolean");
    assert!(type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
}

#[tokio::test]
async fn test_system_type_reflection_integer() {
    let int_val = FhirPathValue::Integer(42);
    let type_obj = ValueTypeAnalyzer::get_type_object(&int_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "System");
    assert_eq!(type_obj.name, "Integer");
    assert!(type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
}

#[tokio::test]
async fn test_system_type_reflection_string() {
    let str_val = FhirPathValue::String("hello".into());
    let type_obj = ValueTypeAnalyzer::get_type_object(&str_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "System");
    assert_eq!(type_obj.name, "String");
    assert!(type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
}

#[tokio::test]
async fn test_fhir_type_reflection_patient() {
    let patient_json = json!({
        "resourceType": "Patient",
        "id": "example",
        "active": true
    });
    let json_val = FhirPathValue::JsonValue(JsonValue::new(patient_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "Patient");
    assert!(!type_obj.metadata.is_primitive);
    assert!(type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("DomainResource".to_string()));
}

#[tokio::test]
async fn test_fhir_primitive_type_boolean() {
    // FHIR boolean primitive (as JSON) should be FHIR.boolean, not System.Boolean
    let fhir_bool_json = json!(true);
    let json_val = FhirPathValue::JsonValue(JsonValue::new(fhir_bool_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "boolean"); // Note: lowercase for FHIR primitives
    assert!(!type_obj.metadata.is_primitive); // FHIR primitives are not System primitives
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_primitive_type_string() {
    let fhir_string_json = json!("hello");
    let json_val = FhirPathValue::JsonValue(JsonValue::new(fhir_string_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "string"); // Note: lowercase for FHIR primitives
    assert!(!type_obj.metadata.is_primitive); // FHIR primitives are not System primitives
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_primitive_type_integer() {
    let fhir_int_json = json!(42);
    let json_val = FhirPathValue::JsonValue(JsonValue::new(fhir_int_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "integer"); // Note: lowercase for FHIR primitives
    assert!(!type_obj.metadata.is_primitive); // FHIR primitives are not System primitives
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_complex_type_coding() {
    let coding_json = json!({
        "system": "http://hl7.org/fhir/administrative-gender",
        "code": "male",
        "display": "Male"
    });
    let json_val = FhirPathValue::JsonValue(JsonValue::new(coding_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "Coding");
    assert!(!type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_complex_type_quantity() {
    let quantity_json = json!({
        "value": 185,
        "unit": "lbs",
        "system": "http://unitsofmeasure.org",
        "code": "[lb_av]"
    });
    let json_val = FhirPathValue::JsonValue(JsonValue::new(quantity_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "Quantity");
    assert!(!type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_complex_type_reference() {
    let reference_json = json!({
        "reference": "Patient/example",
        "display": "Amy Shaw"
    });
    let json_val = FhirPathValue::JsonValue(JsonValue::new(reference_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "Reference");
    assert!(!type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_complex_type_human_name() {
    let name_json = json!({
        "use": "official",
        "family": "Doe",
        "given": ["John", "F."]
    });
    let json_val = FhirPathValue::JsonValue(JsonValue::new(name_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "HumanName");
    assert!(!type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_fhir_complex_type_address() {
    let address_json = json!({
        "line": ["123 Main Street"],
        "city": "Springfield",
        "state": "IL",
        "postalCode": "62701"
    });
    let json_val = FhirPathValue::JsonValue(JsonValue::new(address_json));
    let type_obj = ValueTypeAnalyzer::get_type_object(&json_val, None)
        .await
        .unwrap();

    assert_eq!(type_obj.namespace, "FHIR");
    assert_eq!(type_obj.name, "Address");
    assert!(!type_obj.metadata.is_primitive);
    assert!(!type_obj.metadata.is_resource);
    assert_eq!(type_obj.base_type, Some("Element".to_string()));
}

#[tokio::test]
async fn test_type_object_to_fhir_path_value() {
    let type_obj = FhirPathTypeObject::system_type("Boolean");
    let value = type_obj.to_fhir_path_value();

    match value {
        FhirPathValue::JsonValue(json) => {
            let sonic_val = json.as_value();
            assert_eq!(
                sonic_val.get("namespace").and_then(|v| v.as_str()),
                Some("System")
            );
            assert_eq!(
                sonic_val.get("name").and_then(|v| v.as_str()),
                Some("Boolean")
            );
            assert_eq!(
                sonic_val.get("isPrimitive").and_then(|v| v.as_bool()),
                Some(true)
            );
            assert_eq!(
                sonic_val.get("isResource").and_then(|v| v.as_bool()),
                Some(false)
            );
        }
        _ => panic!("Expected JsonValue for type object representation"),
    }
}

#[tokio::test]
async fn test_type_object_to_type_info_object() {
    let type_obj = FhirPathTypeObject::fhir_type("Patient", Some("DomainResource".to_string()));
    let value = type_obj.to_type_info_object();

    match value {
        FhirPathValue::TypeInfoObject { namespace, name } => {
            assert_eq!(namespace.as_ref(), "FHIR");
            assert_eq!(name.as_ref(), "Patient");
        }
        _ => panic!("Expected TypeInfoObject"),
    }
}

#[tokio::test]
async fn test_collection_type_inference() {
    // Test that collections return the type of their first element
    let items = vec![
        FhirPathValue::String("hello".into()),
        FhirPathValue::String("world".into()),
    ];
    let collection =
        FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from_vec(items));

    let type_obj = ValueTypeAnalyzer::get_type_object(&collection, None)
        .await
        .unwrap();
    assert_eq!(type_obj.namespace, "System");
    assert_eq!(type_obj.name, "String");
    assert!(type_obj.metadata.is_primitive);
}

#[tokio::test]
async fn test_empty_collection_error() {
    let empty_collection = FhirPathValue::Collection(octofhir_fhirpath_model::Collection::new());

    let result = ValueTypeAnalyzer::get_type_object(&empty_collection, None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Empty collection has no type"));
}

#[tokio::test]
async fn test_type_info_object_self_reflection() {
    let type_info = FhirPathValue::TypeInfoObject {
        namespace: Arc::from("System"),
        name: Arc::from("Boolean"),
    };

    let type_obj = ValueTypeAnalyzer::get_type_object(&type_info, None)
        .await
        .unwrap();
    assert_eq!(type_obj.namespace, "System");
    assert_eq!(type_obj.name, "Boolean");
    assert!(type_obj.metadata.is_primitive); // System namespace = primitive
}

#[tokio::test]
async fn test_value_type_analyzer_convenience_methods() {
    // Test get_type_name
    let bool_val = FhirPathValue::Boolean(true);
    let type_name = ValueTypeAnalyzer::get_type_name(&bool_val).await;
    assert_eq!(type_name, Some("Boolean".to_string()));

    // Test get_namespace
    let namespace = ValueTypeAnalyzer::get_namespace(&bool_val).await;
    assert_eq!(namespace, Some("System".to_string()));

    // Test with FHIR type
    let patient_json = json!({"resourceType": "Patient", "id": "test"});
    let patient_val = FhirPathValue::JsonValue(JsonValue::new(patient_json));
    let type_name = ValueTypeAnalyzer::get_type_name(&patient_val).await;
    assert_eq!(type_name, Some("Patient".to_string()));
    let namespace = ValueTypeAnalyzer::get_namespace(&patient_val).await;
    assert_eq!(namespace, Some("FHIR".to_string()));
}

#[tokio::test]
async fn test_type_compatibility_checking() {
    let bool_val = FhirPathValue::Boolean(true);

    // Direct match
    assert!(ValueTypeAnalyzer::is_compatible_with_type(&bool_val, "System", "Boolean").await);

    // Wrong namespace
    assert!(!ValueTypeAnalyzer::is_compatible_with_type(&bool_val, "FHIR", "Boolean").await);

    // Wrong name
    assert!(!ValueTypeAnalyzer::is_compatible_with_type(&bool_val, "System", "String").await);
}

#[tokio::test]
async fn test_performance_benchmark() {
    // Test that type determination is fast
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        let bool_val = FhirPathValue::Boolean(true);
        let _type_obj = ValueTypeAnalyzer::get_type_object(&bool_val, None)
            .await
            .unwrap();
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "Type analysis too slow: {elapsed:?}"
    );
}

#[test]
fn test_type_object_constructors() {
    // Test system type constructor
    let system_type = FhirPathTypeObject::system_type("Integer");
    assert_eq!(system_type.namespace, "System");
    assert_eq!(system_type.name, "Integer");
    assert!(system_type.metadata.is_primitive);
    assert!(!system_type.metadata.is_resource);
    assert_eq!(system_type.base_type, None);

    // Test FHIR type constructor
    let fhir_type = FhirPathTypeObject::fhir_type("Patient", Some("DomainResource".to_string()));
    assert_eq!(fhir_type.namespace, "FHIR");
    assert_eq!(fhir_type.name, "Patient");
    assert!(!fhir_type.metadata.is_primitive);
    assert!(fhir_type.metadata.is_resource);
    assert_eq!(fhir_type.base_type, Some("DomainResource".to_string()));

    // Test non-resource FHIR type
    let element_type = FhirPathTypeObject::fhir_type("HumanName", Some("Element".to_string()));
    assert_eq!(element_type.namespace, "FHIR");
    assert_eq!(element_type.name, "HumanName");
    assert!(!element_type.metadata.is_primitive);
    assert!(!element_type.metadata.is_resource); // HumanName is not a resource
    assert_eq!(element_type.base_type, Some("Element".to_string()));
}
