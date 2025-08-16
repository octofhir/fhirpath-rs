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

//! StartsWith function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// StartsWith function: returns true if the input string starts with the given prefix
pub struct StartsWithFunction;

impl Default for StartsWithFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl StartsWithFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("startsWith", OperationType::Function)
            .description("Returns true if the input string starts with the given prefix")
            .example("'hello world'.startsWith('hello')")
            .example("Patient.name.family.startsWith('Sm')")
            .parameter(
                "prefix",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for StartsWithFunction {
    fn identifier(&self) -> &str {
        "startsWith"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(StartsWithFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        self.evaluate_starts_with(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_starts_with(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl StartsWithFunction {
    fn evaluate_starts_with(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "startsWith() requires exactly one argument (prefix)".to_string(),
            });
        }

        // Get prefix parameter - handle both direct strings and collections containing strings
        let prefix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(s) => s.as_ref(),
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        message: "startsWith() prefix parameter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(FhirPathError::EvaluationError {
                    message: "startsWith() prefix parameter must be a string".to_string(),
                });
            }
        };

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                self.process_single_value(value, prefix)
            }
            _ => {
                // Process as single value
                self.process_single_value(input, prefix)
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, prefix: &str) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                let result = s.as_ref().starts_with(prefix);
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Boolean(result),
                ])))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "startsWith() requires input to be a string".to_string(),
            }),
        }
    }
}
