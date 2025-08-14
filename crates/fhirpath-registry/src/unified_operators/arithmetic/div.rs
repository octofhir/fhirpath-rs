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

//! Integer division 'div' operator implementation with enhanced metadata

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
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// Integer division 'div' operator implementation
/// Performs truncated integer division according to FHIRPath specification
pub struct UnifiedDivOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedDivOperator {
    /// Create a new integer division operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "div",
            OperatorCategory::Arithmetic,
            4, // FHIRPath spec: *, /, div, mod have precedence #4
            Associativity::Left,
        )
        .display_name("Integer Division (div)")
        .description("Performs truncated integer division, returning the integer part of the division result.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("7 div 3", "Integer division (2)")
        .example("10 div 4", "Integer division (2)")
        .example("15.7 div 3.2", "Decimal to integer division (4)")
        .keywords(vec!["div", "division", "integer", "truncate", "arithmetic"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedDivOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedDivOperator {
    fn metadata(&self) -> &EnhancedOperatorMetadata {
        &self.metadata
    }

    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Both operands should be single values for div operation
        let left_collection = left.to_collection();
        let right_collection = right.to_collection();

        if left_collection.len() != 1 || right_collection.len() != 1 {
            return Ok(FhirPathValue::Empty);
        }

        let left_val = left_collection.first().unwrap();
        let right_val = right_collection.first().unwrap();

        match (left_val, right_val) {
            // Integer div Integer
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                if *r == 0 {
                    return Ok(FhirPathValue::Empty); // Division by zero
                }
                Ok(FhirPathValue::Integer(l / r))
            }
            
            // Decimal div Decimal (result is integer)
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                if r.is_zero() {
                    return Ok(FhirPathValue::Empty); // Division by zero
                }
                let result = (l / r).trunc(); // Truncate to integer part
                if let Some(int_result) = result.to_i64() {
                    Ok(FhirPathValue::Integer(int_result))
                } else {
                    Ok(FhirPathValue::Empty) // Overflow
                }
            }
            
            // Mixed integer and decimal
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                if r.is_zero() {
                    return Ok(FhirPathValue::Empty); // Division by zero
                }
                let left_decimal = Decimal::from(*l);
                let result = (left_decimal / r).trunc(); // Truncate to integer part
                if let Some(int_result) = result.to_i64() {
                    Ok(FhirPathValue::Integer(int_result))
                } else {
                    Ok(FhirPathValue::Empty) // Overflow
                }
            }
            
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                if *r == 0 {
                    return Ok(FhirPathValue::Empty); // Division by zero
                }
                let right_decimal = Decimal::from(*r);
                let result = (l / right_decimal).trunc(); // Truncate to integer part
                if let Some(int_result) = result.to_i64() {
                    Ok(FhirPathValue::Integer(int_result))
                } else {
                    Ok(FhirPathValue::Empty) // Overflow
                }
            }
            
            // Invalid operand types
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_div_integers() {
        let operator = UnifiedDivOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 7 div 3 = 2
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(7),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));

        // 10 div 4 = 2
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(10),
                FhirPathValue::Integer(4),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));

        // -7 div 3 = -2 (truncated toward zero)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(-7),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(-2));
    }

    #[tokio::test]
    async fn test_div_decimals() {
        let operator = UnifiedDivOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 7.8 div 2.3 = 3 (7.8/2.3 = 3.39..., truncated to 3)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(78, 1)), // 7.8
                FhirPathValue::Decimal(Decimal::new(23, 1)), // 2.3
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // 15.7 div 3.2 = 4 (15.7/3.2 = 4.90..., truncated to 4)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(157, 1)), // 15.7
                FhirPathValue::Decimal(Decimal::new(32, 1)),  // 3.2
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(4));
    }

    #[tokio::test]
    async fn test_div_mixed_types() {
        let operator = UnifiedDivOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 7 div 2.0 = 3
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(7),
                FhirPathValue::Decimal(Decimal::from(2)),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // 8.5 div 3 = 2 (8.5/3 = 2.83..., truncated to 2)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(85, 1)), // 8.5
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));
    }

    #[tokio::test]
    async fn test_div_division_by_zero() {
        let operator = UnifiedDivOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 5 div 0 = {} (empty)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(0),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5.0 div 0.0 = {} (empty)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::from(5)),
                FhirPathValue::Decimal(Decimal::ZERO),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_div_with_empty() {
        let operator = UnifiedDivOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // {} div 5 = {}
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5 div {} = {}
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Empty,
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_div_invalid_types() {
        let operator = UnifiedDivOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // "hello" div 5 = {}
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5 div true = {}
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Boolean(true),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_operator_metadata() {
        let operator = UnifiedDivOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "div");
        assert_eq!(metadata.basic.display_name, "Integer Division (div)");
        assert_eq!(metadata.basic.precedence, 4);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Arithmetic);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // Division is not commutative
    }
}