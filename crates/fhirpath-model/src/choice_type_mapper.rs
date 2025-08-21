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

//! Choice Type Mapping System for FHIR Polymorphic Properties
//!
//! This module provides comprehensive mapping and resolution of FHIR choice types
//! (value[x] patterns) to enable proper navigation of polymorphic properties.

use sonic_rs::JsonValueTrait;
use std::collections::HashMap;
use std::sync::Arc;

/// Comprehensive choice type mapping and resolution
#[derive(Debug, Clone)]
pub struct ChoiceTypeMapper {
    /// Maps base property to possible choice types
    /// e.g., "value" -> ["valueString", "valueQuantity", "valueBoolean", ...]
    choice_mappings: HashMap<String, Vec<ChoiceVariant>>,

    /// Maps type-specific properties back to base
    /// e.g., "valueQuantity" -> "value"
    reverse_mappings: HashMap<String, String>,

    /// Type priority for ambiguous cases
    type_priority: HashMap<String, u32>,
}

/// Represents a specific choice type variant
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceVariant {
    /// The specific property name (e.g., "valueQuantity")
    pub property_name: String,

    /// The target type (e.g., "Quantity")
    pub target_type: String,

    /// FHIR type code for matching (e.g., "Quantity", "string", "boolean")
    pub type_code: String,

    /// Priority for type resolution (lower = higher priority)
    pub priority: u32,
}

impl ChoiceTypeMapper {
    /// Create mapper with comprehensive FHIR R4 definitions
    pub fn new_fhir_r4() -> Self {
        let mut mapper = Self {
            choice_mappings: HashMap::new(),
            reverse_mappings: HashMap::new(),
            type_priority: HashMap::new(),
        };

        // Register all FHIR R4 choice types
        mapper.register_observation_choice_types();
        mapper.register_patient_choice_types();
        mapper.register_medication_choice_types();
        mapper.register_common_choice_types();

        mapper
    }

    /// Register Observation resource choice types
    fn register_observation_choice_types(&mut self) {
        self.register_choice_type(
            "Observation",
            "value",
            vec![
                ChoiceVariant {
                    property_name: "valueQuantity".to_string(),
                    target_type: "Quantity".to_string(),
                    type_code: "Quantity".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "valueCodeableConcept".to_string(),
                    target_type: "CodeableConcept".to_string(),
                    type_code: "CodeableConcept".to_string(),
                    priority: 2,
                },
                ChoiceVariant {
                    property_name: "valueString".to_string(),
                    target_type: "string".to_string(),
                    type_code: "string".to_string(),
                    priority: 3,
                },
                ChoiceVariant {
                    property_name: "valueBoolean".to_string(),
                    target_type: "boolean".to_string(),
                    type_code: "boolean".to_string(),
                    priority: 4,
                },
                ChoiceVariant {
                    property_name: "valueInteger".to_string(),
                    target_type: "integer".to_string(),
                    type_code: "integer".to_string(),
                    priority: 5,
                },
                ChoiceVariant {
                    property_name: "valueRange".to_string(),
                    target_type: "Range".to_string(),
                    type_code: "Range".to_string(),
                    priority: 6,
                },
                ChoiceVariant {
                    property_name: "valueRatio".to_string(),
                    target_type: "Ratio".to_string(),
                    type_code: "Ratio".to_string(),
                    priority: 7,
                },
                ChoiceVariant {
                    property_name: "valueSampledData".to_string(),
                    target_type: "SampledData".to_string(),
                    type_code: "SampledData".to_string(),
                    priority: 8,
                },
                ChoiceVariant {
                    property_name: "valueTime".to_string(),
                    target_type: "time".to_string(),
                    type_code: "time".to_string(),
                    priority: 9,
                },
                ChoiceVariant {
                    property_name: "valueDateTime".to_string(),
                    target_type: "dateTime".to_string(),
                    type_code: "dateTime".to_string(),
                    priority: 10,
                },
                ChoiceVariant {
                    property_name: "valuePeriod".to_string(),
                    target_type: "Period".to_string(),
                    type_code: "Period".to_string(),
                    priority: 11,
                },
            ],
        );
    }

