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

//! Collection operations evaluator

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_registry::{
    FhirPathRegistry,
    operations::EvaluationContext as RegistryEvaluationContext,
};
use std::sync::Arc;

/// Specialized evaluator for collection operations
pub struct CollectionEvaluator;

impl CollectionEvaluator {
    /// Helper method to evaluate binary collection operations via registry
    async fn evaluate_binary_operation(
        symbol: &str,
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation(symbol).await {
            operation.evaluate(&[left.clone(), right.clone()], context).await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Collection operation '{symbol}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Collection operation '{symbol}' not found in registry"),
            })
        }
    }

    /// Helper method to evaluate unary collection operations via registry
    async fn evaluate_unary_operation(
        name: &str,
        target: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation(name).await {
            operation.evaluate(&[target.clone()], context).await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Collection operation '{name}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Collection operation '{name}' not found in registry"),
            })
        }
    }
    
    /// Evaluate union operation
    pub async fn evaluate_union(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("|", left, right, registry, context).await
    }
    
    /// Evaluate contains operation
    pub async fn evaluate_contains(
        collection: &FhirPathValue,
        item: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("contains", collection, item, registry, context).await
    }
    
    /// Evaluate in operation
    pub async fn evaluate_in(
        item: &FhirPathValue,
        collection: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("in", item, collection, registry, context).await
    }
    
    /// Evaluate distinct operation
    pub async fn evaluate_distinct(
        collection: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_unary_operation("distinct", collection, registry, context).await
    }
    
    /// Evaluate intersect operation
    pub async fn evaluate_intersect(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation("intersect").await {
            operation.evaluate(&[left.clone(), right.clone()], context).await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Intersect operation error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "Intersect operation not found in registry".to_string(),
            })
        }
    }
}