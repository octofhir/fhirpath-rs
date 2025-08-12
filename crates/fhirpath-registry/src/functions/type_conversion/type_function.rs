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

//! type() function - returns the type of the value

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

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

        // Detect if this specific value came from a FHIR resource property navigation
        // Rather than checking the root context, we need to determine if this particular
        // value was the result of navigating a FHIR resource property
        let is_fhir_primitive = match &context.input {
            // If the current input is exactly the root input and root is FHIR resource,
            // then we're directly accessing the resource, not a navigated property
            FhirPathValue::Resource(res) if context.root == context.input => false,
            // If we're in FHIR context and the current value is a primitive that's different from the root,
            // it likely came from resource navigation
            _ => match &context.root {
                FhirPathValue::Resource(res) => {
                    res.resource_type().is_some()
                        && context.input != context.root
                        && matches!(
                            input_item,
                            FhirPathValue::String(_)
                                | FhirPathValue::Integer(_)
                                | FhirPathValue::Decimal(_)
                                | FhirPathValue::Boolean(_)
                        )
                }
                _ => false,
            },
        };

        let type_info = match input_item {
            FhirPathValue::String(_) => FhirPathValue::TypeInfoObject {
                namespace: if is_fhir_primitive { "FHIR" } else { "System" }.into(),
                name: if is_fhir_primitive {
                    "string"
                } else {
                    "String"
                }
                .into(),
            },
            FhirPathValue::Integer(_) => FhirPathValue::TypeInfoObject {
                namespace: if is_fhir_primitive { "FHIR" } else { "System" }.into(),
                name: if is_fhir_primitive {
                    "integer"
                } else {
                    "Integer"
                }
                .into(),
            },
            FhirPathValue::Decimal(_) => FhirPathValue::TypeInfoObject {
                namespace: if is_fhir_primitive { "FHIR" } else { "System" }.into(),
                name: if is_fhir_primitive {
                    "decimal"
                } else {
                    "Decimal"
                }
                .into(),
            },
            FhirPathValue::Boolean(_) => FhirPathValue::TypeInfoObject {
                namespace: if is_fhir_primitive { "FHIR" } else { "System" }.into(),
                name: if is_fhir_primitive {
                    "boolean"
                } else {
                    "Boolean"
                }
                .into(),
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

                // Check if this resource has source property metadata (from our boxing system)
                if let Some(source_property) = resource
                    .get_property("_fhir_source_property")
                    .and_then(|v| v.as_str())
                {
                    // Use the source property to determine the correct FHIR type
                    let fhir_type =
                        if source_property.starts_with("value") && source_property.len() > 5 {
                            // This is a value[x] property - extract the type from the suffix
                            let type_suffix = &source_property[5..]; // Remove "value" prefix
                            match type_suffix {
                                "String" => "string".to_string(),
                                "Integer" => "integer".to_string(),
                                "Decimal" => "decimal".to_string(),
                                "Boolean" => "boolean".to_string(),
                                "Date" => "date".to_string(),
                                "DateTime" => "dateTime".to_string(),
                                "Time" => "time".to_string(),
                                "Uuid" => "uuid".to_string(),
                                "Uri" => "uri".to_string(),
                                "Code" => "code".to_string(),
                                _ => type_suffix.to_lowercase(),
                            }
                        } else {
                            // Regular property - infer from the wrapped value
                            if let Some(wrapped_value) = resource.get_property("value") {
                                self.infer_fhir_type_from_value(wrapped_value)
                            } else {
                                self.infer_fhir_type_from_value(resource.as_json())
                            }
                        };

                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".into(),
                        name: fhir_type.into(),
                    }
                } else if let Some(wrapped_value) = resource.get_property("value") {
                    // This might be a wrapped primitive value - check the source property
                    if let Some(source_property) = resource
                        .get_property("_fhir_source_property")
                        .and_then(|v| v.as_str())
                    {
                        let fhir_type =
                            if source_property.starts_with("value") && source_property.len() > 5 {
                                let type_suffix = &source_property[5..];
                                match type_suffix {
                                    "String" => "string".to_string(),
                                    "Integer" => "integer".to_string(),
                                    "Decimal" => "decimal".to_string(),
                                    "Boolean" => "boolean".to_string(),
                                    "Date" => "date".to_string(),
                                    "DateTime" => "dateTime".to_string(),
                                    "Time" => "time".to_string(),
                                    "Uuid" => "uuid".to_string(),
                                    "Uri" => "uri".to_string(),
                                    "Code" => "code".to_string(),
                                    _ => type_suffix.to_lowercase(),
                                }
                            } else {
                                self.infer_fhir_type_from_value(wrapped_value)
                            };

                        FhirPathValue::TypeInfoObject {
                            namespace: "FHIR".into(),
                            name: fhir_type.into(),
                        }
                    } else {
                        // Fallback to value-based inference
                        let inferred_type = self.infer_fhir_type_from_value(wrapped_value);
                        FhirPathValue::TypeInfoObject {
                            namespace: "FHIR".into(),
                            name: inferred_type.into(),
                        }
                    }
                }
                // Check if this is a FHIR primitive type by examining the value
                else if let Some(_json_value) = resource.as_json().as_bool() {
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
            FhirPathValue::JsonValue(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".into(),
                name: "JsonValue".into(),
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };
        Ok(type_info)
    }
}

impl TypeFunction {
    /// Infer FHIR type from JSON value
    fn infer_fhir_type_from_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Bool(_) => "boolean".to_string(),
            serde_json::Value::Number(n) => {
                if n.is_f64() {
                    "decimal".to_string()
                } else {
                    "integer".to_string()
                }
            }
            serde_json::Value::String(s) => {
                if s.starts_with("urn:uuid:") {
                    "uuid".to_string()
                } else if s.starts_with("http://")
                    || s.starts_with("https://")
                    || s.starts_with("urn:")
                {
                    "uri".to_string()
                } else {
                    "string".to_string()
                }
            }
            _ => "Resource".to_string(),
        }
    }
}
