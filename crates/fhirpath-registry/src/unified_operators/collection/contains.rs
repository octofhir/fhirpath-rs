// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Contains 'contains' operator implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::UnifiedFhirPathOperator;
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Contains 'contains' operator implementation
/// Tests whether a collection contains a specific value according to FHIRPath semantics
pub struct UnifiedContainsOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedContainsOperator {
    /// Create a new 'contains' operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "contains",
            OperatorCategory::Collection,
            10, // FHIRPath spec: in, contains have precedence #10
            Associativity::Left,
        )
        .display_name("Contains")
        .description("Tests whether a collection contains a specific value using FHIRPath equality semantics.")
        .complexity(OperatorComplexity::Linear)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("[1, 2, 5] contains 5", "Contains test (true)")
        .example("['hello', 'world'] contains 'test'", "Contains test (false)")
        .example("[1, 2] contains {}", "Empty operand (empty)")
        .keywords(vec!["contains", "includes", "has", "member", "element"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedContainsOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedContainsOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Per FHIRPath spec for 'contains' operator:
        // - If both operands are empty, return empty
        // - If left operand is empty (but right is not), return false  
        // - If right operand is empty (but left is not), return empty
        // - Otherwise test containment

        if left.is_empty() && right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if left.is_empty() {
            return Ok(FhirPathValue::Boolean(false));
        }

        if right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let left_collection = left.to_collection();
        let contains_value = left_collection.contains(&right);
        
        Ok(FhirPathValue::Boolean(contains_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};

    #[tokio::test]
    async fn test_contains_basic() {
        let operator = UnifiedContainsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // [1, 2, 5] contains 5 = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // [1, 2, 5] contains 3 = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_contains_with_strings() {
        let operator = UnifiedContainsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // ['hello', 'world'] contains 'hello' = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::String("hello".into()),
                    FhirPathValue::String("world".into()),
                ])),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // ['hello', 'world'] contains 'test' = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::String("hello".into()),
                    FhirPathValue::String("world".into()),
                ])),
                FhirPathValue::String("test".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_contains_with_empty() {
        let operator = UnifiedContainsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // {} contains 5 = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // [1, 2] contains {} = {} (empty)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                ])),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // {} contains {} = {} (empty)
        let result = operator
            .evaluate_binary(FhirPathValue::Empty, FhirPathValue::Empty, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_contains_single_values() {
        let operator = UnifiedContainsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Single value treated as collection: 5 contains 5 = true  
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Single value mismatch: 5 contains 3 = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_contains_mixed_types() {
        let operator = UnifiedContainsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Mixed types: [1, 'hello', 5] contains 'hello' = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::String("hello".into()),
                    FhirPathValue::Integer(5),
                ])),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Type mismatch: [1, 2, 5] contains 'hello' = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_contains_with_collections() {
        let operator = UnifiedContainsOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Collection contains collection (exact match): [[1, 2], 3] contains [1, 2] = true
        let inner_collection = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]));
        
        let outer_collection = FhirPathValue::Collection(Collection::from_vec(vec![
            inner_collection.clone(),
            FhirPathValue::Integer(3),
        ]));

        let result = operator
            .evaluate_binary(outer_collection, inner_collection, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedContainsOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "contains");
        assert_eq!(metadata.basic.display_name, "Contains");
        assert_eq!(metadata.basic.precedence, 10);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Collection);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // 'contains' is not commutative (order matters)
    }
}