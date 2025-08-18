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

//! Join function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Join function: joins a collection of strings into a single string using the specified separator
pub struct JoinFunction;

impl Default for JoinFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl JoinFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("join", OperationType::Function)
            .description(
                "Joins a collection of strings into a single string using the specified separator",
            )
            .example("('a' | 'b' | 'c').join(',')")
            .example("Patient.name.given.join(' ')")
            .parameter(
                "separator",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for JoinFunction {
    fn identifier(&self) -> &str {
        "join"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(JoinFunction::create_metadata);
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

        self.evaluate_join(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_join(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl JoinFunction {
    fn evaluate_join(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "join() requires exactly one argument (separator)".to_string(),
            });
        }

        // Extract and convert separator parameter to string (handle collections)
        let separator = self.extract_string_from_value(&args[0])?;
        if separator.is_none() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "join() separator parameter must be a string".to_string(),
            });
        }
        let separator = separator.unwrap();

        // Get input collection - always convert input to collection for consistent handling
        let collection = match &context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            // Single item becomes a single-item collection
            single => vec![single.clone()].into(),
        };

        // Convert all items to strings and join
        let string_items: Result<Vec<String>> = collection
            .iter()
            .map(|item| match item {
                FhirPathValue::String(s) => Ok(s.as_ref().to_string()),
                FhirPathValue::Integer(i) => Ok(i.to_string()),
                FhirPathValue::Decimal(d) => Ok(d.to_string()),
                FhirPathValue::Boolean(b) => Ok(b.to_string()),
                FhirPathValue::DateTime(dt) => Ok(dt.to_string()),
                FhirPathValue::Date(d) => Ok(d.to_string()),
                FhirPathValue::Time(t) => Ok(t.to_string()),
                FhirPathValue::JsonValue(json_val) => {
                    // Convert JsonValue to string
                    match json_val.as_str() {
                        Some(s) => Ok(s.to_string()),
                        None => {
                            // For non-string JSON values, use JSON representation
                            Ok(json_val.as_inner().to_string())
                        }
                    }
                }
                FhirPathValue::Empty => Ok("".to_string()),
                _ => Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: format!("join() cannot convert {item:?} to string"),
                }),
            })
            .collect();

        let strings = string_items?;

        // If collection is empty, return empty string
        if strings.is_empty() {
            return Ok(FhirPathValue::String("".into()));
        }

        let result = strings.join(&separator);
        Ok(FhirPathValue::String(result.into()))
    }

    /// Extract a string from a FhirPathValue, handling collections and type conversion
    fn extract_string_from_value(&self, value: &FhirPathValue) -> Result<Option<String>> {
        match value {
            FhirPathValue::String(s) => Ok(Some(s.as_ref().to_string())),
            FhirPathValue::Integer(i) => Ok(Some(i.to_string())),
            FhirPathValue::Decimal(d) => Ok(Some(d.to_string())),
            FhirPathValue::Boolean(b) => Ok(Some(b.to_string())),
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(None)
                } else if items.len() == 1 {
                    // Single element collection - recursively extract
                    self.extract_string_from_value(items.first().unwrap())
                } else {
                    // Multiple elements - can't convert
                    Ok(None)
                }
            }
            _ => Ok(None), // Other types can't be converted
        }
    }
}
