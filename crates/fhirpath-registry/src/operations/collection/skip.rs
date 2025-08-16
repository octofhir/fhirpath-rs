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

//! Skip function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Skip function: returns a collection skipping the first num items
pub struct SkipFunction;

impl Default for SkipFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SkipFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("skip", OperationType::Function)
            .description("Returns a collection containing all but the first num items in the input collection. If num is negative or zero, returns the entire collection. If num is greater than the collection length, returns an empty collection.")
            .example("Patient.name.skip(1)")
            .example("Bundle.entry.skip(5)")
            .parameter("num", TypeConstraint::Specific(FhirPathType::Integer), false)
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SkipFunction {
    fn identifier(&self) -> &str {
        "skip"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(SkipFunction::create_metadata);
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

        // Fallback to async evaluation (though skip is always sync)
        self.evaluate_skip(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_skip(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SkipFunction {
    fn evaluate_skip(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "skip() requires exactly one integer argument".to_string(),
            });
        }
        // Extract the skip count
        let skip_count = match &args[0] {
            FhirPathValue::Integer(n) => *n,
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.iter().next().unwrap() {
                    FhirPathValue::Integer(n) => *n,
                    _ => {
                        return Err(FhirPathError::InvalidArguments {
                            message: "take() argument must be an integer".to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(FhirPathError::InvalidArguments {
                    message: "skip() argument must be an integer".to_string(),
                });
            }
        };

        // Handle negative or zero skip count
        if skip_count <= 0 {
            return Ok(context.input.clone());
        }

        let skip_count = skip_count as usize;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if skip_count >= items.len() {
                    Ok(FhirPathValue::collection(vec![]))
                } else {
                    Ok(FhirPathValue::collection(
                        items.as_arc().as_ref()[skip_count..].to_vec(),
                    ))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            _ => {
                // Single item - skip it if skip_count > 0, otherwise return it
                if skip_count > 0 {
                    Ok(FhirPathValue::collection(vec![]))
                } else {
                    Ok(context.input.clone())
                }
            }
        }
    }
}
