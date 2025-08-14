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

//! Not equivalent operator (!~) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::UnifiedFhirPathOperator;
use crate::unified_operators::comparison::equivalent::UnifiedEquivalentOperator;
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Not equivalent operator (!~) implementation
/// This is the logical negation of the equivalent operator (~)
pub struct UnifiedNotEquivalentOperator {
    metadata: EnhancedOperatorMetadata,
    equivalent_operator: UnifiedEquivalentOperator,
}

impl UnifiedNotEquivalentOperator {
    /// Create a new not equivalent operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "!~",
            OperatorCategory::Comparison,
            6, // FHIRPath spec: ~ and !~ have precedence #06
            Associativity::Left,
        )
        .display_name("Not Equivalent")
        .description("Tests for non-equivalence between two values, using FHIRPath equivalence semantics.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Linear)
        .example("5 !~ 3", "Different integers (true)")
        .example("{} !~ {}", "Empty collections are equivalent (false)")
        .example("5 !~ {}", "Value vs empty collection (true)")
        .keywords(vec!["not", "equivalent", "different", "comparison", "negation"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self {
            metadata,
            equivalent_operator: UnifiedEquivalentOperator::new(),
        }
    }
}

impl Default for UnifiedNotEquivalentOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedNotEquivalentOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Evaluate equivalence and negate the result
        let equivalent_result = self.equivalent_operator
            .evaluate_binary(left, right, context)
            .await?;

        match equivalent_result {
            FhirPathValue::Boolean(value) => Ok(FhirPathValue::Boolean(!value)),
            _ => Ok(FhirPathValue::Boolean(false)), // Should not happen for well-formed equivalence
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_not_equivalent_empty_collections() {
        let operator = UnifiedNotEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Empty !~ Empty should be false (they are equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Empty !~ non-empty should be true (they are not equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_not_equivalent_integers() {
        let operator = UnifiedNotEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Same integers should be false (they are equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Different integers should be true (they are not equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_not_equivalent_mixed_numeric() {
        let operator = UnifiedNotEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer and equivalent decimal should be false (they are equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::new(5, 0)), // 5.0
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Integer and non-equivalent decimal should be true (they are not equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::new(51, 1)), // 5.1
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_not_equivalent_strings() {
        let operator = UnifiedNotEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Same strings should be false (they are equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Different strings should be true (they are not equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("world".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_not_equivalent_different_types() {
        let operator = UnifiedNotEquivalentOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer and string (different types) should be true (not equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::String("5".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Boolean and integer should be true (not equivalent)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Integer(1),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedNotEquivalentOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "!~");
        assert_eq!(metadata.basic.display_name, "Not Equivalent");
        assert_eq!(metadata.basic.precedence, 6);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Comparison);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(operator.is_commutative());
    }
}