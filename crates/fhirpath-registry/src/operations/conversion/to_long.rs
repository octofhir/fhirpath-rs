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

//! ToLong function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;
use std::any::Any;

/// ToLong function - converts values to 64-bit integers
#[derive(Debug, Clone)]
pub struct ToLongFunction;

impl Default for ToLongFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToLongFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toLong", OperationType::Function)
            .description("Convert value to 64-bit integer (Long)")
            .example("42.toLong()")
            .example("'12345'.toLong()")
            .example("123.0.toLong()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ToLongFunction {
    fn identifier(&self) -> &str {
        "toLong"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToLongFunction::create_metadata);
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
                message: "toLong() takes no arguments".to_string(),
            });
        }

        let mut results = Vec::new();

        for value in context.input.clone().to_collection().iter() {
            match value {
                FhirPathValue::Integer(i) => {
                    // Integer is already 64-bit in Rust
                    results.push(FhirPathValue::Integer(*i));
                }
                FhirPathValue::String(s) => match s.parse::<i64>() {
                    Ok(long_val) => results.push(FhirPathValue::Integer(long_val)),
                    Err(_) => {
                        return Err(FhirPathError::EvaluationError {
                            expression: None,
                            location: None,
                            message: format!("Cannot convert '{s}' to Long"),
                        });
                    }
                },
                FhirPathValue::Decimal(d) => {
                    if d.fract().is_zero() {
                        let long_val = d.trunc().to_i64().unwrap_or(0);
                        results.push(FhirPathValue::Integer(long_val));
                    } else {
                        return Err(FhirPathError::EvaluationError {
                            expression: None,
                            location: None,
                            message: "Cannot convert decimal with fractional part to Long"
                                .to_string(),
                        });
                    }
                }
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "Cannot convert value to Long".to_string(),
                    });
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "toLong() takes no arguments".to_string(),
            });
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
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_to_long_integer() {
        let context = create_test_context(FhirPathValue::Integer(42));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(42));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_large_integer() {
        let large_int = 9223372036854775807i64; // i64::MAX
        let context = create_test_context(FhirPathValue::Integer(large_int));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(large_int));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_negative_integer() {
        let context = create_test_context(FhirPathValue::Integer(-123));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(-123));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_string_number() {
        let context = create_test_context(FhirPathValue::String("12345".into()));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(12345));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_string_negative() {
        let context = create_test_context(FhirPathValue::String("-999".into()));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(-999));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_decimal_without_fraction() {
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::try_from(123.0).unwrap(),
        ));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(123));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_negative_decimal_without_fraction() {
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::try_from(-456.0).unwrap(),
        ));
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(-456));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_collection() {
        let collection = FhirPathValue::Collection(
            vec![
                FhirPathValue::Integer(42),
                FhirPathValue::String("123".into()),
                FhirPathValue::Decimal(rust_decimal::Decimal::try_from(456.0).unwrap()),
            ]
            .into(),
        );
        let context = create_test_context(collection);
        let result = ToLongFunction::new().evaluate(&[], &context).await.unwrap();

        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 3);
                assert_eq!(values.get(0).unwrap(), &FhirPathValue::Integer(42));
                assert_eq!(values.get(1).unwrap(), &FhirPathValue::Integer(123));
                assert_eq!(values.get(2).unwrap(), &FhirPathValue::Integer(456));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_to_long_error_on_decimal_with_fraction() {
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::try_from(123.45).unwrap(),
        ));
        let result = ToLongFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_to_long_error_on_invalid_string() {
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = ToLongFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_to_long_error_on_string_with_fraction() {
        let context = create_test_context(FhirPathValue::String("123.45".into()));
        let result = ToLongFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_to_long_error_on_boolean() {
        let context = create_test_context(FhirPathValue::Boolean(true));
        let result = ToLongFunction::new().evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_to_long_error_with_arguments() {
        let context = create_test_context(FhirPathValue::Integer(123));
        let args = vec![FhirPathValue::String("test".into())];
        let result = ToLongFunction::new().evaluate(&args, &context).await;
        assert!(result.is_err());
    }
}
