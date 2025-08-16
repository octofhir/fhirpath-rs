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

//! Length function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Length function: returns the length of a string
pub struct LengthFunction;

impl Default for LengthFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl LengthFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("length", OperationType::Function)
            .description("Returns the number of characters in a string")
            .example("Patient.name.given.first().length()")
            .example("'hello world'.length()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LengthFunction {
    fn identifier(&self) -> &str {
        "length"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(LengthFunction::create_metadata);
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

        self.evaluate_length(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_length(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl LengthFunction {
    fn evaluate_length(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                message: "length() takes no arguments".to_string(),
            });
        }

        // Handle collection inputs
        let input = &context.input;

        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Empty);
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                self.process_single_value(value)
            }
            _ => {
                // Process as single value
                self.process_single_value(input)
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                let length = s.chars().count() as i64;
                Ok(FhirPathValue::Integer(length))
            }
            FhirPathValue::Collection(items) => {
                let length = items.len() as i64;
                Ok(FhirPathValue::Integer(length))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // For other types, try to convert to string first
                if let Some(string_val) = value.to_string_value() {
                    let length = string_val.chars().count() as i64;
                    Ok(FhirPathValue::Integer(length))
                } else {
                    Err(FhirPathError::EvaluationError {
                        message: "length() can only be called on string values or collections"
                            .to_string(),
                    })
                }
            }
        }
    }
}
