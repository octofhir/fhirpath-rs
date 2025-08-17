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

//! FHIR Type Registry and Type System
//!
//! This module provides a lightweight type registry for FHIR types without full struct generation.
//! It supports type hierarchy checking and polymorphic element detection.

use std::collections::HashMap;
use std::collections::HashSet;

/// FHIR Type Registry for lightweight type checking and hierarchy support
#[derive(Debug, Clone)]
pub struct FhirTypeRegistry {
    /// Maps type names to their parent types (inheritance hierarchy)
    type_hierarchy: HashMap<String, String>,
    /// Set of all known FHIR types
    known_types: HashSet<String>,
    /// Maps polymorphic element base names to their possible suffixes
    polymorphic_elements: HashMap<String, Vec<String>>,
}

impl Default for FhirTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FhirTypeRegistry {
    /// Create a new FHIR type registry with default type hierarchy
    pub fn new() -> Self {
        let mut registry = FhirTypeRegistry {
            type_hierarchy: HashMap::new(),
            known_types: HashSet::new(),
            polymorphic_elements: HashMap::new(),
        };

        registry.initialize_type_hierarchy();
        registry.initialize_polymorphic_elements();
        registry
    }

    /// Initialize the basic FHIR type hierarchy
    fn initialize_type_hierarchy(&mut self) {
        // System types (primitive types)
        let system_types = vec![
            "System.Boolean",
            "System.Integer",
            "System.Decimal",
            "System.String",
            "System.Date",
            "System.DateTime",
            "System.Time",
            "System.Quantity",
        ];

        // FHIR primitive types
        let primitive_types = vec![
            "boolean",
            "integer",
            "decimal",
            "string",
            "date",
            "dateTime",
            "time",
            "instant",
            "uri",
            "url",
            "canonical",
            "oid",
            "uuid",
            "id",
            "markdown",
            "base64Binary",
            "code",
            "unsignedInt",
            "positiveInt",
            "xhtml",
        ];

        // FHIR complex types (inherit from Element)
        let complex_types = vec![
            "Quantity",
            "CodeableConcept",
            "Coding",
            "Identifier",
            "HumanName",
            "Address",
            "ContactPoint",
            "Period",
            "Range",
            "Ratio",
            "SampledData",
            "Attachment",
            "Annotation",
            "Signature",
            "Money",
            "Duration",
            "Count",
            "Distance",
            "Age",
            "SimpleQuantity",
            "Timing",
            "Dosage",
            "Meta",
            "Narrative",
            "Extension",
            "Reference",
            "ContactDetail",
            "Contributor",
            "DataRequirement",
            "Expression",
            "ParameterDefinition",
            "RelatedArtifact",
            "TriggerDefinition",
            "UsageContext",
        ];

        // FHIR resources (all inherit from Resource)
        let resource_types = vec![
            // Base resources
            "Resource",
            "DomainResource",
            // Clinical resources
            "Patient",
            "Practitioner",
            "PractitionerRole",
            "Organization",
            "Location",
            "HealthcareService",
            "Endpoint",
            "Person",
            "RelatedPerson",
            "Group",
            // Clinical summary
            "AllergyIntolerance",
            "Condition",
            "Procedure",
            "FamilyMemberHistory",
            "ClinicalImpression",
            "DetectedIssue",
            "RiskAssessment",
            // Diagnostics
            "Observation",
            "DiagnosticReport",
            "ImagingStudy",
            "Specimen",
            "BodyStructure",
            "Media",
            "DocumentReference",
            "DocumentManifest",
            // Medications
            "Medication",
            "MedicationRequest",
            "MedicationDispense",
            "MedicationStatement",
            "MedicationAdministration",
            "MedicationKnowledge",
            "Immunization",
            "ImmunizationEvaluation",
            "ImmunizationRecommendation",
            // Care provision
            "CarePlan",
            "CareTeam",
            "Goal",
            "ServiceRequest",
            "NutritionOrder",
            "VisionPrescription",
            "RequestGroup",
            "Communication",
            "CommunicationRequest",
            // Request & response
            "DeviceRequest",
            "DeviceUseStatement",
            "GuidanceResponse",
            "SupplyRequest",
            "SupplyDelivery",
            "Task",
            // Foundation
            "Encounter",
            "EpisodeOfCare",
            "Flag",
            "List",
            "Library",
            "Basic",
            // Security
            "AuditEvent",
            "Provenance",
            "Consent",
            // Documents
            "Composition",
            "Bundle",
            // Financial
            "Account",
            "Coverage",
            "CoverageEligibilityRequest",
            "CoverageEligibilityResponse",
            "EnrollmentRequest",
            "EnrollmentResponse",
            "Claim",
            "ClaimResponse",
            "Invoice",
            "PaymentNotice",
            "PaymentReconciliation",
            "ExplanationOfBenefit",
            // Workflow
            "Appointment",
            "AppointmentResponse",
            "Schedule",
            "Slot",
            "VerificationResult",
            // Specialized
            "ResearchStudy",
            "ResearchSubject",
            "ActivityDefinition",
            "DeviceDefinition",
            "EventDefinition",
            "ObservationDefinition",
            "PlanDefinition",
            "Questionnaire",
            "QuestionnaireResponse",
            "Measure",
            "MeasureReport",
            "TestScript",
            "TestReport",
            // Terminology
            "CodeSystem",
            "ValueSet",
            "ConceptMap",
            "NamingSystem",
            "TerminologyCapabilities",
            // Conformance
            "CapabilityStatement",
            "StructureDefinition",
            "ImplementationGuide",
            "SearchParameter",
            "MessageDefinition",
            "OperationDefinition",
            "CompartmentDefinition",
            "StructureMap",
            "GraphDefinition",
            "ExampleScenario",
            // Operations
            "Parameters",
            "OperationOutcome",
            // Infrastructure
            "Binary",
            "MessageHeader",
            "Subscription",
            "SubscriptionStatus",
            "SubscriptionTopic",
        ];

        // Add all types to known_types set
        for type_name in &system_types {
            self.known_types.insert(type_name.to_string());
        }
        for type_name in &primitive_types {
            self.known_types.insert(type_name.to_string());
        }
        for type_name in &complex_types {
            self.known_types.insert(type_name.to_string());
        }
        for type_name in &resource_types {
            self.known_types.insert(type_name.to_string());
        }

        // Set up inheritance hierarchy
        // Complex types inherit from Element
        for type_name in &complex_types {
            self.type_hierarchy
                .insert(type_name.to_string(), "Element".to_string());
        }

        // All resources inherit from Resource
        for type_name in &resource_types {
            if *type_name != "Resource" {
                if *type_name == "DomainResource" {
                    self.type_hierarchy
                        .insert(type_name.to_string(), "Resource".to_string());
                } else if !matches!(
                    *type_name,
                    "Binary" | "Bundle" | "Parameters" | "OperationOutcome"
                ) {
                    // Most resources inherit from DomainResource
                    self.type_hierarchy
                        .insert(type_name.to_string(), "DomainResource".to_string());
                } else {
                    // Some resources inherit directly from Resource
                    self.type_hierarchy
                        .insert(type_name.to_string(), "Resource".to_string());
                }
            }
        }

        // Add base types
        self.known_types.insert("Element".to_string());
        self.known_types.insert("BackboneElement".to_string());
        self.type_hierarchy
            .insert("BackboneElement".to_string(), "Element".to_string());
    }

