//! XOR (exclusive or) operator implementation
//!
//! Implements FHIRPath XOR logical operator with three-valued logic.

use std::sync::Arc;
use async_trait::async_trait;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};

/// XOR operator evaluator
pub struct XorOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl XorOperatorEvaluator {
    /// Create a new XOR operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_xor_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Convert FhirPathValue to boolean for XOR operation
    fn to_boolean(&self, value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Boolean(b, _, _) => Some(*b),
            FhirPathValue::Integer(i, _, _) => Some(*i != 0),
            FhirPathValue::Decimal(d, _, _) => Some(!d.is_zero()),
            FhirPathValue::String(s, _, _) => Some(!s.is_empty()),
            FhirPathValue::Collection(coll) => Some(!coll.is_empty()),
            _ => Some(true), // Non-empty values are truthy
        }
    }

    /// Perform XOR operation with three-valued logic
    fn xor_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        let left_bool = self.to_boolean(left);
        let right_bool = self.to_boolean(right);

        match (left_bool, right_bool) {
            (Some(l), Some(r)) => {
                // XOR: true if exactly one operand is true
                Some((l && !r) || (!l && r))
            }
            _ => None, // If either operand cannot be converted to boolean, result is empty
        }
    }
}

#[async_trait]
impl OperationEvaluator for XorOperatorEvaluator {
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

        // For logical operations, we use the first elements (singleton evaluation)
        let left_value = left.first().unwrap();
        let right_value = right.first().unwrap();

        match self.xor_values(left_value, right_value) {
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

/// Create metadata for the XOR operator
fn create_xor_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "xor".to_string(),
        description: "Exclusive OR logical operator with three-valued logic".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                TypeSignature::new(vec![FhirPathType::Boolean, FhirPathType::Boolean], FhirPathType::Boolean),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 4, // FHIRPath logical precedence (same as AND/OR)
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_xor_true_false() {
        let evaluator = XorOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(false)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_xor_false_true() {
        let evaluator = XorOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(false)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_xor_true_true() {
        let evaluator = XorOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_xor_false_false() {
        let evaluator = XorOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(false)];
        let right = vec![FhirPathValue::boolean(false)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_xor_integer_truthy() {
        let evaluator = XorOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(1)]; // truthy
        let right = vec![FhirPathValue::integer(0)]; // falsy

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_xor_empty_propagation() {
        let evaluator = XorOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![]; // Empty collection

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert!(result.value.is_empty());
    }
}