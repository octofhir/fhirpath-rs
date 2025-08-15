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

//! Join function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Join function: joins a collection of strings into a single string using the specified separator
pub struct JoinFunction;

impl JoinFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("join", OperationType::Function)
            .description("Joins a collection of strings into a single string using the specified separator")
            .example("('a' | 'b' | 'c').join(',')")
            .example("Patient.name.given.join(' ')")
            .parameter("separator", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for JoinFunction {
    fn identifier(&self) -> &str {
        "join"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            JoinFunction::create_metadata()
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

        self.evaluate_join(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_join(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl JoinFunction {
    fn evaluate_join(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "join() requires exactly one argument (separator)".to_string(),
            });
        }

        // Extract and convert separator parameter to string (handle collections)
        let separator = self.extract_string_from_value(&args[0])?;
        if separator.is_none() {
            return Err(FhirPathError::EvaluationError {
                message: "join() separator parameter must be a string".to_string(),
            });
        }
        let separator = separator.unwrap();

        // Get input collection - always convert input to collection for consistent handling
        let collection = match &context.input {
            FhirPathValue::Collection(items) => items.clone(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            // Single item becomes a single-item collection
            single => vec![single.clone()].into(),
        };

        // Convert all items to strings and join
        let string_items: Result<Vec<String>> = collection
            .iter()
            .map(|item| match item {
                FhirPathValue::String(s) => Ok(s.as_ref().to_string()),
                FhirPathValue::Integer(i) => Ok(i.to_string()),
                FhirPathValue::Decimal(d) => Ok(d.to_string()),
                FhirPathValue::Boolean(b) => Ok(b.to_string()),
                FhirPathValue::DateTime(dt) => Ok(dt.to_string()),
                FhirPathValue::Date(d) => Ok(d.to_string()),
                FhirPathValue::Time(t) => Ok(t.to_string()),
                FhirPathValue::Empty => Ok("".to_string()),
                _ => Err(FhirPathError::EvaluationError {
                    message: format!("join() cannot convert {:?} to string", item),
                })
            })
            .collect();

        let strings = string_items?;
        
        // If collection is empty, return empty string
        if strings.is_empty() {
            return Ok(FhirPathValue::String("".into()));
        }

        let result = strings.join(&separator);
        Ok(FhirPathValue::String(result.into()))
    }

    /// Extract a string from a FhirPathValue, handling collections and type conversion
    fn extract_string_from_value(&self, value: &FhirPathValue) -> Result<Option<String>> {
        match value {
            FhirPathValue::String(s) => Ok(Some(s.as_ref().to_string())),
            FhirPathValue::Integer(i) => Ok(Some(i.to_string())),
            FhirPathValue::Decimal(d) => Ok(Some(d.to_string())),
            FhirPathValue::Boolean(b) => Ok(Some(b.to_string())),
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(None)
                } else if items.len() == 1 {
                    // Single element collection - recursively extract
                    self.extract_string_from_value(items.first().unwrap())
                } else {
                    // Multiple elements - can't convert
                    Ok(None)
                }
            }
            _ => Ok(None), // Other types can't be converted
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
    async fn test_join_function() {
        let join_fn = JoinFunction::new();

        // Test basic join with comma
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("a,b,c".into()));

        // Test join with space
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
            FhirPathValue::String("test".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(" ".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world test".into()));

        // Test join with empty separator
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("abc".into()));

        // Test join single item
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("single".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("single".into()));

        // Test join empty collection
        let collection = FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]));
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));

        // Test join with mixed types
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(" ".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello 42 true".into()));

        // Test join with empty strings
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("a,,b".into()));

        // Test join with multi-character separator
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
            FhirPathValue::String("test".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(" :: ".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello :: world :: test".into()));

        // Test join with unicode characters
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("héllo".into()),
            FhirPathValue::String("wörld".into()),
            FhirPathValue::String("世界".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("•".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("héllo•wörld•世界".into()));

        // Test with single item (not a collection)
        let single = FhirPathValue::String("single".into());
        let context = create_test_context(single);
        let args = vec![FhirPathValue::String(",".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("single".into()));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String(",".into())];
        let result = join_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let join_fn = JoinFunction::new();
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("x".into()),
            FhirPathValue::String("y".into()),
            FhirPathValue::String("z".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("-".into())];

        let sync_result = join_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("x-y-z".into()));
        assert!(join_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let join_fn = JoinFunction::new();
        
        // Test with non-string separator argument
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("test".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::Integer(42)];
        let result = join_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("test".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![];
        let result = join_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}