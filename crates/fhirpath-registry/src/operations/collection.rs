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

//! Essential collection function implementations for FHIRPath
//!
//! This module contains implementations of core collection functions:
//! count, empty, exists, first, last, single with both sync and async support.

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType
};
use crate::enhanced_metadata::PerformanceComplexity;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::function::EvaluationContext;

/// Count function: returns the number of items in a collection
pub struct CountFunction;

impl CountFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("count", OperationType::Function)
            .description("Returns the number of items in a collection")
            .example("Patient.name.count()")
            .example("Bundle.entry.count()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for CountFunction {
    fn identifier(&self) -> &str {
        "count"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            CountFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync path first for performance
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        // Fallback to async evaluation (though count is always sync)
        match &context.input {
            FhirPathValue::Collection(collection) => Ok(FhirPathValue::Integer(collection.len() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match &context.input {
            FhirPathValue::Collection(collection) => Ok(FhirPathValue::Integer(collection.len() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "count".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Empty function: returns true if collection is empty
pub struct EmptyFunction;

impl EmptyFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("empty", OperationType::Function)
            .description("Returns true if the input collection is empty")
            .example("Patient.name.empty()")
            .example("Bundle.entry.empty()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for EmptyFunction {
    fn identifier(&self) -> &str {
        "empty"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            EmptyFunction::create_metadata()
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

        let is_empty = match &context.input {
            FhirPathValue::Collection(collection) => collection.is_empty(),
            FhirPathValue::Empty => true,
            _ => false,
        };
        Ok(FhirPathValue::Boolean(is_empty))
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let is_empty = match &context.input {
            FhirPathValue::Collection(collection) => collection.is_empty(),
            FhirPathValue::Empty => true,
            _ => false,
        };
        Some(Ok(FhirPathValue::Boolean(is_empty)))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "empty".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Exists function: returns true if collection is not empty
pub struct ExistsFunction;

impl ExistsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("exists", OperationType::Function)
            .description("Returns true if the input collection is not empty")
            .example("Patient.name.exists()")
            .example("Bundle.entry.exists()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ExistsFunction {
    fn identifier(&self) -> &str {
        "exists"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ExistsFunction::create_metadata()
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

        let exists = match &context.input {
            FhirPathValue::Collection(collection) => !collection.is_empty(),
            FhirPathValue::Empty => false,
            _ => true,
        };
        Ok(FhirPathValue::Boolean(exists))
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let exists = match &context.input {
            FhirPathValue::Collection(collection) => !collection.is_empty(),
            FhirPathValue::Empty => false,
            _ => true,
        };
        Some(Ok(FhirPathValue::Boolean(exists)))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "exists".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// First function: returns the first item in a collection
pub struct FirstFunction;

impl FirstFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("first", OperationType::Function)
            .description("Returns the first item in the input collection")
            .example("Patient.name.first()")
            .example("Bundle.entry.first()")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for FirstFunction {
    fn identifier(&self) -> &str {
        "first"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            FirstFunction::create_metadata()
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

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(collection.first().unwrap().clone())
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            value => Ok(value.clone()),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(collection.first().unwrap().clone())
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            value => Ok(value.clone()),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "first".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Last function: returns the last item in a collection
pub struct LastFunction;

impl LastFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("last", OperationType::Function)
            .description("Returns the last item in the input collection")
            .example("Patient.name.last()")
            .example("Bundle.entry.last()")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LastFunction {
    fn identifier(&self) -> &str {
        "last"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            LastFunction::create_metadata()
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

        match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(collection.last().unwrap().clone())
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            value => Ok(value.clone()),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match &context.input {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(collection.last().unwrap().clone())
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            value => Ok(value.clone()),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "last".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Single function: returns the single item in a collection, error if not exactly one
pub struct SingleFunction;

impl SingleFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("single", OperationType::Function)
            .description("Returns the single item in the input collection, error if not exactly one")
            .example("Patient.active.single()")
            .example("Bundle.entry.where(resource.id = 'patient123').single()")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SingleFunction {
    fn identifier(&self) -> &str {
        "single"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SingleFunction::create_metadata()
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

        match &context.input {
            FhirPathValue::Collection(collection) => {
                match collection.len() {
                    0 => Ok(FhirPathValue::Empty),
                    1 => Ok(collection.first().unwrap().clone()),
                    _ => Err(FhirPathError::EvaluationError {
                        message: format!(
                            "single() function called on collection with {} items, expected exactly 1",
                            collection.len()
                        ),
                    }),
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            value => Ok(value.clone()),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match &context.input {
            FhirPathValue::Collection(collection) => {
                match collection.len() {
                    0 => Ok(FhirPathValue::Empty),
                    1 => Ok(collection.first().unwrap().clone()),
                    _ => Err(FhirPathError::EvaluationError {
                        message: format!(
                            "single() function called on collection with {} items, expected exactly 1",
                            collection.len()
                        ),
                    }),
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            value => Ok(value.clone()),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "single".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext {
            input: input.clone(),
            root: input,
            variables: Default::default(),
            model_provider: Arc::new(MockModelProvider::new()),
        }
    }

    #[tokio::test]
    async fn test_count_function() {
        let count_fn = CountFunction::new();

        // Test with collection
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection);
        let result = count_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = count_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test with single value
        let single = FhirPathValue::String("test".into());
        let context = create_test_context(single);
        let result = count_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
    }

    #[tokio::test]
    async fn test_empty_function() {
        let empty_fn = EmptyFunction::new();

        // Test with non-empty collection
        let collection = FhirPathValue::Collection(vec![FhirPathValue::String("a".into())]);
        let context = create_test_context(collection);
        let result = empty_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = empty_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with Empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = empty_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_exists_function() {
        let exists_fn = ExistsFunction::new();

        // Test with non-empty collection
        let collection = FhirPathValue::Collection(vec![FhirPathValue::String("a".into())]);
        let context = create_test_context(collection);
        let result = exists_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = exists_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with single value
        let single = FhirPathValue::String("test".into());
        let context = create_test_context(single);
        let result = exists_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_first_function() {
        let first_fn = FirstFunction::new();

        // Test with collection
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("third".into()),
        ]);
        let context = create_test_context(collection);
        let result = first_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("first".into()));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = first_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with single value
        let single = FhirPathValue::String("only".into());
        let context = create_test_context(single.clone());
        let result = first_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, single);
    }

    #[tokio::test]
    async fn test_last_function() {
        let last_fn = LastFunction::new();

        // Test with collection
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
            FhirPathValue::String("last".into()),
        ]);
        let context = create_test_context(collection);
        let result = last_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("last".into()));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = last_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with single value
        let single = FhirPathValue::String("only".into());
        let context = create_test_context(single.clone());
        let result = last_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, single);
    }

    #[tokio::test]
    async fn test_single_function() {
        let single_fn = SingleFunction::new();

        // Test with single item collection
        let collection = FhirPathValue::Collection(vec![FhirPathValue::String("only".into())]);
        let context = create_test_context(collection);
        let result = single_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("only".into()));

        // Test with empty collection
        let empty_collection = FhirPathValue::Collection(vec![]);
        let context = create_test_context(empty_collection);
        let result = single_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with multiple items - should error
        let multi_collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ]);
        let context = create_test_context(multi_collection);
        let result = single_fn.evaluate(&[], &context).await;
        assert!(result.is_err());

        // Test with single value
        let single = FhirPathValue::String("only".into());
        let context = create_test_context(single.clone());
        let result = single_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, single);
    }

    #[test]
    fn test_sync_evaluation() {
        let count_fn = CountFunction::new();
        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
        ]);
        let context = create_test_context(collection);
        
        let sync_result = count_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Integer(2));
        assert!(count_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let count_fn = CountFunction::new();
        let metadata = count_fn.metadata();
        
        assert_eq!(metadata.basic.name, "count");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}