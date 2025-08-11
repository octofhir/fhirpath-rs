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

//! Type mapping system for converting FHIR types to FHIRPath types

use super::types::TypeInfo;
use std::collections::HashMap;

/// Maps FHIR type names to FHIRPath TypeInfo
pub struct TypeMapper {
    /// Cached mappings for performance
    cache: HashMap<String, TypeInfo>,
}

impl TypeMapper {
    /// Create a new type mapper
    pub fn new() -> Self {
        let mut mapper = Self {
            cache: HashMap::new(),
        };
        mapper.initialize_mappings();
        mapper
    }

    /// Map a FHIR type name to FHIRPath TypeInfo
    pub fn map_fhir_type(&self, fhir_type: &str) -> Option<TypeInfo> {
        self.cache.get(fhir_type).cloned()
    }

    /// Map a FHIR primitive type to FHIRPath type
    pub fn map_primitive_type(&self, primitive_type: &str) -> Option<TypeInfo> {
        match primitive_type {
            "boolean" => Some(TypeInfo::Boolean),
            "integer" | "int" | "positiveInt" | "unsignedInt" => Some(TypeInfo::Integer),
            "decimal" => Some(TypeInfo::Decimal),
            "string" | "code" | "id" | "markdown" | "uri" | "url" | "canonical" | "oid"
            | "uuid" => Some(TypeInfo::String),
            "date" => Some(TypeInfo::Date),
            "dateTime" | "instant" => Some(TypeInfo::DateTime),
            "time" => Some(TypeInfo::Time),
            "Quantity" => Some(TypeInfo::Quantity),
            _ => None,
        }
    }

    /// Map a FHIR complex type to FHIRPath type
    pub fn map_complex_type(&self, complex_type: &str) -> Option<TypeInfo> {
        match complex_type {
            "CodeableConcept" => Some(TypeInfo::Resource("CodeableConcept".to_string())),
            "Coding" => Some(TypeInfo::Resource("Coding".to_string())),
            "Identifier" => Some(TypeInfo::Resource("Identifier".to_string())),
            "Reference" => Some(TypeInfo::Resource("Reference".to_string())),
            "Quantity" => Some(TypeInfo::Quantity),
            "Period" => Some(TypeInfo::Resource("Period".to_string())),
            "Range" => Some(TypeInfo::Resource("Range".to_string())),
            "Ratio" => Some(TypeInfo::Resource("Ratio".to_string())),
            "SampledData" => Some(TypeInfo::Resource("SampledData".to_string())),
            "Attachment" => Some(TypeInfo::Resource("Attachment".to_string())),
            "ContactPoint" => Some(TypeInfo::Resource("ContactPoint".to_string())),
            "HumanName" => Some(TypeInfo::Resource("HumanName".to_string())),
            "Address" => Some(TypeInfo::Resource("Address".to_string())),
            "Money" => Some(TypeInfo::Resource("Money".to_string())),
            "Age" => Some(TypeInfo::Quantity),
            "Count" => Some(TypeInfo::Quantity),
            "Distance" => Some(TypeInfo::Quantity),
            "Duration" => Some(TypeInfo::Quantity),
            _ => None,
        }
    }

