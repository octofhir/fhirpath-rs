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

//! Integer division operation (div) implementation for FHIRPath

use crate::metadata::{
    MetadataBuilder, OperationType, TypeConstraint, FhirPathType,
    OperationMetadata, PerformanceComplexity, Associativity,
};
use crate::operation::FhirPathOperation;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// Integer division operation (div) - returns integer result
pub struct DivOperation;

impl DivOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("div", OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        })
            .description("Integer division operation - returns integer result by truncating towards zero")
            .example("10 div 3")
            .example("7 div 2")
            .example("-10 div 3")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    pub fn div_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Special handling for division by zero - return empty instead of error for div operation
        let check_zero = |val: &FhirPathValue| -> bool {
            match val {
                FhirPathValue::Integer(i) => *i == 0,
                FhirPathValue::Decimal(d) => *d == Decimal::ZERO,
                _ => false,
            }
        };

        if check_zero(right) {
            // Return empty for division by zero
            return Err(FhirPathError::ArithmeticError {
                message: "Division by zero".to_string()
            });
        }

        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                // Integer division - truncate towards zero
                Ok(FhirPathValue::Integer(a / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                let result = a / b;
                // Truncate towards zero
                let truncated = result.trunc();
                
                match truncated.to_i64() {
                    Some(int_result) => Ok(FhirPathValue::Integer(int_result)),
                    None => Err(FhirPathError::ArithmeticError {
                        message: "Integer division result too large".to_string()
                    })
                }
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                match Decimal::try_from(*a) {
                    Ok(a_decimal) => {
                        let result = a_decimal / b;
                        let truncated = result.trunc();
                        match truncated.to_i64() {
                            Some(int_result) => Ok(FhirPathValue::Integer(int_result)),
                            None => Err(FhirPathError::ArithmeticError {
                                message: "Integer division result too large".to_string()
                            })
                        }
                    }
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    })
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                match Decimal::try_from(*b) {
                    Ok(b_decimal) => {
                        let result = a / b_decimal;
                        let truncated = result.trunc();
                        match truncated.to_i64() {
                            Some(int_result) => Ok(FhirPathValue::Integer(int_result)),
                            None => Err(FhirPathError::ArithmeticError {
                                message: "Integer division result too large".to_string()
                            })
                        }
                    }
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot perform integer division of {} by {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for DivOperation {
    fn identifier(&self) -> &str {
        "div"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            DivOperation::create_metadata()
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
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // Special handling for div - division by zero returns empty collection instead of error
        match binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::div_values) {
            Ok(result) => Ok(result),
            Err(FhirPathError::ArithmeticError { message }) if message == "Division by zero" => {
                use octofhir_fhirpath_model::Collection;
                Ok(FhirPathValue::Collection(Collection::from(vec![])))
            }
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        // Special handling for div - division by zero returns empty collection instead of error
        match binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::div_values) {
            Ok(result) => Some(Ok(result)),
            Err(FhirPathError::ArithmeticError { message }) if message == "Division by zero" => {
                use octofhir_fhirpath_model::Collection;
                Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))))
            }
            Err(e) => Some(Err(e)),
        }
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
    async fn test_integer_div() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // 10 div 3 = 3 (truncated towards zero)
        let args = vec![FhirPathValue::Integer(10), FhirPathValue::Integer(3)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));

        // 7 div 2 = 3
        let args = vec![FhirPathValue::Integer(7), FhirPathValue::Integer(2)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));

        // -10 div 3 = -3 (truncated towards zero)
        let args = vec![FhirPathValue::Integer(-10), FhirPathValue::Integer(3)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(-3)])));

        // 10 div -3 = -3 (truncated towards zero)
        let args = vec![FhirPathValue::Integer(10), FhirPathValue::Integer(-3)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(-3)])));
    }

    #[tokio::test]
    async fn test_decimal_div() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // 7.8 div 2.0 = 3 (truncated)
        let dec1 = Decimal::from_str("7.8").unwrap();
        let dec2 = Decimal::from_str("2.0").unwrap();
        let args = vec![FhirPathValue::Decimal(dec1), FhirPathValue::Decimal(dec2)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));

        // -7.8 div 2.0 = -3 (truncated towards zero)
        let dec1 = Decimal::from_str("-7.8").unwrap();
        let dec2 = Decimal::from_str("2.0").unwrap();
        let args = vec![FhirPathValue::Decimal(dec1), FhirPathValue::Decimal(dec2)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(-3)])));
    }

    #[tokio::test]
    async fn test_mixed_type_div() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Integer div by decimal
        let args = vec![FhirPathValue::Integer(7), FhirPathValue::Decimal(Decimal::from_str("2.0").unwrap())];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));

        // Decimal div by integer
        let args = vec![FhirPathValue::Decimal(Decimal::from_str("7.5").unwrap()), FhirPathValue::Integer(2)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));
    }

    #[tokio::test]
    async fn test_div_by_zero() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Division by zero should return empty collection
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(0)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Decimal division by zero should return empty collection
        let args = vec![FhirPathValue::Decimal(Decimal::from_str("5.0").unwrap()), FhirPathValue::Decimal(Decimal::ZERO)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[tokio::test]
    async fn test_div_with_empty() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Empty operands should return empty collection
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Empty];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        let args = vec![FhirPathValue::Empty, FhirPathValue::Integer(5)];
        let result = div_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(10), FhirPathValue::Integer(3)];
        let sync_result = div_op.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));
        assert!(div_op.supports_sync());
    }

    #[tokio::test]
    async fn test_type_errors() {
        let div_op = DivOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Cannot divide by string
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::String("hello".into())];
        let result = div_op.evaluate(&args, &context).await;
        assert!(result.is_err());
    }
}