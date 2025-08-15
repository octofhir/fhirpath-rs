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

//! Time conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::NaiveTime;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ConvertsToTime function: returns true if the input can be converted to Time
pub struct ConvertsToTimeFunction;

impl ConvertsToTimeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToTime", OperationType::Function)
            .description("Returns true if the input can be converted to Time")
            .example("'10:00:00'.convertsToTime()")
            .example("@T10:00:00.convertsToTime()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_time(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Time(_) => Ok(true),
            FhirPathValue::String(s) => {
                // Try to parse as time
                Ok(NaiveTime::parse_from_str(s, "%H:%M:%S").is_ok()
                    || NaiveTime::parse_from_str(s, "%H:%M:%S%.f").is_ok())
            }
            FhirPathValue::Empty => Ok(true), // Empty collection returns true result
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns true result
                } else if c.len() == 1 {
                    Self::can_convert_to_time(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message: "convertsToTime() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToTimeFunction {
    fn identifier(&self) -> &str {
        "convertsToTime"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ConvertsToTimeFunction::create_metadata());
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

        match Self::can_convert_to_time(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_time(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        };
        Some(result)
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
    async fn test_converts_to_time() {
        let func = ConvertsToTimeFunction::new();

        // Test with time
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let ctx = create_test_context(FhirPathValue::Time(time));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with string that can be parsed as time
        let ctx = create_test_context(FhirPathValue::String("10:00:00".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with string with fractional seconds that can be parsed as time
        let ctx = create_test_context(FhirPathValue::String("10:00:00.123".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with string that cannot be parsed as time
        let ctx = create_test_context(FhirPathValue::String("invalid-time".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_converts_to_time_sync() {
        let func = ConvertsToTimeFunction::new();
        let time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let ctx = create_test_context(FhirPathValue::Time(time));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        assert!(func.supports_sync());
    }
}
