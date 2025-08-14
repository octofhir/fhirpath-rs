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

//! Union '|' operator implementation with enhanced metadata

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

/// Union '|' operator implementation
/// Merges two collections, removing duplicates according to FHIRPath semantics
pub struct UnifiedUnionOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedUnionOperator {
    /// Create a new union operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "|",
            OperatorCategory::Collection,
            7, // FHIRPath spec: | has precedence #7
            Associativity::Left,
        )
        .display_name("Union (|)")
        .description("Merges two collections, eliminating duplicates while preserving order.")
        .complexity(OperatorComplexity::Linear)
        .memory_usage(OperatorMemoryUsage::Linear)
        .example("[1, 2] | [2, 3]", "Collection union ([1, 2, 3])")
        .example("{} | [1, 2]", "Union with empty ([1, 2])")
        .example("5 | [1, 2]", "Single value union ([5, 1, 2])")
        .keywords(vec!["union", "merge", "combine", "collect", "|"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedUnionOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedUnionOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Convert both sides to collections
        let mut left_collection = left.to_collection();
        let right_collection = right.to_collection();
        
        // Add items from right collection, avoiding duplicates
        for item in right_collection.iter() {
            if !left_collection.contains(item) {
                left_collection.push(item.clone());
            }
        }
        
        // Return as collection (will auto-unwrap single items)
        if left_collection.len() == 1 {
            Ok(left_collection.into_iter().next().unwrap())
        } else if left_collection.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::Collection(left_collection))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};

    #[tokio::test]
    async fn test_union_basic() {
        let operator = UnifiedUnionOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Basic union: [1, 2] | [3, 4] = [1, 2, 3, 4]
        let left = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]));
        let right = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]));

        let result = operator
            .evaluate_binary(left, right, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items.iter().nth(0).unwrap(), &FhirPathValue::Integer(1));
                assert_eq!(items.iter().nth(1).unwrap(), &FhirPathValue::Integer(2));
                assert_eq!(items.iter().nth(2).unwrap(), &FhirPathValue::Integer(3));
                assert_eq!(items.iter().nth(3).unwrap(), &FhirPathValue::Integer(4));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_union_with_duplicates() {
        let operator = UnifiedUnionOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Union with duplicates: [1, 2] | [2, 3] = [1, 2, 3]
        let left = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]));
        let right = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));

        let result = operator
            .evaluate_binary(left, right, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items.iter().nth(0).unwrap(), &FhirPathValue::Integer(1));
                assert_eq!(items.iter().nth(1).unwrap(), &FhirPathValue::Integer(2));
                assert_eq!(items.iter().nth(2).unwrap(), &FhirPathValue::Integer(3));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_union_with_empty() {
        let operator = UnifiedUnionOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Union with empty: {} | [1, 2] = [1, 2]
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

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items.iter().nth(0).unwrap(), &FhirPathValue::Integer(1));
                assert_eq!(items.iter().nth(1).unwrap(), &FhirPathValue::Integer(2));
            }
            _ => panic!("Expected collection result"),
        }

        // Union empty with empty: {} | {} = {}
        let result = operator
            .evaluate_binary(FhirPathValue::Empty, FhirPathValue::Empty, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_union_single_values() {
        let operator = UnifiedUnionOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Single value union: 5 | 3 = [5, 3]
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items.iter().nth(0).unwrap(), &FhirPathValue::Integer(5));
                assert_eq!(items.iter().nth(1).unwrap(), &FhirPathValue::Integer(3));
            }
            _ => panic!("Expected collection result"),
        }

        // Single value with duplicate: 5 | 5 = 5 (unwrapped)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));
    }

    #[tokio::test]
    async fn test_union_mixed_types() {
        let operator = UnifiedUnionOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Mixed type union: [1, "hello"] | [2, "world"] = [1, "hello", 2, "world"]
        let left = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::String("hello".into()),
        ]));
        let right = FhirPathValue::Collection(Collection::from_vec(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::String("world".into()),
        ]));

        let result = operator
            .evaluate_binary(left, right, &context)
            .await
            .unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 4);
                assert_eq!(items.iter().nth(0).unwrap(), &FhirPathValue::Integer(1));
                assert_eq!(items.iter().nth(1).unwrap(), &FhirPathValue::String("hello".into()));
                assert_eq!(items.iter().nth(2).unwrap(), &FhirPathValue::Integer(2));
                assert_eq!(items.iter().nth(3).unwrap(), &FhirPathValue::String("world".into()));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedUnionOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "|");
        assert_eq!(metadata.basic.display_name, "Union (|)");
        assert_eq!(metadata.basic.precedence, 7);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Collection);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(operator.is_commutative()); // Union is commutative (order doesn't matter for final result)
    }
}