    /// Initialize polymorphic element mappings
    fn initialize_polymorphic_elements(&mut self) {
        // Common FHIR polymorphic elements with their possible type suffixes
        let polymorphic_mappings = vec![
            (
                "value",
                vec![
                    "Boolean",
                    "Integer",
                    "Decimal",
                    "String",
                    "Date",
                    "DateTime",
                    "Time",
                    "Instant",
                    "Uri",
                    "Canonical",
                    "Base64Binary",
                    "Code",
                    "Oid",
                    "Id",
                    "UnsignedInt",
                    "PositiveInt",
                    "Markdown",
                    "Uuid",
                    "Url",
                    "Quantity",
                    "CodeableConcept",
                    "Coding",
                    "Attachment",
                    "Reference",
                    "Period",
                    "Range",
                    "Ratio",
                    "SampledData",
                    "Signature",
                    "HumanName",
                    "Address",
                    "ContactPoint",
                    "Timing",
                    "Meta",
                    "Identifier",
                ],
            ),
            ("effective", vec!["DateTime", "Period", "Timing", "Instant"]),
            (
                "onset",
                vec!["DateTime", "Age", "Period", "Range", "String"],
            ),
            (
                "abatement",
                vec!["DateTime", "Age", "Boolean", "Period", "Range", "String"],
            ),
            ("occurrence", vec!["DateTime", "Period", "Timing"]),
            (
                "performed",
                vec!["DateTime", "Period", "String", "Age", "Range"],
            ),
            ("deceased", vec!["Boolean", "DateTime"]),
            ("multipleBirth", vec!["Boolean", "Integer"]),
            ("bodySite", vec!["CodeableConcept", "Reference"]),
            ("method", vec!["CodeableConcept", "Reference"]),
            ("specimen", vec!["Reference", "CodeableConcept"]),
            ("device", vec!["Reference", "CodeableConcept"]),
            ("reason", vec!["CodeableConcept", "Reference"]),
            ("medication", vec!["CodeableConcept", "Reference"]),
            ("subject", vec!["Reference", "CodeableConcept"]),
        ];

        for (base_name, suffixes) in polymorphic_mappings {
            self.polymorphic_elements.insert(
                base_name.to_string(),
                suffixes.into_iter().map(|s| s.to_string()).collect(),
            );
        }
    }

