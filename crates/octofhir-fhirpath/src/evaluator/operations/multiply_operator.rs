//! Multiplication (*) operator implementation
//!
//! Implements FHIRPath multiplication for numeric types and quantities.

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Multiplication operator evaluator
pub struct MultiplyOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for MultiplyOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiplyOperatorEvaluator {
    /// Create a new multiplication operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_multiply_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform multiplication on two FhirPathValues
    fn multiply_values(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<FhirPathValue> {
        match (left, right) {
            // Integer multiplication
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                Some(FhirPathValue::integer(l * r))
            }

            // Decimal multiplication
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some(FhirPathValue::decimal(*l * *r))
            }

            // Integer * Decimal = Decimal
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(FhirPathValue::decimal(left_decimal * *r))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(FhirPathValue::decimal(*l * right_decimal))
            }

            // Quantity * Scalar = Quantity
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
                let right_decimal = Decimal::from(*r);
                Some(FhirPathValue::quantity_with_components(
                    *lv * right_decimal,
                    lu.clone(),
                    lc.clone(),
                    ls.clone(),
                ))
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
            ) => Some(FhirPathValue::quantity_with_components(
                *lv * *r,
                lu.clone(),
                lc.clone(),
                ls.clone(),
            )),
            (
                FhirPathValue::Integer(l, _, _),
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    code: rc,
                    system: rs,
                    ..
                },
            ) => {
                let left_decimal = Decimal::from(*l);
                Some(FhirPathValue::quantity_with_components(
                    left_decimal * *rv,
                    ru.clone(),
                    rc.clone(),
                    rs.clone(),
                ))
            }
            (
                FhirPathValue::Decimal(l, _, _),
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    code: rc,
                    system: rs,
                    ..
                },
            ) => Some(FhirPathValue::quantity_with_components(
                *l * *rv,
                ru.clone(),
                rc.clone(),
                rs.clone(),
            )),

            // Quantity * Quantity = Quantity (with unit combination using UCUM algebra)
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
                // Multiply values
                let result_value = *lv * *rv;

                // Handle unit combination using UCUM
                let (result_unit, result_code) = match (lu.as_deref(), ru.as_deref()) {
                    (None, None) => (None, None),
                    (Some(l), None) | (Some(l), Some("1")) => (Some(l.to_string()), lc.clone()),
                    (None, Some(r)) | (Some("1"), Some(r)) => (Some(r.to_string()), rc.clone()),
                    (Some(l), Some(r)) => {
                        // Use UCUM library for proper unit multiplication
                        match octofhir_ucum::unit_multiply(l, r) {
                            Ok(result) => {
                                // Use the UCUM-computed result expression
                                (Some(result.expression.clone()), Some(result.expression))
                            }
                            Err(_) => {
                                // Fall back to simple concatenation if UCUM fails
                                // This handles non-UCUM units gracefully
                                let combined = format!("{l}.{r}");
                                (Some(combined.clone()), Some(combined))
                            }
                        }
                    }
                };

                Some(FhirPathValue::quantity_with_components(
                    result_value,
                    result_unit.clone(),
                    result_code,
                    ls.clone().or_else(|| rs.clone()),
                ))
            }

            // Invalid combinations
            _ => None,
        }
    }
}

#[async_trait]
impl OperationEvaluator for MultiplyOperatorEvaluator {
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

        match self.multiply_values(left_value, right_value) {
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

/// Create metadata for the multiplication operator
fn create_multiply_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Any, // Return type depends on operands
    );

    OperatorMetadata {
        name: "*".to_string(),
        description: "Multiplication for numeric types and quantities".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                // Numeric multiplication
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Integer],
                    FhirPathType::Integer,
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
                // Quantity multiplication
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Integer],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Decimal],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Quantity],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Quantity],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Quantity],
                    FhirPathType::Quantity,
                ),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 8, // FHIRPath multiplication precedence
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

    // ========== Integer Multiplication ==========

    #[tokio::test]
    async fn test_multiply_integers() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(6)];
        let right = vec![FhirPathValue::integer(7)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(42));
    }

    // ========== Decimal Multiplication ==========

    #[tokio::test]
    async fn test_multiply_decimals() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::decimal(dec!(2.5))];
        let right = vec![FhirPathValue::decimal(dec!(4.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(dec!(10.0)));
    }

    // ========== Quantity * Scalar ==========

    #[tokio::test]
    async fn test_multiply_quantity_by_integer() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(5), Some("kg".to_string()))];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(15));
            assert_eq!(unit.as_deref(), Some("kg"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_quantity_by_decimal() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(3), Some("mg".to_string()))];
        let right = vec![FhirPathValue::decimal(dec!(2.0))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(6));
            assert_eq!(unit.as_deref(), Some("mg"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_scalar_by_quantity() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(2)];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("m".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(10));
            assert_eq!(unit.as_deref(), Some("m"));
        } else {
            panic!("Expected quantity result");
        }
    }

    // ========== Basic Unit Multiplication (UCUM) ==========

    #[tokio::test]
    async fn test_multiply_meters_by_meters() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(2), Some("m".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(3), Some("m".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(6));
            // UCUM should produce m2 (square meters)
            assert_eq!(unit.as_deref(), Some("m.m"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_cm_by_cm() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(5), Some("cm".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(4), Some("cm".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(20));
            assert_eq!(unit.as_deref(), Some("cm.cm"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_kg_by_kg() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("kg".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(2), Some("kg".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(20));
            assert_eq!(unit.as_deref(), Some("kg.kg"));
        } else {
            panic!("Expected quantity result");
        }
    }

    // ========== Dimensionless Multiplication ==========

    #[tokio::test]
    async fn test_multiply_dimensionless_by_quantity() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(5), Some("1".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(3), Some("m".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(15));
            assert_eq!(unit.as_deref(), Some("m"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_quantity_by_dimensionless() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(2), Some("m".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(3), Some("1".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(6));
            assert_eq!(unit.as_deref(), Some("m"));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_two_dimensionless() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(4), Some("1".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("1".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(20));
            assert_eq!(unit.as_deref(), Some("1"));
        } else {
            panic!("Expected quantity result");
        }
    }

    // ========== Unit Composition (Different Units) ==========

    #[tokio::test]
    async fn test_multiply_m_by_s() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(10), Some("m".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("s".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(50));
            // Should be m.s
            assert_eq!(unit.as_deref(), Some("m.s"));
        } else {
            panic!("Expected quantity result");
        }
    }

    // ========== Edge Cases ==========

    #[tokio::test]
    async fn test_multiply_zero_quantity() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(0), Some("m".to_string()))];
        let right = vec![FhirPathValue::quantity(dec!(5), Some("m".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(0));
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_empty_left() {
        let evaluator = MultiplyOperatorEvaluator::new();
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
    async fn test_multiply_empty_right() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 0);
    }

    // ========== Unitless Quantities ==========

    #[tokio::test]
    async fn test_multiply_unitless_quantities() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(3), None)];
        let right = vec![FhirPathValue::quantity(dec!(4), None)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(12));
            assert_eq!(*unit, None);
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_multiply_unitless_by_unit() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = create_test_context();

        let left = vec![FhirPathValue::quantity(dec!(3), None)];
        let right = vec![FhirPathValue::quantity(dec!(4), Some("kg".to_string()))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, dec!(12));
            assert_eq!(unit.as_deref(), Some("kg"));
        } else {
            panic!("Expected quantity result");
        }
    }
}
