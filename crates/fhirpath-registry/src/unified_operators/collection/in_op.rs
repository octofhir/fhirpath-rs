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

//! In 'in' operator implementation with enhanced metadata

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

/// In 'in' operator implementation
/// Tests whether a value is a member of a collection according to FHIRPath semantics
pub struct UnifiedInOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedInOperator {
    /// Create a new 'in' operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "in",
            OperatorCategory::Collection,
            10, // FHIRPath spec: in, contains have precedence #10
            Associativity::Left,
        )
        .display_name("Membership (in)")
        .description("Tests whether a value is a member of a collection using FHIRPath equality semantics.")
        .complexity(OperatorComplexity::Linear)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("5 in [1, 2, 5]", "Membership test (true)")
        .example("'hello' in ['world', 'test']", "Membership test (false)")
        .example("{} in [1, 2]", "Empty operand (empty)")
        .keywords(vec!["in", "membership", "contains", "member", "element"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedInOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedInOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Per FHIRPath spec for 'in' operator:
        // - If left operand is empty, return empty
        // - If right operand is empty, return false
        // - If left operand has multiple items, return empty
        // - Otherwise test membership

        if left.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if right.is_empty() {
            return Ok(FhirPathValue::Boolean(false));
        }

        let left_collection = left.to_collection();
        let right_collection = right.to_collection();

        // If left has multiple items, return empty (based on FHIRPath spec)
        if left_collection.len() > 1 {
            return Ok(FhirPathValue::Empty);
        }

        // Single item test
        if let Some(single_item) = left_collection.first() {
            let is_member = right_collection.contains(single_item);
            Ok(FhirPathValue::Boolean(is_member))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};

    #[tokio::test]
    async fn test_in_basic_membership() {
        let operator = UnifiedInOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 5 in [1, 2, 5] = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // 3 in [1, 2, 5] = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(3),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_in_with_strings() {
        let operator = UnifiedInOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 'hello' in ['hello', 'world'] = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::String("hello".into()),
                    FhirPathValue::String("world".into()),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // 'test' in ['hello', 'world'] = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("test".into()),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::String("hello".into()),
                    FhirPathValue::String("world".into()),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_in_with_empty() {
        let operator = UnifiedInOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // {} in [1, 2] = {} (empty)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5 in {} = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // {} in {} = {} (empty)
        let result = operator
            .evaluate_binary(FhirPathValue::Empty, FhirPathValue::Empty, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_in_with_multi_item_left() {
        let operator = UnifiedInOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // [1, 2] in [1, 2, 3] = {} (empty, multi-item left operand)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                ])),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(3),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_in_single_item_collections() {
        let operator = UnifiedInOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Single item in collection: [5] in [1, 2, 5] = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Collection(Collection::from_vec(vec![FhirPathValue::Integer(5)])),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_in_mixed_types() {
        let operator = UnifiedInOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Mixed types: 5 in [1, 'hello', 5] = true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::String("hello".into()),
                    FhirPathValue::Integer(5),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Type mismatch: 'hello' in [1, 2, 5] = false
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::Collection(Collection::from_vec(vec![
                    FhirPathValue::Integer(1),
                    FhirPathValue::Integer(2),
                    FhirPathValue::Integer(5),
                ])),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedInOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "in");
        assert_eq!(metadata.basic.display_name, "Membership (in)");
        assert_eq!(metadata.basic.precedence, 10);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Collection);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // 'in' is not commutative (order matters)
    }
}