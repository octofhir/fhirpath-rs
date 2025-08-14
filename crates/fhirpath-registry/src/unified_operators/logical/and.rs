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

//! Logical AND operator (and) implementation with enhanced metadata

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

/// Logical AND operator implementation
/// FHIRPath logical AND follows three-valued logic with empty collections
pub struct UnifiedAndOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedAndOperator {
    /// Create a new logical AND operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "and",
            OperatorCategory::Logical,
            3, // FHIRPath spec: and has precedence #03
            Associativity::Left,
        )
        .display_name("Logical AND")
        .description("Performs logical AND operation using FHIRPath three-valued logic.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("true and false", "Boolean AND (false)")
        .example("true and {}", "Boolean and empty collection (false)")
        .example("{} and true", "Empty collection and boolean (false)")
        .keywords(vec!["and", "logical", "boolean", "conjunction"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .short_circuits(true)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedAndOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedAndOperator {
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

impl LogicalOperator for UnifiedAndOperator {
    fn apply_logical(&self, left_bool: bool, right_bool: bool) -> bool {
        left_bool && right_bool
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_and_boolean_values() {
        let operator = UnifiedAndOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true and true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // true and false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // false and true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // false and false
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
    async fn test_and_with_empty_collections() {
        let operator = UnifiedAndOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true and empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // empty and true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // empty and empty
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
    async fn test_and_with_non_boolean_values() {
        let operator = UnifiedAndOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer values - should convert to boolean (truthy/falsy)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Both are truthy

        // String values
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("world".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Both non-empty strings are truthy

        // Mixed values
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Integer(0),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // true and false (0 is falsy)
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedAndOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "and");
        assert_eq!(metadata.basic.display_name, "Logical AND");
        assert_eq!(metadata.basic.precedence, 3);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Logical);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(operator.is_commutative());
        assert!(operator.can_short_circuit());
    }

    #[test]
    fn test_logical_operations() {
        let operator = UnifiedAndOperator::new();
        
        // Test the logic application
        assert!(operator.apply_logical(true, true));
        assert!(!operator.apply_logical(true, false));
        assert!(!operator.apply_logical(false, true));
        assert!(!operator.apply_logical(false, false));
    }
}