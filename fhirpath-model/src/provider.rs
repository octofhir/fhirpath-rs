//! Model provider trait for FHIR type information

use crate::types::TypeInfo;

/// FHIR version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FhirVersion {
    /// FHIR R4
    R4,
    /// FHIR R4B
    R4B,
    /// FHIR R5
    R5,
}

impl FhirVersion {
    /// Get the version string
    pub fn as_str(&self) -> &'static str {
        match self {
            FhirVersion::R4 => "R4",
            FhirVersion::R4B => "R4B",
            FhirVersion::R5 => "R5",
        }
    }
}

/// Search parameter definition
#[derive(Debug, Clone)]
pub struct SearchParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (string, token, reference, etc.)
    pub param_type: String,
    /// Path expression
    pub expression: String,
}

/// Trait for providing FHIR model information
pub trait ModelProvider: Send + Sync {
    /// Get type information for a type name
    fn get_type_info(&self, type_name: &str) -> Option<TypeInfo>;

    /// Get property type for a given parent type and property name
    fn get_property_type(&self, parent_type: &str, property: &str) -> Option<TypeInfo>;

    /// Check if a property is polymorphic (e.g., value[x])
    fn is_polymorphic(&self, property: &str) -> bool {
        property.ends_with("[x]")
    }

    /// Get all possible type suffixes for a polymorphic property
    fn get_polymorphic_types(&self, property: &str) -> Vec<String> {
        if !self.is_polymorphic(property) {
            return vec![];
        }

        // Common FHIR polymorphic types
        vec![
            "Boolean".to_string(),
            "Integer".to_string(),
            "String".to_string(),
            "Decimal".to_string(),
            "Uri".to_string(),
            "Url".to_string(),
            "Canonical".to_string(),
            "Base64Binary".to_string(),
            "Instant".to_string(),
            "Date".to_string(),
            "DateTime".to_string(),
            "Time".to_string(),
            "Code".to_string(),
            "Oid".to_string(),
            "Id".to_string(),
            "Markdown".to_string(),
            "UnsignedInt".to_string(),
            "PositiveInt".to_string(),
            "Uuid".to_string(),
            "Quantity".to_string(),
            "Age".to_string(),
            "Distance".to_string(),
            "Duration".to_string(),
            "Count".to_string(),
            "Money".to_string(),
            "Range".to_string(),
            "Period".to_string(),
            "Ratio".to_string(),
            "RatioRange".to_string(),
            "SampledData".to_string(),
            "Signature".to_string(),
            "HumanName".to_string(),
            "Address".to_string(),
            "ContactPoint".to_string(),
            "Timing".to_string(),
            "Reference".to_string(),
            "Annotation".to_string(),
            "Attachment".to_string(),
            "CodeableConcept".to_string(),
            "Identifier".to_string(),
            "Coding".to_string(),
            "Meta".to_string(),
        ]
    }

    /// Get search parameters for a resource type
    fn get_search_params(&self, resource_type: &str) -> Vec<SearchParameter>;

    /// Check if a type is a resource type
    fn is_resource_type(&self, type_name: &str) -> bool;

    /// Check if a type is a primitive type
    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "boolean"
                | "integer"
                | "string"
                | "decimal"
                | "uri"
                | "url"
                | "canonical"
                | "base64Binary"
                | "instant"
                | "date"
                | "dateTime"
                | "time"
                | "code"
                | "oid"
                | "id"
                | "markdown"
                | "unsignedInt"
                | "positiveInt"
                | "uuid"
        )
    }

    /// Check if a type is a complex type
    fn is_complex_type(&self, type_name: &str) -> bool {
        !self.is_primitive_type(type_name) && !self.is_resource_type(type_name)
    }

    /// Get the FHIR version
    fn fhir_version(&self) -> FhirVersion;

    /// Check if a type is a subtype of another
    fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool;

    /// Get all properties for a type
    fn get_properties(&self, type_name: &str) -> Vec<(String, TypeInfo)>;

    /// Get the base type for a given type
    fn get_base_type(&self, type_name: &str) -> Option<String>;
}

/// Empty model provider for testing
#[derive(Debug, Clone)]
pub struct EmptyModelProvider;

impl ModelProvider for EmptyModelProvider {
    fn get_type_info(&self, _type_name: &str) -> Option<TypeInfo> {
        None
    }

    fn get_property_type(&self, _parent_type: &str, _property: &str) -> Option<TypeInfo> {
        None
    }

    fn get_search_params(&self, _resource_type: &str) -> Vec<SearchParameter> {
        Vec::new()
    }

    fn is_resource_type(&self, _type_name: &str) -> bool {
        false
    }

    fn fhir_version(&self) -> FhirVersion {
        FhirVersion::R5
    }

    fn is_subtype_of(&self, _child_type: &str, _parent_type: &str) -> bool {
        false
    }

    fn get_properties(&self, _type_name: &str) -> Vec<(String, TypeInfo)> {
        Vec::new()
    }

    fn get_base_type(&self, _type_name: &str) -> Option<String> {
        None
    }
}
