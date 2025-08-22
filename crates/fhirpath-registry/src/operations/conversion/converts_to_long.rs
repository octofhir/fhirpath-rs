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

//! ConvertsToLong function implementation

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

/// ConvertsToLong function - checks if values can be converted to 64-bit integers
#[derive(Debug, Clone)]
pub struct ConvertsToLongFunction;

impl Default for ConvertsToLongFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConvertsToLongFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToLong", OperationType::Function)
            .description("Check if value can be converted to 64-bit integer (Long)")
            .example("42.convertsToLong() = true")
            .example("'12345'.convertsToLong() = true")
            .example("'hello'.convertsToLong() = false")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToLongFunction {
    fn identifier(&self) -> &str {
        "convertsToLong"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConvertsToLongFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "convertsToLong() takes no arguments".to_string(),
            });
        }

        let mut results = Vec::new();

        for value in context.input.clone().to_collection().iter() {
            let can_convert = match value {
                FhirPathValue::Integer(_) => true,
                FhirPathValue::String(s) => s.parse::<i64>().is_ok(),
                FhirPathValue::Decimal(d) => d.fract().is_zero(),
                _ => false,
            };

            results.push(FhirPathValue::Boolean(can_convert));
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "convertsToLong() takes no arguments".to_string(),
            });
        }
        Ok(())
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
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
    use octofhir_fhirpath_model::{MockModelProvider, PrecisionDate as Date};
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_converts_to_long_integer() {
        let context = create_test_context(FhirPathValue::Integer(42));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_large_integer() {
        let large_int = 9223372036854775807i64; // i64::MAX
        let context = create_test_context(FhirPathValue::Integer(large_int));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_negative_integer() {
        let context = create_test_context(FhirPathValue::Integer(-123));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_valid_string_number() {
        let context = create_test_context(FhirPathValue::String("12345".into()));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_string_negative() {
        let context = create_test_context(FhirPathValue::String("-999".into()));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_decimal_without_fraction() {
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::try_from(123.0).unwrap(),
        ));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_negative_decimal_without_fraction() {
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::try_from(-456.0).unwrap(),
        ));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_false_on_decimal_with_fraction() {
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::try_from(123.45).unwrap(),
        ));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(false));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_false_on_invalid_string() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(false));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_false_on_string_with_fraction() {
        let context = create_test_context(FhirPathValue::String("123.45".into()));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(false));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_false_on_boolean() {
        let context = create_test_context(FhirPathValue::Boolean(true));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(false));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_false_on_date() {
        let date = Date::new(
            chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            octofhir_fhirpath_model::TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(false));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_mixed_collection() {
        let collection = FhirPathValue::Collection(
            vec![
                FhirPathValue::Integer(42),
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("123".into()),
                FhirPathValue::Decimal(rust_decimal::Decimal::try_from(45.6).unwrap()),
                FhirPathValue::Decimal(rust_decimal::Decimal::try_from(78.0).unwrap()),
            ]
            .into(),
        );
        let context = create_test_context(collection);
        let result = ConvertsToLongFunction::new()
            .evaluate(&[], &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 5);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Boolean(true)); // 42
                assert_eq!(values.get(1).unwrap(), &FhirPathValue::Boolean(false)); // "hello"
                assert_eq!(values.get(2).unwrap(), &FhirPathValue::Boolean(true)); // "123"
                assert_eq!(values.get(3).unwrap(), &FhirPathValue::Boolean(false)); // 45.6
                assert_eq!(values.get(4).unwrap(), &FhirPathValue::Boolean(true)); // 78.0
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_converts_to_long_error_with_arguments() {
        let context = create_test_context(FhirPathValue::Integer(123));
        let args = vec![FhirPathValue::String("test".into())];
        let result = ConvertsToLongFunction::new()
            .evaluate(&args, &context)
            .await;
        assert!(result.is_err());
    }
}
