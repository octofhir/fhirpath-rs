//! Division (/) operator implementation
//!
//! Implements FHIRPath division for numeric types and quantities.

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Division operator evaluator
pub struct DivideOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for DivideOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
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
                    return Ok(None);
                }
                let left_decimal = Decimal::from(*l);
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::decimal(left_decimal / right_decimal)))
            }

            // Decimal division
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if *r == Decimal::ZERO {
                    return Ok(None);
                }
                Ok(Some(FhirPathValue::decimal(*l / *r)))
            }

            // Integer / Decimal = Decimal
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if *r == Decimal::ZERO {
                    return Ok(None);
                }
                let left_decimal = Decimal::from(*l);
                Ok(Some(FhirPathValue::decimal(left_decimal / *r)))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                if *r == 0 {
                    return Ok(None);
                }
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::decimal(*l / right_decimal)))
            }

            // Quantity / Scalar = Quantity
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    code: lc,
                    system: ls,
                    ..
                },
                FhirPathValue::Integer(r, _, _),
            ) => {
                if *r == 0 {
                    return Ok(None);
                }
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::quantity_with_components(
                    *lv / right_decimal,
                    lu.clone(),
                    lc.clone(),
                    ls.clone(),
                )))
            }
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    code: lc,
                    system: ls,
                    ..
                },
                FhirPathValue::Decimal(r, _, _),
            ) => {
                if *r == Decimal::ZERO {
                    return Ok(None);
                }
                Ok(Some(FhirPathValue::quantity_with_components(
                    *lv / *r,
                    lu.clone(),
                    lc.clone(),
                    ls.clone(),
                )))
            }

            // Quantity / Quantity = Decimal (when units cancel) or Quantity (with unit division)
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    code: lc,
                    system: ls,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    code: rc,
                    system: rs,
                    ..
                },
            ) => {
                if *rv == Decimal::ZERO {
                    return Ok(None);
                }

                // If same units, units cancel and result is a Decimal (not Quantity)
                // Per FHIRPath spec: "6 'kg' / 2 'kg' -> 3.0"
                if lu == ru {
                    Ok(Some(FhirPathValue::decimal(*lv / *rv)))
                } else {
                    // TODO: Implement proper unit division using UCUM
                    // For now, simple concatenation with division
                    let combined_unit = match (lu, ru) {
                        (None, None) => None,
                        (Some(l), None) => Some(l.clone()),
                        (None, Some(r)) => Some(format!("1/{r}")),
                        (Some(l), Some(r)) => Some(format!("{l}/{r}")),
                    };
                    Ok(Some(FhirPathValue::quantity_with_components(
                        *lv / *rv,
                        combined_unit,
                        lc.clone().or_else(|| rc.clone()),
                        ls.clone().or_else(|| rs.clone()),
                    )))
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
    use rust_decimal_macros::dec;

    fn create_test_context() -> EvaluationContext {
        EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
    }

    // ========== Integer / Integer → Decimal (CRITICAL) ==========

    #[tokio::test]
    async fn test_integer_divide_integer_returns_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(2.5)));
    }

    #[tokio::test]
    async fn test_integer_divide_yields_exact_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

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
    async fn test_integer_divide_fraction() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(1)];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(0.5)));
    }

    #[tokio::test]
    async fn test_integer_divide_with_remainder() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(7)];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(3.5)));
    }

    // ========== Decimal Division ==========

    #[tokio::test]
    async fn test_divide_decimals() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::decimal(dec!(7.5))];
        let right = vec![FhirPathValue::decimal(dec!(2.5))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(3.0)));
    }

    #[tokio::test]
    async fn test_divide_decimals_with_result() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::decimal(dec!(10.0))];
        let right = vec![FhirPathValue::decimal(dec!(4.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(2.5)));
    }

    // ========== Mixed Integer/Decimal ==========

    #[tokio::test]
    async fn test_integer_divide_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![FhirPathValue::decimal(dec!(2.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(2.5)));
    }

    #[tokio::test]
    async fn test_decimal_divide_integer() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::decimal(dec!(7.5))];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(3.75)));
    }

    // ========== Quantity / Quantity → Decimal ==========

    #[tokio::test]
    async fn test_quantity_divide_quantity_same_units_returns_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(6), Some("kg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(2), Some("kg".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        // Should return Decimal, not Quantity
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(3.0)));
    }

    #[tokio::test]
    async fn test_quantity_divide_quantity_mg() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("mg".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(2.0)));
    }

    #[tokio::test]
    async fn test_quantity_divide_quantity_cm() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(100), Some("cm".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(50), Some("cm".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(2.0)));
    }

    // ========== Quantity / Number → Quantity ==========

    #[tokio::test]
    async fn test_quantity_divide_integer() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(5));
            assert_eq!(unit.as_deref(), Some("mg"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_quantity_divide_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::decimal(dec!(2.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(5));
            assert_eq!(unit.as_deref(), Some("mg"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_quantity_cm_divide_integer() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(100), Some("cm".to_string()))];
        let right = vec![FhirPathValue::integer(4)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(25));
            assert_eq!(unit.as_deref(), Some("cm"));
        } else {
            panic!("Expected quantity result");
        }
    }

    // ========== Division by Zero ==========

    #[tokio::test]
    async fn test_integer_divide_by_zero() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![FhirPathValue::integer(0)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    #[tokio::test]
    async fn test_decimal_divide_by_zero() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::decimal(dec!(5.0))];
        let right = vec![FhirPathValue::decimal(dec!(0.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    #[tokio::test]
    async fn test_quantity_divide_by_zero_integer() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::integer(0)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    #[tokio::test]
    async fn test_quantity_divide_by_zero_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::decimal(dec!(0.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    #[tokio::test]
    async fn test_quantity_divide_by_zero_quantity() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(0), Some("mg".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    // ========== Edge Cases ==========

    #[tokio::test]
    async fn test_zero_divide_by_integer() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(0)];
        let right = vec![FhirPathValue::integer(5)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(0.0)));
    }

    #[tokio::test]
    async fn test_zero_decimal_divide_by_decimal() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::decimal(dec!(0.0))];
        let right = vec![FhirPathValue::decimal(dec!(5.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(0.0)));
    }

    #[tokio::test]
    async fn test_empty_left_operand() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![];
        let right = vec![FhirPathValue::integer(5)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    #[tokio::test]
    async fn test_empty_right_operand() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    // ========== Incompatible Quantity Division ==========

    #[tokio::test]
    async fn test_quantity_divide_different_units_creates_compound_unit() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("cm".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        // Should create a compound unit (basic implementation without UCUM)
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(2));
            assert_eq!(unit.as_deref(), Some("mg/cm"));
        } else {
            panic!("Expected quantity result");
        }
    }
}
