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

//! AllTrue function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// AllTrue function: returns true if all boolean items in the collection are true
pub struct AllTrueFunction;

impl Default for AllTrueFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AllTrueFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("allTrue", OperationType::Function)
            .description("Returns true if all items in the collection are true. Non-boolean items are treated as false. Returns true for an empty collection.")
            .example("Patient.active.allTrue()")
            .example("Bundle.entry.resource.active.allTrue()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for AllTrueFunction {
    fn identifier(&self) -> &str {
        "allTrue"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AllTrueFunction::create_metadata);
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

        // Fallback to async evaluation (though allTrue is always sync)
        self.evaluate_all_true(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_all_true(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AllTrueFunction {
    fn evaluate_all_true(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments {
                message: "allTrue() takes no arguments".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                // If empty collection, return true
                if items.is_empty() {
                    return Ok(FhirPathValue::Boolean(true));
                }

                // Check all items - non-boolean items are treated as false
                let all_true = items.iter().all(|item| match item {
                    FhirPathValue::Boolean(b) => *b,
                    _ => false, // Non-boolean values are treated as false
                });

                Ok(FhirPathValue::Boolean(all_true))
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => {
                // Non-boolean single item - return true (no boolean values to check)
                Ok(FhirPathValue::Boolean(true))
            }
        }
    }
}
