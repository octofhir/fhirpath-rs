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

            // Number / Quantity: the number is a Quantity with unit '1' (`60 / 1 's'` is
            // `60 '/s'`). Calendar durations do not divide.
            (
                FhirPathValue::Integer(..) | FhirPathValue::Decimal(..),
                FhirPathValue::Quantity {
                    value: rv,
                    code: rc,
                    unit: ru,
                    system: rs,
                    calendar_unit: rcal,
                    ..
                },
            ) => {
                if *rv == Decimal::ZERO || rcal.is_some() {
                    return Ok(None);
                }
                let number = match left {
                    FhirPathValue::Integer(n, _, _) => Decimal::from(*n),
                    FhirPathValue::Decimal(n, _, _) => *n,
                    _ => return Ok(None),
                };
                let right_unit = rc.as_deref().or(ru.as_deref()).unwrap_or("1");
                let unit = crate::evaluator::quantity_utils::combine_units("1", right_unit, true);
                Ok(number.checked_div(*rv).map(|value| {
                    FhirPathValue::quantity_with_components(
                        value,
                        Some(unit.clone()),
                        Some(unit),
                        rs.clone(),
                    )
                }))
            }

            // Quantity / Quantity = Quantity, units combined by UCUM unit algebra
            // (`120 'm' / 60 's'` is `2 'm/s'`, `1 'm' / 1 'm'` is `1 '1'`). Calendar
            // durations do not divide.
            (
                FhirPathValue::Quantity {
                    value: lv,
                    code: lc,
                    unit: lu,
                    system: ls,
                    calendar_unit: lcal,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    code: rc,
                    unit: ru,
                    system: rs,
                    calendar_unit: rcal,
                    ..
                },
            ) => {
                if *rv == Decimal::ZERO || lcal.is_some() || rcal.is_some() {
                    return Ok(None);
                }
                let left_unit = lc.as_deref().or(lu.as_deref()).unwrap_or("1");
                let right_unit = rc.as_deref().or(ru.as_deref()).unwrap_or("1");
                let unit =
                    crate::evaluator::quantity_utils::combine_units(left_unit, right_unit, true);
                Ok(lv.checked_div(*rv).map(|value| {
                    FhirPathValue::quantity_with_components(
                        value,
                        Some(unit.clone()),
                        Some(unit),
                        ls.clone().or_else(|| rs.clone()),
                    )
                }))
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
        __input: Collection,
        _context: &EvaluationContext,
        left: Collection,
        right: Collection,
    ) -> Result<EvaluationResult> {
        self.evaluate_sync(__input, _context, left, right)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(
        &self,
        __input: Collection,
        _context: &EvaluationContext,
        left: Collection,
        right: Collection,
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(3.75)));
    }

    // ========== Quantity / Quantity with the same unit → Quantity '1' ==========

    fn assert_unity(value: &FhirPathValue, expected: Decimal) {
        match value {
            FhirPathValue::Quantity { value, unit, .. } => {
                assert_eq!(*value, expected);
                assert_eq!(unit.as_deref(), Some("1"));
            }
            other => panic!("expected a Quantity, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_quantity_divide_quantity_same_units_returns_unity() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(6), Some("kg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(2), Some("kg".to_string()))];

        let result = evaluator
            .evaluate(Collection::empty(), &context, left.into(), right.into())
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        // A Quantity in the unit '1' (FHIRPath §1.6.6.2: Quantity inputs give a Quantity)
        assert_unity(result.value.first().unwrap(), dec!(3));
    }

    #[tokio::test]
    async fn test_quantity_divide_quantity_mg() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("mg".to_string()))];

        let result = evaluator
            .evaluate(Collection::empty(), &context, left.into(), right.into())
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_unity(result.value.first().unwrap(), dec!(2));
    }

    #[tokio::test]
    async fn test_quantity_divide_quantity_cm() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(100), Some("cm".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(50), Some("cm".to_string()))];

        let result = evaluator
            .evaluate(Collection::empty(), &context, left.into(), right.into())
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_unity(result.value.first().unwrap(), dec!(2));
    }

    // ========== Quantity / Number → Quantity ==========

    #[tokio::test]
    async fn test_quantity_divide_integer() {
        let evaluator = DivideOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("mg".to_string()))];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
            .evaluate(Collection::empty(), &context, left.into(), right.into())
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
