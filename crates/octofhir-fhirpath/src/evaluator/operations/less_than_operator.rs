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
use crate::evaluator::quantity_utils;
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Less than operator evaluator
pub struct LessThanOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for LessThanOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
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

            // String comparison - try temporal parsing first, fall back to lexicographic
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => {
                self.compare_strings_with_temporal_parsing(l, r)
            }

            // Date comparison
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => {
                // Use PartialOrd for proper temporal precision handling
                match l.partial_cmp(r) {
                    Some(std::cmp::Ordering::Less) => Some(true),
                    Some(_) => Some(false), // Equal or Greater
                    None => None,           // Uncertain due to precision differences
                }
            }

            // DateTime comparison
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                // Use PartialOrd for proper temporal precision handling
                match l.partial_cmp(r) {
                    Some(std::cmp::Ordering::Less) => Some(true),
                    Some(_) => Some(false), // Equal or Greater
                    None => None,           // Uncertain due to precision differences
                }
            }

            // Time comparison
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => {
                // Use PartialOrd for proper temporal precision handling
                match l.partial_cmp(r) {
                    Some(std::cmp::Ordering::Less) => Some(true),
                    Some(_) => Some(false), // Equal or Greater
                    None => None,           // Uncertain due to precision differences
                }
            }

            // Cross-type temporal comparisons: Date vs DateTime
            // Allow comparison by promoting Date to DateTime for practical use cases like boundary comparisons
            (FhirPathValue::Date(date, _, _), FhirPathValue::DateTime(datetime, _, _)) => {
                // Convert Date to DateTime with time 00:00:00 and compare
                // Promote Date precision to match DateTime precision to enable comparison
                use chrono::FixedOffset;
                let naive_datetime = date.date.and_hms_opt(0, 0, 0).unwrap();
                let date_as_datetime = PrecisionDateTime {
                    datetime: naive_datetime
                        .and_local_timezone(FixedOffset::east_opt(0).unwrap())
                        .single()
                        .unwrap(),
                    precision: datetime.precision, // Use DateTime's precision for comparison compatibility
                    tz_specified: datetime.tz_specified, // Match timezone specification of the other value
                };
                match date_as_datetime.partial_cmp(datetime) {
                    Some(std::cmp::Ordering::Less) => Some(true),
                    Some(_) => Some(false), // Equal or Greater
                    None => None,           // Uncertain due to precision differences
                }
            }
            (FhirPathValue::DateTime(datetime, _, _), FhirPathValue::Date(date, _, _)) => {
                // Convert Date to DateTime with time 00:00:00 and compare
                // Promote Date precision to match DateTime precision to enable comparison
                use chrono::FixedOffset;
                let naive_datetime = date.date.and_hms_opt(0, 0, 0).unwrap();
                let date_as_datetime = PrecisionDateTime {
                    datetime: naive_datetime
                        .and_local_timezone(FixedOffset::east_opt(0).unwrap())
                        .single()
                        .unwrap(),
                    precision: datetime.precision, // Use DateTime's precision for comparison compatibility
                    tz_specified: datetime.tz_specified, // Match timezone specification of the other value
                };
                match datetime.partial_cmp(&date_as_datetime) {
                    Some(std::cmp::Ordering::Less) => Some(true),
                    Some(_) => Some(false), // Equal or Greater
                    None => None,           // Uncertain due to precision differences
                }
            }

            // Other cross-type temporal comparisons are not supported
            (FhirPathValue::Date(_, _, _), FhirPathValue::Time(_, _, _)) => None,
            (FhirPathValue::Time(_, _, _), FhirPathValue::Date(_, _, _)) => None,
            (FhirPathValue::DateTime(_, _, _), FhirPathValue::Time(_, _, _)) => None,
            (FhirPathValue::Time(_, _, _), FhirPathValue::DateTime(_, _, _)) => None,

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
                    Ok(Some(std::cmp::Ordering::Less)) => Some(true),
                    Ok(Some(_)) => Some(false), // Equal or Greater
                    Ok(None) | Err(_) => None,  // Not comparable or conversion failed
                }
            }

            // Other types are not orderable
            _ => None,
        }
    }

    /// Compare two strings, attempting to parse as temporal values first
    fn compare_strings_with_temporal_parsing(&self, left: &str, right: &str) -> Option<bool> {
        // Try to parse both strings as temporal values
        match (PrecisionDate::parse(left), PrecisionDate::parse(right)) {
            (Some(l_date), Some(r_date)) => {
                // Both parsed as dates - use temporal comparison with precision awareness
                match l_date.partial_cmp(&r_date) {
                    Some(std::cmp::Ordering::Less) => Some(true),
                    Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {
                        Some(false)
                    }
                    None => None, // Different precisions or uncertain comparison
                }
            }
            _ => {
                // Try parsing as datetimes
                match (
                    PrecisionDateTime::parse(left),
                    PrecisionDateTime::parse(right),
                ) {
                    (Some(l_dt), Some(r_dt)) => {
                        // Both parsed as datetimes - use temporal comparison with precision awareness
                        match l_dt.partial_cmp(&r_dt) {
                            Some(std::cmp::Ordering::Less) => Some(true),
                            Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {
                                Some(false)
                            }
                            None => None, // Different precisions or uncertain comparison
                        }
                    }
                    _ => {
                        // Try mixed date/datetime parsing
                        match (PrecisionDate::parse(left), PrecisionDateTime::parse(right)) {
                            (Some(_), Some(_)) => {
                                // According to FHIRPath spec: different precision levels return empty
                                None
                            }
                            _ => {
                                match (PrecisionDateTime::parse(left), PrecisionDate::parse(right))
                                {
                                    (Some(_), Some(_)) => {
                                        // According to FHIRPath spec: different precision levels return empty
                                        None
                                    }
                                    _ => {
                                        // Neither string could be parsed as temporal - fall back to lexicographic comparison
                                        Some(left < right)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl OperationEvaluator for LessThanOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
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

        // Detect invalid numeric vs string comparison and raise execution error
        let is_numeric = matches!(left_value, FhirPathValue::Integer(_, _, _) | FhirPathValue::Decimal(_, _, _) | FhirPathValue::Quantity { .. })
            || matches!(right_value, FhirPathValue::Integer(_, _, _) | FhirPathValue::Decimal(_, _, _) | FhirPathValue::Quantity { .. });
        let is_string_pair = matches!(left_value, FhirPathValue::String(_, _, _)) || matches!(right_value, FhirPathValue::String(_, _, _));
        if is_numeric && is_string_pair {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0051,
                "Type mismatch: cannot compare numeric and string with '<'".to_string(),
            ));
        }

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
