// FHIR Resource Schema Definitions
//
// This module provides basic FHIR resource schema definitions for context validation

use std::collections::HashMap;
use std::collections::HashSet;

/// FHIR resource schema validator
pub struct FhirSchemaValidator {
    schemas: HashMap<String, FhirResourceSchema>,
}

/// Schema definition for a FHIR resource type
#[derive(Debug, Clone)]
pub struct FhirResourceSchema {
    pub resource_type: String,
    pub properties: HashSet<String>,
    pub nested_properties: HashMap<String, HashSet<String>>,
}

impl FhirSchemaValidator {
    /// Create a new schema validator with built-in FHIR R4 schemas
    pub fn new() -> Self {
        let mut schemas = HashMap::new();

        // Patient resource schema
        let patient_schema = FhirResourceSchema {
            resource_type: "Patient".to_string(),
            properties: [
                "resourceType", "id", "meta", "implicitRules", "language", "text",
                "contained", "extension", "modifierExtension", "identifier", "active",
                "name", "telecom", "gender", "birthDate", "deceased", "address",
                "maritalStatus", "multipleBirth", "photo", "contact", "communication",
                "generalPractitioner", "managingOrganization", "link"
            ].iter().map(|s| s.to_string()).collect(),
            nested_properties: {
                let mut nested = HashMap::new();
                nested.insert("name".to_string(), [
                    "use", "text", "family", "given", "prefix", "suffix", "period"
                ].iter().map(|s| s.to_string()).collect());
                nested.insert("telecom".to_string(), [
                    "system", "value", "use", "rank", "period"
                ].iter().map(|s| s.to_string()).collect());
                nested.insert("address".to_string(), [
                    "use", "type", "text", "line", "city", "district", "state",
                    "postalCode", "country", "period"
                ].iter().map(|s| s.to_string()).collect());
                nested
            },
        };
        schemas.insert("Patient".to_string(), patient_schema);

        // Encounter resource schema
        let encounter_schema = FhirResourceSchema {
            resource_type: "Encounter".to_string(),
            properties: [
                "resourceType", "id", "meta", "implicitRules", "language", "text",
                "contained", "extension", "modifierExtension", "identifier", "status",
                "statusHistory", "class", "classHistory", "type", "serviceType",
                "priority", "subject", "episodeOfCare", "basedOn", "participant",
                "appointment", "period", "length", "reasonCode", "reasonReference",
                "diagnosis", "account", "hospitalization", "location", "serviceProvider",
                "partOf"
            ].iter().map(|s| s.to_string()).collect(),
            nested_properties: {
                let mut nested = HashMap::new();
                nested.insert("class".to_string(), [
                    "system", "version", "code", "display", "userSelected"
                ].iter().map(|s| s.to_string()).collect());
                nested.insert("participant".to_string(), [
                    "type", "period", "individual"
                ].iter().map(|s| s.to_string()).collect());
                nested.insert("diagnosis".to_string(), [
                    "condition", "use", "rank"
                ].iter().map(|s| s.to_string()).collect());
                nested
            },
        };
        schemas.insert("Encounter".to_string(), encounter_schema);

        // Observation resource schema
        let observation_schema = FhirResourceSchema {
            resource_type: "Observation".to_string(),
            properties: [
                "resourceType", "id", "meta", "implicitRules", "language", "text",
                "contained", "extension", "modifierExtension", "identifier", "basedOn",
                "partOf", "status", "category", "code", "subject", "focus", "encounter",
                "effective", "issued", "performer", "value", "dataAbsentReason",
                "interpretation", "note", "bodySite", "method", "specimen", "device",
                "referenceRange", "hasMember", "derivedFrom", "component",
                // Value[x] choice types
                "valueQuantity", "valueCodeableConcept", "valueString", "valueBoolean",
                "valueInteger", "valueRange", "valueRatio", "valueSampledData",
                "valueTime", "valueDateTime", "valuePeriod"
            ].iter().map(|s| s.to_string()).collect(),
            nested_properties: {
                let mut nested = HashMap::new();
                nested.insert("valueQuantity".to_string(), [
                    "value", "comparator", "unit", "system", "code"
                ].iter().map(|s| s.to_string()).collect());
                nested.insert("code".to_string(), [
                    "coding", "text"
                ].iter().map(|s| s.to_string()).collect());
                nested.insert("component".to_string(), [
                    "code", "value", "dataAbsentReason", "interpretation", "referenceRange"
                ].iter().map(|s| s.to_string()).collect());
                nested
            },
        };
        schemas.insert("Observation".to_string(), observation_schema);

        Self { schemas }
    }

