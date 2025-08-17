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

//! IndexOf function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// IndexOf function: returns the 0-based index of the first position substring is found in the input string, or -1 if not found
pub struct IndexOfFunction;

impl Default for IndexOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("indexOf", OperationType::Function)
            .description("Returns the 0-based index of the first position substring is found in the input string, or -1 if it is not found")
            .example("'hello world'.indexOf('world')")
            .example("'abcdef'.indexOf('cd')")
            .parameter("substring", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for IndexOfFunction {
    fn identifier(&self) -> &str {
        "indexOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(IndexOfFunction::create_metadata);
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

        self.evaluate_index_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_index_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl IndexOfFunction {
    fn evaluate_index_of(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "indexOf() requires exactly one argument (substring)".to_string(),
            });
        }

        // Get substring parameter first - handle both direct strings and single-element collections
        let substring = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                match items.len() {
                    0 => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
                    1 => match items.first().unwrap() {
                        FhirPathValue::String(s) => s,
                        _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
                    },
                    _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))), // Multiple values not allowed
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => return Ok(FhirPathValue::Collection(Collection::from(vec![]))), // Non-string parameters result in empty
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
                self.process_single_value(value, substring)
            }
            _ => {
                // Process as single value
                self.process_single_value(input, substring)
            }
        }
    }

    fn process_single_value(
        &self,
        value: &FhirPathValue,
        substring: &str,
    ) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                // Handle empty substring (returns 0 per spec)
                if substring.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![
                        FhirPathValue::Integer(0),
                    ])));
                }

                // Find index
                let index = match s.find(substring) {
                    Some(idx) => idx as i64,
                    None => -1,
                };

                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Integer(index),
                ])))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Ok(FhirPathValue::Collection(Collection::from(vec![]))), // Non-string values result in empty
        }
    }
}
