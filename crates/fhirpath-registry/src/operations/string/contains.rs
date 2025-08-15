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

//! Contains function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;

/// Contains function: returns true if the input string contains the given substring
pub struct ContainsFunction;

impl ContainsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("contains", OperationType::Function)
            .description("Returns true if the input string contains the given substring OR if a collection contains an item")
            .example("'hello world'.contains('world')")
            .example("Patient.name.family.contains('John')")
            .example("{1, 2, 3} contains 2")
            .parameter("substring_or_item", TypeConstraint::Any, false)
            .parameter("item", TypeConstraint::Any, true) // Optional second parameter for collection containership
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ContainsFunction {
    fn identifier(&self) -> &str {
        "contains"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ContainsFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle both method call syntax (1 arg) and operator syntax (2 args)
        match args.len() {
            1 => {
                // Method call: 'string'.contains('substring')
                if let Some(result) = self.try_evaluate_sync(args, context) {
                    return result;
                }
                self.evaluate_contains(args, context)
            }
            2 => {
                // Binary operator: collection contains item
                self.evaluate_collection_contains(&args[0], &args[1])
            }
            _ => Err(FhirPathError::EvaluationError {
                message: format!("contains() expects 1 argument (substring) or 2 arguments (collection, item), got {}", args.len()),
            })
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        match args.len() {
            1 => Some(self.evaluate_contains(args, context)),
            2 => Some(self.evaluate_collection_contains(&args[0], &args[1])),
            _ => Some(Err(FhirPathError::EvaluationError {
                message: format!("contains() expects 1 argument (substring) or 2 arguments (collection, item), got {}", args.len()),
            }))
        }
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ContainsFunction {
    fn evaluate_contains(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "contains() requires exactly one argument (substring)".to_string(),
            });
        }

        // Get substring parameter - handle both direct strings and collections containing strings
        let substring = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.first().unwrap() {
                    FhirPathValue::String(s) => s.as_ref(),
                    _ => return Err(FhirPathError::EvaluationError {
                        message: "contains() substring parameter must be a string".to_string(),
                    }),
                }
            },
            _ => return Err(FhirPathError::EvaluationError {
                message: "contains() substring parameter must be a string".to_string(),
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
                return self.process_single_value(value, substring);
            }
            _ => {
                // Process as single value
                return self.process_single_value(input, substring);
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, substring: &str) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::String(s) => {
                let result = s.as_ref().contains(substring);
                Ok(FhirPathValue::Collection(Collection::from(vec![
                    FhirPathValue::Boolean(result)
                ])))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::EvaluationError {
                message: "contains() requires input to be a string".to_string(),
            }),
        }
    }

    /// Collection containership operation: returns true if left collection contains right operand
    fn evaluate_collection_contains(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        use crate::operations::comparison::equals::EqualsOperation;
        
        // Right operand must be single item
        let right_collection = right.clone().to_collection();
        if right_collection.len() != 1 {
            return Err(FhirPathError::InvalidArguments { message: 
                "Right operand of 'contains' must be a single item".to_string()
            });
        }

        let search_item = right_collection.get(0).unwrap();
        let search_collection = left.clone().to_collection();

        // If left-hand side is empty, result is false
        if search_collection.is_empty() {
            return Ok(FhirPathValue::Boolean(false));
        }

        // Search for the item using equality semantics
        for item in search_collection.iter() {
            if EqualsOperation::compare_equal(item, search_item)? {
                return Ok(FhirPathValue::Boolean(true));
            }
        }

        Ok(FhirPathValue::Boolean(false))
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
    async fn test_contains_function() {
        let contains_fn = ContainsFunction::new();

        // Test positive case - substring in middle
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("lo wo".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test positive case - substring at start
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test positive case - substring at end
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("world".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test negative case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("xyz".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty substring (always true)
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test exact match
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test substring longer than string
        let string = FhirPathValue::String("hi".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with unicode characters
        let string = FhirPathValue::String("héllo 世界 test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("世界".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test case sensitivity
        let string = FhirPathValue::String("Hello World".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String("test".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let contains_fn = ContainsFunction::new();
        let string = FhirPathValue::String("test string".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("string".into())];

        let sync_result = contains_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(contains_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let contains_fn = ContainsFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::String("test".into())];
        let result = contains_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(42)];
        let result = contains_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![];
        let result = contains_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}