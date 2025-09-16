//! Multiplication (*) operator implementation
//!
//! Implements FHIRPath multiplication for numeric types and quantities.

use std::sync::Arc;
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};

/// Multiplication operator evaluator
pub struct MultiplyOperatorEvaluator {
    metadata: OperatorMetadata,
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
    fn multiply_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
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
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(FhirPathValue::quantity(*lv * right_decimal, lu.clone()))
            }
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Decimal(r, _, _)) => {
                Some(FhirPathValue::quantity(*lv * *r, lu.clone()))
            }
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                let left_decimal = Decimal::from(*l);
                Some(FhirPathValue::quantity(left_decimal * *rv, ru.clone()))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                Some(FhirPathValue::quantity(*l * *rv, ru.clone()))
            }

            // Quantity * Quantity = Quantity (with unit combination)
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                // TODO: Implement proper unit multiplication using UCUM
                // For now, simple concatenation
                let combined_unit = match (lu, ru) {
                    (None, None) => None,
                    (Some(l), None) => Some(l.clone()),
                    (None, Some(r)) => Some(r.clone()),
                    (Some(l), Some(r)) => Some(format!("{}.{}", l, r)),
                };
                Some(FhirPathValue::quantity(*lv * *rv, combined_unit))
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
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Integer], FhirPathType::Integer),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Integer], FhirPathType::Decimal),

                // Quantity multiplication
                TypeSignature::new(vec![FhirPathType::Quantity, FhirPathType::Integer], FhirPathType::Quantity),
                TypeSignature::new(vec![FhirPathType::Quantity, FhirPathType::Decimal], FhirPathType::Quantity),
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Quantity], FhirPathType::Quantity),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Quantity], FhirPathType::Quantity),
                TypeSignature::new(vec![FhirPathType::Quantity, FhirPathType::Quantity], FhirPathType::Quantity),
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

    #[tokio::test]
    async fn test_multiply_integers() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(6)];
        let right = vec![FhirPathValue::integer(7)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(42));
    }

    #[tokio::test]
    async fn test_multiply_quantity_by_scalar() {
        let evaluator = MultiplyOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::quantity(5.0, "kg".to_string())];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from_f64_retain(15.0).unwrap());
            assert_eq!(*unit, "kg");
        } else {
            panic!("Expected quantity result");
        }
    }
}