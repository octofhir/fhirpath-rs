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

//! EndsWith function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// EndsWith function: returns true if the input string ends with the given suffix
pub struct EndsWithFunction;

impl Default for EndsWithFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl EndsWithFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("endsWith", OperationType::Function)
            .description("Returns true if the input string ends with the given suffix")
            .example("'hello world'.endsWith('world')")
            .example("Patient.name.family.endsWith('son')")
            .parameter(
                "suffix",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for EndsWithFunction {
    fn identifier(&self) -> &str {
        "endsWith"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(EndsWithFunction::create_metadata);
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

        self.evaluate_ends_with(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_ends_with(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl EndsWithFunction {
    fn evaluate_ends_with(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "endsWith() requires exactly one argument (suffix)".to_string(),
            });
        }

        // Get suffix parameter - handle both direct strings and collections containing strings
        let suffix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(s) => s.as_ref(),
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "endsWith() suffix parameter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "endsWith() suffix parameter must be a string".to_string(),
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
                self.process_single_value(value, suffix)
            }
            _ => {
                // Process as single value
                self.process_single_value(input, suffix)
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, suffix: &str) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                let result = s.as_ref().ends_with(suffix);
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Boolean(result),
                ])))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "endsWith() requires input to be a string".to_string(),
            }),
        }
    }
}