    /// Validate if a property path is valid for the given resource type
    pub fn validate_property_path(&self, resource_type: &str, path: &str) -> Result<(), crate::errors::FhirPathError> {
        let schema = self.schemas.get(resource_type)
            .ok_or_else(|| crate::errors::FhirPathError::ResourceTypeError {
                resource_type: resource_type.to_string(),
                reason: "Unknown resource type".to_string(),
            })?;

        // Split the path into components (e.g., "name.given" -> ["name", "given"])
        let path_components: Vec<&str> = path.split('.').collect();

        if path_components.is_empty() {
            return Ok(());
        }

        let first_component = path_components[0];

        // Check if the first component is a valid property
        if !schema.properties.contains(first_component) {
            return Err(crate::errors::FhirPathError::InvalidContextPath {
                path: path.to_string(),
                resource_type: resource_type.to_string(),
                available_properties: schema.properties.iter().cloned().collect(),
            });
        }

        // If there are nested components, validate them
        if path_components.len() > 1 {
            let second_component = path_components[1];

            if let Some(nested_props) = schema.nested_properties.get(first_component) {
                if !nested_props.contains(second_component) {
                    return Err(crate::errors::FhirPathError::InvalidContextPath {
                        path: path.to_string(),
                        resource_type: resource_type.to_string(),
                        available_properties: nested_props.iter().cloned().collect(),
                    });
                }
            }
            // If no nested properties are defined, we allow the path (for flexibility)
        }

        Ok(())
    }

    /// Get available properties for a resource type
    pub fn get_available_properties(&self, resource_type: &str) -> Option<Vec<String>> {
        self.schemas.get(resource_type)
            .map(|schema| schema.properties.iter().cloned().collect())
    }

    /// Check if a resource type is supported
    pub fn is_supported_resource_type(&self, resource_type: &str) -> bool {
        self.schemas.contains_key(resource_type)
    }
}

impl Default for FhirSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_schema_validation() {
        let validator = FhirSchemaValidator::new();

        // Valid Patient properties
        assert!(validator.validate_property_path("Patient", "name").is_ok());
        assert!(validator.validate_property_path("Patient", "name.given").is_ok());
        assert!(validator.validate_property_path("Patient", "telecom.system").is_ok());
        assert!(validator.validate_property_path("Patient", "birthDate").is_ok());

        // Invalid Patient properties
        assert!(validator.validate_property_path("Patient", "class").is_err());
        assert!(validator.validate_property_path("Patient", "status").is_err());
        assert!(validator.validate_property_path("Patient", "valueQuantity").is_err());
    }

    #[test]
    fn test_encounter_schema_validation() {
        let validator = FhirSchemaValidator::new();

        // Valid Encounter properties
        assert!(validator.validate_property_path("Encounter", "status").is_ok());
        assert!(validator.validate_property_path("Encounter", "class").is_ok());
        assert!(validator.validate_property_path("Encounter", "class.code").is_ok());

        // Invalid Encounter properties
        assert!(validator.validate_property_path("Encounter", "name").is_err());
        assert!(validator.validate_property_path("Encounter", "birthDate").is_err());
        assert!(validator.validate_property_path("Encounter", "valueQuantity").is_err());
    }

    #[test]
    fn test_observation_schema_validation() {
        let validator = FhirSchemaValidator::new();

        // Valid Observation properties
        assert!(validator.validate_property_path("Observation", "status").is_ok());
        assert!(validator.validate_property_path("Observation", "valueQuantity").is_ok());
        assert!(validator.validate_property_path("Observation", "valueQuantity.value").is_ok());

        // Invalid Observation properties
        assert!(validator.validate_property_path("Observation", "name").is_err());
        assert!(validator.validate_property_path("Observation", "birthDate").is_err());
    }

    #[test]
    fn test_unknown_resource_type() {
        let validator = FhirSchemaValidator::new();

        assert!(validator.validate_property_path("UnknownResource", "someProperty").is_err());
    }
}
