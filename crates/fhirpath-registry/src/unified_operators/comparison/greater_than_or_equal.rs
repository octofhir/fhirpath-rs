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

//! Greater than or equal operator (>=) implementation with enhanced metadata

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

/// Greater than or equal operator (>=) implementation
pub struct UnifiedGreaterThanOrEqualOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedGreaterThanOrEqualOperator {
    /// Create a new greater than or equal operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            ">=",
            OperatorCategory::Comparison,
            8, // FHIRPath spec: >, <, >=, <= have precedence #08
            Associativity::Left,
        )
        .display_name("Greater Than or Equal")
        .description("Tests if the left operand is greater than or equal to the right operand.")
        .complexity(OperatorComplexity::TypeDependent)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("5 >= 3", "Integer comparison (true)")
        .example("3 >= 3", "Equal values (true)")
        .example("'b' >= 'a'", "String comparison (true)")
        .keywords(vec!["greater", "equal", "comparison", "order", "gte"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedGreaterThanOrEqualOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedGreaterThanOrEqualOperator {
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

impl ComparisonOperator for UnifiedGreaterThanOrEqualOperator {
    fn compare(&self, ordering: std::cmp::Ordering) -> bool {
        match ordering {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Equal => true,
            std::cmp::Ordering::Less => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_greater_than_or_equal_integers() {
        let operator = UnifiedGreaterThanOrEqualOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Greater than case
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(7),
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

        // Less than case
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_greater_than_or_equal_decimals() {
        let operator = UnifiedGreaterThanOrEqualOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        use rust_decimal::Decimal;

        // Greater than
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(35, 1)), // 3.5
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

        // Less than
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(25, 1)), // 2.5
                FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_greater_than_or_equal_strings() {
        let operator = UnifiedGreaterThanOrEqualOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Greater than (lexicographic)
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("cherry".into()),
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

        // Less than
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("apple".into()),
                FhirPathValue::String("banana".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedGreaterThanOrEqualOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, ">=");
        assert_eq!(metadata.basic.display_name, "Greater Than or Equal");
        assert_eq!(metadata.basic.precedence, 8);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Comparison);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative());
    }
}