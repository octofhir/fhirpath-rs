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

//! Addition operator (+) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorCompletionVisibility, OperatorComplexity,
    OperatorMemoryUsage, OperatorMetadataBuilder, OperatorTypeSignature,
};
use crate::function::EvaluationContext;
use crate::unified_operator::Associativity;
use crate::unified_operator::{
    ArithmeticOperator, OperatorError, OperatorResult, UnifiedFhirPathOperator,
};
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;

/// Addition operator (+) implementation
pub struct UnifiedAdditionOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedAdditionOperator {
    /// Create a new addition operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "+",
            OperatorCategory::Arithmetic,
            12, // High precedence
            Associativity::Left,
        )
        .display_name("Addition")
        .description("Adds two numeric values or concatenates strings. Supports Integer + Integer = Integer, Decimal + Decimal = Decimal, String + String = String, and mixed operations.")
        .commutative(true)
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("3 + 2", "Basic integer addition")
        .example("3.14 + 2.86", "Decimal addition")
        .example("5 + 2.5", "Mixed integer and decimal addition")
        .example("'Hello' + ' World'", "String concatenation")
        .keywords(vec!["add", "addition", "plus", "arithmetic", "sum"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        // Add type signatures
        let mut metadata = metadata;
        metadata.types.type_signatures = vec![
            OperatorTypeSignature {
                left_type: Some("Integer".to_string()),
                right_type: "Integer".to_string(),
                result_type: "Integer".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Decimal".to_string()),
                right_type: "Decimal".to_string(),
                result_type: "Decimal".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: Some("Integer".to_string()),
                right_type: "Decimal".to_string(),
                result_type: "Decimal".to_string(),
                is_preferred: false,
            },
            OperatorTypeSignature {
                left_type: Some("Decimal".to_string()),
                right_type: "Integer".to_string(),
                result_type: "Decimal".to_string(),
                is_preferred: false,
            },
            OperatorTypeSignature {
                left_type: Some("Quantity".to_string()),
                right_type: "Quantity".to_string(),
                result_type: "Quantity".to_string(),
                is_preferred: false,
            },
            OperatorTypeSignature {
                left_type: Some("String".to_string()),
                right_type: "String".to_string(),
                result_type: "String".to_string(),
                is_preferred: true,
            },
        ];

        metadata.usage.related_operators = vec!["-".to_string(), "*".to_string(), "/".to_string()];

        metadata.usage.common_mistakes = vec![
            "Remember that adding Integer + Decimal results in Decimal".to_string(),
            "Quantity addition requires compatible units".to_string(),
        ];

        Self { metadata }
    }
}

impl Default for UnifiedAdditionOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedAdditionOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        // Handle Collection inputs element-wise
        match (&left, &right) {
            // Collection + Collection - element-wise addition
            (Collection(l), Collection(r)) => {
                if l.len() != r.len() {
                    return Ok(Empty); // Per FHIRPath spec, mismatched collections return empty
                }
                let mut results = Vec::new();
                for (left_item, right_item) in l.iter().zip(r.iter()) {
                    match self.evaluate_binary(left_item.clone(), right_item.clone(), context).await {
                        Ok(Collection(items)) => {
                            for item in items.iter() {
                                results.push(item.clone());
                            }
                        },
                        Ok(single) => results.push(single),
                        Err(_) => continue, // Skip errors in collection operations
                    }
                }
                return Ok(FhirPathValue::collection(results));
            },
            // Single + Collection or Collection + Single
            (Collection(items), single) | (single, Collection(items)) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    let (l, r) = if matches!(left, Collection(_)) {
                        (item.clone(), single.clone())
                    } else {
                        (single.clone(), item.clone())
                    };
                    match self.evaluate_binary(l, r, context).await {
                        Ok(Collection(result_items)) => {
                            for item in result_items.iter() {
                                results.push(item.clone());
                            }
                        },
                        Ok(single_result) => results.push(single_result),
                        Err(_) => continue, // Skip errors in collection operations
                    }
                }
                return Ok(FhirPathValue::collection(results));
            },
            _ => {} // Continue with single value operations
        }

        // Handle string concatenation case
        if matches!((&left, &right), (String(_), String(_))) {
            match (&left, &right) {
                (String(l), String(r)) => {
                    let result = format!("{}{}", l, r);
                    return Ok(FhirPathValue::collection(vec![String(result.into())]));
                }
                _ => unreachable!(),
            }
        }
        
        // Otherwise, delegate to arithmetic operations
        self.evaluate_arithmetic_binary(left, right, context).await
    }
}

