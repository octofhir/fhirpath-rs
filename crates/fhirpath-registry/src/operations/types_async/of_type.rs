//! OfType function - async implementation for FunctionRegistry

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{AsyncOperation, EvaluationContext};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, FhirPathValue, JsonValueExt, Result};

/// OfType function - filters collection to items of specific type
#[derive(Debug, Default, Clone)]
pub struct OfTypeFunction;

impl OfTypeFunction {
    pub fn new() -> Self {
        Self
    }

    /// Check if a value matches a type name using ModelProvider's enhanced capabilities
    async fn matches_type(&self, value: &FhirPathValue, type_name: &str, context: &EvaluationContext) -> bool {
        // Use ModelProvider for accurate FHIR type matching
        if let Ok(Some(_type_reflection)) = context.model_provider.get_type_reflection(type_name).await {
            // Check type compatibility using ModelProvider
            if let Ok(compatible) = context.model_provider.is_type_compatible(&value.type_name(), type_name).await {
                if compatible {
                    return true;
                }
            }
            
            // Also check type hierarchy for inheritance relationships
            if let Ok(Some(hierarchy)) = context.model_provider.get_type_hierarchy(&value.type_name()).await {
                if hierarchy.ancestors.iter().any(|ancestor| ancestor == type_name) {
                    return true;
                }
            }
        }
        let actual_type = match value {
            FhirPathValue::String(_) => "System.String",
            FhirPathValue::Integer(_) => "System.Integer",
            FhirPathValue::Decimal(_) => "System.Decimal",
            FhirPathValue::Boolean(_) => "System.Boolean",
            FhirPathValue::DateTime(_) => "System.DateTime",
            FhirPathValue::Date(_) => "System.Date",
            FhirPathValue::Time(_) => "System.Time",
            FhirPathValue::Quantity { value: _, .. } => "System.Quantity",
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
impl AsyncOperation for OfTypeFunction {
    fn name(&self) -> &'static str {
        "ofType"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "ofType",
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
        // ofType() takes exactly one argument - the type name
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "ofType".to_string(),
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
                let ns_str: &str = namespace.as_ref();
                let name_str: &str = name.as_ref();
                if ns_str == "FHIR" {
                    name_str
                } else {
                    // Create full type name for non-FHIR types
                    let full_type_name = format!("{namespace}.{name}");
                    return match &context.input {
                        FhirPathValue::Collection(col) => {
                            let mut filtered_items = Vec::new();
                            for item in col.iter() {
                                if self.matches_type(item, &full_type_name, context).await {
                                    filtered_items.push(item.clone());
                                }
                            }
                            Ok(FhirPathValue::Collection(vec![]))
                        }
                        single => {
                            if self.matches_type(single, &full_type_name, context).await {
                                Ok(FhirPathValue::Collection(vec![]))
                            } else {
                                Ok(FhirPathValue::Collection(vec![]))
                            }
                        }
                    };
                }
            }
            // Handle collections containing TypeInfo
            FhirPathValue::Collection(col) => {
                if col.len() == 1 {
                    if let Some(FhirPathValue::TypeInfoObject { namespace, name }) = col.get(0) {
                        let ns_str: &str = namespace.as_ref();
                        let name_str: &str = name.as_ref();
                        if ns_str == "FHIR" {
                            name_str
                        } else {
                            let full_type_name = format!("{namespace}.{name}");
                            return match &context.input {
                                FhirPathValue::Collection(input_col) => {
                                    let mut filtered_items = Vec::new();
                                    for item in input_col.iter() {
                                        if self.matches_type(item, &full_type_name, context).await {
                                            filtered_items.push(item.clone());
                                        }
                                    }
                                    Ok(FhirPathValue::Collection(vec![]))
                                }
                                single => {
                                    if self.matches_type(single, &full_type_name, context).await {
                                        Ok(FhirPathValue::Collection(vec![]))
                                    } else {
                                        Ok(FhirPathValue::Collection(vec![]))
                                    }
                                }
                            };
                        }
                    } else {
                        return Err(FhirPathError::TypeError {
                            message: format!(
                                "ofType() type argument must be a type identifier or string, got {}",
                                col.get(0).map(|v| v.type_name()).unwrap_or("None".to_string())
                            ),
                        });
                    }
                } else if col.is_empty() {
                    return Ok(FhirPathValue::Collection(vec![].into()));
                } else {
                    return Err(FhirPathError::TypeError {
                        message: "ofType() type argument must be a single type identifier"
                            .to_string(),
                    });
                }
            }
            // Handle empty arguments
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(vec![].into())),
            // For other types, provide a helpful error message
            other => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "ofType() type argument must be a type identifier or string, got {}",
                        other.type_name()
                    ),
                });
            }
        };

        match &context.input {
            FhirPathValue::Collection(col) => {
                let mut filtered_items = Vec::new();

                for item in col.iter() {
                    if self.matches_type(item, type_name, context).await {
                        filtered_items.push(item.clone());
                    }
                }

                Ok(FhirPathValue::Collection(filtered_items))
            }
            FhirPathValue::Empty => {
                // Empty input returns empty collection
                Ok(FhirPathValue::Collection(vec![]))
            }
            _ => {
                // Single item - return it if it matches the type, otherwise empty
                if self.matches_type(&context.input, type_name, context).await {
                    Ok(FhirPathValue::Collection(vec![context.input.clone()]))
                } else {
                    Ok(FhirPathValue::Collection(vec![]))
                }
            }
        }
    }
}
