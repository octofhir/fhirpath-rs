//! Not equals (!=) operator implementation
//!
//! Implements FHIRPath inequality comparison.
//! The not equals operator returns the logical negation of the equals operator.

use async_trait::async_trait;
use std::sync::Arc;

use super::equals_operator::EqualsOperatorEvaluator;
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Not equals operator evaluator
pub struct NotEqualsOperatorEvaluator {
    metadata: OperatorMetadata,
    equals_evaluator: EqualsOperatorEvaluator,
}

impl Default for NotEqualsOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl NotEqualsOperatorEvaluator {
    /// Create a new not equals operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_not_equals_metadata(),
            equals_evaluator: EqualsOperatorEvaluator::new(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl OperationEvaluator for NotEqualsOperatorEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Use the equals evaluator and negate the result
        let equals_result = self
            .equals_evaluator
            .evaluate(input, context, left, right)
            .await?;

        // If equals returned empty, not equals also returns empty
        if equals_result.value.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Negate the boolean result
        if let Some(equals_bool) = equals_result.value.first().and_then(|v| v.as_boolean()) {
            Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(!equals_bool)),
            })
        } else {
            Ok(EvaluationResult {
                value: Collection::empty(),
            })
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the not equals operator
fn create_not_equals_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "!=".to_string(),
        description: "Inequality comparison (negation of equality)".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 5, // Same as equality
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_not_equals_different_values() {
        let evaluator = NotEqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::integer(43)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_not_equals_same_values() {
        let evaluator = NotEqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }
}
