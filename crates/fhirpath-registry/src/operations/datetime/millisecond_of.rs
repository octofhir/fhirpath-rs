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

//! MillisecondOf function implementation

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

/// MillisecondOf function - extracts millisecond component from DateTime or Time (0-999)
#[derive(Debug, Clone)]
pub struct MillisecondOfFunction;

impl Default for MillisecondOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl MillisecondOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("millisecondOf", OperationType::Function)
            .description("Extract millisecond component (0-999) from DateTime or Time value")
            .example("@2023-05-15T14:30:30.500.millisecondOf() = 500")
            .example("Observation.issued.millisecondOf()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MillisecondOfFunction {
    fn identifier(&self) -> &str {
        "millisecondOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(MillisecondOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "millisecondOf() takes no arguments",
            ));
        }

        let mut results = Vec::new();
        let collection = context.input.clone().to_collection();

        for value in collection.iter() {
            match value {
                FhirPathValue::DateTime(datetime) => {
                    let millisecond =
                        (datetime.datetime.timestamp_subsec_nanos() / 1_000_000) % 1000;
                    results.push(FhirPathValue::Integer(millisecond as i64));
                }
                FhirPathValue::Time(time) => {
                    let millisecond = (time.time.nanosecond() / 1_000_000) % 1000;
                    results.push(FhirPathValue::Integer(millisecond as i64));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        "millisecondOf() can only be called on DateTime or Time values",
                    ));
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "millisecondOf() takes no arguments",
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
    async fn test_millisecond_of_datetime_zero() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:30.000Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = MillisecondOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(0));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_millisecond_of_datetime_500() {
        let datetime = DateTime::parse_from_rfc3339("2023-05-15T14:30:30.500Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = MillisecondOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(500));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_millisecond_of_error_on_date() {
        let date = octofhir_fhirpath_model::PrecisionDate::new(
            chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = MillisecondOfFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_millisecond_of_error_on_non_temporal() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = MillisecondOfFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }
}
