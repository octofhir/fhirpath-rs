//! Is operation - async implementation for FunctionRegistry

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{AsyncOperation, EvaluationContext};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use sonic_rs::JsonValueTrait;

/// Is operation - checks if value is of specific type
#[derive(Debug, Default, Clone)]
pub struct IsOperation;

impl IsOperation {
    pub fn new() -> Self {
        Self
    }

    /// Check if a value matches a type name
    async fn matches_type(
        &self,
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> bool {
        // For FHIR types, try to use ModelProvider to get the correct type information
        // This is important for cases like Patient.gender which is stored as a string
        // but should be recognized as a FHIR code type
        if context
            .model_provider
            .is_value_of_type(value, type_name)
            .await
        {
            return true;
        }

        // Also try with FHIR prefix if not already present
        if !type_name.starts_with("FHIR.") {
            let fhir_type_name = format!("FHIR.{type_name}");
            if context
                .model_provider
                .is_value_of_type(value, &fhir_type_name)
                .await
            {
                return true;
            }
        }

        // Also try without FHIR prefix if present
        if let Some(bare_type_name) = type_name.strip_prefix("FHIR.") {
            if context
                .model_provider
                .is_value_of_type(value, bare_type_name)
                .await
            {
                return true;
            }
        }

        // Fall back to basic type checking for System types and basic values
        let actual_type = match value {
            FhirPathValue::String(_) => "System.String",
            FhirPathValue::Integer(_) => "System.Integer",
            FhirPathValue::Decimal(_) => "System.Decimal",
            FhirPathValue::Boolean(_) => "System.Boolean",
            FhirPathValue::DateTime(_) => "System.DateTime",
            FhirPathValue::Date(_) => "System.Date",
            FhirPathValue::Time(_) => "System.Time",
            FhirPathValue::Quantity(_) => "System.Quantity",
            FhirPathValue::JsonValue(json_val) => {
                // Try to match FHIR resource type
                if let Some(resource_type) = json_val.as_inner().get("resourceType") {
                    if let Some(type_str) = resource_type.as_str() {
                        let fhir_type = format!("FHIR.{type_str}");
                        return type_name == fhir_type || type_name == type_str;
                    }
                }
                "System.Object"
            }
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.resource_type() {
                    let fhir_type = format!("FHIR.{resource_type}");
                    return type_name == fhir_type || type_name == resource_type;
                }
                "System.Object"
            }
            FhirPathValue::TypeInfoObject { namespace, name } => {
                let full_type = format!("{namespace}.{name}");
                return type_name == full_type
                    || type_name.eq_ignore_ascii_case(&full_type)
                    || type_name.eq_ignore_ascii_case(name);
            }
            FhirPathValue::Collection(_) => "System.Collection",
            FhirPathValue::Empty => return false, // Empty never matches any type
        };

        // Direct match or case-insensitive match
        actual_type.eq_ignore_ascii_case(type_name) ||
        actual_type == type_name ||
        // Also allow matching without System. prefix
        (actual_type.starts_with("System.") && 
         actual_type[7..].eq_ignore_ascii_case(type_name))
    }
}

#[async_trait]
impl AsyncOperation for IsOperation {
    fn name(&self) -> &'static str {
        "is"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "is",
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
        // is() takes exactly one argument - the type name
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "is".to_string(),
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
                    return Ok(FhirPathValue::Boolean(
                        self.matches_type(&context.input, &format!("{namespace}.{name}"), context)
                            .await,
                    ));
                }
            }
            // Handle collections containing TypeInfo
            FhirPathValue::Collection(col) => {
                if col.len() == 1 {
                    if let Some(FhirPathValue::TypeInfoObject { namespace, name }) = col.get(0) {
                        if namespace.as_ref() == "FHIR" {
                            name.as_ref()
                        } else {
                            return Ok(FhirPathValue::Boolean(
                                self.matches_type(
                                    &context.input,
                                    &format!("{namespace}.{name}"),
                                    context,
                                )
                                .await,
                            ));
                        }
                    } else {
                        return Err(FhirPathError::TypeError {
                            message: format!(
                                "is() type argument must be a type identifier or string, got {}",
                                col.get(0).map(|v| v.type_name()).unwrap_or("None")
                            ),
                        });
                    }
                } else if col.is_empty() {
                    return Ok(FhirPathValue::Boolean(false));
                } else {
                    return Err(FhirPathError::TypeError {
                        message: "is() type argument must be a single type identifier".to_string(),
                    });
                }
            }
            // Handle empty arguments
            FhirPathValue::Empty => return Ok(FhirPathValue::Boolean(false)),
            // For other types, provide a helpful error message
            other => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "is() type argument must be a type identifier or string, got {}",
                        other.type_name()
                    ),
                });
            }
        };

        match &context.input {
            FhirPathValue::Collection(col) => {
                let mut results = Vec::new();

                for item in col.iter() {
                    let matches = self.matches_type(item, type_name, context).await;
                    results.push(FhirPathValue::Boolean(matches));
                }

                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(results),
                ))
            }
            FhirPathValue::Empty => {
                // Empty input returns empty collection
                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ))
            }
            _ => {
                // Single item - check if it matches the type
                let matches = self.matches_type(&context.input, type_name, context).await;
                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![FhirPathValue::Boolean(
                        matches,
                    )]),
                ))
            }
        }
    }
}
