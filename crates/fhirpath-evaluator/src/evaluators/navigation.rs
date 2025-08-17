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
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_registry::{
    FhirPathRegistry, operations::EvaluationContext as RegistryEvaluationContext,
};
use serde_json::Value;
use std::sync::Arc;

/// Specialized evaluator for navigation and member access operations
pub struct NavigationEvaluator;

impl NavigationEvaluator {
    /// Evaluate member access with polymorphic FHIR support (async with boxing for recursion)
    pub fn evaluate_member_access<'a>(
        target: &'a FhirPathValue,
        member: &'a str,
        registry: &'a Arc<FhirPathRegistry>,
        context: &'a LocalEvaluationContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + Send + 'a>,
    > {
        Box::pin(async move {
            match target {
                FhirPathValue::JsonValue(json) => {
                    Self::evaluate_json_member_access(json, member).await
                }

                FhirPathValue::Collection(items) => {
                    let mut result_items = Vec::new();
                    for item in items.iter() {
                        let member_result =
                            Self::evaluate_member_access(item, member, registry, context).await?;
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
                    Ok(FhirPathValue::normalize_collection_result(result_items))
                }

                _ => Ok(FhirPathValue::Empty),
            }
        })
    }

    /// Evaluate member access on JSON values with polymorphic FHIR support
    async fn evaluate_json_member_access(
        json: &Value,
        member: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // Direct property access
        if let Some(value) = json.get(member) {
            return Ok(FhirPathValue::from(value.clone()));
        }

        // FHIR choice type polymorphic access
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key.starts_with(member) && key.len() > member.len() {
                    if let Some(next_char) = key.chars().nth(member.len()) {
                        if next_char.is_uppercase() {
                            return Ok(FhirPathValue::from(value.clone()));
                        }
                    }
                }
            }
        }

        Ok(FhirPathValue::Empty)
    }

    /// Evaluate children operation (get immediate child elements)
    pub async fn evaluate_children(
        target: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation("children").await {
            operation
                .evaluate(&[target.clone()], context)
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
        target: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation("descendants").await {
            operation
                .evaluate(&[target.clone()], context)
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
        target: &FhirPathValue,
        type_name: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation("ofType").await {
            operation
                .evaluate(&[target.clone(), type_name.clone()], context)
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
        target: &FhirPathValue,
        type_name: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation("is").await {
            operation
                .evaluate(&[target.clone(), type_name.clone()], context)
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
