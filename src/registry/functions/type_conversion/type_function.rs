//! type() function - returns the type of the value

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;

/// type() function - returns the type of the value
pub struct TypeFunction;

impl FhirPathFunction for TypeFunction {
    fn name(&self) -> &str {
        "type"
    }
    fn human_friendly_name(&self) -> &str {
        "Type"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "type",
                vec![],
                TypeInfo::Any, // Returns a TypeInfo object
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // type() is a pure type conversion function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let type_info = match input_item {
            FhirPathValue::String(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "String".to_string(),
            },
            FhirPathValue::Integer(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Integer".to_string(),
            },
            FhirPathValue::Decimal(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Decimal".to_string(),
            },
            FhirPathValue::Boolean(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
            },
            FhirPathValue::Date(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Date".to_string(),
            },
            FhirPathValue::DateTime(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "DateTime".to_string(),
            },
            FhirPathValue::Time(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Time".to_string(),
            },
            FhirPathValue::Quantity(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Quantity".to_string(),
            },
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, determine the appropriate type
                let resource_type = resource.resource_type();

                // Check if this is a FHIR primitive type by examining the value
                if let Some(_json_value) = resource.as_json().as_bool() {
                    // Boolean primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "boolean".to_string(),
                    }
                } else if let Some(json_value) = resource.as_json().as_str() {
                    // String-based FHIR primitive
                    // Check if it looks like a UUID or URI
                    let fhir_type = if json_value.starts_with("urn:uuid:") {
                        "uuid"
                    } else if json_value.starts_with("http://")
                        || json_value.starts_with("https://")
                        || json_value.starts_with("urn:")
                    {
                        "uri"
                    } else {
                        "string"
                    };
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: fhir_type.to_string(),
                    }
                } else if let Some(_json_value) = resource.as_json().as_i64() {
                    // Integer primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "integer".to_string(),
                    }
                } else if let Some(_json_value) = resource.as_json().as_f64() {
                    // Decimal primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "decimal".to_string(),
                    }
                } else if resource_type.is_some() {
                    // This is a complex FHIR resource with a resourceType
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: resource_type.unwrap().to_string(),
                    }
                } else {
                    // Unknown resource type
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "Unknown".to_string(),
                    }
                }
            }
            FhirPathValue::Collection(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Collection".to_string(),
            },
            FhirPathValue::TypeInfoObject { .. } => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "TypeInfo".to_string(),
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };
        Ok(FhirPathValue::collection(vec![type_info]))
    }
}
