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

//! Division operation (/) implementation for FHIRPath

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

/// Division operation (/) - returns decimal result
pub struct DivisionOperation;

impl DivisionOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("/", OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        })
            .description("Division operation - returns decimal result")
            .example("10 / 3")
            .example("6.0 / 2.0")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    pub fn divide_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    match (Decimal::try_from(*a), Decimal::try_from(*b)) {
                        (Ok(a_decimal), Ok(b_decimal)) => {
                            Ok(FhirPathValue::Decimal(a_decimal / b_decimal))
                        }
                        _ => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integers to decimal for division".to_string()
                        })
                    }
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if *b == Decimal::ZERO {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Decimal(a / b))
                }
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if *b == Decimal::ZERO {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    match Decimal::try_from(*a) {
                        Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal / b)),
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal".to_string()
                        })
                    }
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    match Decimal::try_from(*b) {
                        Ok(b_decimal) => Ok(FhirPathValue::Decimal(a / b_decimal)),
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal".to_string()
                        })
                    }
                }
            }
            // Quantity / Quantity = Quantity with combined units
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                match a.divide(b) {
                    Some(result) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(result))),
                    None => Ok(FhirPathValue::Empty), // Division by zero returns empty
                }
            }
            // Quantity / Scalar = Quantity (scalar division)
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(scalar)) => {
                if *scalar == 0 {
                    Ok(FhirPathValue::Empty) // Division by zero returns empty
                } else {
                    match Decimal::try_from(*scalar) {
                        Ok(scalar_decimal) => match q.divide_scalar(scalar_decimal) {
                            Some(result) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(result))),
                            None => Ok(FhirPathValue::Empty), // Division by zero returns empty
                        },
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal for quantity division".to_string()
                        })
                    }
                }
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(scalar)) => {
                match q.divide_scalar(*scalar) {
                    Some(result) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(result))),
                    None => Ok(FhirPathValue::Empty), // Division by zero returns empty
                }
            }
            // Scalar / Quantity - this creates a quantity with reciprocal unit
            (FhirPathValue::Integer(scalar), FhirPathValue::Quantity(q)) => {
                if q.value.is_zero() {
                    Ok(FhirPathValue::Empty) // Division by zero returns empty
                } else {
                    match Decimal::try_from(*scalar) {
                        Ok(scalar_decimal) => {
                            let result_value = scalar_decimal / q.value;
                            let result_unit = if let Some(unit) = &q.unit {
                                if unit == "1" || unit.is_empty() {
                                    Some("1".to_string())
                                } else {
                                    Some(format!("1/{}", unit))
                                }
                            } else {
                                Some("1".to_string())
                            };
                            Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                                octofhir_fhirpath_model::Quantity::new(result_value, result_unit)
                            )))
                        },
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal for quantity division".to_string()
                        })
                    }
                }
            }
            (FhirPathValue::Decimal(scalar), FhirPathValue::Quantity(q)) => {
                if q.value.is_zero() {
                    Ok(FhirPathValue::Empty) // Division by zero returns empty
                } else {
                    let result_value = scalar / q.value;
                    let result_unit = if let Some(unit) = &q.unit {
                        if unit == "1" || unit.is_empty() {
                            Some("1".to_string())
                        } else {
                            Some(format!("1/{}", unit))
                        }
                    } else {
                        Some("1".to_string())
                    };
                    Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                        octofhir_fhirpath_model::Quantity::new(result_value, result_unit)
                    )))
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot divide {} by {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for DivisionOperation {
    fn identifier(&self) -> &str {
        "/"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            DivisionOperation::create_metadata()
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
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::divide_values)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::divide_values))
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
    use octofhir_fhirpath_model::Collection;
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
    async fn test_integer_division() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Integer division returns decimal wrapped in collection
        let args = vec![FhirPathValue::Integer(10), FhirPathValue::Integer(3)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        
        // 10/3 = 3.333... wrapped in collection
        if let FhirPathValue::Collection(coll) = result {
            assert_eq!(coll.len(), 1);
            if let FhirPathValue::Decimal(dec) = coll.first().unwrap() {
                assert!((dec - Decimal::from_str("3.333333333333333333333333333").unwrap()).abs() < Decimal::new(1, 20));
            } else {
                panic!("Expected decimal result in collection");
            }
        } else {
            panic!("Expected collection result");
        }
    }

    #[tokio::test]
    async fn test_decimal_division() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Decimal division wrapped in collection
        let dec1 = Decimal::from_str("6.0").unwrap();
        let dec2 = Decimal::from_str("2.0").unwrap();
        let args = vec![FhirPathValue::Decimal(dec1), FhirPathValue::Decimal(dec2)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        
        let expected = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Decimal(Decimal::from_str("3.0").unwrap())
        ]));
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_mixed_type_division() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Integer divided by decimal wrapped in collection
        let args = vec![FhirPathValue::Integer(6), FhirPathValue::Decimal(Decimal::from_str("2.0").unwrap())];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        let expected = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Decimal(Decimal::from_str("3.0").unwrap())
        ]));
        assert_eq!(result, expected);

        // Decimal divided by integer wrapped in collection
        let args = vec![FhirPathValue::Decimal(Decimal::from_str("6.0").unwrap()), FhirPathValue::Integer(2)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        let expected = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Decimal(Decimal::from_str("3.0").unwrap())
        ]));
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_division_by_zero() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Division by zero should return empty collection according to FHIRPath specification
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(0)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Decimal division by zero should return empty collection according to FHIRPath specification
        let args = vec![FhirPathValue::Decimal(Decimal::from_str("5.0").unwrap()), FhirPathValue::Decimal(Decimal::ZERO)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Mixed type division by zero should return empty collection
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Decimal(Decimal::ZERO)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        let args = vec![FhirPathValue::Decimal(Decimal::from_str("5.0").unwrap()), FhirPathValue::Integer(0)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[tokio::test]
    async fn test_division_with_empty() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Empty operands return empty collections
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Empty];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        let args = vec![FhirPathValue::Empty, FhirPathValue::Integer(5)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(6), FhirPathValue::Integer(2)];
        let sync_result = div_op.try_evaluate_sync(&args, &context).unwrap().unwrap();
        let expected = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Decimal(Decimal::from_str("3").unwrap())
        ]));
        assert_eq!(sync_result, expected);
        assert!(div_op.supports_sync());
    }

    #[tokio::test]
    async fn test_type_errors() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Cannot divide by string
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::String("hello".into())];
        let result = div_op.evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_collection_handling() {
        let div_op = DivisionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Single-element collections should be unwrapped
        let left = FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(6)]));
        let right = FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(2)]));
        let args = vec![left, right];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        let expected = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Decimal(Decimal::from_str("3").unwrap())
        ]));
        assert_eq!(result, expected);

        // Empty collections should return empty collection
        let left = FhirPathValue::Collection(Collection::from(vec![]));
        let right = FhirPathValue::Integer(2);
        let args = vec![left, right];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Multi-element collections should return empty collection
        let left = FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Integer(4), 
            FhirPathValue::Integer(6)
        ]));
        let right = FhirPathValue::Integer(2);
        let args = vec![left, right];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }
}