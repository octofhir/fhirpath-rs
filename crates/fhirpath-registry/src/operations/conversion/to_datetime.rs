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
use chrono::{DateTime, FixedOffset};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ToDateTime function: converts input to DateTime
pub struct ToDateTimeFunction;

impl ToDateTimeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toDateTime", OperationType::Function)
            .description("Converts input to DateTime")
            .example("'2023-01-01T10:00:00Z'.toDateTime()")
            .example("@2023-01-01T10:00:00Z.toDateTime()")
            .returns(TypeConstraint::Specific(FhirPathType::DateTime))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn convert_to_datetime(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::DateTime(dt) => Ok(FhirPathValue::DateTime(*dt)),
            FhirPathValue::Date(d) => {
                // Convert date to datetime at midnight UTC
                let dt = d
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(FixedOffset::east_opt(0).unwrap())
                    .unwrap();
                Ok(FhirPathValue::DateTime(dt.fixed_offset()))
            }
            FhirPathValue::String(s) => {
                // Try to parse as datetime
                match DateTime::parse_from_rfc3339(s) {
                    Ok(dt) => Ok(FhirPathValue::DateTime(dt.fixed_offset())),
                    Err(_) => Ok(FhirPathValue::Empty), // Cannot convert
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    Self::convert_to_datetime(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message: "toDateTime() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }
            _ => Ok(FhirPathValue::Empty), // Cannot convert
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToDateTimeFunction {
    fn identifier(&self) -> &str {
        "toDateTime"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ToDateTimeFunction::create_metadata());
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

        Self::convert_to_datetime(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_datetime(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_to_datetime() {
        let func = ToDateTimeFunction::new();

        // Test with datetime
        let dt = DateTime::parse_from_rfc3339("2023-01-01T10:00:00Z").unwrap();
        let ctx = create_test_context(FhirPathValue::DateTime(dt));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::DateTime(dt));

        // Test with date
        let date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let expected_dt = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(FixedOffset::east_opt(0).unwrap())
            .unwrap()
            .fixed_offset();
        let ctx = create_test_context(FhirPathValue::Date(date));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::DateTime(expected_dt));

        // Test with string that can be parsed as datetime
        let ctx = create_test_context(FhirPathValue::String("2023-01-01T10:00:00Z".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::DateTime(dt));

        // Test with string that cannot be parsed as datetime
        let ctx = create_test_context(FhirPathValue::String("invalid-datetime".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with empty
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_to_datetime_sync() {
        let func = ToDateTimeFunction::new();
        let dt = DateTime::parse_from_rfc3339("2023-01-01T10:00:00Z").unwrap();
        let ctx = create_test_context(FhirPathValue::DateTime(dt));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::DateTime(dt));
        assert!(func.supports_sync());
    }
}
