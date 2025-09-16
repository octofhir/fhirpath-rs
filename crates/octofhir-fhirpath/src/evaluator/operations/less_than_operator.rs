//! Less than (<) operator implementation
//!
//! Implements FHIRPath less than comparison for ordered types.

use std::sync::Arc;
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};

/// Less than operator evaluator
pub struct LessThanOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl LessThanOperatorEvaluator {
    /// Create a new less than operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_less_than_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Compare two FhirPathValues for less than relationship
    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            // Integer comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l < r),

            // Decimal comparison
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => Some(l < r),

            // Integer vs Decimal comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(left_decimal < *r)
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(*l < right_decimal)
            }

            // String comparison (lexicographic)
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => Some(l < r),

            // Date comparison
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => Some(l < r),

            // DateTime comparison
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => Some(l < r),

            // Time comparison
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l < r),

            // Quantity comparison (with same units)
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                if lu == ru {
                    Some(lv < rv)
                } else {
                    // Different units - would need proper unit conversion
                    None
                }
            }

            // Other types are not orderable
            _ => None,
        }
    }
}

#[async_trait]
impl OperationEvaluator for LessThanOperatorEvaluator {
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

        // For comparison, we compare the first elements
        let left_value = left.first().unwrap();
        let right_value = right.first().unwrap();

        match self.compare_values(left_value, right_value) {
            Some(result) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(result)),
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

/// Create metadata for the less than operator
fn create_less_than_metadata() -> OperatorMetadata {
    // Support multiple ordered types
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any], // Will be validated at runtime for ordered types
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "<".to_string(),
        description: "Less than comparison for ordered types".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Integer], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Decimal], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Decimal], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Integer], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::String, FhirPathType::String], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::Date, FhirPathType::Date], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::DateTime, FhirPathType::DateTime], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::Time, FhirPathType::Time], FhirPathType::Boolean),
                TypeSignature::new(vec![FhirPathType::Quantity, FhirPathType::Quantity], FhirPathType::Boolean),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 6, // FHIRPath comparison precedence
        associativity: Associativity::Left,
    }
}