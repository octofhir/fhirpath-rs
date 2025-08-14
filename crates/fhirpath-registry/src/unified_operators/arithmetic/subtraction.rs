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

//! Subtraction operator (-) implementation with enhanced metadata

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
    

/// Subtraction operator (-) implementation
pub struct UnifiedSubtractionOperator {
    metadata: EnhancedOperatorMetadata,
}

impl UnifiedSubtractionOperator {
    /// Create a new subtraction operator
    pub fn new() -> Self {
        let metadata = OperatorMetadataBuilder::new(
            "-",
            OperatorCategory::Arithmetic,
            12, // Same precedence as addition
            Associativity::Left,
        )
        .display_name("Subtraction")
        .description("Subtracts the right operand from the left operand. Supports Integer - Integer = Integer, Decimal - Decimal = Decimal, and mixed operations resulting in Decimal.")
        .supports_unary(true) // Unary minus
        .complexity(OperatorComplexity::Constant)
        .memory_usage(OperatorMemoryUsage::Minimal)
        .example("5 - 3", "Basic integer subtraction")
        .example("7.5 - 2.3", "Decimal subtraction")
        .example("10 - 3.5", "Mixed integer and decimal subtraction")
        .example("-5", "Unary minus (negation)")
        .keywords(vec!["subtract", "subtraction", "minus", "arithmetic", "difference", "negate"])
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
            // Unary minus
            OperatorTypeSignature {
                left_type: None,
                right_type: "Integer".to_string(),
                result_type: "Integer".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: None,
                right_type: "Decimal".to_string(),
                result_type: "Decimal".to_string(),
                is_preferred: true,
            },
            OperatorTypeSignature {
                left_type: None,
                right_type: "String".to_string(),
                result_type: "String".to_string(),
                is_preferred: false, // Used for sort function, not general arithmetic
            },
        ];

        metadata.usage.related_operators = vec![
            "+".to_string(),
            "*".to_string(),
            "/".to_string(),
        ];

        metadata.usage.common_mistakes = vec![
            "Subtraction is not commutative: a - b ≠ b - a".to_string(),
            "Remember that subtracting Integer - Decimal results in Decimal".to_string(),
        ];

        Self { metadata }
    }
}

impl Default for UnifiedSubtractionOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedFhirPathOperator for UnifiedSubtractionOperator {
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

    async fn evaluate_unary(
        &self,
        operand: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        match operand {
            Integer(i) => {
                match i.checked_neg() {
                    Some(result) => Ok(Collection(vec![Integer(result)].into())),
                    None => Err(OperatorError::EvaluationError {
                        operator: "-".to_string(),
                        message: "Integer overflow in unary minus".to_string(),
                    }.into()),
                }
            }
            Decimal(d) => Ok(Collection(vec![Decimal(-d)].into())),
            Collection(items) => {
                if items.len() == 1 {
                    // Single-item collection, extract and negate
                    if let Some(item) = items.iter().next() {
                        Box::pin(self.evaluate_unary(item.clone(), _context)).await
                    } else {
                        Ok(Collection(vec![Empty].into()))
                    }
                } else {
                    // Multi-item collection - negate each item
                    let mut results = Vec::new();
                    for item in items.iter() {
                        match Box::pin(self.evaluate_unary(item.clone(), _context)).await? {
                            Collection(sub_items) => {
                                for sub_item in sub_items.iter() {
                                    results.push(sub_item.clone());
                                }
                            }
                            single_result => results.push(single_result),
                        }
                    }
                    Ok(Collection(results.into()))
                }
            }
            Empty => Ok(Empty),
            String(s) => {
                // For sorting purposes, unary minus on strings should return a value that sorts in reverse order
                // We create a string that will sort in reverse alphabetical order by inverting each character
                let inverted: std::string::String = s.chars()
                    .map(|c| char::from_u32(0x10FFFF - c as u32).unwrap_or(c))
                    .collect();
                Ok(Collection(vec![String(inverted.into())].into()))
            }
            _ => {
                Err(OperatorError::InvalidUnaryOperandType {
                    operator: "-".to_string(),
                    operand_type: operand.type_name().to_string(),
                }.into())
            }
        }
    }
}

impl ArithmeticOperator for UnifiedSubtractionOperator {
    fn apply_integer(&self, left: i64, right: i64) -> OperatorResult<i64> {
        left.checked_sub(right)
            .ok_or_else(|| OperatorError::EvaluationError {
                operator: "-".to_string(),
                message: "Integer overflow in subtraction".to_string(),
            })
    }

    fn apply_decimal(&self, left: rust_decimal::Decimal, right: rust_decimal::Decimal) -> OperatorResult<rust_decimal::Decimal> {
        match left.checked_sub(right) {
            Some(result) => Ok(result),
            None => Err(OperatorError::EvaluationError {
                operator: "-".to_string(),
                message: "Decimal overflow in subtraction".to_string(),
            })
        }
    }

