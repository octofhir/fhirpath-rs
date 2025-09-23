//! Greater than (>) operator implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::quantity_utils;
use crate::evaluator::{EvaluationContext, EvaluationResult};

pub struct GreaterThanOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for GreaterThanOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl GreaterThanOperatorEvaluator {
    pub fn new() -> Self {
        let signature = TypeSignature::polymorphic(
            vec![FhirPathType::Any, FhirPathType::Any],
            FhirPathType::Boolean,
        );

        Self {
            metadata: OperatorMetadata {
                name: ">".to_string(),
                description: "Greater than comparison for ordered types".to_string(),
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
                    ],
                },
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                precedence: 6,
                associativity: Associativity::Left,
            },
        }
    }

    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Try to parse a string as a temporal value
    fn try_parse_string_as_temporal(&self, s: &str) -> Option<FhirPathValue> {
        if let Some(date) = PrecisionDate::parse(s) {
            return Some(FhirPathValue::date(date));
        }
        if let Some(datetime) = PrecisionDateTime::parse(s) {
            return Some(FhirPathValue::datetime(datetime));
        }
        if let Some(time) = PrecisionTime::parse(s) {
            return Some(FhirPathValue::time(time));
        }
        None
    }

    /// Check if a value is a temporal type
    fn is_temporal(&self, value: &FhirPathValue) -> bool {
        matches!(
            value,
            FhirPathValue::Date(_, _, _) | FhirPathValue::DateTime(_, _, _) | FhirPathValue::Time(_, _, _)
        )
    }

    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        // Handle string-to-temporal conversions
        match (left, right) {
            // String vs temporal types - try to parse string as temporal
            (FhirPathValue::String(s, _, _), temporal) if self.is_temporal(temporal) => {
                if let Some(parsed) = self.try_parse_string_as_temporal(s) {
                    return self.compare_values(&parsed, temporal);
                }
            }
            (temporal, FhirPathValue::String(s, _, _)) if self.is_temporal(temporal) => {
                if let Some(parsed) = self.try_parse_string_as_temporal(s) {
                    return self.compare_values(temporal, &parsed);
                }
            }
            _ => {}
        }

        match (left, right) {
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l > r),
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => Some(l > r),
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(left_decimal > *r)
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(*l > right_decimal)
            }
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => Some(l > r),
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => {
                // Use PartialOrd for proper temporal precision handling
                match l.partial_cmp(r) {
                    Some(std::cmp::Ordering::Greater) => Some(true),
                    Some(_) => Some(false), // Equal or Less
                    None => None,           // Uncertain due to precision differences
                }
            }
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                // Use PartialOrd for proper temporal precision handling
                match l.partial_cmp(r) {
                    Some(std::cmp::Ordering::Greater) => Some(true),
                    Some(_) => Some(false), // Equal or Less
                    None => None,           // Uncertain due to precision differences
                }
            }
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => {
                // Use PartialOrd for proper temporal precision handling
                match l.partial_cmp(r) {
                    Some(std::cmp::Ordering::Greater) => Some(true),
                    Some(_) => Some(false), // Equal or Less
                    None => None,           // Uncertain due to precision differences
                }
            }
            // Cross-type temporal comparisons: DateTime vs Date by comparing date components
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::Date(r, _, _)) => {
                let l_date = l.date();
                match l_date.partial_cmp(r) {
                    Some(std::cmp::Ordering::Greater) => Some(true),
                    Some(std::cmp::Ordering::Less) => Some(false),
                    Some(std::cmp::Ordering::Equal) => None, // Same day, time unknown
                    None => None,
                }
            }
            (FhirPathValue::Date(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                let r_date = r.date();
                match l.partial_cmp(&r_date) {
                    Some(std::cmp::Ordering::Greater) => Some(true),
                    Some(std::cmp::Ordering::Less) => Some(false),
                    Some(std::cmp::Ordering::Equal) => None, // Same day, time unknown
                    None => None,
                }
            }
            // Quantity comparison (with unit conversion)
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    calendar_unit: lc,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    calendar_unit: rc,
                    ..
                },
            ) => {
                // Use the quantity utilities for proper unit conversion
                match quantity_utils::compare_quantities(*lv, lu, lc, *rv, ru, rc) {
                    Ok(Some(std::cmp::Ordering::Greater)) => Some(true),
                    Ok(Some(_)) => Some(false), // Equal or Less
                    Ok(None) | Err(_) => None,  // Not comparable or conversion failed
                }
            }
            _ => None,
        }
    }
}

#[async_trait]
impl OperationEvaluator for GreaterThanOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        if left.is_empty() || right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        match self.compare_values(left.first().unwrap(), right.first().unwrap()) {
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
