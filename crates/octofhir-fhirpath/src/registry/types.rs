//! Type checking and casting functions for FHIRPath
//!
//! This module implements comprehensive type system support including type checking,
//! casting, and FHIR type hierarchy with proper subtype relationships.

use super::type_utils::TypeUtils;
use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::error_code::FP0055;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::register_function;

/// Complete FHIRPath type system with proper FHIR type hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FhirPathType {
    // Primitive types
    Boolean,
    Integer,
    Decimal,
    String,
    Date,
    DateTime,
    Time,
    Quantity,

    // FHIR primitive types
    Code,
    Uri,
    Url,
    Canonical,
    Oid,
    Uuid,
    Id,
    Markdown,
    Base64Binary,
    Instant,

    // Complex types
    CodeableConcept,
    Coding,
    Identifier,
    HumanName,
    Address,
    ContactPoint,
    Reference,
    Attachment,
    Period,
    Range,
    Ratio,
    SampledData,
    Signature,

    // Resource types
    Resource,
    DomainResource,
    Patient,
    Observation,
    Organization,
    Practitioner,
    Bundle,

    // Collections
    Collection,

    // System types
    Any,
    System,
    TypeInfo,
    Empty,
}

impl FhirPathType {
    /// Get the type name as used in FHIRPath expressions
    pub fn type_name(&self) -> &'static str {
        match self {
            FhirPathType::Boolean => "Boolean",
            FhirPathType::Integer => "Integer",
            FhirPathType::Decimal => "Decimal",
            FhirPathType::String => "String",
            FhirPathType::Date => "Date",
            FhirPathType::DateTime => "DateTime",
            FhirPathType::Time => "Time",
            FhirPathType::Quantity => "Quantity",
            FhirPathType::Code => "code",
            FhirPathType::Uri => "uri",
            FhirPathType::Url => "url",
            FhirPathType::Canonical => "canonical",
            FhirPathType::Oid => "oid",
            FhirPathType::Uuid => "uuid",
            FhirPathType::Id => "id",
            FhirPathType::Markdown => "markdown",
            FhirPathType::Base64Binary => "base64Binary",
            FhirPathType::Instant => "instant",
            FhirPathType::CodeableConcept => "CodeableConcept",
            FhirPathType::Coding => "Coding",
            FhirPathType::Identifier => "Identifier",
            FhirPathType::HumanName => "HumanName",
            FhirPathType::Address => "Address",
            FhirPathType::ContactPoint => "ContactPoint",
            FhirPathType::Reference => "Reference",
            FhirPathType::Attachment => "Attachment",
            FhirPathType::Period => "Period",
            FhirPathType::Range => "Range",
            FhirPathType::Ratio => "Ratio",
            FhirPathType::SampledData => "SampledData",
            FhirPathType::Signature => "Signature",
            FhirPathType::Resource => "Resource",
            FhirPathType::DomainResource => "DomainResource",
            FhirPathType::Patient => "Patient",
            FhirPathType::Observation => "Observation",
            FhirPathType::Organization => "Organization",
            FhirPathType::Practitioner => "Practitioner",
            FhirPathType::Bundle => "Bundle",
            FhirPathType::Collection => "Collection",
            FhirPathType::Any => "Any",
            FhirPathType::System => "System",
            FhirPathType::TypeInfo => "TypeInfo",
            FhirPathType::Empty => "empty",
        }
    }

    /// Parse a type name from a string
    pub fn from_type_name(name: &str) -> Option<FhirPathType> {
        // Allow namespaced System.* type names
        let name = if let Some(stripped) = name.strip_prefix("System.") {
            stripped
        } else {
            name
        };
        match name {
            "Boolean" => Some(FhirPathType::Boolean),
            "Integer" => Some(FhirPathType::Integer),
            "Decimal" => Some(FhirPathType::Decimal),
            "String" => Some(FhirPathType::String),
            "Date" => Some(FhirPathType::Date),
            "DateTime" => Some(FhirPathType::DateTime),
            "Time" => Some(FhirPathType::Time),
            "Quantity" => Some(FhirPathType::Quantity),
            "code" => Some(FhirPathType::Code),
            "uri" => Some(FhirPathType::Uri),
            "url" => Some(FhirPathType::Url),
            "canonical" => Some(FhirPathType::Canonical),
            "oid" => Some(FhirPathType::Oid),
            "uuid" => Some(FhirPathType::Uuid),
            "id" => Some(FhirPathType::Id),
            "markdown" => Some(FhirPathType::Markdown),
            "base64Binary" => Some(FhirPathType::Base64Binary),
            "instant" => Some(FhirPathType::Instant),
            "CodeableConcept" => Some(FhirPathType::CodeableConcept),
            "Coding" => Some(FhirPathType::Coding),
            "Identifier" => Some(FhirPathType::Identifier),
            "HumanName" => Some(FhirPathType::HumanName),
            "Address" => Some(FhirPathType::Address),
            "ContactPoint" => Some(FhirPathType::ContactPoint),
            "Reference" => Some(FhirPathType::Reference),
            "Attachment" => Some(FhirPathType::Attachment),
            "Period" => Some(FhirPathType::Period),
            "Range" => Some(FhirPathType::Range),
            "Ratio" => Some(FhirPathType::Ratio),
            "SampledData" => Some(FhirPathType::SampledData),
            "Signature" => Some(FhirPathType::Signature),
            "Resource" => Some(FhirPathType::Resource),
            "DomainResource" => Some(FhirPathType::DomainResource),
            "Patient" => Some(FhirPathType::Patient),
            "Observation" => Some(FhirPathType::Observation),
            "Organization" => Some(FhirPathType::Organization),
            "Practitioner" => Some(FhirPathType::Practitioner),
            "Bundle" => Some(FhirPathType::Bundle),
            "Collection" => Some(FhirPathType::Collection),
            "Any" => Some(FhirPathType::Any),
            "System" => Some(FhirPathType::System),
            "TypeInfo" => Some(FhirPathType::TypeInfo),
            "empty" => Some(FhirPathType::Empty),
            _ => None,
        }
    }

    /// Check if this type is a subtype of another type according to FHIR hierarchy
    pub fn is_subtype_of(&self, other: &FhirPathType) -> bool {
        if self == other || *other == FhirPathType::Any {
            return true;
        }

        match (self, other) {
            // All FHIR resources are subtypes of Resource
            (FhirPathType::Patient, FhirPathType::Resource) => true,
            (FhirPathType::Observation, FhirPathType::Resource) => true,
            (FhirPathType::Organization, FhirPathType::Resource) => true,
            (FhirPathType::Practitioner, FhirPathType::Resource) => true,
            (FhirPathType::Bundle, FhirPathType::Resource) => true,

            // DomainResource is a subtype of Resource
            (FhirPathType::DomainResource, FhirPathType::Resource) => true,

            // Most resources are subtypes of DomainResource
            (FhirPathType::Patient, FhirPathType::DomainResource) => true,
            (FhirPathType::Observation, FhirPathType::DomainResource) => true,
            (FhirPathType::Organization, FhirPathType::DomainResource) => true,
            (FhirPathType::Practitioner, FhirPathType::DomainResource) => true,

            // FHIR primitive types are subtypes of their base types
            (FhirPathType::Code, FhirPathType::String) => true,
            (FhirPathType::Uri, FhirPathType::String) => true,
            (FhirPathType::Url, FhirPathType::String) => true,
            (FhirPathType::Canonical, FhirPathType::String) => true,
            (FhirPathType::Oid, FhirPathType::String) => true,
            (FhirPathType::Uuid, FhirPathType::String) => true,
            (FhirPathType::Id, FhirPathType::String) => true,
            (FhirPathType::Markdown, FhirPathType::String) => true,
            (FhirPathType::Base64Binary, FhirPathType::String) => true,
            (FhirPathType::Instant, FhirPathType::DateTime) => true,

            _ => false,
        }
    }
}

