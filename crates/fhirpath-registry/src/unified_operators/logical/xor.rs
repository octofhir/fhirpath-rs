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

//! Logical XOR operator (xor) implementation with enhanced metadata

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

/// Logical XOR (exclusive or) operator implementation
/// FHIRPath logical XOR follows three-valued logic with empty collections
pub struct UnifiedXorOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedXorOperator {
    /// Create a new logical XOR operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "xor",
            OperatorCategory::Logical,
            4, // FHIRPath spec: xor has precedence #04 (between and/or)
            Associativity::Left,
        )
        .display_name("Logical XOR")
        .description("Performs logical exclusive OR operation using FHIRPath three-valued logic.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("true xor false", "Boolean XOR (true)")
        .example("true xor true", "Boolean XOR (false)")
        .example("false xor {}", "Boolean xor empty collection (false)")
        .keywords(vec!["xor", "exclusive", "or", "logical", "boolean"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedXorOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedXorOperator {
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
        false // XOR cannot short-circuit as it needs both operands
    }
}

impl LogicalOperator for UnifiedXorOperator {
    fn apply_logical(&self, left_bool: bool, right_bool: bool) -> bool {
        left_bool != right_bool // XOR is true when operands differ
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_xor_boolean_values() {
        let operator = UnifiedXorOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true xor true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // true xor false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // false xor true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // false xor false
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
    async fn test_xor_with_empty_collections() {
        let operator = UnifiedXorOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true xor empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // true xor false

        // false xor empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // false xor false

        // empty xor empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // false xor false
    }

    #[tokio::test]
    async fn test_xor_with_non_boolean_values() {
        let operator = UnifiedXorOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer values - should convert to boolean (truthy/falsy)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(0),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // true xor false

        // String values
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::String("world".into()),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // true xor true

        // Mixed values - different truthiness
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Integer(0),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // true xor false (0 is falsy)

        // Mixed values - same truthiness
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Integer(1),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // true xor true
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedXorOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "xor");
        assert_eq!(metadata.basic.display_name, "Logical XOR");
        assert_eq!(metadata.basic.precedence, 4);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Logical);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(operator.is_commutative());
        assert!(!operator.can_short_circuit());
    }

    #[test]
    fn test_xor_logic() {
        let operator = UnifiedXorOperator::new();
        
        // Test the logic application
        assert!(!operator.apply_logical(true, true));   // same -> false
        assert!(operator.apply_logical(true, false));   // different -> true
        assert!(operator.apply_logical(false, true));   // different -> true
        assert!(!operator.apply_logical(false, false)); // same -> false
    }
}