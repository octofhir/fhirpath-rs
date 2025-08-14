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

//! Division operator (/) implementation with enhanced metadata

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

/// Division operator (/) implementation
pub struct UnifiedDivisionOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedDivisionOperator {
    /// Create a new division operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "/",
            OperatorCategory::Arithmetic,
            13, // Same precedence as multiplication
            Associativity::Left,
        )
        .display_name("Division")
        .description("Divides the left operand by the right operand. Always returns Decimal result to maintain precision. Division by zero results in an error.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("10 / 2", "Basic division")
        .example("7 / 3", "Division with remainder")
        .example("15.6 / 2.6", "Decimal division")
        .keywords(vec!["divide", "division", "arithmetic", "quotient"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        // Add type signatures - all division results in Decimal per FHIRPath spec
        let mut metadata = metadata;
        metadata.types.type_signatures = vec![
            OperatorTypeSignature {
                left_type: Some("Integer".to_string()),
                right_type: "Integer".to_string(),
                result_type: "Decimal".to_string(),
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

        metadata.types.default_result_type = Some("Decimal".to_string());

        metadata.usage.related_operators = vec![
            "*".to_string(),
            "div".to_string(),
            "mod".to_string(),
            "+".to_string(),
            "-".to_string(),
        ];

        metadata.usage.common_mistakes = vec![
            "Division by zero causes an error".to_string(),
            "Division always returns Decimal, never Integer".to_string(),
            "For integer division, use 'div' operator instead".to_string(),
        ];

        Self { metadata }
    }
}

impl Default for UnifiedDivisionOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedDivisionOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        // Convert all to decimal for division per FHIRPath spec
        let (left_decimal, right_decimal) = match (left, right) {
            (Integer(l), Integer(r)) => (rust_decimal::Decimal::from(l), rust_decimal::Decimal::from(r)),
            (Decimal(l), Decimal(r)) => (l, r),
            (Integer(l), Decimal(r)) => (rust_decimal::Decimal::from(l), r),
            (Decimal(l), Integer(r)) => (l, rust_decimal::Decimal::from(r)),
            (left_val, right_val) => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: "/".to_string(),
                    left_type: left_val.type_name().to_string(),
                    right_type: right_val.type_name().to_string(),
                }.into());
            }
        };

        match self.apply_decimal(left_decimal, right_decimal) {
            Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
            Err(e) => Err(e.into()),
        }
    }
}

impl ArithmeticOperator for UnifiedDivisionOperator {
    fn apply_integer(&self, left: i64, right: i64) -> OperatorResult<i64> {
        // Division should return decimal, so this shouldn't be called
        // But implement for completeness
        if right == 0 {
            return Err(OperatorError::EvaluationError {
                operator: "/".to_string(),
                message: "Division by zero".to_string(),
            });
        }
        Ok(left / right)
    }

    fn apply_decimal(&self, left: rust_decimal::Decimal, right: rust_decimal::Decimal) -> OperatorResult<rust_decimal::Decimal> {
        if right.is_zero() {
            return Err(OperatorError::EvaluationError {
                operator: "/".to_string(),
                message: "Division by zero".to_string(),
            });
        }

        match left.checked_div(right) {
            Some(result) => Ok(result),
            None => Err(OperatorError::EvaluationError {
                operator: "/".to_string(),
                message: "Decimal overflow or invalid result in division".to_string(),
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
    async fn test_integer_division() {
        let op = UnifiedDivisionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(10),
                FhirPathValue::Integer(2),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));
    }

    #[tokio::test]
    async fn test_decimal_division() {
        let op = UnifiedDivisionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(156, 1)),
                FhirPathValue::Decimal(Decimal::new(26, 1)),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(6)));
    }

    #[tokio::test]
    async fn test_division_with_remainder() {
        let op = UnifiedDivisionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(7),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        
        // Should be approximately 2.333...
        if let FhirPathValue::Decimal(val) = result {
            let expected = Decimal::new(2333333333333i64, 12); // 2.333333333333
            let tolerance = Decimal::new(1, 4); // 0.0001
            assert!((val - expected).abs() < tolerance);
        } else {
            panic!("Expected Decimal result");
        }
    }

    #[tokio::test]
    async fn test_division_by_zero() {
        let op = UnifiedDivisionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(10),
                FhirPathValue::Integer(0),
                &context,
            )
            .await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_division_by_zero_decimal() {
        let op = UnifiedDivisionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(105, 1)),
                FhirPathValue::Decimal(Decimal::from(0)),
                &context,
            )
            .await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mixed_division() {
        let op = UnifiedDivisionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(15),
                FhirPathValue::Decimal(Decimal::from(3)),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(5)));
    }

    #[test]
    fn test_metadata() {
        let op = UnifiedDivisionOperator::new();
        let metadata = op.metadata();
        
        assert_eq!(metadata.basic.symbol, "/");
        assert_eq!(metadata.basic.display_name, "Division");
        assert_eq!(metadata.basic.precedence, 13); // Same as multiplication
        assert!(!metadata.basic.is_commutative); // Division is not commutative
        assert!(metadata.basic.supports_binary);
        assert!(!metadata.basic.supports_unary);
        
        // All type signatures should result in Decimal
        assert!(metadata.types.type_signatures.iter().all(|sig| sig.result_type == "Decimal"));
    }

    #[test]
    fn test_arithmetic_trait_methods() {
        let op = UnifiedDivisionOperator::new();
        
        assert_eq!(op.apply_decimal(Decimal::from(10), Decimal::from(2)).unwrap(), Decimal::from(5));
        let result = op.apply_decimal(Decimal::from(7), Decimal::from(3)).unwrap();
        assert_eq!(result, Decimal::from(7) / Decimal::from(3));
        
        // Test division by zero
        assert!(op.apply_decimal(Decimal::from(10), Decimal::from(0)).is_err());
        assert!(op.apply_integer(10, 0).is_err());
    }

    #[test]
    fn test_result_type() {
        let op = UnifiedDivisionOperator::new();
        
        // All division operations should result in Decimal
        assert_eq!(op.result_type(Some("Integer"), "Integer"), Some("Decimal".to_string()));
        assert_eq!(op.result_type(Some("Decimal"), "Decimal"), Some("Decimal".to_string()));
        assert_eq!(op.result_type(Some("Integer"), "Decimal"), Some("Decimal".to_string()));
    }
}