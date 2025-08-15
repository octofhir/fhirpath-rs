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

//! Split function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, 
    PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// Split function: splits a string into a collection by separator
pub struct SplitFunction;

impl SplitFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("split", OperationType::Function)
            .description("Splits a string into a collection of strings using the specified separator")
            .example("'a,b,c'.split(',')")
            .example("Patient.name.family.split(' ')")
            .parameter("separator", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Collection(Box::new(TypeConstraint::Specific(FhirPathType::String))))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SplitFunction {
    fn identifier(&self) -> &str {
        "split"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SplitFunction::create_metadata()
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

        self.evaluate_split(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_split(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SplitFunction {
    fn evaluate_split(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "split() requires exactly one argument (separator)".to_string(),
            });
        }

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![])));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![])));
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                return self.process_single_value(value, args);
            }
            _ => {
                // Process as single value
                return self.process_single_value(input, args);
            }
        }
    }

    fn process_single_value(&self, value: &FhirPathValue, args: &[FhirPathValue]) -> Result<FhirPathValue> {
        // Convert input to string (including numeric values)
        let input_str = match value {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))),
            _ => return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))), // Return empty for other non-convertible types
        };

        // Extract and convert separator parameter to string (handle collections)
        let separator = self.extract_string_from_value(&args[0])?;
        if separator.is_none() {
            return Ok(FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]))); // Return empty for non-convertible types
        }
        let separator = separator.unwrap();

        // Split the string
        let parts: Vec<FhirPathValue> = if separator.is_empty() {
            // Empty separator means split into individual characters
            input_str
                .chars()
                .map(|c| FhirPathValue::String(c.to_string().into()))
                .collect()
        } else {
            // Split by the separator
            input_str
                .split(&separator)
                .map(|s| FhirPathValue::String(s.to_string().into()))
                .collect()
        };

        Ok(FhirPathValue::Collection(parts.into()))
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
    async fn test_split_function() {
        let split_fn = SplitFunction::new();

        // Test basic split by comma
        let string = FhirPathValue::String("a,b,c".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(",".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        assert_eq!(result, expected);

        // Test split by space
        let string = FhirPathValue::String("hello world test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(" ".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
            FhirPathValue::String("test".into()),
        ]);
        assert_eq!(result, expected);

        // Test with empty parts
        let string = FhirPathValue::String("a,,b".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(",".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("".into()),
            FhirPathValue::String("b".into()),
        ]);
        assert_eq!(result, expected);

        // Test single item (no separator found)
        let string = FhirPathValue::String("hello".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(",".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
        ]);
        assert_eq!(result, expected);

        // Test empty string
        let string = FhirPathValue::String("".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(",".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("".into()),
        ]);
        assert_eq!(result, expected);

        // Test empty separator (split into characters)
        let string = FhirPathValue::String("abc".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        assert_eq!(result, expected);

        // Test with multi-character separator
        let string = FhirPathValue::String("hello::world::test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("::".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
            FhirPathValue::String("test".into()),
        ]);
        assert_eq!(result, expected);

        // Test with unicode characters
        let string = FhirPathValue::String("héllo•wörld•test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String("•".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("héllo".into()),
            FhirPathValue::String("wörld".into()),
            FhirPathValue::String("test".into()),
        ]);
        assert_eq!(result, expected);

        // Test with empty value
        let empty = FhirPathValue::Empty;
        let context = create_test_context(empty);
        let args = vec![FhirPathValue::String(",".into())];
        let result = split_fn.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_sync_evaluation() {
        let split_fn = SplitFunction::new();
        let string = FhirPathValue::String("x,y,z".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::String(",".into())];

        let sync_result = split_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        let expected = FhirPathValue::Collection(vec![
            FhirPathValue::String("x".into()),
            FhirPathValue::String("y".into()),
            FhirPathValue::String("z".into()),
        ]);
        assert_eq!(sync_result, expected);
        assert!(split_fn.supports_sync());
    }

    #[test]
    fn test_error_conditions() {
        let split_fn = SplitFunction::new();
        
        // Test with non-string input
        let integer = FhirPathValue::Integer(42);
        let context = create_test_context(integer);
        let args = vec![FhirPathValue::String(",".into())];
        let result = split_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with non-string separator argument
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![FhirPathValue::Integer(42)];
        let result = split_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());

        // Test with wrong number of arguments
        let string = FhirPathValue::String("test".into());
        let context = create_test_context(string);
        let args = vec![];
        let result = split_fn.try_evaluate_sync(&args, &context).unwrap();
        assert!(result.is_err());
    }
}