    /// Register Patient resource choice types
    fn register_patient_choice_types(&mut self) {
        self.register_choice_type(
            "Patient",
            "deceased",
            vec![
                ChoiceVariant {
                    property_name: "deceasedBoolean".to_string(),
                    target_type: "boolean".to_string(),
                    type_code: "boolean".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "deceasedDateTime".to_string(),
                    target_type: "dateTime".to_string(),
                    type_code: "dateTime".to_string(),
                    priority: 2,
                },
            ],
        );

        self.register_choice_type(
            "Patient",
            "multipleBirth",
            vec![
                ChoiceVariant {
                    property_name: "multipleBirthBoolean".to_string(),
                    target_type: "boolean".to_string(),
                    type_code: "boolean".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "multipleBirthInteger".to_string(),
                    target_type: "integer".to_string(),
                    type_code: "integer".to_string(),
                    priority: 2,
                },
            ],
        );
    }

    /// Register Medication resource choice types
    fn register_medication_choice_types(&mut self) {
        self.register_choice_type(
            "Medication",
            "effective",
            vec![
                ChoiceVariant {
                    property_name: "effectiveDateTime".to_string(),
                    target_type: "dateTime".to_string(),
                    type_code: "dateTime".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "effectivePeriod".to_string(),
                    target_type: "Period".to_string(),
                    type_code: "Period".to_string(),
                    priority: 2,
                },
            ],
        );
    }

    /// Register common choice types across multiple resources
    fn register_common_choice_types(&mut self) {
        // Condition onset choice types
        self.register_choice_type(
            "Condition",
            "onset",
            vec![
                ChoiceVariant {
                    property_name: "onsetDateTime".to_string(),
                    target_type: "dateTime".to_string(),
                    type_code: "dateTime".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "onsetAge".to_string(),
                    target_type: "Age".to_string(),
                    type_code: "Age".to_string(),
                    priority: 2,
                },
                ChoiceVariant {
                    property_name: "onsetPeriod".to_string(),
                    target_type: "Period".to_string(),
                    type_code: "Period".to_string(),
                    priority: 3,
                },
                ChoiceVariant {
                    property_name: "onsetRange".to_string(),
                    target_type: "Range".to_string(),
                    type_code: "Range".to_string(),
                    priority: 4,
                },
                ChoiceVariant {
                    property_name: "onsetString".to_string(),
                    target_type: "string".to_string(),
                    type_code: "string".to_string(),
                    priority: 5,
                },
            ],
        );

        // AllergyIntolerance onset choice types
        self.register_choice_type(
            "AllergyIntolerance",
            "onset",
            vec![
                ChoiceVariant {
                    property_name: "onsetDateTime".to_string(),
                    target_type: "dateTime".to_string(),
                    type_code: "dateTime".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "onsetAge".to_string(),
                    target_type: "Age".to_string(),
                    type_code: "Age".to_string(),
                    priority: 2,
                },
                ChoiceVariant {
                    property_name: "onsetPeriod".to_string(),
                    target_type: "Period".to_string(),
                    type_code: "Period".to_string(),
                    priority: 3,
                },
                ChoiceVariant {
                    property_name: "onsetRange".to_string(),
                    target_type: "Range".to_string(),
                    type_code: "Range".to_string(),
                    priority: 4,
                },
                ChoiceVariant {
                    property_name: "onsetString".to_string(),
                    target_type: "string".to_string(),
                    type_code: "string".to_string(),
                    priority: 5,
                },
            ],
        );

        // Dosage dose choice types
        self.register_choice_type(
            "Dosage",
            "dose",
            vec![
                ChoiceVariant {
                    property_name: "doseRange".to_string(),
                    target_type: "Range".to_string(),
                    type_code: "Range".to_string(),
                    priority: 1,
                },
                ChoiceVariant {
                    property_name: "doseQuantity".to_string(),
                    target_type: "Quantity".to_string(),
                    type_code: "Quantity".to_string(),
                    priority: 2,
                },
            ],
        );
    }

