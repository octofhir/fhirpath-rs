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

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::{Local, Timelike, NaiveTime};

/// TimeOfDay function - returns the current time
#[derive(Debug, Clone)]
pub struct TimeOfDayFunction;

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
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            TimeOfDayFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            });
        }

        // Get current local time
        let now = Local::now();

        // Create NaiveTime from current time
        let naive_time = NaiveTime::from_hms_milli_opt(
            now.hour(),
            now.minute(),
            now.second(),
            now.nanosecond() / 1_000_000
        ).unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        Ok(FhirPathValue::Time(naive_time))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            }));
        }

        // Get current local time
        let now = Local::now();

        // Create NaiveTime from current time
        let naive_time = NaiveTime::from_hms_milli_opt(
            now.hour(),
            now.minute(),
            now.second(),
            now.nanosecond() / 1_000_000
        ).unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        Some(Ok(FhirPathValue::Time(naive_time)))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_time_of_day_function() -> Result<()> {
        let function = TimeOfDayFunction::new();
        let registry = std::sync::Arc::new(crate::FhirPathRegistry::new());
        let model_provider = std::sync::Arc::new(octofhir_fhirpath_model::MockModelProvider::new());
        let context = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Time(naive_time) => {
                // Just validate that we got a valid NaiveTime
                assert!(naive_time.hour() < 24);
                assert!(naive_time.minute() < 60);
                assert!(naive_time.second() < 60);
            }
            _ => panic!("Expected Time value"),
        }

        Ok(())
    }

    #[test]
    fn test_time_of_day_sync() -> Result<()> {
        let function = TimeOfDayFunction::new();
        let registry = std::sync::Arc::new(crate::FhirPathRegistry::new());
        let model_provider = std::sync::Arc::new(octofhir_fhirpath_model::MockModelProvider::new());
        let context = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        let result = function.try_evaluate_sync(&[], &context)
            .unwrap()?;

        match result {
            FhirPathValue::Time(naive_time) => {
                // Just validate that we got a valid NaiveTime
                assert!(naive_time.hour() < 24);
                assert!(naive_time.minute() < 60);
                assert!(naive_time.second() < 60);
            }
            _ => panic!("Expected Time value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_time_of_day_invalid_args() -> () {
        let function = TimeOfDayFunction::new();
        let registry = std::sync::Arc::new(crate::FhirPathRegistry::new());
        let model_provider = std::sync::Arc::new(octofhir_fhirpath_model::MockModelProvider::new());
        let context = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        let result = function.evaluate(&[FhirPathValue::String("invalid".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = result {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }
}
