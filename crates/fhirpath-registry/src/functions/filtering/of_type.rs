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

//! ofType() function - filters collection to items of specified type

use crate::function::EvaluationContext;
use crate::function::{AsyncFhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::provider::ModelProvider;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
use std::sync::Arc;

/// ofType() function - filters collection to items of specified type
pub struct OfTypeFunction;

#[async_trait]
impl AsyncFhirPathFunction for OfTypeFunction {
    fn name(&self) -> &str {
        "ofType"
    }
    fn human_friendly_name(&self) -> &str {
        "OfType"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "ofType",
                vec![ParameterInfo::required("type", TypeInfo::Any)], // Accept any type (String or TypeInfoObject)
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
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

        let type_name = match &args[0] {
            FhirPathValue::String(t) => t.as_ref(),
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // Handle TypeInfoObject arguments like Patient, FHIR.Patient, etc.
                if namespace.as_ref() == "FHIR" || namespace.is_empty() {
                    name.as_ref()
                } else {
                    // For other namespaces, use fully qualified name
                    return self.handle_namespaced_type(&args[0], context).await;
                }
            }
            FhirPathValue::Resource(resource) => {
                // Handle case where the argument is a resource (e.g., Patient in ofType(Patient))
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

        // Get the collection to filter
        let items = match &context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])),
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // ModelProvider is required for sophisticated type matching
        let model_provider =
            context
                .model_provider
                .as_ref()
                .ok_or_else(|| FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "ModelProvider is required for ofType function".to_string(),
                })?;

        // Filter items by type using ModelProvider
        for item in items {
            if self
                .matches_type_with_provider(item, type_name, model_provider)
                .await
            {
                results.push((*item).clone());
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

impl OfTypeFunction {
    /// Handle namespaced type (e.g., System.String, FHIR.Patient)
    async fn handle_namespaced_type(
        &self,
        type_arg: &FhirPathValue,
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if let FhirPathValue::TypeInfoObject { namespace, name } = type_arg {
            let full_type_name = format!("{namespace}.{name}");

            // Get the collection to filter
            let items = match &context.input {
                FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
                FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])),
                single => vec![single],
            };

            let mut results = Vec::new();

            let model_provider =
                context
                    .model_provider
                    .as_ref()
                    .ok_or_else(|| FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "ModelProvider is required for namespaced type matching"
                            .to_string(),
                    })?;

            // Filter items by the full namespaced type
            for item in items {
                if self
                    .matches_type_with_provider(item, &full_type_name, model_provider)
                    .await
                {
                    results.push((*item).clone());
                }
            }

            Ok(FhirPathValue::collection(results))
        } else {
            Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Expected TypeInfoObject for namespaced type handling".to_string(),
            })
        }
    }
    /// Check if a value matches the specified type name using ModelProvider
    async fn matches_type_with_provider(
        &self,
        value: &FhirPathValue,
        type_name: &str,
        model_provider: &Arc<dyn ModelProvider>,
    ) -> bool {
        // First try the basic type matching
        if self.matches_type(value, type_name) {
            return true;
        }

        // For more sophisticated type checking, use the ModelProvider
        match value {
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.resource_type() {
                    // Check if the resource type is compatible with the requested type (includes inheritance)
                    model_provider
                        .is_type_compatible(resource_type, type_name)
                        .await
                } else {
                    false
                }
            }
            _ => {
                // For non-resource types, try to get type information and check subtypes
                // This is a placeholder for more advanced type checking
                false
            }
        }
    }

    /// Check if a value matches the specified type name (basic matching)
    fn matches_type(&self, value: &FhirPathValue, type_name: &str) -> bool {
        match value {
            FhirPathValue::Boolean(_) => {
                matches!(
                    type_name,
                    "Boolean" | "System.Boolean" | "boolean" | "FHIR.boolean"
                )
            }
            FhirPathValue::Integer(_) => {
                matches!(
                    type_name,
                    "Integer" | "System.Integer" | "integer" | "FHIR.integer"
                )
            }
            FhirPathValue::Decimal(_) => {
                matches!(
                    type_name,
                    "Decimal" | "System.Decimal" | "decimal" | "FHIR.decimal"
                )
            }
            FhirPathValue::String(_) => {
                matches!(
                    type_name,
                    "String"
                        | "System.String"
                        | "string"
                        | "FHIR.string"
                        | "uri"
                        | "FHIR.uri"
                        | "uuid"
                        | "FHIR.uuid"
                        | "code"
                        | "FHIR.code"
                        | "id"
                        | "FHIR.id"
                )
            }
            FhirPathValue::Date(_) => {
                matches!(type_name, "Date" | "System.Date" | "date" | "FHIR.date")
            }
            FhirPathValue::DateTime(_) => {
                matches!(
                    type_name,
                    "DateTime" | "System.DateTime" | "dateTime" | "FHIR.dateTime"
                )
            }
            FhirPathValue::Time(_) => {
                matches!(type_name, "Time" | "System.Time" | "time" | "FHIR.time")
            }
            FhirPathValue::Quantity { .. } => {
                matches!(type_name, "Quantity" | "System.Quantity" | "FHIR.Quantity")
            }
            FhirPathValue::Resource(resource) => {
                // Check FHIR resource type - support both with and without FHIR prefix
                if let Some(resource_type) = resource.resource_type() {
                    resource_type == type_name
                        || type_name == format!("FHIR.{resource_type}")
                        || type_name == format!("FHIR.`{resource_type}`")
                        // Handle case-insensitive matching for common FHIR resources
                        || resource_type.to_lowercase() == type_name.to_lowercase()
                } else {
                    false
                }
            }
            FhirPathValue::Collection(_) => {
                matches!(type_name, "Collection")
            }
            FhirPathValue::TypeInfoObject { .. } => {
                matches!(type_name, "TypeInfo" | "System.TypeInfo")
            }
            FhirPathValue::JsonValue(_) => {
                matches!(type_name, "JsonValue" | "Object" | "Any")
            }
            FhirPathValue::Empty => false,
        }
    }
}
