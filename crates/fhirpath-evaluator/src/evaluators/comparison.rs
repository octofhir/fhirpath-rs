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

//! Comparison operations evaluator

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_registry::{
    FhirPathRegistry,
    operations::EvaluationContext as RegistryEvaluationContext,
};
use std::sync::Arc;

/// Specialized evaluator for comparison operations
pub struct ComparisonEvaluator;

impl ComparisonEvaluator {
    /// Helper method to evaluate comparison operations via registry
    async fn evaluate_comparison_operation(
        symbol: &str,
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation(symbol).await {
            operation.evaluate(&[left.clone(), right.clone()], context).await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Comparison operation '{symbol}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Comparison operation '{symbol}' not found in registry"),
            })
        }
    }
    
    /// Evaluate equals operation
    pub async fn evaluate_equals(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation("=", left, right, registry, context).await
    }
    
    /// Evaluate not equals operation
    pub async fn evaluate_not_equals(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation("!=", left, right, registry, context).await
    }
    
    /// Evaluate less than operation
    pub async fn evaluate_less_than(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation("<", left, right, registry, context).await
    }
    
    /// Evaluate less than or equal operation
    pub async fn evaluate_less_than_or_equal(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation("<=", left, right, registry, context).await
    }
    
    /// Evaluate greater than operation
    pub async fn evaluate_greater_than(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation(">", left, right, registry, context).await
    }
    
    /// Evaluate greater than or equal operation
    pub async fn evaluate_greater_than_or_equal(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation(">=", left, right, registry, context).await
    }
    
    /// Evaluate equivalence operation
    pub async fn evaluate_equivalent(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation("~", left, right, registry, context).await
    }
    
    /// Evaluate not equivalent operation
    pub async fn evaluate_not_equivalent(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_comparison_operation("!~", left, right, registry, context).await
    }
}