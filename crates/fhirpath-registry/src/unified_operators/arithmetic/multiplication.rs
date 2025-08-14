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

//! Multiplication operator (*) implementation with enhanced metadata

use crate::enhanced_operator_metadata::{
    EnhancedOperatorMetadata, OperatorCategory, OperatorComplexity, OperatorMemoryUsage,
    OperatorCompletionVisibility, OperatorTypeSignature, OperatorMetadataBuilder,
};
use crate::unified_operator::Associativity;
use crate::unified_operator::{ArithmeticOperator, OperatorError, OperatorResult, UnifiedFhirPathOperator};
use async_trait::async_trait;
use octofhir_fhirpath_core::EvaluationResult;
use crate::function::EvaluationContext;
use octofhir_fhirpath_model::FhirPathValue;
    

/// Multiplication operator (*) implementation
pub struct UnifiedMultiplicationOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedMultiplicationOperator {
    /// Create a new multiplication operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "*",
            OperatorCategory::Arithmetic,
            13, // Higher precedence than addition/subtraction
            Associativity::Left,
        )
        .display_name("Multiplication")
        .description("Multiplies two numeric values. Supports Integer * Integer = Integer, Decimal * Decimal = Decimal, and mixed operations resulting in Decimal.")
        .commutative(true)
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("3 * 4", "Basic integer multiplication")
        .example("2.5 * 1.6", "Decimal multiplication")
        .example("5 * 2.0", "Mixed integer and decimal multiplication")
        .keywords(vec!["multiply", "multiplication", "times", "arithmetic", "product"])
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
        ];

        metadata.usage.related_operators = vec![
            "/".to_string(),
            "+".to_string(),
            "-".to_string(),
            "div".to_string(),
            "mod".to_string(),
        ];

        metadata.usage.common_mistakes = vec![
            "Multiplying large integers may cause overflow".to_string(),
            "Remember that multiplying Integer * Decimal results in Decimal".to_string(),
        ];

        Self { metadata }
    }
}

impl Default for UnifiedMultiplicationOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedMultiplicationOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        self.evaluate_arithmetic_binary(left, right, context).await
    }
}

impl ArithmeticOperator for UnifiedMultiplicationOperator {
    fn apply_integer(&self, left: i64, right: i64) -> OperatorResult<i64> {
        left.checked_mul(right)
            .ok_or_else(|| OperatorError::EvaluationError {
                operator: "*".to_string(),
                message: "Integer overflow in multiplication".to_string(),
            })
    }

    fn apply_decimal(&self, left: rust_decimal::Decimal, right: rust_decimal::Decimal) -> OperatorResult<rust_decimal::Decimal> {
        match left.checked_mul(right) {
            Some(result) => Ok(result),
            None => Err(OperatorError::EvaluationError {
                operator: "*".to_string(),
                message: "Decimal overflow in multiplication".to_string(),
            })
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
    async fn test_integer_multiplication() {
        let op = UnifiedMultiplicationOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(4),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(12));
    }

    #[tokio::test]
    async fn test_decimal_multiplication() {
        let op = UnifiedMultiplicationOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(25, 1)),
                FhirPathValue::Decimal(Decimal::new(16, 1)),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(4)));
    }

    #[tokio::test]
    async fn test_mixed_multiplication() {
        let op = UnifiedMultiplicationOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Decimal(Decimal::from(2)),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(10)));
    }

    #[tokio::test]
    async fn test_zero_multiplication() {
        let op = UnifiedMultiplicationOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(0),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(0));
    }

    #[tokio::test]
    async fn test_integer_overflow() {
        let op = UnifiedMultiplicationOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(i64::MAX),
                FhirPathValue::Integer(2),
                &context,
            )
            .await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let op = UnifiedMultiplicationOperator::new();
        let metadata = op.metadata();
        
        assert_eq!(metadata.basic.symbol, "*");
        assert_eq!(metadata.basic.display_name, "Multiplication");
        assert_eq!(metadata.basic.precedence, 13); // Higher than addition
        assert!(metadata.basic.is_commutative);
        assert!(metadata.basic.supports_binary);
        assert!(!metadata.basic.supports_unary);
    }

    #[test]
    fn test_arithmetic_trait_methods() {
        let op = UnifiedMultiplicationOperator::new();
        
        assert_eq!(op.apply_integer(3, 4).unwrap(), 12);
        assert_eq!(op.apply_decimal(Decimal::new(25, 1), Decimal::new(16, 1)).unwrap(), Decimal::from(4));
        
        // Test overflow
        assert!(op.apply_integer(i64::MAX, 2).is_err());
    }
}