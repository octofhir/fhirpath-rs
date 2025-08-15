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

//! StartsWith function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// StartsWith function: returns true if the input string starts with the given prefix
pub struct StartsWithFunction;

impl StartsWithFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("startsWith", OperationType::Function)
            .description("Returns true if the input string starts with the given prefix")
            .example("'hello world'.startsWith('hello')")
            .example("Patient.name.family.startsWith('Sm')")
            .parameter("prefix", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for StartsWithFunction {
    fn identifier(&self) -> &str {
        "startsWith"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            StartsWithFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        self.evaluate_starts_with(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_starts_with(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl StartsWithFunction {
    fn evaluate_starts_with(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "startsWith() requires exactly one argument (prefix)".to_string(),
            });
        }

        // Get prefix parameter - handle both direct strings and collections containing strings
        let prefix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.first().unwrap() {
                    FhirPathValue::String(s) => s.as_ref(),
                    _ => return Err(FhirPathError::EvaluationError {
                        message: "startsWith() prefix parameter must be a string".to_string(),
                    }),
                }
            },
            _ => return Err(FhirPathError::EvaluationError {
                message: "startsWith() prefix parameter must be a string".to_string(),
            }),
        };

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(Collection::from(vec![])));
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                return self.process_single_value(value, prefix);
            }
            _ => {
                // Process as single value
                return self.process_single_value(input, prefix);
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, prefix: &str) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                let result = s.as_ref().starts_with(prefix);
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Boolean(result)
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "startsWith() requires input to be a string".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use crate::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_starts_with_function() {
        let starts_with_fn = StartsWithFunction::new();

        // Test positive case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test negative case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("world".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty prefix (always true)
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test exact match
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test prefix longer than string
        let string = FhirPathValue::String("hi".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with unicode characters
        let string = FhirPathValue::String("héllo 世界".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("héllo".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test case sensitivity
        let string = FhirPathValue::String("Hello World".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String("test".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let starts_with_fn = StartsWithFunction::new();
        let string = FhirPathValue::String("test string".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("test".into())];

        let sync_result = starts_with_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(starts_with_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let starts_with_fn = StartsWithFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::String("test".into())];
        let result = starts_with_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(42)];
        let result = starts_with_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![];
        let result = starts_with_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}