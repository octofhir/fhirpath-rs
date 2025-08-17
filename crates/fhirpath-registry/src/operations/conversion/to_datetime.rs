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

//! DateTime conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::DateTime;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ToDateTime function: converts input to DateTime
pub struct ToDateTimeFunction;

impl Default for ToDateTimeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToDateTimeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toDateTime", OperationType::Function)
            .description("Converts input to DateTime")
            .example("'2023-01-01T10:00:00Z'.toDateTime()")
            .example("@2023-01-01T10:00:00Z.toDateTime()")
            .returns(TypeConstraint::Specific(FhirPathType::DateTime))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn convert_to_datetime(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::DateTime(dt) => {
                // For existing datetime, convert to date-only format
                let date = dt.date_naive();
                Ok(FhirPathValue::Date(date))
            }
            FhirPathValue::Date(d) => {
                // Date already in correct format
                Ok(FhirPathValue::Date(*d))
            }
            FhirPathValue::String(s) => {
                // Try to parse as full datetime first
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    let date = dt.date_naive();
                    return Ok(FhirPathValue::Date(date));
                }

                // Try to parse as date-only format (YYYY-MM-DD)
                if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    return Ok(FhirPathValue::Date(date));
                }

                // Try to parse partial date formats
                if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d")
                {
                    return Ok(FhirPathValue::Date(date));
                }

                if let Ok(date) =
                    chrono::NaiveDate::parse_from_str(&format!("{s}-01-01"), "%Y-%m-%d")
                {
                    return Ok(FhirPathValue::Date(date));
                }

                Ok(FhirPathValue::Empty) // Cannot convert
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    Self::convert_to_datetime(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message:
                            "toDateTime() requires a single item, but collection has multiple items"
                                .to_string(),
                    })
                }
            }
            _ => Ok(FhirPathValue::Empty), // Cannot convert
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToDateTimeFunction {
    fn identifier(&self) -> &str {
        "toDateTime"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToDateTimeFunction::create_metadata);
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

        Self::convert_to_datetime(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_datetime(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
