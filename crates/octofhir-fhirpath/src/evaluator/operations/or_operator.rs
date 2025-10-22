//! Logical OR operator implementation
//!
//! Implements three-valued logic for the FHIRPath 'or' operator.
//! Truth table:
//! - true or true = true
//! - true or false = true (short-circuit)
//! - false or true = true (short-circuit)
//! - false or false = false
//! - true or {} = true (short-circuit)
//! - false or {} = {}
//! - {} or true = true (short-circuit)
//! - {} or false = {}
//! - {} or {} = {}

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Logical OR operator evaluator
pub struct OrOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for OrOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl OrOperatorEvaluator {
    /// Create a new OR operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_or_metadata(),
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
impl OperationEvaluator for OrOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Extract boolean values from both operands
        let left_bool = self.extract_boolean(&left);
        let right_bool = self.extract_boolean(&right);

        // Three-valued logic for OR:
        // If either operand is true, result is true (short-circuit)
        if left_bool == Some(true) || right_bool == Some(true) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // If both operands are false, result is false
        if left_bool == Some(false) && right_bool == Some(false) {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
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

/// Create metadata for the OR operator
fn create_or_metadata() -> OperatorMetadata {
    let signature = TypeSignature::new(
        vec![FhirPathType::Boolean, FhirPathType::Boolean],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "or".to_string(),
        description: "Logical OR operation with three-valued logic".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Custom, // Custom logic for short-circuiting
        deterministic: true,
        precedence: 2, // FHIRPath OR precedence (lower than AND)
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_or_true_true() {
        let evaluator = OrOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

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
    async fn test_or_true_false() {
        let evaluator = OrOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(false)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_or_false_false() {
        let evaluator = OrOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::boolean(false)];
        let right = vec![FhirPathValue::boolean(false)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_or_true_short_circuit() {
        let evaluator = OrOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_or_empty_propagation() {
        let evaluator = OrOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::boolean(false)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert!(result.value.is_empty());
    }
}
