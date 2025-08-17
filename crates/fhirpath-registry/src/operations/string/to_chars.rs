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

//! ToChars function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ToChars function: returns the list of characters in the input string
pub struct ToCharsFunction;

impl Default for ToCharsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToCharsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toChars", OperationType::Function)
            .description("Returns the list of characters in the input string")
            .example("'abc'.toChars()")
            .example("Patient.name.family.toChars()")
            .returns(TypeConstraint::Collection(Box::new(
                TypeConstraint::Specific(FhirPathType::String),
            )))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ToCharsFunction {
    fn identifier(&self) -> &str {
        "toChars"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToCharsFunction::create_metadata);
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

        self.evaluate_to_chars(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_to_chars(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ToCharsFunction {
    fn evaluate_to_chars(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "toChars() takes no arguments".to_string(),
            });
        }

        // Get input string from context - handle both single strings and collections with single strings
        match &context.input {
            FhirPathValue::String(s) => {
                // Convert each character to a FhirPathValue::String and collect into a collection
                let chars: Vec<FhirPathValue> = s
                    .as_ref()
                    .chars()
                    .map(|c| FhirPathValue::String(c.to_string().into()))
                    .collect();

                Ok(FhirPathValue::Collection(chars.into()))
            }
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.iter().next().unwrap() {
                    FhirPathValue::String(s) => {
                        // Convert each character to a FhirPathValue::String and collect into a collection
                        let chars: Vec<FhirPathValue> = s
                            .as_ref()
                            .chars()
                            .map(|c| FhirPathValue::String(c.to_string().into()))
                            .collect();

                        Ok(FhirPathValue::Collection(chars.into()))
                    }
                    _ => Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "toChars() requires input to be a string".to_string(),
                    }),
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "toChars() requires input to be a string".to_string(),
            }),
        }
    }
}
