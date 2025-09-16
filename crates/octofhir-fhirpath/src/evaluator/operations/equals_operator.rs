//! Equality (=) operator implementation
//!
//! Implements FHIRPath equality comparison with type-aware semantics.
//! The equality operator performs type-specific comparison and returns empty
//! if either operand is empty.

use std::sync::Arc;
use async_trait::async_trait;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};
use rust_decimal::Decimal;

/// Equality operator evaluator
pub struct EqualsOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl EqualsOperatorEvaluator {
    /// Create a new equality operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_equals_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Compare two FhirPathValues for equality
    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            // Boolean equality
            (FhirPathValue::Boolean(l, _, _), FhirPathValue::Boolean(r, _, _)) => Some(l == r),

            // String equality
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => Some(l == r),

            // Integer equality
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l == r),

            // Decimal equality
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some((l - r).abs() < Decimal::new(1, 10)) // Small epsilon for decimal comparison
            }

            // Integer vs Decimal comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some((left_decimal - r).abs() < Decimal::new(1, 10))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some((l - right_decimal).abs() < Decimal::new(1, 10))
            }

            // Date equality
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => Some(l == r),

            // DateTime equality
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => Some(l == r),

            // Time equality
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l == r),

            // Quantity equality (considering units)
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                // For now, simple equality - in real implementation we'd need unit conversion
                if lu == ru {
                    Some((lv - rv).abs() < Decimal::new(1, 10)) // Small epsilon for decimal comparison
                } else {
                    // Different units - would need proper unit conversion
                    Some(false)
                }
            }

            // Different types are not equal
            _ => Some(false),
        }
    }
}

#[async_trait]
impl OperationEvaluator for EqualsOperatorEvaluator {
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

        // For equality, we compare the first elements (singleton evaluation)
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

/// Create metadata for the equality operator
fn create_equals_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "=".to_string(),
        description: "Equality comparison with type-aware semantics".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 5, // FHIRPath equality precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_equals_boolean() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equals_integer() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equals_integer_decimal() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::decimal(42.0)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equals_different_types() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::string("42".to_string())];
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_equals_empty_propagation() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![]; // Empty collection

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert!(result.value.is_empty());
    }
}