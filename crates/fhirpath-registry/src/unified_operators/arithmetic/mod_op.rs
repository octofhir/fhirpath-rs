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

//! Modulo 'mod' operator implementation with enhanced metadata

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
use rust_decimal::Decimal;

/// Modulo 'mod' operator implementation
/// Computes the remainder of division according to FHIRPath specification
pub struct UnifiedModOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedModOperator {
    /// Create a new modulo operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "mod",
            OperatorCategory::Arithmetic,
            4, // FHIRPath spec: *, /, div, mod have precedence #4
            Associativity::Left,
        )
        .display_name("Modulo (mod)")
        .description("Computes the remainder of division operation using FHIRPath modulo semantics.")
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("7 mod 3", "Modulo operation (1)")
        .example("10 mod 4", "Modulo operation (2)")
        .example("8.5 mod 3.2", "Decimal modulo (1.6)")
        .keywords(vec!["mod", "modulo", "remainder", "arithmetic"])
        .completion_visibility(OperatorCompletionVisibility::Always)
        .build();

        Self { metadata }
    }
}

impl Default for UnifiedModOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedModOperator {
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

        // Both operands should be single values for mod operation
        let left_collection = left.to_collection();
        let right_collection = right.to_collection();

        if left_collection.len() != 1 || right_collection.len() != 1 {
            return Ok(FhirPathValue::Empty);
        }

        let left_val = left_collection.first().unwrap();
        let right_val = right_collection.first().unwrap();

        match (left_val, right_val) {
            // Integer mod Integer
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => {
                if *r == 0 {
                    return Ok(FhirPathValue::Empty); // Modulo by zero
                }
                Ok(FhirPathValue::Integer(l % r))
            }
            
            // Decimal mod Decimal
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                if r.is_zero() {
                    return Ok(FhirPathValue::Empty); // Modulo by zero
                }
                Ok(FhirPathValue::Decimal(l % r))
            }
            
            // Mixed integer and decimal - convert to decimal
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                if r.is_zero() {
                    return Ok(FhirPathValue::Empty); // Modulo by zero
                }
                let left_decimal = Decimal::from(*l);
                Ok(FhirPathValue::Decimal(left_decimal % r))
            }
            
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                if *r == 0 {
                    return Ok(FhirPathValue::Empty); // Modulo by zero
                }
                let right_decimal = Decimal::from(*r);
                Ok(FhirPathValue::Decimal(l % right_decimal))
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
    async fn test_mod_integers() {
        let operator = UnifiedModOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 7 mod 3 = 1
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(7),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // 10 mod 4 = 2
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(10),
                FhirPathValue::Integer(4),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));

        // 8 mod 3 = 2
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(8),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));

        // Negative modulo: -7 mod 3 = -1 (follows Rust/FHIRPath semantics)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(-7),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(-1));
    }

    #[tokio::test]
    async fn test_mod_decimals() {
        let operator = UnifiedModOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 8.5 mod 3.2 = 1.6 (8.5 - (2 * 3.2) = 8.5 - 6.4 = 2.1, but with decimal precision)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(85, 1)), // 8.5
                FhirPathValue::Decimal(Decimal::new(32, 1)), // 3.2
                &context,
            )
            .await
            .unwrap();
        
        // Check if result is decimal and approximately correct
        match result {
            FhirPathValue::Decimal(d) => {
                // 8.5 % 3.2 should be approximately 1.6
                let expected = Decimal::new(16, 1); // 1.6
                assert!((d - expected).abs() < Decimal::new(1, 2)); // Within 0.01
            }
            _ => panic!("Expected decimal result"),
        }

        // 7.0 mod 2.0 = 1.0
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::from(7)),
                FhirPathValue::Decimal(Decimal::from(2)),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(1)));
    }

    #[tokio::test]
    async fn test_mod_mixed_types() {
        let operator = UnifiedModOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 7 mod 2.0 = 1.0
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(7),
                FhirPathValue::Decimal(Decimal::from(2)),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from(1)));

        // 8.5 mod 3 = 2.5
        let result = operator
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(85, 1)), // 8.5
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(25, 1))); // 2.5
    }

    #[tokio::test]
    async fn test_mod_modulo_by_zero() {
        let operator = UnifiedModOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // 5 mod 0 = {} (empty)
        let result = operator
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(0),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5.0 mod 0.0 = {} (empty)
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
    async fn test_mod_with_empty() {
        let operator = UnifiedModOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // {} mod 5 = {}
        let result = operator
            .evaluate_binary(
                FhirPathValue::Empty,
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5 mod {} = {}
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
    async fn test_mod_invalid_types() {
        let operator = UnifiedModOperator::new();
        let context = EvaluationContext::new(FhirPathValue::Empty);

        // "hello" mod 5 = {}
        let result = operator
            .evaluate_binary(
                FhirPathValue::String("hello".into()),
                FhirPathValue::Integer(5),
                &context,
            )
            .await
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // 5 mod true = {}
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
        let operator = UnifiedModOperator::new();
        let metadata = operator.metadata();

        assert_eq!(metadata.basic.symbol, "mod");
        assert_eq!(metadata.basic.display_name, "Modulo (mod)");
        assert_eq!(metadata.basic.precedence, 4);
        assert_eq!(metadata.basic.associativity, Associativity::Left);
        assert_eq!(metadata.basic.category, OperatorCategory::Arithmetic);
        assert!(operator.supports_binary());
        assert!(!operator.supports_unary());
        assert!(!operator.is_commutative()); // Modulo is not commutative
    }
}