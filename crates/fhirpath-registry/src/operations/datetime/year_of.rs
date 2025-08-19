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

//! YearOf function implementation

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

/// YearOf function - extracts year component from Date or DateTime
#[derive(Debug, Clone)]
pub struct YearOfFunction;

impl Default for YearOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl YearOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("yearOf", OperationType::Function)
            .description("Extract year component from Date or DateTime value")
            .example("@2023-05-15.yearOf() = 2023")
            .example("Patient.birthDate.yearOf()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for YearOfFunction {
    fn identifier(&self) -> &str {
        "yearOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(YearOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "yearOf() takes no arguments",
            ));
        }

        let mut results = Vec::new();

        for value in context.input.clone().to_collection().iter() {
            match value {
                FhirPathValue::Date(date) => {
                    results.push(FhirPathValue::Integer(date.date.year() as i64));
                }
                FhirPathValue::DateTime(datetime) => {
                    results.push(FhirPathValue::Integer(datetime.datetime.year() as i64));
                }
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        "yearOf() can only be called on Date or DateTime values",
                    ));
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                "yearOf() takes no arguments",
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
    async fn test_year_of_date() {
        let date = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = YearOfFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(2023));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_year_of_datetime() {
        let datetime = DateTime::parse_from_rfc3339("2023-12-25T14:30:00Z").unwrap();
        let context = create_test_context(FhirPathValue::DateTime(
            octofhir_fhirpath_model::PrecisionDateTime::new(
                datetime,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond,
            ),
        ));
        let result = YearOfFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(2023));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_year_of_collection() {
        let date1 = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let date2 = Date::new(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let collection = FhirPathValue::Collection(
            vec![FhirPathValue::Date(date1), FhirPathValue::Date(date2)].into(),
        );
        let context = create_test_context(collection);
        let result = YearOfFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(2023));
                assert_eq!(values.get(1).unwrap(), &FhirPathValue::Integer(2024));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_year_of_error_on_non_date() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = YearOfFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_year_of_error_with_arguments() {
        let date = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = YearOfFunction::new()
            .evaluate(&[FhirPathValue::String("arg".into())], &context)
            .await;
        assert!(result.is_err());
    }
}
