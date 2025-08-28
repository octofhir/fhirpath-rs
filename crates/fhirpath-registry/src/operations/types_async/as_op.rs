//! As operation - async implementation for FunctionRegistry

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{AsyncOperation, EvaluationContext};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// As operation - converts values to specific type (returns empty if conversion fails)
#[derive(Debug, Default, Clone)]
pub struct AsOperation;

impl AsOperation {
    pub fn new() -> Self {
        Self
    }

    /// Check if a value matches a type name and return it, or empty if not
    fn try_cast_to_type(&self, value: &FhirPathValue, type_name: &str) -> Option<FhirPathValue> {
        // For 'as', we return the original value if it matches the type, empty otherwise
        let matches = match value {
            FhirPathValue::String(_) => {
                type_name.eq_ignore_ascii_case("System.String")
                    || type_name.eq_ignore_ascii_case("String")
            }
            FhirPathValue::Integer(_) => {
                type_name.eq_ignore_ascii_case("System.Integer")
                    || type_name.eq_ignore_ascii_case("Integer")
            }
            FhirPathValue::Decimal(_) => {
                type_name.eq_ignore_ascii_case("System.Decimal")
                    || type_name.eq_ignore_ascii_case("Decimal")
            }
            FhirPathValue::Boolean(_) => {
                type_name.eq_ignore_ascii_case("System.Boolean")
                    || type_name.eq_ignore_ascii_case("Boolean")
            }
            FhirPathValue::DateTime(_) => {
                type_name.eq_ignore_ascii_case("System.DateTime")
                    || type_name.eq_ignore_ascii_case("DateTime")
            }
            FhirPathValue::Date(_) => {
                type_name.eq_ignore_ascii_case("System.Date")
                    || type_name.eq_ignore_ascii_case("Date")
            }
            FhirPathValue::Time(_) => {
                type_name.eq_ignore_ascii_case("System.Time")
                    || type_name.eq_ignore_ascii_case("Time")
            }
            FhirPathValue::Quantity(_) => {
                type_name.eq_ignore_ascii_case("System.Quantity")
                    || type_name.eq_ignore_ascii_case("Quantity")
            }
            FhirPathValue::JsonValue(json_val) => {
                // Try to match FHIR resource type
                if let Some(resource_type) = json_val.as_inner().get("resourceType") {
                    if let Some(type_str) = resource_type.as_str() {
                        let fhir_type = format!("FHIR.{type_str}");
                        return if type_name == fhir_type || type_name == type_str {
                            Some(value.clone())
                        } else {
                            None
                        };
                    }
                }
                type_name.eq_ignore_ascii_case("System.Object")
                    || type_name.eq_ignore_ascii_case("Object")
            }
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.resource_type() {
                    let fhir_type = format!("FHIR.{resource_type}");
                    return if type_name == fhir_type || type_name == resource_type {
                        Some(value.clone())
                    } else {
                        None
                    };
                }
                type_name.eq_ignore_ascii_case("System.Object")
                    || type_name.eq_ignore_ascii_case("Object")
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                let full_type = format!("{namespace}.{name}");
                return if type_name == full_type
                    || type_name.eq_ignore_ascii_case(&full_type)
                    || type_name.eq_ignore_ascii_case(name)
                {
                    Some(value.clone())
                } else {
                    None
                };
            }
            FhirPathValue::Collection(_) => {
                type_name.eq_ignore_ascii_case("System.Collection")
                    || type_name.eq_ignore_ascii_case("Collection")
            }
            FhirPathValue::Empty => false, // Empty never matches any type
        };

        if matches { Some(value.clone()) } else { None }
    }
}

#[async_trait]
impl AsyncOperation for AsOperation {
    fn name(&self) -> &'static str {
        "as"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "as",
                parameters: vec![ParameterType::Any], // Type identifier or string
                return_type: ValueType::Collection,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    async fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // as() takes exactly one argument - the type name
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "as".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let type_name = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            // Handle TypeInfo values - this is the correct type for type identifiers in FHIRPath
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // For FHIR types, use just the name (e.g., "code" not "FHIR.code")
                // For System types, use the full name (e.g., "System.String")
                if namespace.as_ref() == "FHIR" {
                    name.as_ref()
                } else {
                    // Create full type name for non-FHIR types
                    return match &context.input {
                        FhirPathValue::Collection(col) => {
                            let mut cast_items = Vec::new();
                            let full_type_name = format!("{namespace}.{name}");
                            for item in col.iter() {
                                if let Some(cast_item) =
                                    self.try_cast_to_type(item, &full_type_name)
                                {
                                    cast_items.push(cast_item);
                                }
                            }
                            Ok(FhirPathValue::Collection(cast_items.into()))
                        }
                        single => {
                            let full_type_name = format!("{namespace}.{name}");
                            if let Some(cast_item) = self.try_cast_to_type(single, &full_type_name)
                            {
                                Ok(cast_item)
                            } else {
                                Ok(FhirPathValue::Collection(vec![].into()))
                            }
                        }
                    };
                }
            }
            // Handle collections containing TypeInfo
            FhirPathValue::Collection(col) => {
                if col.len() == 1 {
                    if let Some(FhirPathValue::TypeInfoObject { namespace, name }) = col.get(0) {
                        if namespace.as_ref() == "FHIR" {
                            name.as_ref()
                        } else {
                            let full_type_name = format!("{namespace}.{name}");
                            return match &context.input {
                                FhirPathValue::Collection(input_col) => {
                                    let mut cast_items = Vec::new();
                                    for item in input_col.iter() {
                                        if let Some(cast_item) =
                                            self.try_cast_to_type(item, &full_type_name)
                                        {
                                            cast_items.push(cast_item);
                                        }
                                    }
                                    Ok(FhirPathValue::Collection(cast_items.into()))
                                }
                                single => {
                                    if let Some(cast_item) =
                                        self.try_cast_to_type(single, &full_type_name)
                                    {
                                        Ok(cast_item)
                                    } else {
                                        Ok(FhirPathValue::Collection(vec![].into()))
                                    }
                                }
                            };
                        }
                    } else {
                        return Err(FhirPathError::TypeError {
                            message: format!(
                                "as() type argument must be a type identifier or string, got {}",
                                col.get(0).map(|v| v.type_name()).unwrap_or("None")
                            ),
                        });
                    }
                } else if col.is_empty() {
                    return Ok(FhirPathValue::Collection(vec![].into()));
                } else {
                    return Err(FhirPathError::TypeError {
                        message: "as() type argument must be a single type identifier".to_string(),
                    });
                }
            }
            // Handle empty arguments
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(vec![].into())),
            // For other types, provide a helpful error message
            other => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "as() type argument must be a type identifier or string, got {}",
                        other.type_name()
                    ),
                });
            }
        };

        match &context.input {
            FhirPathValue::Collection(col) => {
                let mut cast_items = Vec::new();

                for item in col.iter() {
                    if let Some(cast_item) = self.try_cast_to_type(item, type_name) {
                        cast_items.push(cast_item);
                    }
                }

                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(cast_items),
                ))
            }
            FhirPathValue::Empty => {
                // Empty input returns empty collection
                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ))
            }
            _ => {
                // Single item - return it if cast succeeds, otherwise empty
                if let Some(cast_item) = self.try_cast_to_type(&context.input, type_name) {
                    Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![cast_item]),
                    ))
                } else {
                    Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![]),
                    ))
                }
            }
        }
    }
}
