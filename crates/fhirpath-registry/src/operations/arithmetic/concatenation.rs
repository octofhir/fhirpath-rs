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

//! String concatenation operation (&) implementation for FHIRPath

use crate::metadata::{
    MetadataBuilder, OperationType, TypeConstraint, FhirPathType,
    OperationMetadata, PerformanceComplexity, Associativity,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// String concatenation operation (&) - concatenates string representations with special empty handling
pub struct ConcatenationOperation;

impl ConcatenationOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("&", OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        })
            .description("String concatenation operation with special empty handling")
            .example("'hello' & ' world'")
            .example("Patient.name.given & ' ' & Patient.name.family")
            .example("'Hello' & {}")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn evaluate_concatenation_sync(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<Result<FhirPathValue>> {
        // Handle special empty cases according to FHIRPath spec
        match (left, right) {
            // If left operand is empty, return right operand
            (FhirPathValue::Empty, right_val) => {
                if let Some(right_str) = Self::value_to_string(right_val) {
                    Some(Ok(FhirPathValue::String(right_str.into())))
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            }
            // If right operand is empty, return left operand
            (left_val, FhirPathValue::Empty) => {
                if let Some(left_str) = Self::value_to_string(left_val) {
                    Some(Ok(FhirPathValue::String(left_str.into())))
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            }
            // Handle collections - if either contains multiple items, error
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                match (l.len(), r.len()) {
                    (1, 1) => {
                        self.evaluate_concatenation_sync(l.get(0).unwrap(), r.get(0).unwrap())
                    }
                    (0, 0) => Some(Ok(FhirPathValue::Empty)),
                    (0, 1) => {
                        // Left is empty, return right as string
                        if let Some(right_str) = Self::value_to_string(r.get(0).unwrap()) {
                            Some(Ok(FhirPathValue::String(right_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    (1, 0) => {
                        // Right is empty, return left as string
                        if let Some(left_str) = Self::value_to_string(l.get(0).unwrap()) {
                            Some(Ok(FhirPathValue::String(left_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    _ => Some(Err(FhirPathError::InvalidArguments { message: 
                        "String concatenation requires single items, not collections".to_string()
                    }))
                }
            }
            (FhirPathValue::Collection(l), right_val) => {
                match l.len() {
                    1 => self.evaluate_concatenation_sync(l.get(0).unwrap(), right_val),
                    0 => {
                        // Left is empty, return right as string
                        if let Some(right_str) = Self::value_to_string(right_val) {
                            Some(Ok(FhirPathValue::String(right_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    _ => Some(Err(FhirPathError::InvalidArguments { message: 
                        "String concatenation requires single items, not collections".to_string()
                    }))
                }
            }
            (left_val, FhirPathValue::Collection(r)) => {
                match r.len() {
                    1 => self.evaluate_concatenation_sync(left_val, r.get(0).unwrap()),
                    0 => {
                        // Right is empty, return left as string
                        if let Some(left_str) = Self::value_to_string(left_val) {
                            Some(Ok(FhirPathValue::String(left_str.into())))
                        } else {
                            Some(Ok(FhirPathValue::Empty))
                        }
                    }
                    _ => Some(Err(FhirPathError::InvalidArguments { message: 
                        "String concatenation requires single items, not collections".to_string()
                    }))
                }
            }
            // Convert both operands to string and concatenate
            (left_val, right_val) => {
                let left_str = Self::value_to_string(left_val);
                let right_str = Self::value_to_string(right_val);
                match (left_str, right_str) {
                    (Some(l), Some(r)) => {
                        Some(Ok(FhirPathValue::String(format!("{}{}", l, r).into())))
                    }
                    _ => Some(Err(FhirPathError::TypeError {
                        message: format!(
                            "Cannot concatenate {} and {}",
                            left_val.type_name(), right_val.type_name()
                        )
                    }))
                }
            }
        }
    }

    /// Convert a FhirPathValue to its string representation for concatenation
    fn value_to_string(value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::String(s) => Some(s.to_string()),
            FhirPathValue::Integer(i) => Some(i.to_string()),
            FhirPathValue::Decimal(d) => Some(d.to_string()),
            FhirPathValue::Boolean(b) => Some(b.to_string()),
            FhirPathValue::Date(d) => Some(d.to_string()),
            FhirPathValue::DateTime(dt) => Some(dt.to_string()),
            FhirPathValue::Time(t) => Some(t.to_string()),
            FhirPathValue::Empty => None, // Empty values cannot be converted to string
            FhirPathValue::Collection(_) => None, // Collections handled separately
            FhirPathValue::Resource(_) => None, // Resources cannot be directly converted
            FhirPathValue::Quantity(_) => None, // Quantities need special handling
            FhirPathValue::JsonValue(_) => None, // JSON values need special handling
            FhirPathValue::TypeInfoObject { .. } => None, // Type info objects cannot be converted
        }
    }
}

#[async_trait]
impl FhirPathOperation for ConcatenationOperation {
    fn identifier(&self) -> &str {
        "&"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ConcatenationOperation::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "&".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // Use sync evaluation
        if let Some(result) = self.evaluate_concatenation_sync(&args[0], &args[1]) {
            result
        } else {
            Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot concatenate {} and {}",
                    args[0].type_name(), args[1].type_name()
                )
            })
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "&".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        self.evaluate_concatenation_sync(&args[0], &args[1])
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
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_string_concatenation() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Basic string concatenation
        let args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String(" world".into())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Empty string concatenation
        let args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("".into())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));
    }

    #[tokio::test]
    async fn test_mixed_type_concatenation() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // String and integer
        let args = vec![
            FhirPathValue::String("Count: ".into()),
            FhirPathValue::Integer(42)
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Count: 42".into()));

        // Integer and string
        let args = vec![
            FhirPathValue::Integer(123),
            FhirPathValue::String(" items".into())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("123 items".into()));

        // String and decimal
        let args = vec![
            FhirPathValue::String("Price: $".into()),
            FhirPathValue::Decimal(Decimal::from_str("19.99").unwrap())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Price: $19.99".into()));

        // String and boolean
        let args = vec![
            FhirPathValue::String("Active: ".into()),
            FhirPathValue::Boolean(true)
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Active: true".into()));
    }

    #[tokio::test]
    async fn test_concatenation_with_empty() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // String concatenated with empty returns empty
        let args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::Empty
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Empty concatenated with string returns empty
        let args = vec![
            FhirPathValue::Empty,
            FhirPathValue::String("world".into())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Empty concatenated with empty returns empty
        let args = vec![
            FhirPathValue::Empty,
            FhirPathValue::Empty
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_concatenation_with_collections() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Single item collections should work
        let args = vec![
            FhirPathValue::Collection(vec![FhirPathValue::String("hello".into())]),
            FhirPathValue::Collection(vec![FhirPathValue::String(" world".into())])
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("hello world".into()));

        // Empty collection concatenated with string returns empty
        let args = vec![
            FhirPathValue::Collection(Collection::new()),
            FhirPathValue::String("world".into())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Multi-item collection should error
        let args = vec![
            FhirPathValue::Collection(vec![
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("hi".into())
            ]),
            FhirPathValue::String(" world".into())
        ];
        let result = concat_op.evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concatenation_with_dates() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Date concatenation
        let args = vec![
            FhirPathValue::String("Date: ".into()),
            FhirPathValue::Date("2023-12-25".to_string())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Date: 2023-12-25".into()));

        // DateTime concatenation
        let args = vec![
            FhirPathValue::String("Time: ".into()),
            FhirPathValue::DateTime("2023-12-25T10:30:00Z".to_string())
        ];
        let result = concat_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("Time: 2023-12-25T10:30:00Z".into()));
    }

    #[test]
    fn test_sync_evaluation() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String(" world".into())
        ];
        let sync_result = concat_op.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::String("hello world".into()));
        assert!(concat_op.supports_sync());
    }

    #[tokio::test]
    async fn test_unsupported_type_concatenation() {
        let concat_op = ConcatenationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Cannot concatenate with resource (for example)
        let args = vec![
            FhirPathValue::String("Resource: ".into()),
            FhirPathValue::Resource(serde_json::json!({"resourceType": "Patient"}))
        ];
        let result = concat_op.evaluate(&args, &context).await;
        assert!(result.is_err());
    }
}