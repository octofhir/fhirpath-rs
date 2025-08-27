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

//! Value type analysis utilities for FHIRPath type reflection

use crate::type_object::{FhirPathTypeObject, TypeObjectMetadata};
use crate::{FhirPathValue, JsonValue};
use sonic_rs::JsonValueTrait;

/// Utility for analyzing FHIRPath value types
pub struct ValueTypeAnalyzer;

impl ValueTypeAnalyzer {
    /// Get the complete type information for a value
    pub async fn get_type_object(
        value: &FhirPathValue,
        _context: Option<&dyn crate::provider::ModelProvider>,
    ) -> Result<FhirPathTypeObject, String> {
        match value {
            // System primitive types
            FhirPathValue::Boolean(_) => Ok(FhirPathTypeObject::system_type("Boolean")),
            FhirPathValue::Integer(_) => Ok(FhirPathTypeObject::system_type("Integer")),
            FhirPathValue::Decimal(_) => Ok(FhirPathTypeObject::system_type("Decimal")),
            FhirPathValue::String(_) => Ok(FhirPathTypeObject::system_type("String")),
            FhirPathValue::Date(_) => Ok(FhirPathTypeObject::system_type("Date")),
            FhirPathValue::DateTime(_) => Ok(FhirPathTypeObject::system_type("DateTime")),
            FhirPathValue::Time(_) => Ok(FhirPathTypeObject::system_type("Time")),
            FhirPathValue::Quantity(_) => Ok(FhirPathTypeObject::system_type("Quantity")),

            // FHIR types from JSON objects
            FhirPathValue::JsonValue(json) => Self::determine_fhir_type_from_json(json).await,

            // Resource types
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.resource_type() {
                    Ok(FhirPathTypeObject::fhir_type(
                        resource_type,
                        Some("DomainResource".to_string()),
                    ))
                } else {
                    Ok(FhirPathTypeObject::fhir_type("Resource", None))
                }
            }

            // Collections - return type of first element
            FhirPathValue::Collection(items) => {
                if let Some(first_item) = items.first() {
                    // Prevent deep recursion for now - handle collection types simply
                    match first_item {
                        FhirPathValue::Boolean(_) => Ok(FhirPathTypeObject::system_type("Boolean")),
                        FhirPathValue::Integer(_) => Ok(FhirPathTypeObject::system_type("Integer")),
                        FhirPathValue::Decimal(_) => Ok(FhirPathTypeObject::system_type("Decimal")),
                        FhirPathValue::String(_) => Ok(FhirPathTypeObject::system_type("String")),
                        _ => Err("Complex collection type not supported yet".to_string()),
                    }
                } else {
                    // Empty collection has no specific type
                    Err("Empty collection has no type".to_string())
                }
            }

            // TypeInfoObject - return its own type information
            FhirPathValue::TypeInfoObject { namespace, name } => {
                let namespace_str = namespace.as_ref().to_string();
                Ok(FhirPathTypeObject {
                    namespace: namespace_str.clone(),
                    name: name.as_ref().to_string(),
                    base_type: None,
                    metadata: TypeObjectMetadata {
                        is_primitive: namespace_str == "System",
                        ..Default::default()
                    },
                })
            }

            FhirPathValue::Empty => Err("Empty value has no type".to_string()),
        }
    }

    /// Get the type name for a value
    pub async fn get_type_name(value: &FhirPathValue) -> Option<String> {
        match Self::get_type_object(value, None).await {
            Ok(type_obj) => Some(type_obj.name),
            Err(_) => None,
        }
    }

    /// Get the namespace for a value
    pub async fn get_namespace(value: &FhirPathValue) -> Option<String> {
        match Self::get_type_object(value, None).await {
            Ok(type_obj) => Some(type_obj.namespace),
            Err(_) => None,
        }
    }

    /// Determine FHIR type from JSON value
    async fn determine_fhir_type_from_json(json: &JsonValue) -> Result<FhirPathTypeObject, String> {
        let sonic_value = json.as_sonic_value();

        // Try to get resourceType first
        if let Some(resource_type) = sonic_value.get("resourceType").and_then(|rt| rt.as_str()) {
            return Ok(FhirPathTypeObject::fhir_type(
                resource_type,
                Some("DomainResource".to_string()),
            ));
        }

        // For FHIR JSON values without resourceType, try to infer type from structure
        if sonic_value.is_object() {
            // Look for common FHIR patterns - check more specific patterns first
            if sonic_value.get("value").is_some() && sonic_value.get("unit").is_some() {
                return Ok(FhirPathTypeObject::fhir_type(
                    "Quantity",
                    Some("Element".to_string()),
                ));
            }

            if sonic_value.get("system").is_some() && sonic_value.get("code").is_some() {
                return Ok(FhirPathTypeObject::fhir_type(
                    "Coding",
                    Some("Element".to_string()),
                ));
            }

            if sonic_value.get("reference").is_some() {
                return Ok(FhirPathTypeObject::fhir_type(
                    "Reference",
                    Some("Element".to_string()),
                ));
            }

            if sonic_value.get("family").is_some() || sonic_value.get("given").is_some() {
                return Ok(FhirPathTypeObject::fhir_type(
                    "HumanName",
                    Some("Element".to_string()),
                ));
            }

            if sonic_value.get("line").is_some() || sonic_value.get("city").is_some() {
                return Ok(FhirPathTypeObject::fhir_type(
                    "Address",
                    Some("Element".to_string()),
                ));
            }
        }

        // For FHIR primitive values, check if this looks like a FHIR primitive
        if sonic_value.is_boolean() {
            // This is likely a FHIR boolean, not a System Boolean
            return Ok(FhirPathTypeObject::fhir_type(
                "boolean",
                Some("Element".to_string()),
            ));
        }

        if sonic_value.is_str() {
            // Could be a FHIR string primitive
            return Ok(FhirPathTypeObject::fhir_type(
                "string",
                Some("Element".to_string()),
            ));
        }

        if sonic_value.is_number() {
            // Could be FHIR integer or decimal
            if sonic_value.as_i64().is_some() {
                return Ok(FhirPathTypeObject::fhir_type(
                    "integer",
                    Some("Element".to_string()),
                ));
            } else {
                return Ok(FhirPathTypeObject::fhir_type(
                    "decimal",
                    Some("Element".to_string()),
                ));
            }
        }

        // Generic fallback for FHIR complex types
        Ok(FhirPathTypeObject::fhir_type("Element", None))
    }

    /// Check if a value is compatible with a target type
    pub async fn is_compatible_with_type(
        value: &FhirPathValue,
        target_namespace: &str,
        target_name: &str,
    ) -> bool {
        if let Ok(actual_type) = Self::get_type_object(value, None).await {
            // Direct match
            if actual_type.namespace == target_namespace && actual_type.name == target_name {
                return true;
            }

            // Simple inheritance checks
            if let Some(base_type) = &actual_type.base_type {
                return Self::check_inheritance_simple(
                    &actual_type.namespace,
                    base_type,
                    target_namespace,
                    target_name,
                );
            }
        }

        false
    }

    /// Simple inheritance checking
    fn check_inheritance_simple(
        actual_namespace: &str,
        actual_base: &str,
        target_namespace: &str,
        target_name: &str,
    ) -> bool {
        if actual_namespace == target_namespace && actual_base == target_name {
            return true;
        }

        // Hardcoded inheritance rules for common FHIR types
        matches!(
            (actual_namespace, actual_base, target_namespace, target_name),
            ("FHIR", "DomainResource", "FHIR", "Resource") | ("FHIR", "Element", "FHIR", "Base")
        )
    }
}
