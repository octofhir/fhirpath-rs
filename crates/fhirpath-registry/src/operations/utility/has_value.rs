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

//! HasValue function implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity, FhirPathType
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// HasValue function - returns true if the input collection contains exactly one item that has a value
#[derive(Debug, Clone)]
pub struct HasValueFunction;

impl HasValueFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("hasValue", OperationType::Function)
            .description("Returns true if the input collection contains exactly one item that has a value (i.e., is not empty)")
            .example("Patient.name.hasValue()")
            .example("'hello'.hasValue()")
            .example("{}.hasValue()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn item_has_value(&self, item: &FhirPathValue) -> bool {
        match item {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => !items.is_empty(),
            FhirPathValue::String(s) => !s.is_empty(),
            FhirPathValue::JsonValue(json) => {
                match json.as_json() {
                    serde_json::Value::Object(obj) => !obj.is_empty(),
                    serde_json::Value::Array(arr) => !arr.is_empty(),
                    serde_json::Value::String(s) => !s.is_empty(),
                    serde_json::Value::Null => false,
                    _ => true,
                }
            },
            // All other value types are considered to have value if they exist
            _ => true,
        }
    }
}

#[async_trait]
impl FhirPathOperation for HasValueFunction {
    fn identifier(&self) -> &str {
        "hasValue"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            HasValueFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            });
        }

        let input = &context.input;

        let has_value = match input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item that is not empty/null
                items.len() == 1 && self.item_has_value(items.get(0).unwrap())
            }
            _ => {
                // Single item - check if it has a value
                self.item_has_value(input)
            }
        };

        Ok(FhirPathValue::Boolean(has_value))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            }));
        }

        let input = &context.input;

        let has_value = match input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item that is not empty/null
                items.len() == 1 && self.item_has_value(items.get(0).unwrap())
            }
            _ => {
                // Single item - check if it has a value
                self.item_has_value(input)
            }
        };

        Some(Ok(FhirPathValue::Boolean(has_value)))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_has_value_string() -> Result<()> {
        let function = HasValueFunction::new();

        // Non-empty string has value
        let context = EvaluationContext::new(FhirPathValue::String("hello".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Empty string has no value
        let context = EvaluationContext::new(FhirPathValue::String("".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(!b);
            }
            _ => panic!("Expected Boolean value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_has_value_collection() -> Result<()> {
        let function = HasValueFunction::new();

        // Single item collection with value
        let context = EvaluationContext::new(FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into())
        ]));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Single item collection with empty value
        let context = EvaluationContext::new(FhirPathValue::Collection(vec![
            FhirPathValue::String("".into())
        ]));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(!b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Multiple item collection (should be false per spec)
        let context = EvaluationContext::new(FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into())
        ]));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(!b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Empty collection has no value
        let context = EvaluationContext::new(FhirPathValue::Collection(Collection::new()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(!b);
            }
            _ => panic!("Expected Boolean value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_has_value_empty() -> Result<()> {
        let function = HasValueFunction::new();

        // Empty value has no value
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(!b);
            }
            _ => panic!("Expected Boolean value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_has_value_object() -> Result<()> {
        let function = HasValueFunction::new();

        // Non-empty object has value
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), FhirPathValue::String("value".into()));
        let context = EvaluationContext::new(FhirPathValue::JsonValue(serde_json::Value::Object(obj)));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Empty object has no value
        let context = EvaluationContext::new(FhirPathValue::JsonValue(serde_json::Value::Object(HashMap::new())));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(!b);
            }
            _ => panic!("Expected Boolean value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_has_value_primitives() -> Result<()> {
        let function = HasValueFunction::new();

        // Number has value
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Boolean has value
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        // Date has value
        let context = EvaluationContext::new(FhirPathValue::Date("2023-06-15".to_string()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        Ok(())
    }

    #[test]
    fn test_has_value_sync() -> Result<()> {
        let function = HasValueFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("hello".into()));
        let result = function.try_evaluate_sync(&[], &context)
            .unwrap()?;

        match result {
            FhirPathValue::Boolean(b) => {
                assert!(b);
            }
            _ => panic!("Expected Boolean value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_has_value_invalid_args() -> () {
        let function = HasValueFunction::new();
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));

        let result = function.evaluate(&[FhirPathValue::String("invalid".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = result {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }
}
