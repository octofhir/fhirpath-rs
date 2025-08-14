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

//! Logical NOT operator (not) implementation with enhanced metadata

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

/// Logical NOT operator implementation
/// FHIRPath logical NOT follows three-valued logic with empty collections
pub struct UnifiedNotOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedNotOperator {
    /// Create a new logical NOT operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "not",
            OperatorCategory::Logical,
            14, // FHIRPath spec: not has high precedence as unary operator
            Associativity::Right,
        )
        .display_name("Logical NOT")
        .description("Performs logical NOT operation using FHIRPath three-valued logic.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .supports_unary(true)
        .example("not true", "Boolean negation (false)")
        .example("not false", "Boolean negation (true)")
        .example("not {}", "Empty collection negation (true)")
        .keywords(vec!["not", "logical", "boolean", "negation", "inverse"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }

    /// Convert FHIRPath value to boolean for logical NOT operation
    fn to_boolean(&self, value: &FhirPathValue) -> bool {
        use FhirPathValue::*;
        match value {
            Boolean(b) => *b,
            Empty => false,
            Collection(items) => !items.is_empty(),
            Integer(i) => *i != 0,
            Decimal(d) => !d.is_zero(),
            String(s) => !s.is_empty(),
            _ => true, // Other types are generally truthy
        }
    }
}

impl Default for UnifiedNotOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedNotOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_unary(
        &self,
        operand: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let boolean_value = self.to_boolean(&operand);
        Ok(FhirPathValue::Boolean(!boolean_value))
    }

    fn supports_unary(&self) -> bool {
        true
    }

    fn supports_binary(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{FhirPathValue, Collection};
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_not_boolean_values() {
        let operator = UnifiedNotOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // not true
        let result = operator
            .evaluate_unary(FhirPathValue::Boolean(true), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // not false
        let result = operator
            .evaluate_unary(FhirPathValue::Boolean(false), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_not_empty_collection() {
        let operator = UnifiedNotOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // not empty
        let result = operator
            .evaluate_unary(FhirPathValue::Empty, &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // not non-empty collection
        let collection = Collection::from_vec(vec![FhirPathValue::Integer(1)]);
        let result = operator
            .evaluate_unary(FhirPathValue::Collection(collection), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_not_numeric_values() {
        let operator = UnifiedNotOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // not 0 (integer)
        let result = operator
            .evaluate_unary(FhirPathValue::Integer(0), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // not 5 (non-zero integer)
        let result = operator
            .evaluate_unary(FhirPathValue::Integer(5), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // not 0.0 (decimal)
        let result = operator
            .evaluate_unary(FhirPathValue::Decimal(Decimal::ZERO), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // not 3.14 (non-zero decimal)
        let result = operator
            .evaluate_unary(
                FhirPathValue::Decimal(Decimal::new(314, 2)), // 3.14
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_not_string_values() {
        let operator = UnifiedNotOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // not empty string
        let result = operator
            .evaluate_unary(FhirPathValue::String("".into()), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // not non-empty string
        let result = operator
            .evaluate_unary(FhirPathValue::String("hello".into()), &context)
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedNotOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "not");
        assert_eq!(metadata.basic.display_name, "Logical NOT");
        assert_eq!(metadata.basic.precedence, 14);
        assert_eq!(metadata.basic.associativity, Associativity::Right);
        assert_eq!(metadata.basic.category, OperatorCategory::Logical);
        assert!(!operator.supports_binary());
        assert!(operator.supports_unary());
        assert!(!operator.is_commutative()); // Unary operators are not commutative
    }

    #[test]
    fn test_boolean_conversion() {
        let operator = UnifiedNotOperator::new();
        
        // Test boolean conversion logic
        assert!(operator.to_boolean(&FhirPathValue::Boolean(true)));
        assert!(!operator.to_boolean(&FhirPathValue::Boolean(false)));
        assert!(!operator.to_boolean(&FhirPathValue::Empty));
        assert!(!operator.to_boolean(&FhirPathValue::Integer(0)));
        assert!(operator.to_boolean(&FhirPathValue::Integer(1)));
        assert!(!operator.to_boolean(&FhirPathValue::Decimal(Decimal::ZERO)));
        assert!(operator.to_boolean(&FhirPathValue::Decimal(Decimal::ONE)));
        assert!(!operator.to_boolean(&FhirPathValue::String("".into())));
        assert!(operator.to_boolean(&FhirPathValue::String("hello".into())));
        
        // Non-empty collection should be truthy
        let collection = Collection::from_vec(vec![FhirPathValue::Integer(1)]);
        assert!(operator.to_boolean(&FhirPathValue::Collection(collection)));
        
        // Empty collection should be falsy
        let empty_collection = Collection::new();
        assert!(!operator.to_boolean(&FhirPathValue::Collection(empty_collection)));
    }
}