    /// Check if a type is known in the registry
    pub fn is_known_type(&self, type_name: &str) -> bool {
        let normalized_name = self.normalize_type_name(type_name);
        self.known_types.contains(&normalized_name)
    }

    /// Check if a value of source_type can be considered as target_type (including inheritance)
    pub fn is_type_compatible(&self, source_type: &str, target_type: &str) -> bool {
        let source = self.normalize_type_name(source_type);
        let target = self.normalize_type_name(target_type);

        // Direct match
        if source == target {
            return true;
        }

        // Check inheritance hierarchy
        self.is_subtype_of(&source, &target)
    }

    /// Check if source_type is a subtype of target_type
    fn is_subtype_of(&self, source_type: &str, target_type: &str) -> bool {
        let mut current_type = source_type.to_string();

        // Walk up the inheritance hierarchy
        while let Some(parent_type) = self.type_hierarchy.get(&current_type) {
            if parent_type == target_type {
                return true;
            }
            current_type = parent_type.clone();
        }

        false
    }

    /// Normalize type name (handle case variations and System. prefixes)
    fn normalize_type_name(&self, type_name: &str) -> String {
        // Handle System. prefixed types
        if type_name.starts_with("System.") {
            return type_name.to_string();
        }

        // Handle FHIR. prefixed types
        if let Some(stripped) = type_name.strip_prefix("FHIR.") {
            return stripped.to_string();
        }

        // For other types, preserve original case for exact matching
        type_name.to_string()
    }

    /// Check if an element name is polymorphic
    pub fn is_polymorphic_element(&self, element_name: &str) -> bool {
        self.polymorphic_elements.contains_key(element_name)
    }

    /// Get possible type suffixes for a polymorphic element
    pub fn get_polymorphic_suffixes(&self, element_name: &str) -> Option<&Vec<String>> {
        self.polymorphic_elements.get(element_name)
    }

    /// Find polymorphic property in a JSON object
    pub fn find_polymorphic_property_name(
        &self,
        properties: &std::collections::BTreeMap<String, serde_json::Value>,
        base_name: &str,
    ) -> Option<String> {
        if let Some(suffixes) = self.get_polymorphic_suffixes(base_name) {
            for suffix in suffixes {
                let property_name = format!("{base_name}{suffix}");
                if properties.contains_key(&property_name) {
                    return Some(property_name);
                }
            }
        }

        // Fallback: look for any property that starts with the base name
        for key in properties.keys() {
            if key.starts_with(base_name) && key.len() > base_name.len() {
                // Make sure the next character is uppercase (following FHIR naming convention)
                if let Some(next_char) = key.chars().nth(base_name.len()) {
                    if next_char.is_uppercase() {
                        return Some(key.clone());
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_registry_creation() {
        let registry = FhirTypeRegistry::new();
        assert!(registry.is_known_type("Patient"));
        assert!(registry.is_known_type("Observation"));
        assert!(registry.is_known_type("string"));
        assert!(registry.is_known_type("System.String"));
    }

    #[test]
    fn test_type_compatibility() {
        let registry = FhirTypeRegistry::new();

        // Direct match
        assert!(registry.is_type_compatible("Patient", "Patient"));

        // Inheritance: Patient -> DomainResource -> Resource
        assert!(registry.is_type_compatible("Patient", "DomainResource"));
        assert!(registry.is_type_compatible("Patient", "Resource"));

        // Not compatible
        assert!(!registry.is_type_compatible("Patient", "Observation"));
    }

    #[test]
    fn test_polymorphic_elements() {
        let registry = FhirTypeRegistry::new();

        assert!(registry.is_polymorphic_element("value"));
        assert!(registry.is_polymorphic_element("effective"));
        assert!(!registry.is_polymorphic_element("name"));

        let value_suffixes = registry.get_polymorphic_suffixes("value").unwrap();
        assert!(value_suffixes.contains(&"String".to_string()));
        assert!(value_suffixes.contains(&"Quantity".to_string()));
    }

    #[test]
    fn test_normalize_type_name() {
        let registry = FhirTypeRegistry::new();

        assert_eq!(
            registry.normalize_type_name("System.String"),
            "System.String"
        );
        assert_eq!(registry.normalize_type_name("FHIR.Patient"), "Patient");
        assert_eq!(registry.normalize_type_name("Patient"), "Patient");
    }
}
