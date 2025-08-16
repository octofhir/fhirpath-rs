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

//! Take function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Take function: returns a collection containing only the first num items
pub struct TakeFunction;

impl Default for TakeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl TakeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("take", OperationType::Function)
            .description("Returns a collection containing only the first num items in the input collection. If num is negative or zero, returns an empty collection. If num is greater than the collection length, returns the entire collection.")
            .example("Patient.name.take(2)")
            .example("Bundle.entry.take(10)")
            .parameter("num", TypeConstraint::Specific(FhirPathType::Integer), false)
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TakeFunction {
    fn identifier(&self) -> &str {
        "take"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(TakeFunction::create_metadata);
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

        // Fallback to async evaluation (though take is always sync)
        self.evaluate_take(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_take(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TakeFunction {
    fn evaluate_take(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "take() requires exactly one integer argument".to_string(),
            });
        }

        // Extract the take count - handle both single values and collections with single values
        let take_count = match &args[0] {
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
                    message: "take() argument must be an integer".to_string(),
                });
            }
        };

        // Handle negative or zero take count
        if take_count <= 0 {
            return Ok(FhirPathValue::collection(vec![]));
        }

        let take_count = take_count as usize;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if take_count >= items.len() {
                    Ok(context.input.clone())
                } else {
                    Ok(FhirPathValue::collection(
                        items.as_arc().as_ref()[..take_count].to_vec(),
                    ))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![])),
            _ => {
                // Single item - take it if take_count > 0, otherwise return empty
                if take_count > 0 {
                    Ok(FhirPathValue::collection(vec![context.input.clone()]))
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }
}