    /// Map a FHIR resource type to FHIRPath type
    pub fn map_resource_type(&self, resource_type: &str) -> Option<TypeInfo> {
        // FHIR R4/R5 resource types
        match resource_type {
            // Administrative resources
            "Patient"
            | "Practitioner"
            | "PractitionerRole"
            | "RelatedPerson"
            | "Person"
            | "Group"
            | "Organization"
            | "OrganizationAffiliation"
            | "Location"
            | "HealthcareService"
            | "Endpoint" => Some(TypeInfo::Resource(resource_type.to_string())),

            // Clinical resources
            "AllergyIntolerance"
            | "Condition"
            | "Procedure"
            | "Observation"
            | "DiagnosticReport"
            | "ImagingStudy"
            | "Media"
            | "Specimen"
            | "BodyStructure"
            | "MolecularSequence"
            | "DocumentReference"
            | "DocumentManifest"
            | "List"
            | "Composition"
            | "Encounter"
            | "EpisodeOfCare"
            | "ClinicalImpression"
            | "DetectedIssue"
            | "RiskAssessment"
            | "FamilyMemberHistory"
            | "CarePlan"
            | "CareTeam"
            | "Goal"
            | "ServiceRequest"
            | "NutritionOrder"
            | "VisionPrescription"
            | "RequestGroup"
            | "DeviceRequest"
            | "DeviceUseStatement"
            | "SupplyRequest"
            | "SupplyDelivery"
            | "ImmunizationEvaluation"
            | "ImmunizationRecommendation"
            | "Immunization"
            | "MedicationRequest"
            | "MedicationDispense"
            | "MedicationAdministration"
            | "MedicationStatement"
            | "MedicationKnowledge"
            | "Medication"
            | "Substance"
            | "SubstanceSpecification"
            | "SubstancePolymer"
            | "SubstanceReferenceInformation"
            | "SubstanceSourceMaterial"
            | "SubstanceProtein"
            | "SubstanceNucleicAcid" => Some(TypeInfo::Resource(resource_type.to_string())),

            // Foundation resources
            "Resource" | "DomainResource" | "Element" | "BackboneElement" => {
                Some(TypeInfo::Resource(resource_type.to_string()))
            }

            // Financial resources
            "Account"
            | "ChargeItem"
            | "ChargeItemDefinition"
            | "Contract"
            | "Coverage"
            | "CoverageEligibilityRequest"
            | "CoverageEligibilityResponse"
            | "EnrollmentRequest"
            | "EnrollmentResponse"
            | "Claim"
            | "ClaimResponse"
            | "Invoice"
            | "PaymentNotice"
            | "PaymentReconciliation"
            | "ExplanationOfBenefit"
            | "InsurancePlan" => Some(TypeInfo::Resource(resource_type.to_string())),

            // Workflow resources
            "Appointment"
            | "AppointmentResponse"
            | "Schedule"
            | "Slot"
            | "Task"
            | "Communication"
            | "CommunicationRequest" => Some(TypeInfo::Resource(resource_type.to_string())),

            // Definitional resources
            "ActivityDefinition"
            | "PlanDefinition"
            | "Questionnaire"
            | "QuestionnaireResponse"
            | "Measure"
            | "MeasureReport"
            | "Library"
            | "EventDefinition"
            | "ObservationDefinition"
            | "SpecimenDefinition"
            | "DeviceDefinition"
            | "Device"
            | "DeviceMetric" => Some(TypeInfo::Resource(resource_type.to_string())),

            // Other/Unknown resources - default to Resource type
            _ if resource_type
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false) =>
            {
                Some(TypeInfo::Resource(resource_type.to_string()))
            }

            _ => None,
        }
    }

    /// Map special FHIR types that have specific FHIRPath handling
    pub fn map_special_type(&self, special_type: &str) -> Option<TypeInfo> {
        match special_type {
            "Quantity" | "Age" | "Count" | "Distance" | "Duration" => Some(TypeInfo::Quantity),
            "Period" => Some(TypeInfo::Resource("Period".to_string())),
            "Range" => Some(TypeInfo::Resource("Range".to_string())),
            "SimpleQuantity" => Some(TypeInfo::Quantity),
            "MoneyQuantity" => Some(TypeInfo::Resource("Money".to_string())),
            _ => None,
        }
    }

    /// Check if a type is a collection type based on cardinality
    pub fn is_collection_type(&self, _type_name: &str, max_cardinality: Option<u32>) -> bool {
        max_cardinality.map(|max| max > 1).unwrap_or(false)
    }

    /// Wrap a type as a collection if needed
    pub fn wrap_as_collection(&self, type_info: TypeInfo, is_collection: bool) -> TypeInfo {
        if is_collection {
            TypeInfo::Collection(Box::new(type_info))
        } else {
            type_info
        }
    }

    /// Initialize the type mapping cache with common types
    fn initialize_mappings(&mut self) {
        // Primitive types
        self.cache.insert("boolean".to_string(), TypeInfo::Boolean);
        self.cache.insert("integer".to_string(), TypeInfo::Integer);
        self.cache.insert("decimal".to_string(), TypeInfo::Decimal);
        self.cache.insert("string".to_string(), TypeInfo::String);
        self.cache.insert("date".to_string(), TypeInfo::Date);
        self.cache
            .insert("dateTime".to_string(), TypeInfo::DateTime);
        self.cache.insert("time".to_string(), TypeInfo::Time);

        // FHIR primitive type aliases
        self.cache.insert("code".to_string(), TypeInfo::String);
        self.cache.insert("id".to_string(), TypeInfo::String);
        self.cache.insert("markdown".to_string(), TypeInfo::String);
        self.cache.insert("uri".to_string(), TypeInfo::String);
        self.cache.insert("url".to_string(), TypeInfo::String);
        self.cache.insert("canonical".to_string(), TypeInfo::String);
        self.cache.insert("oid".to_string(), TypeInfo::String);
        self.cache.insert("uuid".to_string(), TypeInfo::String);
        self.cache
            .insert("positiveInt".to_string(), TypeInfo::Integer);
        self.cache
            .insert("unsignedInt".to_string(), TypeInfo::Integer);
        self.cache.insert("instant".to_string(), TypeInfo::DateTime);

        // Quantity types
        self.cache
            .insert("Quantity".to_string(), TypeInfo::Quantity);
        self.cache.insert("Age".to_string(), TypeInfo::Quantity);
        self.cache.insert("Count".to_string(), TypeInfo::Quantity);
        self.cache
            .insert("Distance".to_string(), TypeInfo::Quantity);
        self.cache
            .insert("Duration".to_string(), TypeInfo::Quantity);

        // Complex types
        self.cache.insert(
            "CodeableConcept".to_string(),
            TypeInfo::Resource("CodeableConcept".to_string()),
        );
        self.cache.insert(
            "Coding".to_string(),
            TypeInfo::Resource("Coding".to_string()),
        );
        self.cache.insert(
            "Identifier".to_string(),
            TypeInfo::Resource("Identifier".to_string()),
        );
        self.cache.insert(
            "Reference".to_string(),
            TypeInfo::Resource("Reference".to_string()),
        );
        self.cache.insert(
            "Period".to_string(),
            TypeInfo::Resource("Period".to_string()),
        );
        self.cache
            .insert("Range".to_string(), TypeInfo::Resource("Range".to_string()));
        self.cache.insert(
            "HumanName".to_string(),
            TypeInfo::Resource("HumanName".to_string()),
        );
        self.cache.insert(
            "Address".to_string(),
            TypeInfo::Resource("Address".to_string()),
        );
        self.cache.insert(
            "ContactPoint".to_string(),
            TypeInfo::Resource("ContactPoint".to_string()),
        );

        // Common FHIR resources
        self.cache.insert(
            "Patient".to_string(),
            TypeInfo::Resource("Patient".to_string()),
        );
        self.cache.insert(
            "Observation".to_string(),
            TypeInfo::Resource("Observation".to_string()),
        );
        self.cache.insert(
            "Condition".to_string(),
            TypeInfo::Resource("Condition".to_string()),
        );
        self.cache.insert(
            "Procedure".to_string(),
            TypeInfo::Resource("Procedure".to_string()),
        );
        self.cache.insert(
            "Encounter".to_string(),
            TypeInfo::Resource("Encounter".to_string()),
        );
        self.cache.insert(
            "DiagnosticReport".to_string(),
            TypeInfo::Resource("DiagnosticReport".to_string()),
        );
        self.cache.insert(
            "Medication".to_string(),
            TypeInfo::Resource("Medication".to_string()),
        );
        self.cache.insert(
            "MedicationRequest".to_string(),
            TypeInfo::Resource("MedicationRequest".to_string()),
        );
    }

    /// Get all mapped type names
    pub fn get_all_mapped_types(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    /// Clear the mapping cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.initialize_mappings();
    }
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_mapping() {
        let mapper = TypeMapper::new();

        assert_eq!(
            mapper.map_primitive_type("boolean"),
            Some(TypeInfo::Boolean)
        );
        assert_eq!(
            mapper.map_primitive_type("integer"),
            Some(TypeInfo::Integer)
        );
        assert_eq!(
            mapper.map_primitive_type("decimal"),
            Some(TypeInfo::Decimal)
        );
        assert_eq!(mapper.map_primitive_type("string"), Some(TypeInfo::String));
        assert_eq!(mapper.map_primitive_type("date"), Some(TypeInfo::Date));
        assert_eq!(
            mapper.map_primitive_type("dateTime"),
            Some(TypeInfo::DateTime)
        );
        assert_eq!(mapper.map_primitive_type("time"), Some(TypeInfo::Time));
    }

    #[test]
    fn test_complex_type_mapping() {
        let mapper = TypeMapper::new();

        assert_eq!(
            mapper.map_complex_type("CodeableConcept"),
            Some(TypeInfo::Resource("CodeableConcept".to_string()))
        );
        assert_eq!(
            mapper.map_complex_type("Quantity"),
            Some(TypeInfo::Quantity)
        );
        assert_eq!(
            mapper.map_complex_type("Reference"),
            Some(TypeInfo::Resource("Reference".to_string()))
        );
    }

    #[test]
    fn test_resource_type_mapping() {
        let mapper = TypeMapper::new();

        assert_eq!(
            mapper.map_resource_type("Patient"),
            Some(TypeInfo::Resource("Patient".to_string()))
        );
        assert_eq!(
            mapper.map_resource_type("Observation"),
            Some(TypeInfo::Resource("Observation".to_string()))
        );
        assert_eq!(mapper.map_resource_type("unknown"), None);
    }

    #[test]
    fn test_special_type_mapping() {
        let mapper = TypeMapper::new();

        assert_eq!(
            mapper.map_special_type("Quantity"),
            Some(TypeInfo::Quantity)
        );
        assert_eq!(mapper.map_special_type("Age"), Some(TypeInfo::Quantity));
        assert_eq!(
            mapper.map_special_type("Period"),
            Some(TypeInfo::Resource("Period".to_string()))
        );
    }

    #[test]
    fn test_collection_wrapping() {
        let mapper = TypeMapper::new();

        let string_type = TypeInfo::String;
        let collection = mapper.wrap_as_collection(string_type.clone(), true);
        assert_eq!(
            collection,
            TypeInfo::Collection(Box::new(string_type.clone()))
        );

        let non_collection = mapper.wrap_as_collection(string_type.clone(), false);
        assert_eq!(non_collection, string_type);
    }

    #[test]
    fn test_fhir_type_mapping_from_cache() {
        let mapper = TypeMapper::new();

        assert_eq!(mapper.map_fhir_type("boolean"), Some(TypeInfo::Boolean));
        assert_eq!(
            mapper.map_fhir_type("Patient"),
            Some(TypeInfo::Resource("Patient".to_string()))
        );
        assert_eq!(mapper.map_fhir_type("unknown"), None);
    }
}
