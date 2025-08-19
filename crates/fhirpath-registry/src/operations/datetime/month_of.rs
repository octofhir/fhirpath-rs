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

//! MonthOf function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::Datelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::any::Any;

/// MonthOf function - extracts month component from Date or DateTime (1-12)
#[derive(Debug, Clone)]
pub struct MonthOfFunction;

impl Default for MonthOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl MonthOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("monthOf", OperationType::Function)
            .description("Extract month component (1-12) from Date or DateTime value")
            .example("@2023-05-15.monthOf() = 5")
            .example("Observation.effectiveDateTime.monthOf()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MonthOfFunction {
    fn identifier(&self) -> &str {
        "monthOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(MonthOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "monthOf() takes no arguments",
            ));
        }

        let mut results = Vec::new();

        for value in context.input.clone().to_collection().iter() {
            match value {
                FhirPathValue::Date(date) => {
                    results.push(FhirPathValue::Integer(date.date.month() as i64));
                }
                FhirPathValue::DateTime(datetime) => {
                    results.push(FhirPathValue::Integer(datetime.datetime.month() as i64));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        "monthOf() can only be called on Date or DateTime values",
                    ));
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "monthOf() takes no arguments",
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
    use octofhir_fhirpath_model::{MockModelProvider, PrecisionDate as Date};
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_month_of_date() {
        let date = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = MonthOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(5));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_month_of_datetime() {
        let datetime = DateTime::parse_from_rfc3339("2023-12-25T14:30:00Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = MonthOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(12));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_month_of_january() {
        let date = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = MonthOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(1));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_month_of_december() {
        let date = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = MonthOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(12));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_month_of_collection() {
        let date1 = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let date2 = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let date3 = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let collection = FhirPathValue::Collection(
            vec![
                FhirPathValue::Date(date1),
                FhirPathValue::Date(date2),
                FhirPathValue::Date(date3),
            ]
            .into(),
        );
        let context = create_test_context(collection);
        let result = MonthOfFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(1));
                assert_eq!(values.get(1).unwrap(), &FhirPathValue::Integer(6));
                assert_eq!(values.get(2).unwrap(), &FhirPathValue::Integer(12));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_month_of_error_on_non_date() {
        let context = create_test_context(FhirPathValue::Integer(42));
        let result = MonthOfFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }
}
