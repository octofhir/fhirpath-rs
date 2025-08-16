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

//! AllFalse function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// AllFalse function: returns true if all boolean items in the collection are false
pub struct AllFalseFunction;

impl Default for AllFalseFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AllFalseFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("allFalse", OperationType::Function)
            .description("Returns true if all boolean items in the collection are false. Returns true for an empty collection. Non-boolean items are ignored.")
            .example("Patient.active.allFalse()")
            .example("Bundle.entry.resource.active.allFalse()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for AllFalseFunction {
    fn identifier(&self) -> &str {
        "allFalse"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AllFalseFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync path first for performance
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        // Fallback to async evaluation (though allFalse is always sync)
        self.evaluate_all_false(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_all_false(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AllFalseFunction {
    fn evaluate_all_false(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments {
                message: "allFalse() takes no arguments".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Filter to only boolean items
                let boolean_items: Vec<bool> = items
                    .iter()
                    .filter_map(|item| match item {
                        FhirPathValue::Boolean(b) => Some(*b),
                        _ => None,
                    })
                    .collect();

                // If no boolean items, return true (empty collection logic)
                if boolean_items.is_empty() {
                    Ok(FhirPathValue::Boolean(true))
                } else {
                    // All must be false
                    Ok(FhirPathValue::Boolean(boolean_items.iter().all(|&b| !b)))
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => {
                // Non-boolean single item - return true (no boolean values to check)
                Ok(FhirPathValue::Boolean(true))
            }
        }
    }
}
