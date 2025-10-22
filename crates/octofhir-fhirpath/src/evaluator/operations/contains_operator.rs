//! Contains operator implementation
//!
//! Implements FHIRPath 'contains' operator for membership testing.
//! Returns true if the left operand collection contains the right operand.

use async_trait::async_trait;
use std::sync::Arc;

use super::equals_operator::EqualsOperatorEvaluator;
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Contains operator evaluator
pub struct ContainsOperatorEvaluator {
    metadata: OperatorMetadata,
    equals_evaluator: EqualsOperatorEvaluator,
}

impl Default for ContainsOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainsOperatorEvaluator {
    /// Create a new contains operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_contains_metadata(),
            equals_evaluator: EqualsOperatorEvaluator::new(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Check if a collection contains a value using equality semantics
    async fn contains_member(
        &self,
        haystack: &[FhirPathValue],
        needle: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<bool> {
        for item in haystack {
            // Use the equals operator to compare values
            let item_vec = vec![item.clone()];
            let needle_vec = vec![needle.clone()];

            let equals_result = self
                .equals_evaluator
                .evaluate(vec![], context, item_vec, needle_vec)
                .await?;

            // If equals returns true, we found a match
            if let Some(result_value) = equals_result.value.first()
                && let Some(true) = result_value.as_boolean()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[async_trait]
impl OperationEvaluator for ContainsOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if right is empty, result is empty
        // If left is empty, result is false (empty collection contains nothing)
        if right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // For 'contains' operator, we check if the left collection contains the right value
        let needle = right.first().unwrap();

        // Handle different left-hand side types
        match left.first().unwrap() {
            FhirPathValue::Collection(collection) => {
                // Left side is a collection, check if it contains the needle
                let contains_member = self
                    .contains_member(collection.values(), needle, context)
                    .await?;
                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(contains_member)),
                })
            }
            _ => {
                // Left side is a single value, treat it as a collection of one
                let contains_member = self.contains_member(&left, needle, context).await?;
                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(contains_member)),
                })
            }
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the contains operator
fn create_contains_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![
            FhirPathType::Collection(Box::new(FhirPathType::Any)),
            FhirPathType::Any,
        ],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "contains".to_string(),
        description: "Containment test - returns true if left collection contains right operand"
            .to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                // Collection contains value
                TypeSignature::new(
                    vec![
                        FhirPathType::Collection(Box::new(FhirPathType::Any)),
                        FhirPathType::Any,
                    ],
                    FhirPathType::Boolean,
                ),
                // Single value contains value (treated as collection of one)
                TypeSignature::new(
                    vec![FhirPathType::Any, FhirPathType::Any],
                    FhirPathType::Boolean,
                ),
            ],
        },
        empty_propagation: EmptyPropagation::Custom, // Custom empty handling
        deterministic: true,
        precedence: 6, // FHIRPath membership precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_contains_integer() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
            FhirPathValue::integer(3),
        ]))];
        let right = vec![FhirPathValue::integer(2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_contains_integer_not_found() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
            FhirPathValue::integer(3),
        ]))];
        let right = vec![FhirPathValue::integer(4)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_contains_string() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::string("world".to_string()),
            FhirPathValue::string("hello".to_string()),
            FhirPathValue::string("test".to_string()),
        ]))];
        let right = vec![FhirPathValue::string("hello".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_contains_single_value() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::integer(42)]; // Single value, not a collection
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_contains_single_value_not_found() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::integer(42)]; // Single value, not a collection
        let right = vec![FhirPathValue::integer(24)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_contains_empty_left() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![]; // Empty collection
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false)); // Empty collection contains nothing
    }

    #[tokio::test]
    async fn test_contains_empty_right() {
        let evaluator = ContainsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        );

        let left = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
        ]))];
        let right = vec![]; // Empty

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert!(result.value.is_empty()); // Empty propagation
    }
}
