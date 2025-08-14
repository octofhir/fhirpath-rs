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

//! Logical OR operator (or) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::{LogicalOperator, UnifiedFhirPathOperator};
use crate::function::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Logical OR operator implementation
/// FHIRPath logical OR follows three-valued logic with empty collections
pub struct UnifiedOrOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedOrOperator {
    /// Create a new logical OR operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "or",
            OperatorCategory::Logical,
            2, // FHIRPath spec: or has precedence #02
            Associativity::Left,
        )
        .display_name("Logical OR")
        .description("Performs logical OR operation using FHIRPath three-valued logic.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("true or false", "Boolean OR (true)")
        .example("false or {}", "Boolean or empty collection (false)")
        .example("{} or false", "Empty collection or boolean (false)")
        .keywords(vec!["or", "logical", "boolean", "disjunction"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .short_circuits(true)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedOrOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedOrOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        self.evaluate_logical_binary(left, right, context).await
    }

    fn can_short_circuit(&self) -> bool {
        true
    }
}

impl LogicalOperator for UnifiedOrOperator {
    fn apply_logical(&self, left_bool: bool, right_bool: bool) -> bool {
        left_bool || right_bool
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_or_boolean_values() {
        let operator = UnifiedOrOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true or true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // true or false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // false or true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // false or false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_or_with_empty_collections() {
        let operator = UnifiedOrOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true or empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // empty or true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // false or empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // empty or empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_or_with_non_boolean_values() {
        let operator = UnifiedOrOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer values - should convert to boolean (truthy/falsy)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(0),
                FhirPathValue::Integer(1),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false or true

        // String values
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("".into()),
                FhirPathValue::String("hello".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false or true

        // Both falsy values
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(0),
                FhirPathValue::String("".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // false or false
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedOrOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "or");
        assert_eq!(metadata.basic.display_name, "Logical OR");
        assert_eq!(metadata.basic.precedence, 2);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Logical);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(operator.is_commutative());
        assert!(operator.can_short_circuit());
    }

    #[test]
    fn test_logical_operations() {
        let operator = UnifiedOrOperator::new();
        
        // Test the logic application
        assert!(operator.apply_logical(true, true));
        assert!(operator.apply_logical(true, false));
        assert!(operator.apply_logical(false, true));
        assert!(!operator.apply_logical(false, false));
    }
}