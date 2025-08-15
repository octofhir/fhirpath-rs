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

//! Multiplication operation (*) implementation for FHIRPath

use crate::metadata::{
    MetadataBuilder, OperationType, TypeConstraint, FhirPathType,
    OperationMetadata, PerformanceComplexity, Associativity,
};
use crate::operation::FhirPathOperation;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;

/// Multiplication operation (*) for FHIRPath
pub struct MultiplicationOperation;

impl MultiplicationOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("*", OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        })
            .description("Binary multiplication operation")
            .example("3 * 4")
            .example("2.5 * 1.2")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer, 
                FhirPathType::Decimal,
            ]))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    pub fn multiply_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_mul(*b)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::ArithmeticError {
                        message: "Integer overflow in multiplication".to_string()
                    })
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a * b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                match Decimal::try_from(*a) {
                    Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal * b)),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    })
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                match Decimal::try_from(*b) {
                    Ok(b_decimal) => Ok(FhirPathValue::Decimal(a * b_decimal)),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    })
                }
            }
            // Quantity * Quantity = Quantity with combined units
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                Ok(FhirPathValue::Quantity(std::sync::Arc::new(a.multiply(b))))
            }
            // Scalar * Quantity = Quantity (scalar multiplication)
            (FhirPathValue::Integer(scalar), FhirPathValue::Quantity(q)) => {
                match Decimal::try_from(*scalar) {
                    Ok(scalar_decimal) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(q.multiply_scalar(scalar_decimal)))),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal for quantity multiplication".to_string()
                    })
                }
            }
            (FhirPathValue::Decimal(scalar), FhirPathValue::Quantity(q)) => {
                Ok(FhirPathValue::Quantity(std::sync::Arc::new(q.multiply_scalar(*scalar))))
            }
            // Quantity * Scalar = Quantity (scalar multiplication)
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(scalar)) => {
                match Decimal::try_from(*scalar) {
                    Ok(scalar_decimal) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(q.multiply_scalar(scalar_decimal)))),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal for quantity multiplication".to_string()
                    })
                }
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(scalar)) => {
                Ok(FhirPathValue::Quantity(std::sync::Arc::new(q.multiply_scalar(*scalar))))
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot multiply {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for MultiplicationOperation {
    fn identifier(&self) -> &str {
        "*"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            MultiplicationOperation::create_metadata()
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
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::multiply_values)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::multiply_values))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_integer_multiplication() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(3), FhirPathValue::Integer(4)];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(12)])));
    }

    #[tokio::test]
    async fn test_decimal_multiplication() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let dec1 = Decimal::from_str("2.5").unwrap();
        let dec2 = Decimal::from_str("1.2").unwrap();
        let args = vec![FhirPathValue::Decimal(dec1), FhirPathValue::Decimal(dec2)];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Decimal(Decimal::from_str("3.0").unwrap())])));
    }

    #[tokio::test]
    async fn test_mixed_type_multiplication() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(2), FhirPathValue::Decimal(Decimal::from_str("3.5").unwrap())];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Decimal(Decimal::from_str("7.0").unwrap())])));
    }

    #[tokio::test]
    async fn test_empty_multiplication() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(2), FhirPathValue::Empty];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(3), FhirPathValue::Integer(4)];
        let sync_result = mul_op.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(12)])));
        assert!(mul_op.supports_sync());
    }

    #[tokio::test]
    async fn test_overflow_error() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(i64::MAX), FhirPathValue::Integer(2)];
        let result = mul_op.evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let mul_op = MultiplicationOperation::new();
        let metadata = mul_op.metadata();

        assert_eq!(metadata.basic.name, "*");
        if let OperationType::BinaryOperator { precedence, associativity } = metadata.basic.operation_type {
            assert_eq!(precedence, 7);
            assert_eq!(associativity, Associativity::Left);
        } else {
            panic!("Expected BinaryOperator");
        }
    }

    #[tokio::test]
    async fn test_collection_handling() {
        let mul_op = MultiplicationOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Single element collections should unwrap and multiply
        let single_collection_1 = FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)]));
        let single_collection_2 = FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(4)]));
        let args = vec![single_collection_1, single_collection_2];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(12)])));

        // Empty collections should return empty collection
        let empty_collection = FhirPathValue::Collection(Collection::from(vec![]));
        let args = vec![empty_collection, FhirPathValue::Integer(4)];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Multi-element collections should return empty collection
        let multi_collection = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Integer(2), 
            FhirPathValue::Integer(3)
        ]));
        let args = vec![multi_collection, FhirPathValue::Integer(4)];
        let result = mul_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }
}