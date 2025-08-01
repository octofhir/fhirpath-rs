//! is() function - checks FHIR type inheritance

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// is() function - checks FHIR type inheritance
pub struct IsFunction;

impl FhirPathFunction for IsFunction {
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
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let target_type = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
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
            (None, target_type.as_str())
        };

        let result = match &context.input {
            FhirPathValue::String(_) => {
                // String type hierarchy: System.String, String, or FHIR.string
                match (namespace, type_name) {
                    (None, "String") => true,
                    (Some("System"), "String") => true,
                    (Some("FHIR"), "string") => true,
                    _ => false,
                }
            }
            FhirPathValue::Integer(_) => {
                // Integer type hierarchy: System.Integer, Integer, or FHIR.integer
                match (namespace, type_name) {
                    (None, "Integer") => true,
                    (Some("System"), "Integer") => true,
                    (Some("FHIR"), "integer") => true,
                    _ => false,
                }
            }
            FhirPathValue::Decimal(_) => {
                // Decimal type hierarchy: System.Decimal, Decimal, or FHIR.decimal
                match (namespace, type_name) {
                    (None, "Decimal") => true,
                    (Some("System"), "Decimal") => true,
                    (Some("FHIR"), "decimal") => true,
                    _ => false,
                }
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
                if let Some(ns) = namespace {
                    if ns == "FHIR" {
                        // Check for FHIR primitive types
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
                            // Complex FHIR resource
                            check_fhir_resource_type(resource, type_name)
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
                        // Otherwise check resource type
                        check_fhir_resource_type(resource, type_name)
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
            FhirPathValue::Empty => {
                // Empty has no type
                false
            }
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

fn check_fhir_resource_type(resource: &crate::model::FhirResource, target_type: &str) -> bool {
    // Get the resource type from the resource
    if let Some(resource_type) = resource.resource_type() {
        // Check direct match first
        if resource_type == target_type {
            return true;
        }

        // Check FHIR inheritance hierarchy
        match (resource_type, target_type) {
            // Patient inherits from DomainResource
            ("Patient", "DomainResource") => true,
            ("Patient", "Resource") => true,

            // Observation inherits from DomainResource
            ("Observation", "DomainResource") => true,
            ("Observation", "Resource") => true,

            // DomainResource inherits from Resource
            ("DomainResource", "Resource") => true,

            // Add more inheritance relationships as needed
            _ => false,
        }
    } else {
        false
    }
}
