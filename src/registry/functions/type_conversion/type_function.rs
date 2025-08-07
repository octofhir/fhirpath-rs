//! type() function - returns the type of the value

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// type() function - returns the type of the value
pub struct TypeFunction;

#[async_trait]
impl AsyncFhirPathFunction for TypeFunction {
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
    async fn evaluate(
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
                        name: self.name().into(),
                        message: "Input collection contains multiple items".into(),
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
                namespace: "System".into(),
                name: "String".into(),
            },
            FhirPathValue::Integer(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Integer".into(),
            },
            FhirPathValue::Decimal(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Decimal".into(),
            },
            FhirPathValue::Boolean(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Boolean".into(),
            },
            FhirPathValue::Date(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Date".into(),
            },
            FhirPathValue::DateTime(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "DateTime".into(),
            },
            FhirPathValue::Time(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Time".into(),
            },
            FhirPathValue::Quantity(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Quantity".into(),
            },
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, determine the appropriate type
                let resource_type = resource.resource_type();

                // Check if this is a FHIR primitive type by examining the value
                if let Some(_json_value) = resource.as_json().as_bool() {
                    // Boolean primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".into(),
                        name: "boolean".into(),
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
                        namespace: "FHIR".into(),
                        name: fhir_type.into(),
                    }
                } else if let Some(_json_value) = resource.as_json().as_i64() {
                    // Integer primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".into(),
                        name: "integer".into(),
                    }
                } else if let Some(_json_value) = resource.as_json().as_f64() {
                    // Decimal primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".into(),
                        name: "decimal".into(),
                    }
                } else if resource_type.is_some() {
                    // This is a complex FHIR resource with a resourceType
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".into(),
                        name: resource_type.unwrap().into(),
                    }
                } else {
                    // Unknown resource type
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".into(),
                        name: "Unknown".into(),
                    }
                }
            }
            FhirPathValue::Collection(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "Collection".into(),
            },
            FhirPathValue::TypeInfoObject { .. } => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "TypeInfo".into(),
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };
        Ok(type_info)
    }
}
