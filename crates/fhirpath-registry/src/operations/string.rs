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

//! Core string function implementations for FHIRPath
//!
//! This module contains implementations of essential string functions:
//! length, contains, startsWith, endsWith, substring with both sync and async support.

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType
};
use crate::enhanced_metadata::PerformanceComplexity;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::function::EvaluationContext;

/// Length function: returns the length of a string
pub struct LengthFunction;

impl LengthFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("length", OperationType::Function)
            .description("Returns the number of characters in a string")
            .example("Patient.name.given.first().length()")
            .example("'hello world'.length()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for LengthFunction {
    fn identifier(&self) -> &str {
        "length"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            LengthFunction::create_metadata()
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
            FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.chars().count() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "length() can only be called on string values".to_string(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let result = match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.chars().count() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "length() can only be called on string values".to_string(),
            }),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "length".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Contains function: tests if a string contains a substring
pub struct ContainsFunction;

impl ContainsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("contains", OperationType::Function)
            .description("Returns true if the string contains the given substring")
            .example("Patient.name.family.contains('Smith')")
            .example("'hello world'.contains('world')")
            .parameter("substring", TypeConstraint::Specific(FhirPathType::String))
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
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        let substring = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Err(FhirPathError::EvaluationError {
                message: "contains() requires a string argument".to_string(),
            }),
        };

        match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Boolean(s.as_ref().contains(substring)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "contains() can only be called on string values".to_string(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let substring = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Some(Err(FhirPathError::EvaluationError {
                message: "contains() requires a string argument".to_string(),
            })),
        };

        let result = match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Boolean(s.as_ref().contains(substring)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "contains() can only be called on string values".to_string(),
            }),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "contains".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// StartsWith function: tests if a string starts with a prefix
pub struct StartsWithFunction;

impl StartsWithFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("startsWith", OperationType::Function)
            .description("Returns true if the string starts with the given prefix")
            .example("Patient.name.family.startsWith('Mc')")
            .example("'hello world'.startsWith('hello')")
            .parameter("prefix", TypeConstraint::Specific(FhirPathType::String))
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

        let prefix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Err(FhirPathError::EvaluationError {
                message: "startsWith() requires a string argument".to_string(),
            }),
        };

        match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Boolean(s.as_ref().starts_with(prefix)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "startsWith() can only be called on string values".to_string(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let prefix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Some(Err(FhirPathError::EvaluationError {
                message: "startsWith() requires a string argument".to_string(),
            })),
        };

        let result = match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Boolean(s.as_ref().starts_with(prefix)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "startsWith() can only be called on string values".to_string(),
            }),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "startsWith".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// EndsWith function: tests if a string ends with a suffix
pub struct EndsWithFunction;

impl EndsWithFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("endsWith", OperationType::Function)
            .description("Returns true if the string ends with the given suffix")
            .example("Patient.name.family.endsWith('son')")
            .example("'hello world'.endsWith('world')")
            .parameter("suffix", TypeConstraint::Specific(FhirPathType::String))
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for EndsWithFunction {
    fn identifier(&self) -> &str {
        "endsWith"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            EndsWithFunction::create_metadata()
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

        let suffix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Err(FhirPathError::EvaluationError {
                message: "endsWith() requires a string argument".to_string(),
            }),
        };

        match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Boolean(s.as_ref().ends_with(suffix)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "endsWith() can only be called on string values".to_string(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let suffix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Some(Err(FhirPathError::EvaluationError {
                message: "endsWith() requires a string argument".to_string(),
            })),
        };

        let result = match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::Boolean(s.as_ref().ends_with(suffix)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "endsWith() can only be called on string values".to_string(),
            }),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "endsWith".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        Ok(())
    }
}

/// Substring function: extracts a substring from a string
pub struct SubstringFunction;

