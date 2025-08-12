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

//! is() function - checks FHIR type inheritance

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// is() function - checks FHIR type inheritance
pub struct IsFunction;

#[async_trait]
impl AsyncFhirPathFunction for IsFunction {
    fn name(&self) -> &str {
        "is"
    }
    fn human_friendly_name(&self) -> &str {
        "Is Type"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "is",
                vec![ParameterInfo::required("type", TypeInfo::Any)], // Accept String, TypeInfoObject, or Resource
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let target_type = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Handle TypeInfoObject arguments like boolean, FHIR.boolean, etc.
                if namespace.is_empty() {
                    name.as_ref()
                } else {
                    // Return the full qualified name for namespaced types
                    &format!("{namespace}.{name}")
                }
            }
            FhirPathValue::Resource(resource) => {
                // Handle case where the argument is a resource (e.g., Patient in is(Patient))
                // Extract the resource type as the type name
                if let Some(resource_type) = resource.resource_type() {
                    resource_type
                } else {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Resource argument has no resource type".to_string(),
                    });
                }
            }
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String, TypeInfoObject, or Resource".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Parse target type - could be simple name or namespace.name
        let (namespace, type_name) = if target_type.contains('.') {
            let parts: Vec<&str> = target_type.splitn(2, '.').collect();
            // Remove backticks if present (e.g., FHIR.`Patient`)
            let clean_name = parts[1].trim_matches('`');
            (Some(parts[0]), clean_name)
        } else {
            (None, target_type)
        };

        let result = match &context.input {
            FhirPathValue::String(_) => {
                // String type hierarchy: System.String, String, or FHIR.string
                matches!(
                    (namespace, type_name),
                    (None, "String") | (Some("System"), "String") | (Some("FHIR"), "string")
                )
            }
            FhirPathValue::Integer(_) => {
                // Integer type hierarchy: System.Integer, Integer, or FHIR.integer
                matches!(
                    (namespace, type_name),
                    (None, "Integer") | (Some("System"), "Integer") | (Some("FHIR"), "integer")
                )
            }
            FhirPathValue::Decimal(_) => {
                // Decimal type hierarchy: System.Decimal, Decimal, or FHIR.decimal
                matches!(
                    (namespace, type_name),
                    (None, "Decimal") | (Some("System"), "Decimal") | (Some("FHIR"), "decimal")
                )
            }
            FhirPathValue::Boolean(_) => {
                // Boolean type hierarchy: System.Boolean, Boolean, or FHIR.boolean
                match (namespace, type_name) {
                    (None, "Boolean") => true,
                    (Some("System"), "Boolean") => true,
                    (Some("FHIR"), "boolean") => true,
                    _ => false,
                }
            }
            FhirPathValue::Date(_) => {
                // Date type hierarchy: System.Date or Date
                match (namespace, type_name) {
                    (None, "Date") => true,
                    (Some("System"), "Date") => true,
                    _ => false,
                }
            }
            FhirPathValue::DateTime(_) => {
                // DateTime type hierarchy: System.DateTime or DateTime
                match (namespace, type_name) {
                    (None, "DateTime") => true,
                    (Some("System"), "DateTime") => true,
                    _ => false,
                }
            }
            FhirPathValue::Time(_) => {
                // Time type hierarchy: System.Time or Time
                match (namespace, type_name) {
                    (None, "Time") => true,
                    (Some("System"), "Time") => true,
                    _ => false,
                }
            }
            FhirPathValue::Quantity(_) => {
                // Quantity type hierarchy: System.Quantity or Quantity
                match (namespace, type_name) {
                    (None, "Quantity") => true,
                    (Some("System"), "Quantity") => true,
                    _ => false,
                }
            }
            FhirPathValue::Resource(resource) => {
                // FHIR resource type hierarchy
                // Handle both FHIR primitive types and complex resources

                // First check if this resource has boxing metadata
                if let Some(source_property) = resource
                    .get_property("_fhir_source_property")
                    .and_then(|v| v.as_str())
                {
                    // Use boxing metadata to determine correct FHIR type
                    let actual_fhir_type =
                        if source_property.starts_with("value") && source_property.len() > 5 {
                            let type_suffix = &source_property[5..];
                            match type_suffix {
                                "String" => "string",
                                "Integer" => "integer",
                                "Decimal" => "decimal",
                                "Boolean" => "boolean",
                                "Date" => "date",
                                "DateTime" => "dateTime",
                                "Time" => "time",
                                "Uuid" => "uuid",
                                "Uri" => "uri",
                                "Code" => "code",
                                _ => {
                                    let lowercased = type_suffix.to_lowercase();
                                    match lowercased.as_str() {
                                        "string" => "string",
                                        "integer" => "integer",
                                        "decimal" => "decimal",
                                        "boolean" => "boolean",
                                        _ => "Resource",
                                    }
                                }
                            }
                        } else {
                            // Infer from value for non-polymorphic properties
                            match resource
                                .get_property("value")
                                .or_else(|| Some(resource.as_json()))
                            {
                                Some(serde_json::Value::Bool(_)) => "boolean",
                                Some(serde_json::Value::Number(n)) => {
                                    if n.is_f64() {
                                        "decimal"
                                    } else {
                                        "integer"
                                    }
                                }
                                Some(serde_json::Value::String(s)) => {
                                    if s.starts_with("urn:uuid:") {
                                        "uuid"
                                    } else if s.starts_with("http://")
                                        || s.starts_with("https://")
                                        || s.starts_with("urn:")
                                    {
                                        "uri"
                                    } else {
                                        "string"
                                    }
                                }
                                _ => "Resource",
                            }
                        };

                    // Check type compatibility with the metadata
                    if let Some(ns) = namespace {
                        if ns == "FHIR" {
                            Self::is_fhir_type_compatible(actual_fhir_type, type_name)
                        } else {
                            false // FHIR resources don't match non-FHIR namespaces
                        }
                    } else {
                        // No namespace specified - match against the actual type
                        Self::is_fhir_type_compatible(actual_fhir_type, type_name)
                    }
                } else if let Some(ns) = namespace {
                    if ns == "FHIR" {
                        // Check for FHIR primitive types (legacy logic)
                        if let Some(_json_value) = resource.as_json().as_bool() {
                            type_name == "boolean"
                        } else if let Some(json_value) = resource.as_json().as_str() {
                            // Check specific string-based FHIR types
                            match type_name {
                                "string" => true,
                                "uuid" => json_value.starts_with("urn:uuid:"),
                                "uri" => {
                                    json_value.starts_with("http://")
                                        || json_value.starts_with("https://")
                                        || json_value.starts_with("urn:")
                                }
                                _ => false,
                            }
                        } else if let Some(_json_value) = resource.as_json().as_i64() {
                            type_name == "integer"
                        } else if let Some(_json_value) = resource.as_json().as_f64() {
                            type_name == "decimal"
                        } else {
                            // Complex FHIR resource - use ModelProvider for proper type checking
                            if let Some(model_provider) = &context.model_provider {
                                if let Some(resource_type) = resource.resource_type() {
                                    model_provider
                                        .is_type_compatible(resource_type, type_name)
                                        .await
                                } else {
                                    false
                                }
                            } else {
                                return Err(FunctionError::EvaluationError {
                                    name: self.name().to_string(),
                                    message:
                                        "ModelProvider is required for FHIR resource type checking"
                                            .to_string(),
                                });
                            }
                        }
                    } else if ns == "System" {
                        // FHIR resources don't match System types
                        false
                    } else {
                        false
                    }
                } else {
                    // No namespace specified - check if it's a FHIR type name
                    // For lowercase names, check if it's a FHIR primitive
                    if type_name == "boolean" && resource.as_json().as_bool().is_some() {
                        true
                    } else if let Some(str_value) = resource.as_json().as_str() {
                        match type_name {
                            "string" => true,
                            "uuid" => str_value.starts_with("urn:uuid:"),
                            "uri" => {
                                str_value.starts_with("http://")
                                    || str_value.starts_with("https://")
                                    || str_value.starts_with("urn:")
                            }
                            _ => false,
                        }
                    } else if type_name == "integer" && resource.as_json().as_i64().is_some() {
                        true
                    } else if type_name == "decimal" && resource.as_json().as_f64().is_some() {
                        true
                    } else {
                        // Otherwise check resource type using ModelProvider
                        if let Some(model_provider) = &context.model_provider {
                            if let Some(resource_type) = resource.resource_type() {
                                model_provider
                                    .is_type_compatible(resource_type, type_name)
                                    .await
                            } else {
                                false
                            }
                        } else {
                            return Err(FunctionError::EvaluationError {
                                name: self.name().to_string(),
                                message:
                                    "ModelProvider is required for FHIR resource type checking"
                                        .to_string(),
                            });
                        }
                    }
                }
            }
            FhirPathValue::Collection(_) => {
                // Collections don't have a specific type
                false
            }
            FhirPathValue::TypeInfoObject { .. } => {
                // TypeInfo objects have type TypeInfo
                match (namespace, type_name) {
                    (None, "TypeInfo") => true,
                    (Some("System"), "TypeInfo") => true,
                    _ => false,
                }
            }
            FhirPathValue::JsonValue(_) => {
                // JsonValue can match Object or Any type
                match (namespace, type_name) {
                    (None, "JsonValue") => true,
                    (None, "Object") => true,
                    (None, "Any") => true,
                    _ => false,
                }
            }
            FhirPathValue::Empty => {
                // Empty has no type
                false
            }
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

impl IsFunction {
    /// Check if a FHIR type is compatible with the target type (including inheritance)
    fn is_fhir_type_compatible(actual_type: &str, target_type: &str) -> bool {
        // Direct match
        if actual_type == target_type {
            return true;
        }

        // Handle FHIR type hierarchy
        match (actual_type, target_type) {
            // UUID is a subtype of URI in FHIR
            ("uuid", "uri") => true,
            _ => false,
        }
    }
}
