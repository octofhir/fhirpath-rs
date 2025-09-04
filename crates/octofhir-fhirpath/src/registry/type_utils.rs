//! Advanced type utilities for FHIRPath type operations
//!
//! This module provides utilities for complex type operations including compatibility
//! checking, type inference, and advanced type analysis.

use super::types::{FhirPathType, TypeChecker};
use crate::core::{FhirPathValue, FhirPathError, Result};
use std::collections::HashSet;

/// Utilities for advanced type operations
pub struct TypeUtils;

impl TypeUtils {
    /// Get all compatible types for a value (including supertypes)
    pub fn get_compatible_types(value: &FhirPathValue) -> Vec<FhirPathType> {
        let primary_type = TypeChecker::get_type(value);
        let mut types = vec![primary_type.clone()];

        // Add supertypes based on FHIR hierarchy
        match primary_type {
            FhirPathType::Patient => {
                types.push(FhirPathType::DomainResource);
                types.push(FhirPathType::Resource);
                types.push(FhirPathType::Any);
            }
            FhirPathType::Observation => {
                types.push(FhirPathType::DomainResource);
                types.push(FhirPathType::Resource);
                types.push(FhirPathType::Any);
            }
            FhirPathType::Organization | FhirPathType::Practitioner => {
                types.push(FhirPathType::DomainResource);
                types.push(FhirPathType::Resource);
                types.push(FhirPathType::Any);
            }
            FhirPathType::Bundle => {
                types.push(FhirPathType::Resource);
                types.push(FhirPathType::Any);
            }
            FhirPathType::DomainResource => {
                types.push(FhirPathType::Resource);
                types.push(FhirPathType::Any);
            }
            FhirPathType::Resource => {
                types.push(FhirPathType::Any);
            }
            // FHIR primitive types that extend String
            FhirPathType::Code | FhirPathType::Uri | FhirPathType::Url | 
            FhirPathType::Canonical | FhirPathType::Oid | FhirPathType::Uuid |
            FhirPathType::Id | FhirPathType::Markdown | FhirPathType::Base64Binary => {
                types.push(FhirPathType::String);
                types.push(FhirPathType::Any);
            }
            // Instant extends DateTime
            FhirPathType::Instant => {
                types.push(FhirPathType::DateTime);
                types.push(FhirPathType::Any);
            }
            // All other types extend Any
            _ => {
                types.push(FhirPathType::Any);
            }
        }

        types
    }

    /// Find the most specific common type for a collection of values
    pub fn find_common_type(values: &[FhirPathValue]) -> FhirPathType {
        if values.is_empty() {
            return FhirPathType::Any;
        }

        if values.len() == 1 {
            return TypeChecker::get_type(&values[0]);
        }

        // Get all compatible types for each value
        let type_sets: Vec<HashSet<FhirPathType>> = values
            .iter()
            .map(|v| Self::get_compatible_types(v).into_iter().collect())
            .collect();

        // Find intersection of all type sets
        let mut common_types: HashSet<FhirPathType> = type_sets[0].clone();
        for type_set in &type_sets[1..] {
            common_types = common_types.intersection(type_set).cloned().collect();
        }

        // Return the most specific common type
        if common_types.is_empty() {
            FhirPathType::Any
        } else {
            // Convert to Vec and sort by specificity (more specific types first)
            let mut common_types_vec: Vec<FhirPathType> = common_types.into_iter().collect();
            common_types_vec.sort_by(|a, b| {
                Self::type_specificity_score(a).cmp(&Self::type_specificity_score(b))
            });
            common_types_vec.into_iter().next().unwrap_or(FhirPathType::Any)
        }
    }

    /// Get a specificity score for a type (lower = more specific)
    fn type_specificity_score(fhir_type: &FhirPathType) -> u32 {
        match fhir_type {
            FhirPathType::Any => 1000,
            FhirPathType::System => 900,
            FhirPathType::Resource => 100,
            FhirPathType::DomainResource => 90,
            FhirPathType::Collection => 80,
            // Specific resource types are very specific
            FhirPathType::Patient | FhirPathType::Observation | 
            FhirPathType::Organization | FhirPathType::Practitioner |
            FhirPathType::Bundle => 10,
            // Complex types are moderately specific
            FhirPathType::CodeableConcept | FhirPathType::Coding | FhirPathType::Identifier |
            FhirPathType::HumanName | FhirPathType::Address | FhirPathType::ContactPoint |
            FhirPathType::Reference | FhirPathType::Attachment | FhirPathType::Period |
            FhirPathType::Range | FhirPathType::Ratio | FhirPathType::SampledData |
            FhirPathType::Signature => 15,
            // String is a base type but less specific than primitives
            FhirPathType::String => 50,
            // FHIR primitive types are more specific than String
            FhirPathType::Code | FhirPathType::Uri | FhirPathType::Url |
            FhirPathType::Canonical | FhirPathType::Oid | FhirPathType::Uuid |
            FhirPathType::Id | FhirPathType::Markdown | FhirPathType::Base64Binary => 5,
            // DateTime base type
            FhirPathType::DateTime => 20,
            FhirPathType::Instant => 5,
            // Other primitive types are most specific
            _ => 1,
        }
    }

