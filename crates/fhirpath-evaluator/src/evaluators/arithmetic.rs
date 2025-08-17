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

//! Arithmetic operations evaluator

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_registry::{
    FhirPathRegistry,
    operations::EvaluationContext as RegistryEvaluationContext,
};
use std::sync::Arc;

/// Specialized evaluator for arithmetic operations
pub struct ArithmeticEvaluator;

impl ArithmeticEvaluator {
    /// Helper method to evaluate binary arithmetic operations via registry
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
                    message: format!("Arithmetic operation '{symbol}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Arithmetic operation '{symbol}' not found in registry"),
            })
        }
    }

    /// Helper method to evaluate unary arithmetic operations via registry
    async fn evaluate_unary_operation(
        symbol: &str,
        operand: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(operation) = registry.get_operation(symbol).await {
            operation.evaluate(&[operand.clone()], context).await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("Unary arithmetic operation '{symbol}' error: {e}"),
                })
        } else {
            Err(EvaluationError::InvalidOperation {
                message: format!("Unary arithmetic operation '{symbol}' not found in registry"),
            })
        }
    }
    
    /// Evaluate addition operation
    pub async fn evaluate_addition(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("+", left, right, registry, context).await
    }
    
    /// Evaluate subtraction operation
    pub async fn evaluate_subtraction(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("-", left, right, registry, context).await
    }
    
    /// Evaluate multiplication operation
    pub async fn evaluate_multiplication(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("*", left, right, registry, context).await
    }
    
    /// Evaluate division operation
    pub async fn evaluate_division(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("/", left, right, registry, context).await
    }
    
    /// Evaluate modulo operation
    pub async fn evaluate_modulo(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("mod", left, right, registry, context).await
    }
    
    /// Evaluate integer division operation
    pub async fn evaluate_integer_division(
        left: &FhirPathValue,
        right: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_binary_operation("div", left, right, registry, context).await
    }
    
    /// Evaluate unary plus operation
    pub async fn evaluate_unary_plus(
        operand: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_unary_operation("+", operand, registry, context).await
    }
    
    /// Evaluate unary minus operation
    pub async fn evaluate_unary_minus(
        operand: &FhirPathValue,
        registry: &Arc<FhirPathRegistry>,
        context: &RegistryEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        Self::evaluate_unary_operation("-", operand, registry, context).await
    }
}