impl ArithmeticOperator for UnifiedAdditionOperator {
    fn apply_integer(&self, left: i64, right: i64) -> OperatorResult<i64> {
        left.checked_add(right)
            .ok_or_else(|| OperatorError::EvaluationError {
                operator: "+".to_string(),
                message: "Integer overflow in addition".to_string(),
            })
    }

    fn apply_decimal(
        &self,
        left: rust_decimal::Decimal,
        right: rust_decimal::Decimal,
    ) -> OperatorResult<rust_decimal::Decimal> {
        match left.checked_add(right) {
            Some(result) => Ok(result),
            None => Err(OperatorError::EvaluationError {
                operator: "+".to_string(),
                message: "Decimal overflow in addition".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;

    fn create_test_context() -> EvaluationContext {
        EvaluationContext::new(FhirPathValue::Empty)
    }

    #[tokio::test]
    async fn test_integer_addition() {
        let op = UnifiedAdditionOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(2),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Integer(5));
    }

    #[tokio::test]
    async fn test_decimal_addition() {
        let op = UnifiedAdditionOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(314, 2)),
                FhirPathValue::Decimal(Decimal::new(286, 2)),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(6)));
    }

    #[tokio::test]
    async fn test_mixed_addition() {
        let op = UnifiedAdditionOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::new(25, 1)),
                &context,
            )
            .await
            .unwrap();

        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(75, 1)));
    }

    #[tokio::test]
    async fn test_integer_overflow() {
        let op = UnifiedAdditionOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(i64::MAX),
                FhirPathValue::Integer(1),
                &context,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_types() {
        let op = UnifiedAdditionOperator::new();
        let context = create_test_context();

        let result = op
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::Integer(1),
                &context,
            )
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let op = UnifiedAdditionOperator::new();
        let metadata = op.metadata();

        assert_eq!(metadata.basic.symbol, "+");
        assert_eq!(metadata.basic.display_name, "Addition");
        assert_eq!(metadata.basic.precedence, 12);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert!(metadata.basic.is_commutative);
        assert!(metadata.basic.supports_binary);
        assert!(!metadata.basic.supports_unary);

        assert_eq!(
            metadata.performance.complexity,
            OperatorComplexity::Constant
        );
        assert_eq!(
            metadata.performance.memory_usage,
            OperatorMemoryUsage::Minimal
        );

        assert!(!metadata.usage.examples.is_empty());
        assert!(!metadata.types.type_signatures.is_empty());
    }

    #[test]
    fn test_arithmetic_trait_methods() {
        let op = UnifiedAdditionOperator::new();

        assert_eq!(op.apply_integer(3, 2).unwrap(), 5);
        assert_eq!(
            op.apply_decimal(Decimal::new(314, 2), Decimal::new(286, 2))
                .unwrap(),
            Decimal::from(6)
        );

        // Test overflow
        assert!(op.apply_integer(i64::MAX, 1).is_err());
    }

    #[test]
    fn test_type_validation() {
        let op = UnifiedAdditionOperator::new();

        assert!(op.validates_types(Some("Integer"), "Integer"));
        assert!(op.validates_types(Some("Decimal"), "Decimal"));
        assert!(op.validates_types(Some("Integer"), "Decimal"));
        assert!(!op.validates_types(Some("String"), "Integer"));
    }

    #[test]
    fn test_result_type() {
        let op = UnifiedAdditionOperator::new();

        assert_eq!(
            op.result_type(Some("Integer"), "Integer"),
            Some("Integer".to_string())
        );
        assert_eq!(
            op.result_type(Some("Decimal"), "Decimal"),
            Some("Decimal".to_string())
        );
        assert_eq!(
            op.result_type(Some("Integer"), "Decimal"),
            Some("Decimal".to_string())
        );
    }

    #[test]
    fn test_operator_properties() {
        let op = UnifiedAdditionOperator::new();

        assert_eq!(op.symbol(), "+");
        assert_eq!(op.display_name(), "Addition");
        assert_eq!(op.precedence(), 12);
        assert_eq!(op.associativity(), Associativity::Left);
        assert!(op.supports_binary());
        assert!(!op.supports_unary());
        assert!(op.is_commutative());
        assert!(op.is_pure());
    }
}
