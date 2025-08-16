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

/// ConvertsToDateTime function: returns true if the input can be converted to DateTime
pub struct ConvertsToDateTimeFunction;

impl Default for ConvertsToDateTimeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertsToDateTimeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToDateTime", OperationType::Function)
            .description("Returns true if the input can be converted to DateTime")
            .example("'2023-01-01T10:00:00Z'.convertsToDateTime()")
            .example("@2023-01-01T10:00:00Z.convertsToDateTime()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_datetime(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::DateTime(_) => Ok(true),
            FhirPathValue::Date(_) => Ok(true),
            FhirPathValue::String(s) => {
                // Try to parse as various datetime formats
                let s = s.as_ref();

                // Try the full RFC3339 format first
                if DateTime::parse_from_rfc3339(s).is_ok() {
                    return Ok(true);
                }

                // Try partial date/datetime formats that FHIRPath supports
                // Year only: "2015"
                if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(true);
                }

                // Year-month: "2015-02"
                if s.len() == 7
                    && s.matches('-').count() == 1
                    && chrono::NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d").is_ok()
                {
                    return Ok(true);
                }

                // Full date: "2015-02-04"
                if s.len() == 10
                    && s.matches('-').count() == 2
                    && chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
                {
                    return Ok(true);
                }

                // Date with hour: "2015-02-04T14"
                if s.len() == 13
                    && s.contains('T')
                    && chrono::NaiveDateTime::parse_from_str(
                        &format!("{s}:00:00"),
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .is_ok()
                {
                    return Ok(true);
                }

                // Date with hour and minute: "2015-02-04T14:34"
                if s.len() == 16
                    && s.contains('T')
                    && s.matches(':').count() == 1
                    && chrono::NaiveDateTime::parse_from_str(
                        &format!("{s}:00"),
                        "%Y-%m-%dT%H:%M:%S",
                    )
                    .is_ok()
                {
                    return Ok(true);
                }

                // Date with hour, minute, second: "2015-02-04T14:34:28"
                if s.len() == 19
                    && s.contains('T')
                    && s.matches(':').count() == 2
                    && chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").is_ok()
                {
                    return Ok(true);
                }

                // Date with milliseconds: "2015-02-04T14:34:28.123"
                if s.len() == 23
                    && s.contains('T')
                    && s.contains('.')
                    && chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f").is_ok()
                {
                    return Ok(true);
                }

                Ok(false)
            }
            FhirPathValue::Empty => Ok(true), // Empty collection returns true result
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns true result
                } else if c.len() == 1 {
                    Self::can_convert_to_datetime(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message: "convertsToDateTime() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToDateTimeFunction {
    fn identifier(&self) -> &str {
        "convertsToDateTime"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConvertsToDateTimeFunction::create_metadata);
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

        match Self::can_convert_to_datetime(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_datetime(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