impl SubstringFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("substring", OperationType::Function)
            .description("Returns a substring starting at the given index, optionally with length")
            .example("'hello world'.substring(6)")
            .example("'hello world'.substring(0, 5)")
            .parameter("start", TypeConstraint::Specific(FhirPathType::Integer))
            .parameter("length", TypeConstraint::Specific(FhirPathType::Integer))
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SubstringFunction {
    fn identifier(&self) -> &str {
        "substring"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SubstringFunction::create_metadata()
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

        let start_index = match &args[0] {
            FhirPathValue::Integer(i) => *i as usize,
            _ => return Err(FhirPathError::EvaluationError {
                message: "substring() requires an integer start index".to_string(),
            }),
        };

        let length = if args.len() > 1 {
            match &args[1] {
                FhirPathValue::Integer(i) => Some(*i as usize),
                _ => return Err(FhirPathError::EvaluationError {
                    message: "substring() length must be an integer".to_string(),
                }),
            }
        } else {
            None
        };

        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                
                if start_index >= chars.len() {
                    return Ok(FhirPathValue::String("".into()));
                }

                let end_index = if let Some(len) = length {
                    std::cmp::min(start_index + len, chars.len())
                } else {
                    chars.len()
                };

                let substring: String = chars[start_index..end_index].iter().collect();
                Ok(FhirPathValue::String(substring.into()))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "substring() can only be called on string values".to_string(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        let start_index = match &args[0] {
            FhirPathValue::Integer(i) => *i as usize,
            _ => return Some(Err(FhirPathError::EvaluationError {
                message: "substring() requires an integer start index".to_string(),
            })),
        };

        let length = if args.len() > 1 {
            match &args[1] {
                FhirPathValue::Integer(i) => Some(*i as usize),
                _ => return Some(Err(FhirPathError::EvaluationError {
                    message: "substring() length must be an integer".to_string(),
                })),
            }
        } else {
            None
        };

        let result = match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                
                if start_index >= chars.len() {
                    Ok(FhirPathValue::String("".into()))
                } else {
                    let end_index = if let Some(len) = length {
                        std::cmp::min(start_index + len, chars.len())
                    } else {
                        chars.len()
                    };

                    let substring: String = chars[start_index..end_index].iter().collect();
                    Ok(FhirPathValue::String(substring.into()))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                message: "substring() can only be called on string values".to_string(),
            }),
        };
        Some(result)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() < 1 || args.len() > 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "substring".to_string(),
                expected: if args.len() < 1 { 1 } else { 2 },
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
    async fn test_length_function() {
        let length_fn = LengthFunction::new();

        // Test with string
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(11));

        // Test with empty string
        let empty_string = FhirPathValue::String("".into());
        let context = create_test_context(empty_string);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test with unicode characters
        let unicode_string = FhirPathValue::String("héllo 世界".into());
        let context = create_test_context(unicode_string);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(8));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let result = length_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with non-string should error
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let result = length_fn.evaluate(&[], &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_contains_function() {
        let contains_fn = ContainsFunction::new();

        // Test contains positive case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("world".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test contains negative case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("xyz".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty substring
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with empty string
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String("test".into())];
        let result = contains_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_starts_with_function() {
        let starts_with_fn = StartsWithFunction::new();

        // Test startsWith positive case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test startsWith negative case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("world".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty prefix
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = starts_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_ends_with_function() {
        let ends_with_fn = EndsWithFunction::new();

        // Test endsWith positive case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("world".into())];
        let result = ends_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test endsWith negative case
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("hello".into())];
        let result = ends_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty suffix
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = ends_with_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_substring_function() {
        let substring_fn = SubstringFunction::new();

        // Test substring with start index only
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(6)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("world".into()));

        // Test substring with start and length
        let string = FhirPathValue::String("hello world".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(5)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));

        // Test substring with start index beyond string length
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(10)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("".into()));

        // Test substring with length extending beyond string
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(3), FhirPathValue::Integer(10)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("lo".into()));

        // Test substring with unicode characters
        let string = FhirPathValue::String("héllo 世界".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(6), FhirPathValue::Integer(2)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("世界".into()));

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::Integer(0)];
        let result = substring_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let length_fn = LengthFunction::new();
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        
        let sync_result = length_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Integer(4));
        assert!(length_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let length_fn = LengthFunction::new();
        let metadata = length_fn.metadata();
        
        assert_eq!(metadata.basic.name, "length");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }

    #[test]
    fn test_argument_validation() {
        let length_fn = LengthFunction::new();
        let contains_fn = ContainsFunction::new();
        let substring_fn = SubstringFunction::new();

        // length() should take no arguments
        assert!(length_fn.validate_args(&[]).is_ok());
        assert!(length_fn.validate_args(&[FhirPathValue::String("test".into())]).is_err());

        // contains() should take exactly 1 argument
        assert!(contains_fn.validate_args(&[]).is_err());
        assert!(contains_fn.validate_args(&[FhirPathValue::String("test".into())]).is_ok());
        assert!(contains_fn.validate_args(&[FhirPathValue::String("test".into()), FhirPathValue::String("extra".into())]).is_err());

        // substring() should take 1 or 2 arguments
        assert!(substring_fn.validate_args(&[]).is_err());
        assert!(substring_fn.validate_args(&[FhirPathValue::Integer(0)]).is_ok());
        assert!(substring_fn.validate_args(&[FhirPathValue::Integer(0), FhirPathValue::Integer(5)]).is_ok());
        assert!(substring_fn.validate_args(&[FhirPathValue::Integer(0), FhirPathValue::Integer(5), FhirPathValue::Integer(10)]).is_err());
    }
}