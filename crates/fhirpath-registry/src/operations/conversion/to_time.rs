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

//! Time conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::NaiveTime;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ToTime function: converts input to Time
pub struct ToTimeFunction;

impl Default for ToTimeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToTimeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toTime", OperationType::Function)
            .description("Converts input to Time")
            .example("'10:00:00'.toTime()")
            .example("@T10:00:00.toTime()")
            .returns(TypeConstraint::Specific(FhirPathType::Time))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn convert_to_time(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Time(t) => Ok(FhirPathValue::Time(*t)),
            FhirPathValue::String(s) => {
                // Try to parse as time
                match NaiveTime::parse_from_str(s, "%H:%M:%S") {
                    Ok(t) => Ok(FhirPathValue::Time(t)),
                    Err(_) => match NaiveTime::parse_from_str(s, "%H:%M:%S%.f") {
                        Ok(t) => Ok(FhirPathValue::Time(t)),
                        Err(_) => Ok(FhirPathValue::Empty), // Cannot convert
                    },
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    Self::convert_to_time(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message:
                            "toTime() requires a single item, but collection has multiple items"
                                .to_string(),
                    })
                }
            }
            _ => Ok(FhirPathValue::Empty), // Cannot convert
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToTimeFunction {
    fn identifier(&self) -> &str {
        "toTime"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToTimeFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(_args, context) {
            return result;
        }

        Self::convert_to_time(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_time(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