    /// Register a choice type for a resource
    pub fn register_choice_type(
        &mut self,
        resource_type: &str,
        base_property: &str,
        variants: Vec<ChoiceVariant>,
    ) {
        let key = format!("{resource_type}.{base_property}");

        // Store forward mappings
        self.choice_mappings.insert(key.clone(), variants.clone());

        // Store reverse mappings
        for variant in &variants {
            let specific_key = format!("{}.{}", resource_type, variant.property_name);
            self.reverse_mappings.insert(specific_key, key.clone());

            // Store type priority
            self.type_priority
                .insert(variant.target_type.clone(), variant.priority);
        }
    }

    /// Get choice variants for a resource property
    pub fn get_choice_variants(
        &self,
        resource_type: &str,
        property: &str,
    ) -> Option<&Vec<ChoiceVariant>> {
        let key = format!("{resource_type}.{property}");
        self.choice_mappings.get(&key)
    }

    /// Resolve choice property to actual property based on data
    pub fn resolve_choice_property(
        &self,
        resource_type: &str,
        base_property: &str,
        json_data: &sonic_rs::Value,
    ) -> Option<String> {
        if let Some(variants) = self.get_choice_variants(resource_type, base_property) {
            // Check which specific property exists in the data
            for variant in variants {
                if json_data.get(&variant.property_name).is_some() {
                    return Some(variant.property_name.clone());
                }
            }
        }
        None
    }

    /// Get target type for a choice variant
    pub fn get_target_type(&self, resource_type: &str, property: &str) -> Option<String> {
        // Check if this is a base choice property
        if let Some(variants) = self.get_choice_variants(resource_type, property) {
            // Return the highest priority variant type
            variants
                .iter()
                .min_by_key(|v| v.priority)
                .map(|v| v.target_type.clone())
        } else {
            // Check if this is a specific choice property variant
            let specific_key = format!("{resource_type}.{property}");
            if let Some(base_key) = self.reverse_mappings.get(&specific_key) {
                // Extract the base property from the key
                let base_property = base_key.split('.').nth(1)?;
                if let Some(variants) = self.get_choice_variants(resource_type, base_property) {
                    // Find the specific variant
                    variants
                        .iter()
                        .find(|v| v.property_name == property)
                        .map(|v| v.target_type.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// Check if a property is a choice type base property
    pub fn is_choice_property(&self, resource_type: &str, property: &str) -> bool {
        let key = format!("{resource_type}.{property}");
        self.choice_mappings.contains_key(&key)
    }

    /// Check if a property is a specific choice variant
    pub fn is_choice_variant(&self, resource_type: &str, property: &str) -> bool {
        let key = format!("{resource_type}.{property}");
        self.reverse_mappings.contains_key(&key)
    }

    /// Get the base property for a specific choice variant
    pub fn get_base_property(&self, resource_type: &str, variant_property: &str) -> Option<String> {
        let key = format!("{resource_type}.{variant_property}");
        self.reverse_mappings
            .get(&key)
            .and_then(|base_key| base_key.split('.').nth(1).map(|s| s.to_string()))
    }

    /// Get all possible variant properties for a base property
    pub fn get_variant_properties(&self, resource_type: &str, base_property: &str) -> Vec<String> {
        self.get_choice_variants(resource_type, base_property)
            .map(|variants| variants.iter().map(|v| v.property_name.clone()).collect())
            .unwrap_or_default()
    }

    /// Resolve all possible target types for a choice property
    pub fn get_all_target_types(&self, resource_type: &str, property: &str) -> Vec<String> {
        self.get_choice_variants(resource_type, property)
            .map(|variants| variants.iter().map(|v| v.target_type.clone()).collect())
            .unwrap_or_default()
    }
}

/// Thread-safe wrapper for ChoiceTypeMapper
pub type SharedChoiceTypeMapper = Arc<ChoiceTypeMapper>;

impl Default for ChoiceTypeMapper {
    fn default() -> Self {
        Self::new_fhir_r4()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sonic_rs::json;

    #[test]
    fn test_observation_choice_types() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let variants = mapper.get_choice_variants("Observation", "value").unwrap();

        assert!(variants.iter().any(|v| v.property_name == "valueQuantity"));
        assert!(variants.iter().any(|v| v.property_name == "valueString"));
        assert!(variants.iter().any(|v| v.property_name == "valueBoolean"));

        // Check priorities
        let quantity_variant = variants
            .iter()
            .find(|v| v.property_name == "valueQuantity")
            .unwrap();
        assert_eq!(quantity_variant.priority, 1);
    }

    #[test]
    fn test_choice_property_resolution() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let observation_with_quantity = json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        });

        let resolved =
            mapper.resolve_choice_property("Observation", "value", &observation_with_quantity);

        assert_eq!(resolved, Some("valueQuantity".to_string()));
    }

    #[test]
    fn test_choice_property_resolution_string() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let observation_with_string = json!({
            "resourceType": "Observation",
            "valueString": "Normal"
        });

        let resolved =
            mapper.resolve_choice_property("Observation", "value", &observation_with_string);

        assert_eq!(resolved, Some("valueString".to_string()));
    }

    #[test]
    fn test_target_type_resolution() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let target_type = mapper.get_target_type("Observation", "value");
        assert_eq!(target_type, Some("Quantity".to_string())); // Highest priority

        let specific_target = mapper.get_target_type("Observation", "valueQuantity");
        assert_eq!(specific_target, Some("Quantity".to_string()));

        let string_target = mapper.get_target_type("Observation", "valueString");
        assert_eq!(string_target, Some("string".to_string()));
    }