    async fn evaluate_arithmetic_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        let left_type = left.type_name().to_string();
        let right_type = right.type_name().to_string();

        match (left, right) {
            // Handle Collection operations
            (Collection(coll), val) => {
                if coll.len() == 1 {
                    // Single-item collection, extract and subtract
                    if let Some(first_item) = coll.iter().next() {
                        Box::pin(self.evaluate_arithmetic_binary(first_item.clone(), val, _context)).await
                    } else {
                        Ok(FhirPathValue::collection(vec![Empty]))
                    }
                } else {
                    Err(OperatorError::EvaluationError {
                        operator: "-".to_string(),
                        message: format!("Cannot subtract {} from multi-item collection", val.type_name()),
                    }.into())
                }
            }
            (val, Collection(coll)) => {
                if coll.len() == 1 {
                    // Single-item collection, extract and subtract
                    if let Some(first_item) = coll.iter().next() {
                        Box::pin(self.evaluate_arithmetic_binary(val, first_item.clone(), _context)).await
                    } else {
                        Ok(FhirPathValue::collection(vec![Empty]))
                    }
                } else {
                    Err(OperatorError::EvaluationError {
                        operator: "-".to_string(),
                        message: format!("Cannot subtract multi-item collection from {}", val.type_name()),
                    }.into())
                }
            }
            // Standard arithmetic operations
            (Integer(l), Integer(r)) => {
                match self.apply_integer(l, r) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Integer(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Decimal(l), Decimal(r)) => {
                match self.apply_decimal(l, r) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Integer(l), Decimal(r)) => {
                let left_decimal = rust_decimal::Decimal::from(l);
                match self.apply_decimal(left_decimal, r) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Decimal(l), Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(r);
                match self.apply_decimal(l, right_decimal) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Empty, _) | (_, Empty) => Ok(FhirPathValue::collection(vec![Empty])),
            _ => {
                Err(OperatorError::InvalidOperandTypes {
                    operator: "-".to_string(),
                    left_type,
                    right_type,
                }.into())
            }
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
    async fn test_integer_subtraction() {
        let op = UnifiedSubtractionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Integer(5),
                FhirPathValue::Integer(3),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(2));
    }

    #[tokio::test]
    async fn test_decimal_subtraction() {
        let op = UnifiedSubtractionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_binary(
                FhirPathValue::Decimal(Decimal::new(75, 1)),
                FhirPathValue::Decimal(Decimal::new(23, 1)),
                &context,
            )
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(52, 1)));
    }

    #[tokio::test]
    async fn test_unary_minus_integer() {
        let op = UnifiedSubtractionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_unary(FhirPathValue::Integer(5), &context)
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(-5));
    }

    #[tokio::test]
    async fn test_unary_minus_decimal() {
        let op = UnifiedSubtractionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_unary(FhirPathValue::Decimal(Decimal::new(314, 2)), &context)
            .await
            .unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(-Decimal::new(314, 2)));
    }

    #[tokio::test]
    async fn test_unary_minus_string() {
        let op = UnifiedSubtractionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_unary(FhirPathValue::String("hello".into()), &context)
            .await
            .unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::String(s)) = items.get(0) {
                    // The result should be an inverted string that sorts in reverse order
                    // We'll just verify it's a transformed string (not the original)
                    assert_ne!(s.as_ref(), "hello");
                    assert!(!s.is_empty());
                } else {
                    panic!("Expected String result");
                }
            }
            _ => panic!("Expected Collection result"),
        }
    }

    #[tokio::test]
    async fn test_unary_overflow() {
        let op = UnifiedSubtractionOperator::new();
        let context = create_test_context();
        
        let result = op
            .evaluate_unary(FhirPathValue::Integer(i64::MIN), &context)
            .await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let op = UnifiedSubtractionOperator::new();
        let metadata = op.metadata();
        
        assert_eq!(metadata.basic.symbol, "-");
        assert_eq!(metadata.basic.display_name, "Subtraction");
        assert!(metadata.basic.supports_binary);
        assert!(metadata.basic.supports_unary);
        assert!(!metadata.basic.is_commutative); // Subtraction is not commutative
    }

    #[test]
    fn test_arithmetic_trait_methods() {
        let op = UnifiedSubtractionOperator::new();
        
        assert_eq!(op.apply_integer(5, 3).unwrap(), 2);
        assert_eq!(op.apply_decimal(Decimal::new(75, 1), Decimal::new(23, 1)).unwrap(), Decimal::new(52, 1));
        
        // Test overflow
        assert!(op.apply_integer(i64::MIN, 1).is_err());
    }
}