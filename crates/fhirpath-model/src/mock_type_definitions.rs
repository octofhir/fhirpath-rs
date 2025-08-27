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

//! Basic FHIR type definitions for testing with MockModelProvider

use crate::provider::{ElementInfo, TypeReflectionInfo};

/// Utility for creating basic FHIR type definitions for testing
pub struct MockTypeDefinitions;

impl MockTypeDefinitions {
    /// Initialize basic FHIR types for testing
    pub fn populate_provider(provider: &mut crate::MockModelProvider) {
        // Add primitive types
        provider.add_type(
            "boolean".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
                base_type: None,
            },
        );

        provider.add_type(
            "integer".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Integer".to_string(),
                base_type: None,
            },
        );

        provider.add_type(
            "string".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            },
        );

        // Add Patient resource
        let patient_elements = vec![
            ElementInfo {
                name: "active".to_string(),
                type_info: TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "Boolean".to_string(),
                    base_type: None,
                },
                min_cardinality: 0,
                max_cardinality: Some(1),
                is_modifier: false,
                is_summary: false,
                documentation: Some("Whether the patient record is active".to_string()),
            },
            ElementInfo {
                name: "name".to_string(),
                type_info: TypeReflectionInfo::ListType {
                    element_type: Box::new(TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: "HumanName".to_string(),
                        base_type: Some("Element".to_string()),
                        elements: vec![],
                    }),
                },
                min_cardinality: 0,
                max_cardinality: None,
                is_modifier: false,
                is_summary: true,
                documentation: Some("A human name for the patient".to_string()),
            },
        ];

        provider.add_type(
            "Patient".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "Patient".to_string(),
                base_type: Some("DomainResource".to_string()),
                elements: patient_elements,
            },
        );

        // Add HumanName complex type
        let human_name_elements = vec![
            ElementInfo {
                name: "family".to_string(),
                type_info: TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "String".to_string(),
                    base_type: None,
                },
                min_cardinality: 0,
                max_cardinality: Some(1),
                is_modifier: false,
                is_summary: true,
                documentation: Some("Family name".to_string()),
            },
            ElementInfo {
                name: "given".to_string(),
                type_info: TypeReflectionInfo::ListType {
                    element_type: Box::new(TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: "String".to_string(),
                        base_type: None,
                    }),
                },
                min_cardinality: 0,
                max_cardinality: None,
                is_modifier: false,
                is_summary: true,
                documentation: Some("Given names".to_string()),
            },
        ];

        provider.add_type(
            "HumanName".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "HumanName".to_string(),
                base_type: Some("Element".to_string()),
                elements: human_name_elements,
            },
        );

        // Add properties for easy lookup
        provider.add_property(
            "Patient".to_string(),
            "active".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
                base_type: None,
            },
        );

        provider.add_property(
            "Patient".to_string(),
            "name".to_string(),
            TypeReflectionInfo::ListType {
                element_type: Box::new(TypeReflectionInfo::ClassInfo {
                    namespace: "FHIR".to_string(),
                    name: "HumanName".to_string(),
                    base_type: Some("Element".to_string()),
                    elements: vec![],
                }),
            },
        );

        provider.add_property(
            "HumanName".to_string(),
            "family".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            },
        );

        provider.add_property(
            "HumanName".to_string(),
            "given".to_string(),
            TypeReflectionInfo::ListType {
                element_type: Box::new(TypeReflectionInfo::SimpleType {
                    namespace: "System".to_string(),
                    name: "String".to_string(),
                    base_type: None,
                }),
            },
        );

        // Add Observation resource type
        provider.add_type(
            "Observation".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "Observation".to_string(),
                base_type: Some("DomainResource".to_string()),
                elements: vec![],
            },
        );

        // Add Quantity type
        provider.add_type(
            "Quantity".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "Quantity".to_string(),
                base_type: Some("Element".to_string()),
                elements: vec![],
            },
        );

        // Add Observation properties
        provider.add_property(
            "Observation".to_string(),
            "valueQuantity".to_string(),
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: "Quantity".to_string(),
                base_type: Some("Element".to_string()),
                elements: vec![],
            },
        );

        provider.add_property(
            "Observation".to_string(),
            "valueString".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            },
        );

        provider.add_property(
            "Observation".to_string(),
            "valueBoolean".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
                base_type: None,
            },
        );

        // Add Quantity properties
        provider.add_property(
            "Quantity".to_string(),
            "value".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "Decimal".to_string(),
                base_type: None,
            },
        );

        provider.add_property(
            "Quantity".to_string(),
            "unit".to_string(),
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: "String".to_string(),
                base_type: None,
            },
        );
    }
}
