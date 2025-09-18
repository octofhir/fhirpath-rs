//! Division (/) operator implementation
//!
//! Implements FHIRPath division for numeric types and quantities.

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Division operator evaluator
pub struct DivideOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl DivideOperatorEvaluator {
    /// Create a new division operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_divide_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform division on two FhirPathValues
    fn divide_values(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<FhirPathValue>> {
        match (left, right) {
            // Integer division - always results in Decimal per FHIRPath spec
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                if *r == 0 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let left_decimal = Decimal::from(*l);
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::decimal(left_decimal / right_decimal)))
            }

            // Decimal division
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if *r == Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::decimal(*l / *r)))
            }

            // Integer / Decimal = Decimal
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if *r == Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let left_decimal = Decimal::from(*l);
                Ok(Some(FhirPathValue::decimal(left_decimal / *r)))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                if *r == 0 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::decimal(*l / right_decimal)))
            }

            // Quantity / Scalar = Quantity
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    ..
                },
                FhirPathValue::Integer(r, _, _),
            ) => {
                if *r == 0 {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::quantity(
                    *lv / right_decimal,
                    lu.clone(),
                )))
            }
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    ..
                },
                FhirPathValue::Decimal(r, _, _),
            ) => {
                if *r == Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }
                Ok(Some(FhirPathValue::quantity(*lv / *r, lu.clone())))
            }

            // Quantity / Quantity = Decimal (unitless) or Quantity (with unit division)
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    ..
                },
            ) => {
                if *rv == Decimal::ZERO {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0051,
                        "Division by zero".to_string(),
                    ));
                }

                // If same units, result is unitless decimal
                if lu == ru {
                    Ok(Some(FhirPathValue::decimal(*lv / *rv)))
                } else {
                    // TODO: Implement proper unit division using UCUM
                    // For now, simple concatenation with division
                    let combined_unit = match (lu, ru) {
                        (None, None) => None,
                        (Some(l), None) => Some(l.clone()),
                        (None, Some(r)) => Some(format!("1/{}", r)),
                        (Some(l), Some(r)) => Some(format!("{}/{}", l, r)),
                    };
                    Ok(Some(FhirPathValue::quantity(*lv / *rv, combined_unit)))
                }
            }

            // Invalid combinations
            _ => Ok(None),
        }
    }
}

#[async_trait]
impl OperationEvaluator for DivideOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
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

        match self.divide_values(left_value, right_value)? {
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

/// Create metadata for the division operator
fn create_divide_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Any, // Return type depends on operands
    );

    OperatorMetadata {
        name: "/".to_string(),
        description: "Division for numeric types and quantities".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                // Numeric division (always returns Decimal per FHIRPath spec)
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Integer],
                    FhirPathType::Decimal,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Decimal],
                    FhirPathType::Decimal,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Decimal],
                    FhirPathType::Decimal,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Integer],
                    FhirPathType::Decimal,
                ),
                // Quantity division
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Integer],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Decimal],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Quantity],
                    FhirPathType::Any,
                ), // Can be Decimal or Quantity
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
    async fn test_divide_integers() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(15)];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_decimal(),
            Some(Decimal::from(5))
        );
    }

    #[tokio::test]
    async fn test_divide_decimals() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::decimal(10.0)];
        let right = vec![FhirPathValue::decimal(2.0)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_decimal(),
            Some(Decimal::from(5))
        );
    }

    #[tokio::test]
    async fn test_divide_by_zero() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(10)];
        let right = vec![FhirPathValue::integer(0)];

        let result = evaluator.evaluate(vec![], &context, left, right).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }

    #[tokio::test]
    async fn test_divide_quantity_by_scalar() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::quantity(15.0, "kg".to_string())];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from(5));
            assert_eq!(*unit, "kg");
        } else {
            panic!("Expected quantity result");
        }
    }
}
