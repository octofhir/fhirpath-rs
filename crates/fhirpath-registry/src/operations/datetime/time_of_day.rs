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

//! TimeOfDay function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::{Local, NaiveTime, Timelike};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// TimeOfDay function - returns the current time
#[derive(Debug, Clone)]
pub struct TimeOfDayFunction;

impl Default for TimeOfDayFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeOfDayFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("timeOfDay", OperationType::Function)
            .description("Returns the current time")
            .example("timeOfDay()")
            .returns(TypeConstraint::Specific(FhirPathType::Time))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TimeOfDayFunction {
    fn identifier(&self) -> &str {
        "timeOfDay"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(TimeOfDayFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        // Get current local time
        let now = Local::now();

        // Create NaiveTime from current time
        let naive_time = NaiveTime::from_hms_milli_opt(
            now.hour(),
            now.minute(),
            now.second(),
            now.nanosecond() / 1_000_000,
        )
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        Ok(FhirPathValue::Time(
            octofhir_fhirpath_model::PrecisionTime::new(
                naive_time,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond, // Full precision for timeOfDay()
            ),
        ))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            }));
        }

        // Get current local time
        let now = Local::now();

        // Create NaiveTime from current time
        let naive_time = NaiveTime::from_hms_milli_opt(
            now.hour(),
            now.minute(),
            now.second(),
            now.nanosecond() / 1_000_000,
        )
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        Some(Ok(FhirPathValue::Time(
            octofhir_fhirpath_model::PrecisionTime::new(
                naive_time,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond, // Full precision for timeOfDay()
            ),
        )))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
