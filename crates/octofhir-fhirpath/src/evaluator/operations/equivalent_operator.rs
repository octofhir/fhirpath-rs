//! Equivalent (~) operator implementation
//!
//! Implements FHIRPath equivalence comparison which is similar to equality
//! but has different handling of empty values and string case sensitivity.

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Equivalent operator evaluator
pub struct EquivalentOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl EquivalentOperatorEvaluator {
    /// Create a new equivalent operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_equivalent_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Compare two FhirPathValues for equivalence
    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            // Boolean equivalence
            (FhirPathValue::Boolean(l, _, _), FhirPathValue::Boolean(r, _, _)) => Some(l == r),

            // String equivalence (case-insensitive and whitespace-normalized per FHIRPath spec)
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => {
                // Normalize strings: trim and convert to lowercase
                let l_normalized = l.trim().to_lowercase();
                let r_normalized = r.trim().to_lowercase();
                Some(l_normalized == r_normalized)
            }

            // Integer equivalence
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l == r),

            // Decimal equivalence
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some((l - r).abs() < Decimal::new(1, 10)) // Small epsilon for decimal comparison
            }

            // Integer vs Decimal comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some((left_decimal - r).abs() < Decimal::new(1, 10))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some((l - right_decimal).abs() < Decimal::new(1, 10))
            }

            // Date equivalence (considering precision)
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => {
                // For equivalence, dates should match considering their precision
                // This is a simplified implementation - the full spec requires more complex precision handling
                Some(l == r)
            }

            // DateTime equivalence (considering precision and timezone normalization)
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                // For equivalence, datetimes should match considering their precision
                // This is a simplified implementation - the full spec requires timezone normalization
                Some(l == r)
            }

            // Time equivalence
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l == r),

            // Quantity equivalence (with unit normalization)
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    ..
                },
            ) => {
                // For now, simple comparison - full implementation would need UCUM normalization
                if lu == ru {
                    Some((lv - rv).abs() < Decimal::new(1, 10))
                } else {
                    // Different units - would need UCUM normalization for true equivalence
                    // For now, return false for different units
                    Some(false)
                }
            }

            // Collection equivalence (recursive)
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.len() != r.len() {
                    return Some(false);
                }

                // Compare each element recursively
                for (l_item, r_item) in l.iter().zip(r.iter()) {
                    if let Some(false) = self.compare_values(l_item, r_item) {
                        return Some(false);
                    }
                }
                Some(true)
            }

            // Different types - for equivalence, this depends on the specific types
            // Some types can be equivalent (e.g., integer and decimal), others cannot
            _ => Some(false),
        }
    }
}

#[async_trait]
impl OperationEvaluator for EquivalentOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Equivalence has different empty handling than equality:
        // - If both are empty, result is true
        // - If one is empty and other is not, result is false
        // - If both are non-empty, compare values

        match (left.is_empty(), right.is_empty()) {
            (true, true) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            }),
            (true, false) | (false, true) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            }),
            (false, false) => {
                // Both sides have values, compare them
                // For collections, we need to compare all elements, not just the first
                if left.len() != right.len() {
                    // Different collection sizes are not equivalent
                    return Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::boolean(false)),
                    });
                }

                // Compare each element pair
                for (left_val, right_val) in left.iter().zip(right.iter()) {
                    match self.compare_values(left_val, right_val) {
                        Some(false) | None => {
                            // If any element pair is not equivalent, the whole comparison is false
                            return Ok(EvaluationResult {
                                value: Collection::single(FhirPathValue::boolean(false)),
                            });
                        }
                        Some(true) => continue,
                    }
                }

                // All elements matched
                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(true)),
                })
            }
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the equivalent operator
fn create_equivalent_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "~".to_string(),
        description: "Equivalence comparison with normalization and special empty handling"
            .to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Custom, // Equivalence has special empty handling
        deterministic: true,
        precedence: 5, // FHIRPath equivalence precedence (same as equality)
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_equivalent_boolean() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_strings_case_insensitive() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![FhirPathValue::string("HELLO".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_strings_whitespace_normalized() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("  hello  ".to_string())];
        let right = vec![FhirPathValue::string("hello".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_both_empty() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![];
        let right = vec![];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_one_empty() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }
}
