//! Integer division (div) operator implementation
//!
//! Implements FHIRPath integer division for numeric types.

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Integer division operator evaluator
pub struct IntegerDivideOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for IntegerDivideOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegerDivideOperatorEvaluator {
    /// Create a new integer division operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_integer_divide_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform integer division on two FhirPathValues
    fn integer_divide_values(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<FhirPathValue>> {
        match (left, right) {
            // Integer division
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                if *r == 0 {
                    return Ok(None);
                }
                Ok(Some(FhirPathValue::integer(l / r)))
            }

            // Decimal integer division (truncate to integer)
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if *r == Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let result = *l / *r;
                // Convert to integer (truncate towards zero)
                let integer_result = result.trunc().to_i64().unwrap_or(0);
                Ok(Some(FhirPathValue::integer(integer_result)))
            }

            // Mixed integer/decimal integer division
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if *r == Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let left_decimal = Decimal::from(*l);
                let result = left_decimal / *r;
                let integer_result = result.trunc().to_i64().unwrap_or(0);
                Ok(Some(FhirPathValue::integer(integer_result)))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                if *r == 0 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let right_decimal = Decimal::from(*r);
                let result = *l / right_decimal;
                let integer_result = result.trunc().to_i64().unwrap_or(0);
                Ok(Some(FhirPathValue::integer(integer_result)))
            }

            // Invalid combinations
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl OperationEvaluator for IntegerDivideOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if either operand is empty, result is empty
        if left.is_empty() || right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // For arithmetic, we use the first elements (singleton evaluation)
        let left_value = left.first().unwrap();
        let right_value = right.first().unwrap();

        match self.integer_divide_values(left_value, right_value)? {
            Some(result) => Ok(EvaluationResult {
                value: Collection::single(result),
            }),
            None => Ok(EvaluationResult {
                value: Collection::empty(),
            }),
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the integer division operator
fn create_integer_divide_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Integer, // Always returns Integer
    );

    OperatorMetadata {
        name: "div".to_string(),
        description: "Integer division (truncates result to integer)".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Integer],
                    FhirPathType::Integer,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Decimal],
                    FhirPathType::Integer,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Decimal],
                    FhirPathType::Integer,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Integer],
                    FhirPathType::Integer,
                ),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 8, // FHIRPath multiplication/division precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_integer_divide() {
        let evaluator = IntegerDivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(17)];
        let right = vec![FhirPathValue::integer(5)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(3));
    }

    #[tokio::test]
    async fn test_integer_divide_decimals() {
        let evaluator = IntegerDivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::decimal(17.8)];
        let right = vec![FhirPathValue::decimal(5.2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(3)); // 17.8 / 5.2 = 3.423... truncated to 3
    }

    #[tokio::test]
    async fn test_integer_divide_by_zero() {
        let evaluator = IntegerDivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(10)];
        let right = vec![FhirPathValue::integer(0)];

        let result = evaluator.evaluate(vec![], &context, left, right).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }
}