/// Core type checker for FHIRPath values
pub struct TypeChecker;

impl TypeChecker {
    /// Get the FHIRPath type of a value
    pub fn get_type(value: &FhirPathValue) -> FhirPathType {
        match value {
            FhirPathValue::Boolean(_) => FhirPathType::Boolean,
            FhirPathValue::Integer(_) => FhirPathType::Integer,
            FhirPathValue::Decimal(_) => FhirPathType::Decimal,
            FhirPathValue::String(_) => FhirPathType::String,
            FhirPathValue::Date(_) => FhirPathType::Date,
            FhirPathValue::DateTime(_) => FhirPathType::DateTime,
            FhirPathValue::Time(_) => FhirPathType::Time,
            FhirPathValue::Quantity { .. } => FhirPathType::Quantity,
            FhirPathValue::Resource(obj) | FhirPathValue::JsonValue(obj) => {
                // Try to determine object type from resourceType or structure
                if let Some(resource_type) = obj.get("resourceType") {
                    if let Some(type_str) = resource_type.as_str() {
                        return FhirPathType::from_type_name(type_str)
                            .unwrap_or(FhirPathType::Resource);
                    }
                }

                // Infer type from object structure if it's an object
                if let Some(obj_map) = obj.as_object() {
                    Self::infer_complex_type(obj_map)
                } else {
                    FhirPathType::System
                }
            }
            FhirPathValue::Id(_) => FhirPathType::Id,
            FhirPathValue::Base64Binary(_) => FhirPathType::Base64Binary,
            FhirPathValue::Uri(_) => FhirPathType::Uri,
            FhirPathValue::Url(_) => FhirPathType::Url,
            FhirPathValue::Collection(_) => FhirPathType::Collection,
            FhirPathValue::TypeInfoObject { .. } => FhirPathType::TypeInfo,
            FhirPathValue::Empty => FhirPathType::Empty,
        }
    }

