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

//! Logical IMPLIES operator (implies) implementation with enhanced metadata

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

/// Logical IMPLIES operator implementation
/// FHIRPath logical IMPLIES follows three-valued logic: A implies B is equivalent to (not A) or B
pub struct UnifiedImpliesOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedImpliesOperator {
    /// Create a new logical IMPLIES operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "implies",
            OperatorCategory::Logical,
            1, // FHIRPath spec: implies has lowest precedence #01
            Associativity::Right, // Right-associative: A implies B implies C = A implies (B implies C)
        )
        .display_name("Logical IMPLIES")
        .description("Performs logical implication using FHIRPath three-valued logic. A implies B is equivalent to (not A) or B.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("true implies false", "Implication (false)")
        .example("false implies true", "Implication (true)")
        .example("false implies false", "Implication (true)")
        .keywords(vec!["implies", "implication", "logical", "conditional"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .short_circuits(true)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedImpliesOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedImpliesOperator {
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

impl LogicalOperator for UnifiedImpliesOperator {
    fn apply_logical(&self, left_bool: bool, right_bool: bool) -> bool {
        // A implies B is equivalent to (not A) or B
        // In other words, implication is false only when A is true and B is false
        !left_bool || right_bool
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_implies_boolean_values() {
        let operator = UnifiedImpliesOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true implies true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // true implies false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // false implies true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // false implies false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(false),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_implies_with_empty_collections() {
        let operator = UnifiedImpliesOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // true implies empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(true),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // true implies false

        // false implies empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Boolean(false),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies false (always true)

        // empty implies true
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies true

        // empty implies empty
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies false
    }

    #[tokio::test]
    async fn test_implies_with_non_boolean_values() {
        let operator = UnifiedImpliesOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Integer values - should convert to boolean (truthy/falsy)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(1),  // true
                FhirPathValue::Integer(0),  // false
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false)); // true implies false

        // String values
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("".into()),     // false
                FhirPathValue::String("hello".into()), // true
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies true

        // Mixed values - antecedent false
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(0),         // false
                FhirPathValue::Boolean(false),     // false
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // false implies false
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedImpliesOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "implies");
        assert_eq!(metadata.basic.display_name, "Logical IMPLIES");
        assert_eq!(metadata.basic.precedence, 1);
        assert_eq!(metadata.basic.associativity, Associativity::Right);
        assert_eq!(metadata.basic.category, OperatorCategory::Logical);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // Implication is not commutative
        assert!(operator.can_short_circuit());
    }

    #[test]
    fn test_implies_logic() {
        let operator = UnifiedImpliesOperator::new();
        
        // Test the logic application (A implies B = (not A) or B)
        assert!(operator.apply_logical(true, true));    // true implies true = true
        assert!(!operator.apply_logical(true, false));  // true implies false = false
        assert!(operator.apply_logical(false, true));   // false implies true = true
        assert!(operator.apply_logical(false, false));  // false implies false = true
    }
}