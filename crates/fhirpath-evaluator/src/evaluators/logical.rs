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

//! Logical operations evaluator

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_registry::{
    FhirPathRegistry, operations::EvaluationContext as RegistryEvaluationContext,
};
use std::sync::Arc;

/// Specialized evaluator for logical operations
pub struct LogicalEvaluator;

impl LogicalEvaluator {
    /// Helper method to evaluate binary logical operations via registry
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
                    message: format!("Logical operation '{symbol}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Logical operation '{symbol}' not found in registry"),
            })
        }
    }

    /// Helper method to evaluate unary logical operations via registry
    async fn evaluate_unary_operation(
        symbol: &str,
        operand: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation(symbol).await {
            operation.evaluate(&[operand.clone()], context).await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Unary logical operation '{symbol}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unary logical operation '{symbol}' not found in registry"),
            })
        }
    }

    /// Evaluate logical AND operation
    pub async fn evaluate_and(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("and", left, right, registry, context).await
    }

    /// Evaluate logical OR operation
    pub async fn evaluate_or(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("or", left, right, registry, context).await
    }

    /// Evaluate logical XOR operation
    pub async fn evaluate_xor(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("xor", left, right, registry, context).await
    }

    /// Evaluate logical NOT operation
    pub async fn evaluate_not(
        operand: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_unary_operation("not", operand, registry, context).await
    }

    /// Evaluate implies operation
    pub async fn evaluate_implies(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("implies", left, right, registry, context).await
    }
}