    /// Check if a value is of a specific type (including subtypes)
    pub fn is_type(value: &FhirPathValue, expected_type: &FhirPathType) -> bool {
        let actual_type = Self::get_type(value);
        actual_type.is_subtype_of(expected_type)
    }

    /// Attempt to cast a value to a specific type
    pub fn cast_to_type(
        value: &FhirPathValue,
        target_type: &FhirPathType,
    ) -> Result<FhirPathValue> {
        let current_type = Self::get_type(value);

        // If already compatible, return as-is
        if current_type.is_subtype_of(target_type) {
            return Ok(value.clone());
        }

        // Attempt safe conversions between compatible types
        match (value, target_type) {
            // Integer to Decimal conversion
            (FhirPathValue::Integer(i), FhirPathType::Decimal) => {
                Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i)))
            }

            // String to numeric type conversions
            (FhirPathValue::String(s), FhirPathType::Integer) => {
                s.parse::<i64>().map(FhirPathValue::Integer).map_err(|_| {
                    FhirPathError::evaluation_error(
                        FP0055,
                        format!("Cannot convert '{}' to Integer", s),
                    )
                })
            }

            (FhirPathValue::String(s), FhirPathType::Decimal) => s
                .parse::<rust_decimal::Decimal>()
                .map(FhirPathValue::Decimal)
                .map_err(|_| {
                    FhirPathError::evaluation_error(
                        FP0055,
                        format!("Cannot convert '{}' to Decimal", s),
                    )
                }),

            (FhirPathValue::String(s), FhirPathType::Boolean) => match s.to_lowercase().as_str() {
                "true" => Ok(FhirPathValue::Boolean(true)),
                "false" => Ok(FhirPathValue::Boolean(false)),
                _ => Err(FhirPathError::evaluation_error(
                    FP0055,
                    format!("Cannot convert '{}' to Boolean", s),
                )),
            },

            // Any value to String conversion
            (_, FhirPathType::String) => match value {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string())),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::String(d.to_string())),
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string())),
                FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),
                FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.to_string())),
                FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.to_string())),
                FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.to_string())),
                FhirPathValue::Uri(u) => Ok(FhirPathValue::String(u.clone())),
                FhirPathValue::Url(u) => Ok(FhirPathValue::String(u.clone())),
                FhirPathValue::Id(id) => Ok(FhirPathValue::String(id.to_string())),
                _ => Err(FhirPathError::evaluation_error(
                    FP0055,
                    format!("Cannot convert {} to String", current_type.type_name()),
                )),
            },

            _ => Err(FhirPathError::evaluation_error(
                FP0055,
                format!(
                    "Cannot cast {} to {}",
                    current_type.type_name(),
                    target_type.type_name()
                ),
            )),
        }
    }

    /// Infer complex type from object structure
    fn infer_complex_type(obj: &serde_json::Map<String, serde_json::Value>) -> FhirPathType {
        // Look for type-indicating fields to infer FHIR complex types
        if obj.contains_key("system") && obj.contains_key("code") {
            return FhirPathType::Coding;
        }

        if obj.contains_key("coding") || obj.contains_key("text") {
            return FhirPathType::CodeableConcept;
        }

        if obj.contains_key("family") || obj.contains_key("given") {
            return FhirPathType::HumanName;
        }

        if obj.contains_key("line") || obj.contains_key("city") || obj.contains_key("state") {
            return FhirPathType::Address;
        }

        if obj.contains_key("reference") {
            return FhirPathType::Reference;
        }

        if obj.contains_key("value") && obj.contains_key("unit") {
            return FhirPathType::Quantity;
        }

        if obj.contains_key("start") || obj.contains_key("end") {
            return FhirPathType::Period;
        }

        if obj.contains_key("use") && obj.contains_key("system") {
            return FhirPathType::Identifier;
        }

        if obj.contains_key("system") && obj.contains_key("value") {
            return FhirPathType::ContactPoint;
        }

        if obj.contains_key("contentType") && obj.contains_key("data") {
            return FhirPathType::Attachment;
        }

        if obj.contains_key("low") || obj.contains_key("high") {
            return FhirPathType::Range;
        }

        if obj.contains_key("numerator") || obj.contains_key("denominator") {
            return FhirPathType::Ratio;
        }

        // Default to System type for unrecognized objects
        FhirPathType::System
    }
}

