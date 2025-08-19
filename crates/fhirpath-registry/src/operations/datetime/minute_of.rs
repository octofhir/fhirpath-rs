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

//! MinuteOf function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::any::Any;

/// MinuteOf function - extracts minute component from DateTime or Time (0-59)
#[derive(Debug, Clone)]
pub struct MinuteOfFunction;

impl Default for MinuteOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl MinuteOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("minuteOf", OperationType::Function)
            .description("Extract minute component (0-59) from DateTime or Time value")
            .example("@2023-05-15T14:30:00.minuteOf() = 30")
            .example("Observation.effectiveDateTime.minuteOf()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MinuteOfFunction {
    fn identifier(&self) -> &str {
        "minuteOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(MinuteOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "minuteOf() takes no arguments",
            ));
        }

        let mut results = Vec::new();

        for value in context.input.clone().to_collection().iter() {
            match value {
                FhirPathValue::DateTime(datetime) => {
                    results.push(FhirPathValue::Integer(datetime.datetime.minute() as i64));
                }
                FhirPathValue::Time(time) => {
                    results.push(FhirPathValue::Integer(time.time.minute() as i64));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        "minuteOf() can only be called on DateTime or Time values",
                    ));
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "minuteOf() takes no arguments",
            ));
        }
        Ok(())
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(futures::executor::block_on(self.evaluate(args, context)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use octofhir_fhirpath_model::{MockModelProvider, PrecisionTime as Time};
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_minute_of_datetime() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:00Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = MinuteOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(30));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_minute_of_time() {
        let time = Time::new(
            chrono::NaiveTime::from_hms_opt(14, 30, 45).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Second,
        );
        let context = create_test_context(FhirPathValue::Time(time));
        let result = MinuteOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(30));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_minute_of_error_on_non_temporal() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = MinuteOfFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }
}