    /// Check if a type conversion is safe (no data loss)
    pub fn is_safe_conversion(from: &FhirPathType, to: &FhirPathType) -> bool {
        if from == to || from.is_subtype_of(to) {
            return true;
        }

        match (from, to) {
            // Numeric promotions are safe
            (FhirPathType::Integer, FhirPathType::Decimal) => true,
            // Any type can be converted to String safely
            (FhirPathType::Integer, FhirPathType::String) => true,
            (FhirPathType::Decimal, FhirPathType::String) => true,
            (FhirPathType::Boolean, FhirPathType::String) => true,
            (FhirPathType::Date, FhirPathType::String) => true,
            (FhirPathType::DateTime, FhirPathType::String) => true,
            (FhirPathType::Time, FhirPathType::String) => true,
            _ => false,
        }
    }

    /// Check if a conversion would potentially lose data
    pub fn is_lossy_conversion(from: &FhirPathType, to: &FhirPathType) -> bool {
        match (from, to) {
            // Decimal to Integer can lose fractional part
            (FhirPathType::Decimal, FhirPathType::Integer) => true,
            // String to numeric can fail or lose precision
            (FhirPathType::String, FhirPathType::Integer) => true,
            (FhirPathType::String, FhirPathType::Decimal) => true,
            (FhirPathType::String, FhirPathType::Boolean) => true,
            // DateTime to Date loses time information
            (FhirPathType::DateTime, FhirPathType::Date) => true,
            (FhirPathType::Instant, FhirPathType::Date) => true,
            _ => false,
        }
    }

    /// Validate a type name
    pub fn is_valid_type_name(name: &str) -> bool {
        FhirPathType::from_type_name(name).is_some()
    }

