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

//! Upper function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// Upper function: converts string to uppercase
pub struct UpperFunction;

impl Default for UpperFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl UpperFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("upper", OperationType::Function)
            .description("Returns the string with all characters converted to uppercase")
            .example("'hello world'.upper()")
            .example("Patient.name.family.upper()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for UpperFunction {
    fn identifier(&self) -> &str {
        "upper"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(UpperFunction::create_metadata);
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

        self.evaluate_upper(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_upper(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UpperFunction {
    fn evaluate_upper(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                message: "upper() takes no arguments".to_string(),
            });
        }

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
                let upper_str = s.as_ref().to_uppercase();
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::String(upper_str.into()),
                ])))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                message: "upper() requires input to be a string".to_string(),
            }),
        }
    }
}
