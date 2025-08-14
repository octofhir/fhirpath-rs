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

//! Equivalent operator (~) implementation with enhanced metadata

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

/// Equivalent operator (~) implementation
/// FHIRPath equivalence differs from equality in its handling of empty collections and null values
pub struct UnifiedEquivalentOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedEquivalentOperator {
    /// Create a new equivalent operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "~",
            OperatorCategory::Comparison,
            6, // FHIRPath spec: ~ and !~ have precedence #06
            Associativity::Left,
        )
        .display_name("Equivalent")
        .description("Tests for equivalence between two values, using FHIRPath equivalence semantics.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Linear)
        .example("5 ~ 5", "Integer equivalence (true)")
        .example("{} ~ {}", "Empty collections are equivalent (true)")
        .example("5 ~ {}", "Value vs empty collection (false)")
        .keywords(vec!["equivalent", "similar", "comparison", "fuzzy", "equivalence"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }

    /// Evaluate equivalence between two FHIRPath values
    /// FHIRPath equivalence rules:
    /// - Empty collections are equivalent to other empty collections
    /// - Empty collections are not equivalent to non-empty collections  
    /// - For non-empty collections, equivalence follows value-specific rules
    fn evaluate_equivalence_sync(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> bool {
        use FhirPathValue::*;

        match (left, right) {
            // Empty values
            (Empty, Empty) => true,
            (Empty, _) | (_, Empty) => false,

            // Collections
            (Collection(left_items), Collection(right_items)) => {
                if left_items.is_empty() && right_items.is_empty() {
                    true
                } else if left_items.is_empty() || right_items.is_empty() {
                    false
                } else if left_items.len() == 1 && right_items.len() == 1 {
                    // Single-item collections - compare the items
                    let left_item = left_items.iter().next().unwrap();
                    let right_item = right_items.iter().next().unwrap();
                    self.evaluate_equivalence_sync(left_item, right_item)
                } else {
                    // Multi-item collections require all items to be equivalent
                    if left_items.len() != right_items.len() {
                        false
                    } else {
                        left_items.iter().zip(right_items.iter())
                            .all(|(l, r)| self.evaluate_equivalence_sync(l, r))
                    }
                }
            }

            (Collection(items), other) | (other, Collection(items)) => {
                if items.is_empty() {
                    false // Empty collection not equivalent to single value
                } else if items.len() == 1 {
                    // Single-item collection - compare with the other value
                    let item = items.iter().next().unwrap();
                    self.evaluate_equivalence_sync(item, other)
                } else {
                    false // Multi-item collection not equivalent to single value
                }
            }

            // Numeric equivalence
            (Integer(l), Integer(r)) => l == r,
            (Decimal(l), Decimal(r)) => l == r,
            (Integer(l), Decimal(r)) => rust_decimal::Decimal::from(*l) == *r,
            (Decimal(l), Integer(r)) => *l == rust_decimal::Decimal::from(*r),

            // String equivalence
            (String(l), String(r)) => l == r,

            // Boolean equivalence
            (Boolean(l), Boolean(r)) => l == r,

            // Date/Time equivalence
            (Date(l), Date(r)) => l == r,
            (DateTime(l), DateTime(r)) => l == r,
            (Time(l), Time(r)) => l == r,

            // Quantity equivalence (same value and unit)
            (Quantity(lq), Quantity(rq)) => {
                lq.value == rq.value && lq.unit == rq.unit
            }

            // Different types are not equivalent
            _ => false,
        }
    }
}

impl Default for UnifiedEquivalentOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedEquivalentOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let result = self.evaluate_equivalence_sync(&left, &right);
        Ok(FhirPathValue::Boolean(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_equivalent_empty_collections() {
        let operator = UnifiedEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Empty ~ Empty should be true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Empty ~ non-empty should be false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_equivalent_integers() {
        let operator = UnifiedEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Same integers
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Different integers
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
    async fn test_equivalent_mixed_numeric() {
        let operator = UnifiedEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer and equivalent decimal
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::new(5, 0)), // 5.0
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Integer and non-equivalent decimal
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::new(51, 1)), // 5.1
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_equivalent_strings() {
        let operator = UnifiedEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Same strings
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Different strings
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("world".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_equivalent_different_types() {
        let operator = UnifiedEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer and string (different types)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("5".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Boolean and integer
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Integer(1),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedEquivalentOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "~");
        assert_eq!(metadata.basic.display_name, "Equivalent");
        assert_eq!(metadata.basic.precedence, 6);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Comparison);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(operator.is_commutative());
    }
}