    #[test]
    fn test_is_choice_property() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        assert!(mapper.is_choice_property("Observation", "value"));
        assert!(mapper.is_choice_property("Patient", "deceased"));
        assert!(!mapper.is_choice_property("Patient", "name"));
    }

    #[test]
    fn test_is_choice_variant() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        assert!(mapper.is_choice_variant("Observation", "valueQuantity"));
        assert!(mapper.is_choice_variant("Observation", "valueString"));
        assert!(mapper.is_choice_variant("Patient", "deceasedBoolean"));
        assert!(!mapper.is_choice_variant("Patient", "name"));
    }

    #[test]
    fn test_get_base_property() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let base = mapper.get_base_property("Observation", "valueQuantity");
        assert_eq!(base, Some("value".to_string()));

        let base_deceased = mapper.get_base_property("Patient", "deceasedBoolean");
        assert_eq!(base_deceased, Some("deceased".to_string()));
    }

    #[test]
    fn test_get_variant_properties() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let variants = mapper.get_variant_properties("Observation", "value");
        assert!(variants.contains(&"valueQuantity".to_string()));
        assert!(variants.contains(&"valueString".to_string()));
        assert!(variants.contains(&"valueBoolean".to_string()));
    }

    #[test]
    fn test_patient_choice_types() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let variants = mapper.get_choice_variants("Patient", "deceased").unwrap();
        assert_eq!(variants.len(), 2);

        assert!(
            variants
                .iter()
                .any(|v| v.property_name == "deceasedBoolean")
        );
        assert!(
            variants
                .iter()
                .any(|v| v.property_name == "deceasedDateTime")
        );
    }

    #[test]
    fn test_no_resolution_when_property_missing() {
        let mapper = ChoiceTypeMapper::new_fhir_r4();

        let observation_without_value = json!({
            "resourceType": "Observation",
            "status": "final"
        });

        let resolved =
            mapper.resolve_choice_property("Observation", "value", &observation_without_value);

        assert_eq!(resolved, None);
    }
}
