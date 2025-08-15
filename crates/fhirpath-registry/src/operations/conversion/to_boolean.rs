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

//! Boolean conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// ToBoolean function: converts input to Boolean
pub struct ToBooleanFunction;

impl ToBooleanFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toBoolean", OperationType::Function)
            .description("Converts input to Boolean")
            .example("'true'.toBoolean()")
            .example("1.toBoolean()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn convert_to_boolean(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            FhirPathValue::Integer(i) => {
                match i {
                    0 => Ok(FhirPathValue::Boolean(false)),
                    1 => Ok(FhirPathValue::Boolean(true)),
                    _ => Ok(FhirPathValue::Empty), // Cannot convert
                }
            }
            FhirPathValue::Decimal(d) => {
                if d.is_zero() {
                    Ok(FhirPathValue::Boolean(false))
                } else if *d == rust_decimal::Decimal::ONE {
                    Ok(FhirPathValue::Boolean(true))
                } else {
                    Ok(FhirPathValue::Empty) // Cannot convert
                }
            }
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => Ok(FhirPathValue::Boolean(true)),
                    "false" | "f" | "no" | "n" | "0" | "0.0" => Ok(FhirPathValue::Boolean(false)),
                    _ => Ok(FhirPathValue::Empty), // Cannot convert
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    Self::convert_to_boolean(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        message:
                            "toBoolean() requires a single item, but collection has multiple items"
                                .to_string(),
                    })
                }
            }
            _ => Ok(FhirPathValue::Empty), // Cannot convert
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToBooleanFunction {
    fn identifier(&self) -> &str {
        "toBoolean"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ToBooleanFunction::create_metadata());
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

        Self::convert_to_boolean(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_boolean(&context.input))
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
    async fn test_empty_to_boolean() {
        let func = ToBooleanFunction::new();
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_to_boolean() {
        let func = ToBooleanFunction::new();
        let ctx = create_test_context(FhirPathValue::Empty);

        // Test with boolean
        let ctx = create_test_context(FhirPathValue::Boolean(true));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with integer 0
        let ctx = create_test_context(FhirPathValue::Integer(0));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with integer 1
        let ctx = create_test_context(FhirPathValue::Integer(1));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with integer 2 (cannot convert)
        let ctx = create_test_context(FhirPathValue::Integer(2));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with string "true"
        let ctx = create_test_context(FhirPathValue::String("true".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with string "false"
        let ctx = create_test_context(FhirPathValue::String("false".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with string "invalid"
        let ctx = create_test_context(FhirPathValue::String("invalid".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_to_boolean_sync() {
        let func = ToBooleanFunction::new();
        let ctx = create_test_context(FhirPathValue::Boolean(false));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
        assert!(func.supports_sync());
    }

}
