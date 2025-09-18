//! Less than (<) operator implementation
//!
//! Implements FHIRPath less than comparison for ordered types.

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Less than operator evaluator
pub struct LessThanOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl LessThanOperatorEvaluator {
    /// Create a new less than operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_less_than_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Try to parse a string as a temporal value
    fn try_parse_string_as_temporal(&self, s: &str) -> Option<FhirPathValue> {
        // Try parsing as Date first
        if let Some(date) = PrecisionDate::parse(s) {
            return Some(FhirPathValue::date(date));
        }

        // Try parsing as DateTime
        if let Some(datetime) = PrecisionDateTime::parse(s) {
            return Some(FhirPathValue::datetime(datetime));
        }

        // Try parsing as Time
        if let Some(time) = PrecisionTime::parse(s) {
            return Some(FhirPathValue::time(time));
        }

        None
    }

    /// Check if a value is a temporal type
    fn is_temporal(&self, value: &FhirPathValue) -> bool {
        matches!(
            value,
            FhirPathValue::Date(_, _, _)
                | FhirPathValue::DateTime(_, _, _)
                | FhirPathValue::Time(_, _, _)
        )
    }

    /// Compare two FhirPathValues for less than relationship with automatic string-to-temporal conversion
    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        // Handle string-to-temporal conversions
        match (left, right) {
            // String vs temporal types - try to parse string as temporal
            (FhirPathValue::String(s, _, _), temporal) if self.is_temporal(temporal) => {
                if let Some(parsed) = self.try_parse_string_as_temporal(s) {
                    return self.compare_values_direct(&parsed, temporal);
                }
                // Fall through to regular comparison if parsing fails
            }
            (temporal, FhirPathValue::String(s, _, _)) if self.is_temporal(temporal) => {
                if let Some(parsed) = self.try_parse_string_as_temporal(s) {
                    return self.compare_values_direct(temporal, &parsed);
                }
                // Fall through to regular comparison if parsing fails
            }
            _ => {}
        }

        // Regular comparison without conversion
        self.compare_values_direct(left, right)
    }

    /// Compare two FhirPathValues for less than relationship without auto-conversion
    fn compare_values_direct(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            // Integer comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l < r),

            // Decimal comparison
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => Some(l < r),

            // Integer vs Decimal comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(left_decimal < *r)
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(*l < right_decimal)
            }

            // String comparison (lexicographic)
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => Some(l < r),

            // Date comparison
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => Some(l < r),

            // DateTime comparison
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => Some(l < r),

            // Time comparison
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l < r),

            // Quantity comparison (with same units)
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
                if lu == ru {
                    Some(lv < rv)
                } else {
                    // Different units - would need proper unit conversion
                    None
                }
            }

            // Other types are not orderable
            _ => None,
        }
    }
}

#[async_trait]
impl OperationEvaluator for LessThanOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if either operand is empty, result is empty
        if left.is_empty() || right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // For comparison, we compare the first elements
        let left_value = left.first().unwrap();
        let right_value = right.first().unwrap();

        match self.compare_values(left_value, right_value) {
            Some(result) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(result)),
            }),
            None => Ok(EvaluationResult {
                value: Collection::empty(),
            }),
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the less than operator
fn create_less_than_metadata() -> OperatorMetadata {
    // Support multiple ordered types
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any], // Will be validated at runtime for ordered types
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "<".to_string(),
        description: "Less than comparison for ordered types".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Integer],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Decimal],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Decimal],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Integer],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::String, FhirPathType::String],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Date, FhirPathType::Date],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::DateTime, FhirPathType::DateTime],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Time, FhirPathType::Time],
                    FhirPathType::Boolean,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Quantity],
                    FhirPathType::Boolean,
                ),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 6, // FHIRPath comparison precedence
        associativity: Associativity::Left,
    }
}
