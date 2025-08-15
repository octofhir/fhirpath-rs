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

//! Integer conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ConvertsToInteger function: returns true if the input can be converted to Integer
pub struct ConvertsToIntegerFunction;

impl ConvertsToIntegerFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("convertsToInteger", OperationType::Function)
            .description("Returns true if the input can be converted to Integer")
            .example("'1'.convertsToInteger()")
            .example("true.convertsToInteger()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn can_convert_to_integer(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Integer(_) => Ok(true),
            FhirPathValue::Boolean(_) => Ok(true),
            FhirPathValue::Decimal(d) => Ok(d.fract().is_zero()),
            FhirPathValue::String(s) => {
                // Try to parse as integer
                Ok(s.trim().parse::<i64>().is_ok())
            }
            FhirPathValue::Empty => Ok(true), // Empty collection returns empty result
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(true) // Empty collection returns empty result
                } else if c.len() == 1 {
                    Self::can_convert_to_integer(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message: "convertsToInteger() requires a single item, but collection has multiple items".to_string(),
                    })
                }
            }
            _ => Ok(false),
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConvertsToIntegerFunction {
    fn identifier(&self) -> &str {
        "convertsToInteger"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ConvertsToIntegerFunction::create_metadata());
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

        match Self::can_convert_to_integer(&context.input) {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match Self::can_convert_to_integer(&context.input) {
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
    async fn test_converts_to_integer() {
        let func = ConvertsToIntegerFunction::new();

        // Test with integer
        let ctx = create_test_context(FhirPathValue::Integer(42));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with boolean
        let ctx = create_test_context(FhirPathValue::Boolean(true));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with decimal that has no fractional part
        let ctx = create_test_context(FhirPathValue::Decimal(Decimal::from(5)));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with decimal that has fractional part
        let ctx = create_test_context(FhirPathValue::Decimal(Decimal::from_str_exact("5.5").unwrap()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with string that can be parsed as integer
        let ctx = create_test_context(FhirPathValue::String("123".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with string that cannot be parsed as integer
        let ctx = create_test_context(FhirPathValue::String("abc".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_converts_to_integer_sync() {
        let func = ConvertsToIntegerFunction::new();
        let ctx = create_test_context(FhirPathValue::Integer(42));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        assert!(func.supports_sync());
    }
}
