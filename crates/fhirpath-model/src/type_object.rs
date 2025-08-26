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

//! FHIRPath type object implementation with namespace and name properties
//!
//! Provides the complete type reflection system required for FHIRPath compliance,
//! enabling proper `type().namespace` and `type().name` functionality.

use crate::{FhirPathValue, JsonValue};
use sonic_rs::{JsonValueTrait, json};
use std::sync::Arc;

/// FHIRPath type object with namespace and name properties
///
/// This represents the result of calling `type()` on a FHIRPath value,
/// providing access to namespace and name via property navigation.
#[derive(Debug, Clone, PartialEq)]
pub struct FhirPathTypeObject {
    /// Type namespace ("System" or "FHIR")
    pub namespace: String,
    /// Type name (e.g., "Integer", "Patient", "boolean")
    pub name: String,
    /// Optional base type for inheritance
    pub base_type: Option<String>,
    /// Additional metadata
    pub metadata: TypeObjectMetadata,
}

/// Additional metadata for type objects
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypeObjectMetadata {
    /// Whether this is a primitive type
    pub is_primitive: bool,
    /// Whether this is a resource type
    pub is_resource: bool,
    /// Whether this is an abstract type
    pub is_abstract: bool,
    /// Available properties (for ClassInfo types)
    pub properties: Vec<String>,
}

impl FhirPathTypeObject {
    /// Create System namespace type object
    pub fn system_type(name: impl Into<String>) -> Self {
        Self {
            namespace: "System".to_string(),
            name: name.into(),
            base_type: None,
            metadata: TypeObjectMetadata {
                is_primitive: true,
                ..Default::default()
            },
        }
    }

    /// Create FHIR namespace type object
    pub fn fhir_type(name: impl Into<String>, base_type: Option<String>) -> Self {
        let name_str = name.into();
        let is_resource = Self::is_resource_type(&name_str);

        Self {
            namespace: "FHIR".to_string(),
            name: name_str,
            base_type,
            metadata: TypeObjectMetadata {
                is_primitive: false,
                is_resource,
                ..Default::default()
            },
        }
    }

    /// Check if type name represents a FHIR resource
    fn is_resource_type(type_name: &str) -> bool {
        // Common FHIR resource types
        matches!(
            type_name,
            "Patient"
                | "Observation"
                | "Practitioner"
                | "Organization"
                | "Bundle"
                | "Condition"
                | "Procedure"
                | "MedicationRequest"
                | "DiagnosticReport"
                | "Encounter"
                | "AllergyIntolerance"
                | "Immunization"
                | "Location"
                | "Device"
                | "Medication"
                | "Substance"
                | "ValueSet"
                | "CodeSystem"
                | "StructureDefinition"
                | "CapabilityStatement"
                | "SearchParameter"
                | "OperationDefinition"
                | "CompartmentDefinition"
                | "ImplementationGuide"
                | "DomainResource"
                | "Resource" // Base resource types
        )
    }

    /// Convert to FhirPathValue with accessible properties
    ///
    /// Creates a JsonValue representation that allows property access
    /// via navigation (e.g., `type().namespace`, `type().name`)
    pub fn to_fhir_path_value(&self) -> FhirPathValue {
        let json_obj = json!({
            "namespace": self.namespace,
            "name": self.name,
            "baseType": self.base_type,
            "isPrimitive": self.metadata.is_primitive,
            "isResource": self.metadata.is_resource,
            "isAbstract": self.metadata.is_abstract,
            "properties": self.metadata.properties
        });

        FhirPathValue::JsonValue(JsonValue::new(json_obj))
    }

    /// Alternative: Convert to TypeInfoObject (current implementation)
    ///
    /// This creates the specialized TypeInfoObject variant that needs
    /// special handling in the navigation evaluator.
    pub fn to_type_info_object(&self) -> FhirPathValue {
        FhirPathValue::TypeInfoObject {
            namespace: Arc::from(self.namespace.as_str()),
            name: Arc::from(self.name.as_str()),
        }
    }
}

// Re-export ValueTypeAnalyzer from the dedicated module
pub use crate::type_analyzer::ValueTypeAnalyzer;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_type_object() {
        let type_obj = FhirPathTypeObject::system_type("Boolean");
        assert_eq!(type_obj.namespace, "System");
        assert_eq!(type_obj.name, "Boolean");
        assert!(type_obj.metadata.is_primitive);
        assert!(!type_obj.metadata.is_resource);
    }

    #[test]
    fn test_fhir_type_object() {
        let type_obj = FhirPathTypeObject::fhir_type("Patient", Some("DomainResource".to_string()));
        assert_eq!(type_obj.namespace, "FHIR");
        assert_eq!(type_obj.name, "Patient");
        assert!(!type_obj.metadata.is_primitive);
        assert!(type_obj.metadata.is_resource);
        assert_eq!(type_obj.base_type, Some("DomainResource".to_string()));
    }

    #[test]
    fn test_to_fhir_path_value() {
        let type_obj = FhirPathTypeObject::system_type("Integer");
        let value = type_obj.to_fhir_path_value();

        match value {
            FhirPathValue::JsonValue(json) => {
                let sonic_val = json.as_sonic_value();
                assert_eq!(
                    sonic_val.get("namespace").and_then(|v| v.as_str()),
                    Some("System")
                );
                assert_eq!(
                    sonic_val.get("name").and_then(|v| v.as_str()),
                    Some("Integer")
                );
                assert_eq!(
                    sonic_val.get("isPrimitive").and_then(|v| v.as_bool()),
                    Some(true)
                );
            }
            _ => panic!("Expected JsonValue"),
        }
    }

    #[tokio::test]
    async fn test_value_type_analyzer() {
        // Test System Boolean
        let bool_val = FhirPathValue::Boolean(true);
        let type_obj = ValueTypeAnalyzer::get_type_object(&bool_val, None)
            .await
            .unwrap();
        assert_eq!(type_obj.namespace, "System");
        assert_eq!(type_obj.name, "Boolean");

        // Test Integer
        let int_val = FhirPathValue::Integer(42);
        let type_obj = ValueTypeAnalyzer::get_type_object(&int_val, None)
            .await
            .unwrap();
        assert_eq!(type_obj.namespace, "System");
        assert_eq!(type_obj.name, "Integer");

        // Test String
        let str_val = FhirPathValue::String("hello".into());
        let type_obj = ValueTypeAnalyzer::get_type_object(&str_val, None)
            .await
            .unwrap();
        assert_eq!(type_obj.namespace, "System");
        assert_eq!(type_obj.name, "String");
    }

    #[tokio::test]
    async fn test_fhir_json_type_inference() {
        // Test FHIR boolean (should be FHIR.boolean, not System.Boolean)
        let json_bool = JsonValue::new(json!(true));
        let bool_val = FhirPathValue::JsonValue(json_bool);
        let type_obj = ValueTypeAnalyzer::get_type_object(&bool_val, None)
            .await
            .unwrap();
        assert_eq!(type_obj.namespace, "FHIR");
        assert_eq!(type_obj.name, "boolean");

        // Test Patient resource
        let patient_json = JsonValue::new(json!({"resourceType": "Patient", "id": "123"}));
        let patient_val = FhirPathValue::JsonValue(patient_json);
        let type_obj = ValueTypeAnalyzer::get_type_object(&patient_val, None)
            .await
            .unwrap();
        assert_eq!(type_obj.namespace, "FHIR");
        assert_eq!(type_obj.name, "Patient");
    }
}
