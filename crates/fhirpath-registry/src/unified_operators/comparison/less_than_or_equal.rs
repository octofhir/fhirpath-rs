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

//! Less than or equal operator (<=) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::{ComparisonOperator, UnifiedFhirPathOperator};
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Less than or equal operator (<=) implementation
pub struct UnifiedLessThanOrEqualOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedLessThanOrEqualOperator {
    /// Create a new less than or equal operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "<=",
            OperatorCategory::Comparison,
            8, // FHIRPath spec: >, <, >=, <= have precedence #08
            Associativity::Left,
        )
        .display_name("Less Than or Equal")
        .description("Tests if the left operand is less than or equal to the right operand.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("3 <= 5", "Integer comparison (true)")
        .example("3 <= 3", "Equal values (true)")
        .example("'a' <= 'b'", "String comparison (true)")
        .keywords(vec!["less", "equal", "comparison", "order", "lte"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedLessThanOrEqualOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedLessThanOrEqualOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        self.evaluate_comparison_binary(left, right, context).await
    }
}

impl ComparisonOperator for UnifiedLessThanOrEqualOperator {
    fn compare(&self, ordering: std::cmp::Ordering) -> bool {
        match ordering {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Equal => true,
            std::cmp::Ordering::Greater => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_less_than_or_equal_integers() {
        let operator = UnifiedLessThanOrEqualOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Less than case
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Equal case
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Greater than case
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(7),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_less_than_or_equal_decimals() {
        let operator = UnifiedLessThanOrEqualOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        use rust_decimal::Decimal;

        // Less than
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(25, 1)), // 2.5
                FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Equal
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
                FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Greater than
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(35, 1)), // 3.5
                FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_less_than_or_equal_strings() {
        let operator = UnifiedLessThanOrEqualOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Less than (lexicographic)
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("apple".into()),
                FhirPathValue::String("banana".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Equal
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("apple".into()),
                FhirPathValue::String("apple".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Greater than
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("cherry".into()),
                FhirPathValue::String("banana".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedLessThanOrEqualOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "<=");
        assert_eq!(metadata.basic.display_name, "Less Than or Equal");
        assert_eq!(metadata.basic.precedence, 8);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Comparison);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative());
    }
}