    /// Get all available type names
    pub fn get_all_type_names() -> Vec<&'static str> {
        vec![
            // Primitive types
            "Boolean", "Integer", "Decimal", "String", "Date", "DateTime", "Time", "Quantity",
            // FHIR primitive types
            "code", "uri", "url", "canonical", "oid", "uuid", "id", "markdown", "base64Binary", "instant",
            // Complex types
            "CodeableConcept", "Coding", "Identifier", "HumanName", "Address", "ContactPoint",
            "Reference", "Attachment", "Period", "Range", "Ratio", "SampledData", "Signature",
            // Resource types
            "Resource", "DomainResource", "Patient", "Observation", "Organization", "Practitioner", "Bundle",
            // System types
            "Collection", "Any", "System"
        ]
    }

    /// Get all primitive type names
    pub fn get_primitive_type_names() -> Vec<&'static str> {
        vec![
            "Boolean", "Integer", "Decimal", "String", "Date", "DateTime", "Time", "Quantity"
        ]
    }

    /// Get all FHIR primitive type names  
    pub fn get_fhir_primitive_type_names() -> Vec<&'static str> {
        vec![
            "code", "uri", "url", "canonical", "oid", "uuid", "id", "markdown", "base64Binary", "instant"
        ]
    }

    /// Get all complex type names
    pub fn get_complex_type_names() -> Vec<&'static str> {
        vec![
            "CodeableConcept", "Coding", "Identifier", "HumanName", "Address", "ContactPoint",
            "Reference", "Attachment", "Period", "Range", "Ratio", "SampledData", "Signature"
        ]
    }

    /// Get all resource type names
    pub fn get_resource_type_names() -> Vec<&'static str> {
        vec![
            "Resource", "DomainResource", "Patient", "Observation", "Organization", "Practitioner", "Bundle"
        ]
    }

    /// Check if a type is a primitive type
    pub fn is_primitive_type(fhir_type: &FhirPathType) -> bool {
        matches!(fhir_type, 
            FhirPathType::Boolean | FhirPathType::Integer | FhirPathType::Decimal |
            FhirPathType::String | FhirPathType::Date | FhirPathType::DateTime |
            FhirPathType::Time | FhirPathType::Quantity
        )
    }

    /// Check if a type is a FHIR primitive type
    pub fn is_fhir_primitive_type(fhir_type: &FhirPathType) -> bool {
        matches!(fhir_type,
            FhirPathType::Code | FhirPathType::Uri | FhirPathType::Url |
            FhirPathType::Canonical | FhirPathType::Oid | FhirPathType::Uuid |
            FhirPathType::Id | FhirPathType::Markdown | FhirPathType::Base64Binary |
            FhirPathType::Instant
        )
    }

    /// Check if a type is a complex type
    pub fn is_complex_type(fhir_type: &FhirPathType) -> bool {
        matches!(fhir_type,
            FhirPathType::CodeableConcept | FhirPathType::Coding | FhirPathType::Identifier |
            FhirPathType::HumanName | FhirPathType::Address | FhirPathType::ContactPoint |
            FhirPathType::Reference | FhirPathType::Attachment | FhirPathType::Period |
            FhirPathType::Range | FhirPathType::Ratio | FhirPathType::SampledData |
            FhirPathType::Signature
        )
    }

    /// Check if a type is a resource type
    pub fn is_resource_type(fhir_type: &FhirPathType) -> bool {
        matches!(fhir_type,
            FhirPathType::Resource | FhirPathType::DomainResource |
            FhirPathType::Patient | FhirPathType::Observation |
            FhirPathType::Organization | FhirPathType::Practitioner |
            FhirPathType::Bundle
        )
    }

    /// Get the base type for a FHIR type (e.g., code -> String)
    pub fn get_base_type(fhir_type: &FhirPathType) -> FhirPathType {
        match fhir_type {
            // FHIR primitives that extend String
            FhirPathType::Code | FhirPathType::Uri | FhirPathType::Url |
            FhirPathType::Canonical | FhirPathType::Oid | FhirPathType::Uuid |
            FhirPathType::Id | FhirPathType::Markdown | FhirPathType::Base64Binary => FhirPathType::String,
            
            // Instant extends DateTime
            FhirPathType::Instant => FhirPathType::DateTime,
            
            // Resources extend Resource
            FhirPathType::Patient | FhirPathType::Observation |
            FhirPathType::Organization | FhirPathType::Practitioner => FhirPathType::DomainResource,
            
            FhirPathType::DomainResource => FhirPathType::Resource,
            
            // Everything else is its own base type
            _ => fhir_type.clone(),
        }
    }

    /// Check if two types are comparable (can be used in equality/comparison operations)
    pub fn are_comparable(type1: &FhirPathType, type2: &FhirPathType) -> bool {
        // Same types are always comparable
        if type1 == type2 {
            return true;
        }

        // Check if one is a subtype of the other
        if type1.is_subtype_of(type2) || type2.is_subtype_of(type1) {
            return true;
        }

        // Numeric types are comparable with each other
        let numeric_types = [FhirPathType::Integer, FhirPathType::Decimal, FhirPathType::Quantity];
        if numeric_types.contains(type1) && numeric_types.contains(type2) {
            return true;
        }

        // Date/time types are comparable with each other
        let datetime_types = [FhirPathType::Date, FhirPathType::DateTime, FhirPathType::Time, FhirPathType::Instant];
        if datetime_types.contains(type1) && datetime_types.contains(type2) {
            return true;
        }

        false
    }

    /// Get suggested conversions for a type
    pub fn get_suggested_conversions(from_type: &FhirPathType) -> Vec<FhirPathType> {
        let mut suggestions = Vec::new();

        match from_type {
            FhirPathType::Integer => {
                suggestions.push(FhirPathType::Decimal);
                suggestions.push(FhirPathType::String);
            }
            FhirPathType::Decimal => {
                suggestions.push(FhirPathType::String);
            }
            FhirPathType::Boolean => {
                suggestions.push(FhirPathType::String);
            }
            FhirPathType::String => {
                suggestions.push(FhirPathType::Integer);
                suggestions.push(FhirPathType::Decimal);
                suggestions.push(FhirPathType::Boolean);
            }
            _ => {
                // Most types can be converted to String
                suggestions.push(FhirPathType::String);
            }
        }

        suggestions
    }
}