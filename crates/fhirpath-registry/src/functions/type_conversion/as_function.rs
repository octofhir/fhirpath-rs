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

//! as() function - type casting function

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use chrono::TimeZone;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

/// as() function - performs type casting
pub struct AsFunction;

#[async_trait]
impl AsyncFhirPathFunction for AsFunction {
    fn name(&self) -> &str {
        "as"
    }

    fn human_friendly_name(&self) -> &str {
        "Type Cast"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "as",
                vec![ParameterInfo::required("type", TypeInfo::Any)], // Accept String, TypeInfoObject, or Resource
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // as() is a pure type conversion function
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Get the type name from the argument
        let type_name = match &args[0] {
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
                // Handle case where the argument is a resource (e.g., Patient in as(Patient))
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

        // Perform type casting based on the input value
        match (&context.input, type_name) {
            // Empty always returns empty
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),

            // String casting - only if actually a string type
            (FhirPathValue::String(s), "string" | "String" | "System.String") => {
                Ok(FhirPathValue::String(s.clone()))
            }

            // Code type casting - in FHIRPath, code is a subtype of string
            // Since we store codes as strings, we accept string values for code type
            (FhirPathValue::String(s), "code" | "Code" | "FHIR.code") => {
                Ok(FhirPathValue::String(s.clone()))
            }

            // Integer casting
            (FhirPathValue::Integer(i), "integer" | "Integer" | "System.Integer") => {
                Ok(FhirPathValue::Integer(*i))
            }
            (FhirPathValue::Decimal(d), "integer" | "Integer" | "System.Integer") => {
                // Try to convert decimal to integer if it has no fractional part
                if d.fract().is_zero() {
                    if let Some(i) = d.to_i64() {
                        Ok(FhirPathValue::Integer(i))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            (FhirPathValue::String(s), "integer" | "Integer" | "System.Integer") => {
                match s.parse::<i64>() {
                    Ok(i) => Ok(FhirPathValue::Integer(i)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }

            // Decimal casting
            (FhirPathValue::Decimal(d), "decimal" | "Decimal" | "System.Decimal") => {
                Ok(FhirPathValue::Decimal(*d))
            }
            (FhirPathValue::Integer(i), "decimal" | "Decimal" | "System.Decimal") => {
                Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i)))
            }
            (FhirPathValue::String(s), "decimal" | "Decimal" | "System.Decimal") => {
                match s.parse::<rust_decimal::Decimal>() {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }

            // Boolean casting
            (FhirPathValue::Boolean(b), "boolean" | "Boolean" | "System.Boolean") => {
                Ok(FhirPathValue::Boolean(*b))
            }
            (FhirPathValue::String(s), "boolean" | "Boolean" | "System.Boolean") => {
                match s.as_ref() {
                    "true" => Ok(FhirPathValue::Boolean(true)),
                    "false" => Ok(FhirPathValue::Boolean(false)),
                    _ => Ok(FhirPathValue::Empty),
                }
            }

            // Date casting
            (FhirPathValue::Date(d), "date" | "Date" | "System.Date") => {
                Ok(FhirPathValue::Date(*d))
            }
            (FhirPathValue::DateTime(dt), "date" | "Date" | "System.Date") => {
                Ok(FhirPathValue::Date(dt.date_naive()))
            }
            (FhirPathValue::String(s), "date" | "Date" | "System.Date") => {
                match chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    Ok(d) => Ok(FhirPathValue::Date(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }

            // DateTime casting
            (FhirPathValue::DateTime(dt), "dateTime" | "DateTime" | "System.DateTime") => {
                Ok(FhirPathValue::DateTime(*dt))
            }
            (FhirPathValue::Date(d), "dateTime" | "DateTime" | "System.DateTime") => {
                let dt = d
                    .and_hms_opt(0, 0, 0)
                    .map(|naive| chrono::Utc.from_utc_datetime(&naive).fixed_offset());
                match dt {
                    Some(datetime) => Ok(FhirPathValue::DateTime(datetime)),
                    None => Ok(FhirPathValue::Empty),
                }
            }

            // Time casting
            (FhirPathValue::Time(t), "time" | "Time" | "System.Time") => {
                Ok(FhirPathValue::Time(*t))
            }
            (FhirPathValue::DateTime(dt), "time" | "Time" | "System.Time") => {
                Ok(FhirPathValue::Time(dt.time()))
            }

            // Quantity casting
            (FhirPathValue::Quantity(q), "Quantity" | "System.Quantity") => {
                Ok(FhirPathValue::Quantity(q.clone()))
            }

            // Resource type casting - use ModelProvider for sophisticated type checking
            (FhirPathValue::Resource(resource), _) => {
                self.cast_resource_with_provider(resource, type_name, context)
                    .await
            }

            // Collection handling - try to cast the collection
            (FhirPathValue::Collection(items), _) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if items.len() == 1 {
                    // Single item collection - cast the item
                    let item_context = EvaluationContext {
                        input: items.first().unwrap().clone(),
                        ..context.clone()
                    };
                    self.evaluate(args, &item_context).await
                } else {
                    // Multiple items - return empty
                    Ok(FhirPathValue::Empty)
                }
            }

            // Default: if the type doesn't match, return empty
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

impl AsFunction {
    /// Cast resource using ModelProvider for sophisticated type checking
    async fn cast_resource_with_provider(
        &self,
        resource: &Arc<octofhir_fhirpath_model::resource::FhirResource>,
        target_type: &str,
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if let Some(model_provider) = &context.model_provider {
            if let Some(resource_type) = resource.resource_type() {
                // Check if the resource type is compatible with the requested type (includes inheritance)
                if model_provider
                    .is_type_compatible(resource_type, target_type)
                    .await
                {
                    // Type is compatible - return the resource
                    Ok(FhirPathValue::Resource(resource.clone()))
                } else {
                    // Type is not compatible - return empty
                    Ok(FhirPathValue::Empty)
                }
            } else {
                Ok(FhirPathValue::Empty)
            }
        } else {
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "ModelProvider is required for FHIR resource type casting".to_string(),
            })
        }
    }
}
