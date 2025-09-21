//! In operator implementation
//!
//! Implements FHIRPath 'in' operator for membership testing.
//! Returns true if the left operand is a member of the right operand collection.

use async_trait::async_trait;
use std::sync::Arc;

use super::equals_operator::EqualsOperatorEvaluator;
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// In operator evaluator
pub struct InOperatorEvaluator {
    metadata: OperatorMetadata,
    equals_evaluator: EqualsOperatorEvaluator,
}

impl Default for InOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl InOperatorEvaluator {
    /// Create a new in operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_in_metadata(),
            equals_evaluator: EqualsOperatorEvaluator::new(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Check if a value is in a collection using equality semantics
    async fn is_member_of(
        &self,
        needle: &FhirPathValue,
        haystack: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<bool> {
        for item in haystack {
            // Use the equals operator to compare values
            let needle_vec = vec![needle.clone()];
            let item_vec = vec![item.clone()];

            let equals_result = self
                .equals_evaluator
                .evaluate(vec![], context, needle_vec, item_vec)
                .await?;

            // If equals returns true, we found a match
            if let Some(result_value) = equals_result.value.first() {
                if let Some(true) = result_value.as_boolean() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

#[async_trait]
impl OperationEvaluator for InOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if left is empty, result is empty
        // If right is empty, result is false (nothing can be in an empty collection)
        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        if right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // For 'in' operator, the left operand must be a single value, not a collection
        if left.len() > 1 {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "Left operand of 'in' operator must be a single value, not a collection"
                    .to_string(),
            ));
        }

        let needle = left.first().unwrap();

        // Handle different right-hand side types
        match right.first().unwrap() {
            FhirPathValue::Collection(collection) => {
                // Right side is a collection, check membership
                let is_member = self
                    .is_member_of(needle, collection.values(), context)
                    .await?;
                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(is_member)),
                })
            }
            _ => {
                // Right side is a single value, treat it as a collection of one
                let is_member = self.is_member_of(needle, &right, context).await?;
                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(is_member)),
                })
            }
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the in operator
fn create_in_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![
            FhirPathType::Any,
            FhirPathType::Collection(Box::new(FhirPathType::Any)),
        ],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "in".to_string(),
        description: "Membership test - returns true if left operand is in right collection"
            .to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                // Value in collection
                TypeSignature::new(
                    vec![
                        FhirPathType::Any,
                        FhirPathType::Collection(Box::new(FhirPathType::Any)),
                    ],
                    FhirPathType::Boolean,
                ),
                // Value in single value (treated as collection of one)
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
    async fn test_in_integer_collection() {
        let evaluator = InOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(2)];
        let right = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
            FhirPathValue::integer(3),
        ]))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_in_integer_not_in_collection() {
        let evaluator = InOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(4)];
        let right = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
            FhirPathValue::integer(3),
        ]))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_in_string_collection() {
        let evaluator = InOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("hello".to_string())];
        let right = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::string("world".to_string()),
            FhirPathValue::string("hello".to_string()),
            FhirPathValue::string("test".to_string()),
        ]))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_in_single_value() {
        let evaluator = InOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::integer(42)]; // Single value, not a collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_in_empty_left() {
        let evaluator = InOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![]; // Empty
        let right = vec![FhirPathValue::Collection(Collection::from_values(vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
        ]))];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert!(result.value.is_empty()); // Empty propagation
    }

    #[tokio::test]
    async fn test_in_empty_right() {
        let evaluator = InOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
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
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false)); // Nothing is in empty collection
    }
}
