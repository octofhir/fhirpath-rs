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

//! Navigation operations evaluator

use crate::context::EvaluationContext as LocalEvaluationContext;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{FhirPathValue, JsonValue};
use octofhir_fhirpath_registry::{
    FunctionRegistry, traits::EvaluationContext as RegistryEvaluationContext,
};
use std::sync::Arc;

/// Specialized evaluator for navigation and member access operations
pub struct NavigationEvaluator;

impl NavigationEvaluator {
    /// Evaluate member access with ModelProvider choice type support
    pub fn evaluate_member_access_with_model_provider<'a>(
        target: &'a FhirPathValue,
        member: &'a str,
        registry: &'a Arc<FunctionRegistry>,
        context: &'a LocalEvaluationContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + Send + 'a>,
    > {
        Box::pin(async move {
            match target {
                FhirPathValue::JsonValue(json) => {
                    // Use ModelProvider for choice type resolution
                    Self::evaluate_json_member_access_with_model_provider(json, member, context)
                        .await
                }

                FhirPathValue::Collection(items) => {
                    let mut result_items = Vec::new();
                    for item in items.iter() {
                        let member_result = Self::evaluate_member_access_with_model_provider(
                            item, member, registry, context,
                        )
                        .await?;
                        match member_result {
                            FhirPathValue::Collection(nested_items) => {
                                result_items.extend(nested_items.iter().cloned());
                            }
                            FhirPathValue::Empty => {
                                // Skip empty results
                            }
                            value => {
                                result_items.push(value);
                            }
                        }
                    }

                    let collection = octofhir_fhirpath_model::Collection::from_vec(result_items);
                    let flattened = collection.flatten();
                    Ok(FhirPathValue::normalize_collection_result(
                        flattened.into_vec(),
                    ))
                }

                // Handle TypeInfoObject property access for .namespace and .name
                FhirPathValue::TypeInfoObject { namespace, name } => match member {
                    "namespace" => Ok(FhirPathValue::String(namespace.clone())),
                    "name" => Ok(FhirPathValue::String(name.clone())),
                    _ => Ok(FhirPathValue::Empty),
                },

                // Handle Quantity property access
                FhirPathValue::Quantity(quantity) => match member {
                    "value" => Ok(FhirPathValue::Decimal(quantity.value)),
                    "unit" => {
                        if let Some(ref unit) = quantity.unit {
                            Ok(FhirPathValue::String(unit.clone().into()))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    _ => Ok(FhirPathValue::Empty),
                },

                _ => Ok(FhirPathValue::Empty),
            }
        })
    }

    /// Enhanced JSON member access using ModelProvider choice type resolution
    async fn evaluate_json_member_access_with_model_provider(
        json: &JsonValue,
        member: &str,
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Direct property access first
        if let Some(value) = json.get_property(member) {
            return Ok(Self::convert_json_to_fhirpath_value(value));
        }

        // Get resource type for choice type resolution
        let resource_type = json
            .get_property("resourceType")
            .and_then(|rt| {
                if rt.is_string() {
                    rt.as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Element".to_string());

        // Check if this is a choice property using ModelProvider
        let model_provider = &context.model_provider;

        if model_provider
            .is_choice_property(&resource_type, member)
            .await
        {
            // Create FhirPathValue from JSON for data checking
            let json_value = FhirPathValue::JsonValue(json.clone());

            // Try to resolve the specific choice property from the data
            if let Some(resolved_property) = model_provider
                .resolve_choice_property(&resource_type, member, &json_value)
                .await
            {
                if let Some(value) = json.get_property(&resolved_property) {
                    return Ok(Self::convert_json_to_fhirpath_value(value));
                }
            }

            // If no specific property found in data, return all possible variants
            let variants = model_provider
                .get_choice_variants(&resource_type, member)
                .await;
            let mut result_items = Vec::new();

            for variant in variants {
                if let Some(value) = json.get_property(&variant.property_name) {
                    result_items.push(Self::convert_json_to_fhirpath_value(value));
                }
            }

            // Return collection result (may be empty if no choice variants exist in data)
            if result_items.is_empty() {
                return Ok(FhirPathValue::Empty);
            } else if result_items.len() == 1 {
                return Ok(result_items.into_iter().next().unwrap());
            } else {
                return Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from_vec(result_items),
                ));
            }
        }

        // Fallback to standard choice type resolution
        Self::evaluate_json_member_access(json, member).await
    }

    /// Get value at a specific path in JSON
    fn get_value_at_path(json: &JsonValue, path: &str) -> Option<JsonValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json.clone();

        for part in parts {
            if let Some(next) = current.get_property(part) {
                current = next;
            } else {
                return None;
            }
        }

        Some(current)
    }
    /// Evaluate member access with polymorphic FHIR support (async with boxing for recursion)
    pub fn evaluate_member_access<'a>(
        target: &'a FhirPathValue,
        member: &'a str,
        registry: &'a Arc<FunctionRegistry>,
        context: &'a LocalEvaluationContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + Send + 'a>,
    > {
        Box::pin(async move {
            match target {
                FhirPathValue::JsonValue(json) => {
                    // Use enhanced choice type resolution with ModelProvider
                    Self::evaluate_json_member_access_with_model_provider(json, member, context)
                        .await
                }

                FhirPathValue::Collection(items) => {
                    let mut result_items = Vec::new();
                    for item in items.iter() {
                        let member_result = Self::evaluate_member_access_with_model_provider(
                            item, member, registry, context,
                        )
                        .await?;
                        match member_result {
                            FhirPathValue::Collection(nested_items) => {
                                result_items.extend(nested_items.iter().cloned());
                            }
                            FhirPathValue::Empty => {
                                // Skip empty results
                            }
                            value => {
                                result_items.push(value);
                            }
                        }
                    }
                    // Create collection and flatten nested collections (FHIRPath navigation semantics)
                    let collection = octofhir_fhirpath_model::Collection::from_vec(result_items);
                    let flattened = collection.flatten();
                    Ok(FhirPathValue::normalize_collection_result(
                        flattened.into_vec(),
                    ))
                }

                // Handle TypeInfoObject property access for .namespace and .name
                FhirPathValue::TypeInfoObject { namespace, name } => match member {
                    "namespace" => Ok(FhirPathValue::String(namespace.clone())),
                    "name" => Ok(FhirPathValue::String(name.clone())),
                    _ => Ok(FhirPathValue::Empty),
                },

                // Handle Quantity property access
                FhirPathValue::Quantity(quantity) => match member {
                    "value" => Ok(FhirPathValue::Decimal(quantity.value)),
                    "unit" => {
                        if let Some(ref unit) = quantity.unit {
                            Ok(FhirPathValue::String(unit.clone().into()))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    _ => Ok(FhirPathValue::Empty),
                },

                _ => Ok(FhirPathValue::Empty),
            }
        })
    }

    /// Evaluate member access on JSON values with polymorphic FHIR support
    async fn evaluate_json_member_access(
        json: &JsonValue,
        member: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // Direct property access first
        if let Some(value) = json.get_property(member) {
            return Ok(Self::convert_json_to_fhirpath_value(value));
        }

        // FHIR choice type polymorphic access - fallback pattern matching
        if json.is_object() {
            // Fallback to pattern-based search for backward compatibility
            if let Some(iter) = json.object_iter() {
                for (key, value) in iter {
                    if key.starts_with(member) && key.len() > member.len() {
                        if let Some(next_char) = key.chars().nth(member.len()) {
                            if next_char.is_uppercase() {
                                return Ok(Self::convert_json_to_fhirpath_value(value));
                            }
                        }
                    }
                }
            }
        }

        Ok(FhirPathValue::Empty)
    }

    /// Convert JsonValue to proper FhirPathValue type using Sonic JSON natively
    pub fn convert_json_to_fhirpath_value(json_value: JsonValue) -> FhirPathValue {
        use octofhir_fhirpath_model::FhirPathValue;
        use rust_decimal::Decimal;
        use std::sync::Arc;

        // Use Sonic JSON API directly to determine the correct FhirPath type
        if json_value.is_boolean() {
            if let Some(b) = json_value.as_bool() {
                FhirPathValue::Boolean(b)
            } else {
                FhirPathValue::JsonValue(json_value)
            }
        } else if json_value.is_number() {
            // Try integer first, then decimal
            if let Some(i) = json_value.as_i64() {
                FhirPathValue::Integer(i)
            } else if let Some(f) = json_value.as_f64() {
                // Convert float to Decimal for precision
                if let Ok(decimal) = Decimal::try_from(f) {
                    FhirPathValue::Decimal(decimal)
                } else {
                    FhirPathValue::JsonValue(json_value)
                }
            } else {
                FhirPathValue::JsonValue(json_value)
            }
        } else if json_value.is_string() {
            if let Some(s) = json_value.as_str() {
                use chrono::{DateTime, NaiveDate, NaiveTime};

                // Try to parse as date/datetime/time first
                if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    FhirPathValue::Date(octofhir_fhirpath_model::temporal::PrecisionDate::new(
                        date,
                        octofhir_fhirpath_model::temporal::TemporalPrecision::Day,
                    ))
                } else if let Ok(datetime) = DateTime::parse_from_rfc3339(s) {
                    FhirPathValue::DateTime(
                        octofhir_fhirpath_model::temporal::PrecisionDateTime::new(
                            datetime.fixed_offset(),
                            octofhir_fhirpath_model::temporal::TemporalPrecision::Millisecond,
                        ),
                    )
                } else if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S") {
                    FhirPathValue::Time(octofhir_fhirpath_model::temporal::PrecisionTime::new(
                        time,
                        octofhir_fhirpath_model::temporal::TemporalPrecision::Second,
                    ))
                } else if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S%.f") {
                    FhirPathValue::Time(octofhir_fhirpath_model::temporal::PrecisionTime::new(
                        time,
                        octofhir_fhirpath_model::temporal::TemporalPrecision::Millisecond,
                    ))
                } else {
                    FhirPathValue::String(Arc::from(s))
                }
            } else {
                FhirPathValue::JsonValue(json_value)
            }
        } else if json_value.is_array() {
            // Convert array elements to proper FhirPath types
            if let Some(iter) = json_value.array_iter() {
                let items: Vec<FhirPathValue> =
                    iter.map(Self::convert_json_to_fhirpath_value).collect();

                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from_vec(items))
                }
            } else {
                FhirPathValue::JsonValue(json_value)
            }
        } else if json_value.is_null() {
            FhirPathValue::Empty
        } else {
            // For complex objects, keep as JsonValue (they might be FHIR resources)
            FhirPathValue::JsonValue(json_value)
        }
    }

    /// Evaluate children operation (get immediate child elements)
    pub async fn evaluate_children(
        _target: &FhirPathValue,
        registry: &Arc<FunctionRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if registry.has_function("children") {
            registry
                .evaluate("children", &[], context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Children operation error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "Children operation not found in registry".to_string(),
            })
        }
    }

    /// Evaluate descendants operation (get all descendant elements)
    pub async fn evaluate_descendants(
        _target: &FhirPathValue,
        registry: &Arc<FunctionRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if registry.has_function("descendants") {
            registry
                .evaluate("descendants", &[], context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Descendants operation error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "Descendants operation not found in registry".to_string(),
            })
        }
    }

    /// Evaluate ofType operation for type filtering
    pub async fn evaluate_of_type(
        _target: &FhirPathValue,
        type_name: &FhirPathValue,
        registry: &Arc<FunctionRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if registry.has_function("ofType") {
            registry
                .evaluate("ofType", &[type_name.clone()], context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("OfType operation error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "OfType operation not found in registry".to_string(),
            })
        }
    }

    /// Evaluate is operation for type checking
    pub async fn evaluate_is(
        _target: &FhirPathValue,
        type_name: &FhirPathValue,
        registry: &Arc<FunctionRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if registry.has_function("is") {
            registry
                .evaluate("is", &[type_name.clone()], context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Is operation error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "Is operation not found in registry".to_string(),
            })
        }
    }
}
