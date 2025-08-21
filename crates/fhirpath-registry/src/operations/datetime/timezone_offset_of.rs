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

//! TimezoneOffsetOf function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::any::Any;

/// TimezoneOffsetOf function - extracts timezone offset in minutes from DateTime
#[derive(Debug, Clone)]
pub struct TimezoneOffsetOfFunction;

impl Default for TimezoneOffsetOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl TimezoneOffsetOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("timezoneOffsetOf", OperationType::Function)
            .description("Extract timezone offset in minutes from DateTime value")
            .example("@2023-05-15T14:30:30+05:00.timezoneOffsetOf() = 300")
            .example("Observation.issued.timezoneOffsetOf()")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TimezoneOffsetOfFunction {
    fn identifier(&self) -> &str {
        "timezoneOffsetOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(TimezoneOffsetOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "timezoneOffsetOf() takes no arguments",
            ));
        }

        let mut results = Vec::new();
        let collection = context.input.clone().to_collection();

        for value in collection.iter() {
            match value {
                FhirPathValue::DateTime(datetime) => {
                    // Get timezone offset in seconds and convert to minutes
                    let offset_seconds = datetime.datetime.offset().local_minus_utc();
                    let offset_minutes = offset_seconds / 60;
                    results.push(FhirPathValue::Decimal(rust_decimal::Decimal::from(
                        offset_minutes,
                    )));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        "timezoneOffsetOf() can only be called on DateTime values",
                    ));
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "timezoneOffsetOf() takes no arguments",
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
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_timezone_offset_of_utc() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:30Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = TimezoneOffsetOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(
                    values.get(0).unwrap(),
                    &FhirPathValue::Decimal(rust_decimal::Decimal::from(0))
                );
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_timezone_offset_of_plus_five() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:30+05:00").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = TimezoneOffsetOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(
                    values.get(0).unwrap(),
                    &FhirPathValue::Decimal(rust_decimal::Decimal::from(300))
                ); // +5 hours = 300 minutes
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_timezone_offset_of_minus_four_thirty() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:30-04:30").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = TimezoneOffsetOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(
                    values.get(0).unwrap(),
                    &FhirPathValue::Decimal(rust_decimal::Decimal::from(-270))
                ); // -4.5 hours = -270 minutes
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_timezone_offset_of_error_on_date() {
        let date = octofhir_fhirpath_model::PrecisionDate::new(
            chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = TimezoneOffsetOfFunction::new()
            .evaluate(&[], &context)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timezone_offset_of_error_on_non_temporal() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = TimezoneOffsetOfFunction::new()
            .evaluate(&[], &context)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timezone_offset_of_error_with_args() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:30Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let args = vec![FhirPathValue::String("test".into())];
        let result = TimezoneOffsetOfFunction::new()
            .evaluate(&args, &context)
            .await;
        assert!(result.is_err());
    }
}
