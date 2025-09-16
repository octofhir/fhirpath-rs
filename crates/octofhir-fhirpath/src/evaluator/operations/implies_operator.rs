//! Logical IMPLIES operator implementation
//!
//! Implements three-valued logic for the FHIRPath 'implies' operator.
//! The implies operator follows the logical implication A → B, which is equivalent to (not A) or B.
//! Truth table:
//! - true implies true = true
//! - true implies false = false
//! - false implies true = true
//! - false implies false = true
//! - true implies {} = {}
//! - false implies {} = true
//! - {} implies true = true
//! - {} implies false = {}
//! - {} implies {} = {}

use std::sync::Arc;
use async_trait::async_trait;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};

/// Logical IMPLIES operator evaluator
pub struct ImpliesOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl ImpliesOperatorEvaluator {
    /// Create a new IMPLIES operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_implies_metadata(),
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
impl OperationEvaluator for ImpliesOperatorEvaluator {
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

        // Three-valued logic for IMPLIES (A → B ≡ ¬A ∨ B):

        // If antecedent (left) is false, implication is true
        if left_bool == Some(false) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // If antecedent is true and consequent is true, implication is true
        if left_bool == Some(true) && right_bool == Some(true) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // If antecedent is true and consequent is false, implication is false
        if left_bool == Some(true) && right_bool == Some(false) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // If right is empty but left is not false, check for special case
        if right_bool.is_none() && left_bool == Some(true) {
            // true implies {} = {}
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // If left is empty but right is true, {} implies true = true
        if left_bool.is_none() && right_bool == Some(true) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // In all other cases involving empty collections, return empty
        Ok(EvaluationResult {
            value: Collection::empty(),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the IMPLIES operator
fn create_implies_metadata() -> OperatorMetadata {
    let signature = TypeSignature::new(
        vec![FhirPathType::Boolean, FhirPathType::Boolean],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "implies".to_string(),
        description: "Logical implication operation with three-valued logic".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Custom, // Custom logic for implication
        deterministic: true,
        precedence: 1, // FHIRPath IMPLIES precedence (lowest)
        associativity: Associativity::Right, // Right-associative per FHIRPath spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_implies_true_true() {
        let evaluator = ImpliesOperatorEvaluator::new();
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
    async fn test_implies_true_false() {
        let evaluator = ImpliesOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(false)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_implies_false_any() {
        let evaluator = ImpliesOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        // false implies true = true
        let left = vec![FhirPathValue::boolean(false)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));

        // false implies false = true
        let right = vec![FhirPathValue::boolean(false)];
        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_implies_empty_propagation() {
        let evaluator = ImpliesOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        // true implies {} = {}
        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![]; // Empty collection

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert!(result.value.is_empty());

        // {} implies true = true
        let left = vec![]; // Empty collection
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }
}