impl FunctionRegistry {
    pub fn register_type_functions(&self) -> Result<()> {
        self.register_is_function()?;
        self.register_as_function()?;
        self.register_oftype_function()?;
        self.register_type_function()?;
        Ok(())
    }

    fn register_is_function(&self) -> Result<()> {
        register_function!(
            self,
            async "is",
            category: FunctionCategory::Type,
            description: "Returns true if the input is of the specified type or a subtype thereof",
            parameters: ["type": Some("string".to_string()) => "The type to check against"],
            return_type: "boolean",
            examples: [
                "Patient.name.is('HumanName')",
                "Observation.value.is('Quantity')",
                "5.is('Integer')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FhirPathValue>>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0055,
                            "is() can only be called on a single value".to_string()
                        ));
                    }

                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0055,
                            "is() requires exactly one type argument".to_string()
                        ));
                    }

                    let type_name = match &context.arguments[0] {
                        FhirPathValue::String(s) => s,
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0055,
                                "is() type argument must be a string".to_string()
                            ));
                        }
                    };

                    // Use ModelProvider instead of hardcoded type checking
                    let current_value_type = Self::get_value_type_name(&context.input[0]);

                    // Check type compatibility using ModelProvider
                    match context.model_provider.is_type_compatible(&current_value_type, type_name).await {
                        Ok(is_compatible) => Ok(vec![FhirPathValue::Boolean(is_compatible)]),
                        Err(_) => {
                            // Fallback to basic type checking for system types
                            let is_compatible = Self::basic_type_compatibility(&context.input[0], type_name);
                            Ok(vec![FhirPathValue::Boolean(is_compatible)])
                        }
                    }
                })
            }
        )
    }

    fn register_as_function(&self) -> Result<()> {
        register_function!(
            self,
            async "as",
            category: FunctionCategory::Type,
            description: "Casts the input to the specified type, returns empty if the cast is not possible",
            parameters: ["type": Some("string".to_string()) => "The type to cast to"],
            return_type: "any",
            examples: [
                "'123'.as('Integer')",
                "Patient.name.as('HumanName')",
                "Observation.value.as('Quantity')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FhirPathValue>>> + Send + '_>> {
                Box::pin(async move {
                    if context.input.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0055,
                            "as() can only be called on a single value".to_string()
                        ));
                    }

                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0055,
                            "as() requires exactly one type argument".to_string()
                        ));
                    }

                    let type_name = match &context.arguments[0] {
                        FhirPathValue::String(s) => s,
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0055,
                                "as() type argument must be a string".to_string()
                            ));
                        }
                    };

                    // Use ModelProvider to check if casting is possible
                    let current_value_type = Self::get_value_type_name(&context.input[0]);

                    match context.model_provider.is_type_compatible(&current_value_type, type_name).await {
                        Ok(true) => {
                            // Cast is possible, return the original value (most cases in FHIRPath)
                            Ok(vec![context.input[0].clone()])
                        },
                        Ok(false) => {
                            // Cast not possible, return empty per FHIRPath spec
                            Ok(vec![])
                        },
                        Err(_) => {
                            // Fallback to basic casting logic
                            if Self::basic_type_compatibility(&context.input[0], type_name) {
                                Ok(vec![context.input[0].clone()])
                            } else {
                                Ok(vec![])
                            }
                        }
                    }
                })
            }
        )
    }

    fn register_oftype_function(&self) -> Result<()> {
        register_function!(
            self,
            async "ofType",
            category: FunctionCategory::Type,
            description: "Returns items from the collection that are of the specified type or a subtype thereof",
            parameters: ["type": Some("string".to_string()) => "The type to filter by"],
            return_type: "collection",
            examples: [
                "Bundle.entry.resource.ofType('Patient')",
                "Patient.telecom.ofType('ContactPoint')",
                "('a', 1, true, 2.5).ofType('Integer')"
            ],
            implementation: |context: &FunctionContext| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<FhirPathValue>>> + Send + '_>> {
                Box::pin(async move {
                    if context.arguments.len() != 1 {
                        return Err(FhirPathError::evaluation_error(
                            FP0055,
                            "ofType() requires exactly one type argument".to_string()
                        ));
                    }

                    let type_name = match &context.arguments[0] {
                        FhirPathValue::String(s) => s,
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                FP0055,
                                "ofType() type argument must be a string".to_string()
                            ));
                        }
                    };

                    // Filter collection using ModelProvider-based type checking
                    let mut result = Vec::new();

                    for value in context.input.iter() {
                        let current_value_type = Self::get_value_type_name(value);

                        let is_compatible = match context.model_provider.is_type_compatible(&current_value_type, type_name).await {
                            Ok(compatible) => compatible,
                            Err(_) => {
                                // Fallback to basic type checking
                                Self::basic_type_compatibility(value, type_name)
                            }
                        };

                        if is_compatible {
                            result.push(value.clone());
                        }
                    }

                    Ok(result)
                })
            }
        )
    }

    fn register_type_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "type",
            category: FunctionCategory::Type,
            description: "Returns the type information of the input value as a TypeInfo object",
            parameters: [],
            return_type: "TypeInfo",
            examples: [
                "Patient.name.type().name",
                "5.type().namespace",
                "true.type()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0055,
                        "type() can only be called on a single value".to_string()
                    ));
                }

                let value_type = TypeChecker::get_type(&context.input[0]);
                let type_name = value_type.type_name();

                // Create TypeInfo object with namespace and name properties
                let namespace = if TypeUtils::is_primitive_type(&value_type) ||
                                  TypeUtils::is_fhir_primitive_type(&value_type) {
                    "System"
                } else {
                    "FHIR"
                };

                let type_info = serde_json::json!({
                    "namespace": namespace,
                    "name": type_name
                });

                Ok(vec![FhirPathValue::JsonValue(type_info)])
            }
        )
    }

    /// Get the type name for a FhirPathValue for ModelProvider operations
    fn get_value_type_name(value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::Boolean(_) => "Boolean".to_string(),
            FhirPathValue::Integer(_) => "Integer".to_string(),
            FhirPathValue::Decimal(_) => "Decimal".to_string(),
            FhirPathValue::String(_) => "String".to_string(),
            FhirPathValue::Date(_) => "Date".to_string(),
            FhirPathValue::DateTime(_) => "DateTime".to_string(),
            FhirPathValue::Time(_) => "Time".to_string(),
            FhirPathValue::Quantity { .. } => "Quantity".to_string(),
            FhirPathValue::Resource(map) | FhirPathValue::JsonValue(map) => {
                // Try to get resourceType for FHIR resources
                if let Some(resource_type) = map.get("resourceType").and_then(|v| v.as_str()) {
                    resource_type.to_string()
                } else {
                    "Resource".to_string()
                }
            }
            FhirPathValue::Id(_) => "Id".to_string(),
            FhirPathValue::Base64Binary(_) => "Base64Binary".to_string(),
            FhirPathValue::Uri(_) => "Uri".to_string(),
            FhirPathValue::Url(_) => "Url".to_string(),
            FhirPathValue::Collection(_) => "Collection".to_string(),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                format!("{}.{}", namespace, name)
            }
            FhirPathValue::Empty => "Empty".to_string(),
        }
    }

    /// Basic type compatibility check as fallback when ModelProvider is unavailable
    fn basic_type_compatibility(value: &FhirPathValue, target_type: &str) -> bool {
        let current_type = Self::get_value_type_name(value);

        // Direct type match
        if current_type == target_type {
            return true;
        }

        // Basic inheritance relationships for essential types
        match (current_type.as_str(), target_type) {
            // All FHIR resources are Resources
            (_, "Resource") => matches!(
                value,
                FhirPathValue::Resource(_) | FhirPathValue::JsonValue(_)
            ),
            // System type compatibility
            ("Boolean", "boolean") | ("boolean", "Boolean") => true,
            ("Integer", "integer") | ("integer", "Integer") => true,
            ("String", "string") | ("string", "String") => true,
            ("Decimal", "decimal") | ("decimal", "Decimal") => true,
            _ => false,
        }
    }
}
