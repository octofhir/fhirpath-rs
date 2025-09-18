//! Logical AND operator implementation
//!
//! Implements three-valued logic for the FHIRPath 'and' operator.
//! Truth table:
//! - true and true = true
//! - true and false = false
//! - false and true = false
//! - false and false = false
//! - true and {} = {}
//! - false and {} = false (short-circuit)
//! - {} and true = {}
//! - {} and false = false (short-circuit)
//! - {} and {} = {}

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Logical AND operator evaluator
pub struct AndOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl AndOperatorEvaluator {
    /// Create a new AND operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_and_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Extract boolean value from a collection
    fn extract_boolean(&self, collection: &[FhirPathValue]) -> Option<bool> {
        if collection.is_empty() {
            None
        } else if let Some(first) = collection.first() {
            first.as_boolean()
        } else {
            None
        }
    }
}

#[async_trait]
impl OperationEvaluator for AndOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Extract boolean values from both operands
        let left_bool = self.extract_boolean(&left);
        let right_bool = self.extract_boolean(&right);

        // Three-valued logic for AND:
        // If either operand is false, result is false (short-circuit)
        if left_bool == Some(false) || right_bool == Some(false) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // If both operands are true, result is true
        if left_bool == Some(true) && right_bool == Some(true) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // In all other cases (involving empty collections), return empty
        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the AND operator
fn create_and_metadata() -> OperatorMetadata {
    let signature = TypeSignature::new(
        vec![FhirPathType::Boolean, FhirPathType::Boolean],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "and".to_string(),
        description: "Logical AND operation with three-valued logic".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Custom, // Custom logic for short-circuiting
        deterministic: true,
        precedence: 3, // FHIRPath AND precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_and_true_true() {
        let evaluator = AndOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_and_true_false() {
        let evaluator = AndOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(false)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_and_false_short_circuit() {
        let evaluator = AndOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(false)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_and_empty_propagation() {
        let evaluator = AndOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert!(result.value.is_empty());
    }
}
