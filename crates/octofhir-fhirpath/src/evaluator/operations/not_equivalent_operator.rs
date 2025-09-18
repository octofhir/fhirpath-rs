//! Not Equivalent (!~) operator implementation
//!
//! Implements FHIRPath not equivalence comparison which is the logical negation
//! of the equivalence operator.

use async_trait::async_trait;
use std::sync::Arc;

use super::equivalent_operator::EquivalentOperatorEvaluator;
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Not Equivalent operator evaluator
pub struct NotEquivalentOperatorEvaluator {
    metadata: OperatorMetadata,
    equivalent_evaluator: EquivalentOperatorEvaluator,
}

impl NotEquivalentOperatorEvaluator {
    /// Create a new not equivalent operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_not_equivalent_metadata(),
            equivalent_evaluator: EquivalentOperatorEvaluator::new(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl OperationEvaluator for NotEquivalentOperatorEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Use the equivalent evaluator and negate the result
        let equivalent_result = self
            .equivalent_evaluator
            .evaluate(input, context, left, right)
            .await?;

        if equivalent_result.value.is_empty() {
            // If equivalent returns empty, not equivalent also returns empty
            Ok(EvaluationResult {
                value: Collection::empty(),
            })
        } else {
            // Negate the boolean result
            let equivalent_bool = equivalent_result
                .value
                .first()
                .and_then(|v| v.as_boolean())
                .unwrap_or(false);

            Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(!equivalent_bool)),
            })
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the not equivalent operator
fn create_not_equivalent_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "!~".to_string(),
        description: "Not equivalence comparison (negation of equivalence)".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Custom, // Same as equivalence
        deterministic: true,
        precedence: 5, // FHIRPath equivalence precedence (same as equality)
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_not_equivalent_boolean_same() {
        let evaluator = NotEquivalentOperatorEvaluator::new();
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
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_not_equivalent_boolean_different() {
        let evaluator = NotEquivalentOperatorEvaluator::new();
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
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_not_equivalent_strings_case_insensitive() {
        let evaluator = NotEquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![FhirPathValue::string("HELLO".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false)); // They are equivalent, so not equivalent is false
    }

    #[tokio::test]
    async fn test_not_equivalent_strings_different() {
        let evaluator = NotEquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![FhirPathValue::string("World".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_not_equivalent_both_empty() {
        let evaluator = NotEquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![];
        let right = vec![];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false)); // Both empty are equivalent, so not equivalent is false
    }

    #[tokio::test]
    async fn test_not_equivalent_one_empty() {
        let evaluator = NotEquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true)); // One empty, one not - not equivalent
    }
}
