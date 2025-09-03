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

//! Comparison operations evaluator

use chrono::{Datelike, Timelike};
use octofhir_fhirpath_core::{EvaluationResult, FhirPathValue, JsonValueExt, PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
use rust_decimal::Decimal;
use std::cmp::Ordering;

/// Specialized evaluator for comparison operations
pub struct ComparisonEvaluator;

impl ComparisonEvaluator {
    /// Evaluate equals operation
    pub async fn evaluate_equals(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_equal_with_collections(left, right) {
            Some(result) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                result,
            )])),
            None => Ok(FhirPathValue::collection(vec![])), // Empty collection per FHIRPath spec
        }
    }

    /// Evaluate not equals operation
    pub async fn evaluate_not_equals(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_equal_with_collections(left, right) {
            Some(result) => Ok(FhirPathValue::Boolean(!result)),
            None => Ok(FhirPathValue::Empty), // Empty result per FHIRPath spec
        }
    }

    /// Evaluate less than operation
    pub async fn evaluate_less_than(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_values(left, right) {
            Some(Ordering::Less) => Ok(FhirPathValue::Boolean(true)),
            Some(_) => Ok(FhirPathValue::Boolean(false)),
            None => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate less than or equal operation
    pub async fn evaluate_less_than_or_equal(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_values(left, right) {
            Some(Ordering::Less) | Some(Ordering::Equal) => Ok(FhirPathValue::Boolean(true)),
            Some(Ordering::Greater) => Ok(FhirPathValue::Boolean(false)),
            None => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate greater than operation
    pub async fn evaluate_greater_than(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_values(left, right) {
            Some(Ordering::Greater) => Ok(FhirPathValue::Boolean(true)),
            Some(_) => Ok(FhirPathValue::Boolean(false)),
            None => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate greater than or equal operation
    pub async fn evaluate_greater_than_or_equal(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_values(left, right) {
            Some(Ordering::Greater) | Some(Ordering::Equal) => Ok(FhirPathValue::Boolean(true)),
            Some(Ordering::Less) => Ok(FhirPathValue::Boolean(false)),
            None => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate equivalence operation
    pub async fn evaluate_equivalent(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Equivalence is like equality but has different empty handling
        // ~ returns false for empty vs non-empty, true for empty vs empty
        match (left, right) {
            (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(FhirPathValue::Boolean(true)),
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Boolean(false))
            }
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r))
                if l.is_empty() && r.is_empty() =>
            {
                Ok(FhirPathValue::Boolean(true))
            }
            (FhirPathValue::Collection(l), _) if l.is_empty() => Ok(FhirPathValue::Boolean(false)),
            (_, FhirPathValue::Collection(r)) if r.is_empty() => Ok(FhirPathValue::Boolean(false)),
            _ => {
                // For non-empty values, use equivalence logic (case-insensitive for strings)
                match Self::compare_equivalent_with_collections(left, right) {
                    Some(result) => Ok(FhirPathValue::Boolean(result)),
                    None => Ok(FhirPathValue::Boolean(false)), // Equivalence treats indeterminate as false
                }
            }
        }
    }

    /// Evaluate not equivalent operation
    pub async fn evaluate_not_equivalent(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        // !~ is the negation of ~
        match Self::evaluate_equivalent(left, right).await? {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            _ => Ok(FhirPathValue::Boolean(true)), // If equivalent returns non-boolean, not equivalent is true
        }
    }

    // Private helper methods for actual comparison operations
    pub fn compare_equal_with_collections(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<bool> {
        match (left, right) {
            // Both empty collections - return empty (not true)
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r))
                if l.is_empty() && r.is_empty() =>
            {
                None
            }
            // Either is empty collection - return empty (not false)
            (FhirPathValue::Collection(l), _) if l.is_empty() => None,
            (_, FhirPathValue::Collection(r)) if r.is_empty() => None,
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => None,

            // Collection comparison - both must have same number of items and be equal element-wise
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.len() != r.len() {
                    return Some(false);
                }

                // Compare element by element using FHIRPath equality
                for (left_item, right_item) in l.iter().zip(r.iter()) {
                    match Self::compare_equal_with_collections(left_item, right_item) {
                        Some(false) => return Some(false), // Any element not equal = whole not equal
                        None => return None, // Any element comparison is empty = whole is empty
                        Some(true) => continue, // This element is equal, check next
                    }
                }
                Some(true) // All elements equal
            }

            // Single value vs collection - unwrap if singleton
            (FhirPathValue::Collection(l), right_val) => {
                if l.len() == 1 {
                    Self::compare_equal_with_collections(l.first().unwrap(), right_val)
                } else {
                    Some(false) // Multi-element collection vs single value
                }
            }
            (left_val, FhirPathValue::Collection(r)) => {
                if r.len() == 1 {
                    Self::compare_equal_with_collections(left_val, r.first().unwrap())
                } else {
                    Some(false) // Single value vs multi-element collection
                }
            }

            // Scalar value comparisons
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Some(a == b),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Some(a == b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                // FHIRPath equivalence uses a reasonable tolerance for decimal comparison
                // This matches common rounding expectations (e.g., 0.666... ~ 0.67)
                Some((a - b).abs() < Decimal::new(5, 3)) // 0.005 tolerance
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Some(a == b),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                // Different precision levels make equality indeterminate
                if a.precision != b.precision {
                    None
                } else {
                    Some(a == b)
                }
            }
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                // For DateTime equality, compare the actual datetime values
                // If times are equal, they're equal regardless of precision
                Some(a.datetime == b.datetime)
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                // For Time equality, compare the actual time values
                Some(a.time == b.time)
            }

            // Cross-type date/datetime equality
            (FhirPathValue::Date(date), FhirPathValue::DateTime(datetime)) => {
                // Convert date to midnight datetime for comparison
                Self::compare_date_with_datetime(&date.date, &datetime.datetime)
                    .map(|ord| ord == Ordering::Equal)
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Date(date)) => {
                // Convert date to midnight datetime for comparison
                Self::compare_date_with_datetime(&date.date, &datetime.datetime)
                    .map(|ord| ord == Ordering::Equal)
            }

            // Cross-type numeric equality
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Some((a_decimal - b).abs() < Decimal::new(1, 10)),
                Err(_) => Some(false),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Some((a - b_decimal).abs() < Decimal::new(1, 10)),
                Err(_) => Some(false),
            },

            // Quantity equality with unit conversion support
            (FhirPathValue::Quantity { value: a_val, unit: a_unit, .. }, 
             FhirPathValue::Quantity { value: b_val, unit: b_unit, .. }) => {
                // For now, simple comparison - units must match exactly
                if a_unit == b_unit {
                    Some((a_val - b_val).abs() < Decimal::new(1, 10)) // Small tolerance
                } else {
                    Some(false) // Different units
                }
            }

            // JsonValue comparisons - handle Sonic JSON values natively
            (FhirPathValue::JsonValue(a), FhirPathValue::JsonValue(b)) => {
                Some(a.as_inner() == b.as_inner())
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::String(string_val)) => {
                // Compare JsonValue with String - use Sonic JSON directly
                if json_val.is_string() {
                    if let Some(json_str) = json_val.as_str() {
                        Some(json_str == string_val.as_str())
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false) // JsonValue is not a string, can't equal a string
                }
            }
            (FhirPathValue::String(string_val), FhirPathValue::JsonValue(json_val)) => {
                // Compare String with JsonValue - use Sonic JSON directly
                if json_val.is_string() {
                    if let Some(json_str) = json_val.as_str() {
                        Some(string_val.as_str() == json_str)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false) // JsonValue is not a string, can't equal a string
                }
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::Boolean(bool_val)) => {
                // Compare JsonValue with Boolean - use Sonic JSON directly
                if json_val.is_boolean() {
                    if let Some(json_bool) = json_val.as_bool() {
                        Some(json_bool == *bool_val)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }
            (FhirPathValue::Boolean(bool_val), FhirPathValue::JsonValue(json_val)) => {
                // Compare Boolean with JsonValue - use Sonic JSON directly
                if json_val.is_boolean() {
                    if let Some(json_bool) = json_val.as_bool() {
                        Some(*bool_val == json_bool)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::Integer(int_val)) => {
                // Compare JsonValue with Integer - use Sonic JSON directly
                if json_val.is_number() {
                    if let Some(json_int) = json_val.as_i64() {
                        Some(json_int == *int_val)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }
            (FhirPathValue::Integer(int_val), FhirPathValue::JsonValue(json_val)) => {
                // Compare Integer with JsonValue - use Sonic JSON directly
                if json_val.is_number() {
                    if let Some(json_int) = json_val.as_i64() {
                        Some(*int_val == json_int)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }

            // Different types are not equal
            _ => Some(false),
        }
    }

    /// Compare values for equivalence (~) - like equality but case-insensitive for strings
    pub fn compare_equivalent_with_collections(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<bool> {
        match (left, right) {
            // Both empty collections - return true (equivalent)
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r))
                if l.is_empty() && r.is_empty() =>
            {
                Some(true)
            }

            // Single item collections - compare the contents
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r))
                if l.len() == 1 && r.len() == 1 =>
            {
                Self::compare_equivalent_with_collections(l.first().unwrap(), r.first().unwrap())
            }

            // Collection vs scalar - extract scalar and compare
            (FhirPathValue::Collection(l), right) if l.len() == 1 => {
                Self::compare_equivalent_with_collections(l.first().unwrap(), right)
            }
            (left, FhirPathValue::Collection(r)) if r.len() == 1 => {
                Self::compare_equivalent_with_collections(left, r.first().unwrap())
            }

            // Multi-element collections
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.len() != r.len() {
                    return Some(false); // Different lengths can't be equivalent
                }

                // For equivalence, order doesn't matter - each item in left must have a match in right
                let all_left_match = l.iter().all(|left_item| {
                    r.iter().any(|right_item| {
                        Self::compare_equivalent_with_collections(left_item, right_item)
                            .unwrap_or(false)
                    })
                });

                if !all_left_match {
                    return Some(false);
                }

                // Also check that each item in right has a match in left (for duplicates)
                let all_right_match = r.iter().all(|right_item| {
                    l.iter().any(|left_item| {
                        Self::compare_equivalent_with_collections(left_item, right_item)
                            .unwrap_or(false)
                    })
                });

                Some(all_right_match)
            }
            (FhirPathValue::Collection(_), _) | (_, FhirPathValue::Collection(_)) => {
                Some(false) // Single value vs multi-element collection
            }

            // Scalar value comparisons - mostly same as equality except strings
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Some(a == b),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Some(a == b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                // FHIRPath equivalence uses a reasonable tolerance for decimal comparison
                // This matches common rounding expectations (e.g., 0.666... ~ 0.67)
                Some((a - b).abs() < Decimal::new(5, 3)) // 0.005 tolerance
            }

            // String comparison - CASE INSENSITIVE for equivalence
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Some(a.to_lowercase() == b.to_lowercase())
            }

            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                // Different precision levels make equivalence indeterminate
                if a.precision != b.precision {
                    None
                } else {
                    Some(a == b)
                }
            }
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                // For DateTime equivalence, compare the actual datetime values
                Some(a.datetime == b.datetime)
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                // For Time equivalence, compare the actual time values
                Some(a.time == b.time)
            }

            // Cross-type date/datetime equality
            (FhirPathValue::Date(date), FhirPathValue::DateTime(datetime)) => {
                Self::compare_date_with_datetime(&date.date, &datetime.datetime)
                    .map(|ord| ord == Ordering::Equal)
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Date(date)) => {
                Self::compare_date_with_datetime(&date.date, &datetime.datetime)
                    .map(|ord| ord == Ordering::Equal)
            }

            // Cross-type numeric equality
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Some((a_decimal - b).abs() < Decimal::new(1, 10)),
                Err(_) => Some(false),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Some((a - b_decimal).abs() < Decimal::new(1, 10)),
                Err(_) => Some(false),
            },

            // Quantity equivalence with unit conversion support
            (FhirPathValue::Quantity { value: a_val, unit: a_unit, .. }, 
             FhirPathValue::Quantity { value: b_val, unit: b_unit, .. }) => {
                // For now, simple comparison - units must match exactly
                if a_unit == b_unit {
                    Some((a_val - b_val).abs() < Decimal::new(5, 3)) // 0.005 tolerance for equivalence
                } else {
                    Some(false) // Different units
                }
            }

            // JsonValue comparisons - handle Sonic JSON values natively for equivalence
            (FhirPathValue::JsonValue(a), FhirPathValue::JsonValue(b)) => {
                Some(a.as_inner() == b.as_inner())
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::String(string_val)) => {
                // Compare JsonValue with String - case insensitive for equivalence
                if json_val.is_string() {
                    if let Some(json_str) = json_val.as_str() {
                        Some(json_str.to_lowercase() == string_val.to_lowercase())
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false) // JsonValue is not a string, can't equal a string
                }
            }
            (FhirPathValue::String(string_val), FhirPathValue::JsonValue(json_val)) => {
                // Compare String with JsonValue - case insensitive for equivalence
                if json_val.is_string() {
                    if let Some(json_str) = json_val.as_str() {
                        Some(string_val.to_lowercase() == json_str.to_lowercase())
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false) // JsonValue is not a string, can't equal a string
                }
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::Boolean(bool_val)) => {
                // Compare JsonValue with Boolean for equivalence
                if json_val.is_boolean() {
                    if let Some(json_bool) = json_val.as_bool() {
                        Some(json_bool == *bool_val)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }
            (FhirPathValue::Boolean(bool_val), FhirPathValue::JsonValue(json_val)) => {
                // Compare Boolean with JsonValue for equivalence
                if json_val.is_boolean() {
                    if let Some(json_bool) = json_val.as_bool() {
                        Some(*bool_val == json_bool)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::Integer(int_val)) => {
                // Compare JsonValue with Integer for equivalence
                if json_val.is_number() {
                    if let Some(json_int) = json_val.as_i64() {
                        Some(json_int == *int_val)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }
            (FhirPathValue::Integer(int_val), FhirPathValue::JsonValue(json_val)) => {
                // Compare Integer with JsonValue for equivalence
                if json_val.is_number() {
                    if let Some(json_int) = json_val.as_i64() {
                        Some(*int_val == json_int)
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                }
            }

            // Different types are not equivalent
            _ => Some(false),
        }
    }

    fn compare_values(left: &FhirPathValue, right: &FhirPathValue) -> Option<Ordering> {
        // Handle collections - must be singletons for comparison
        let (left_val, right_val) = match (left, right) {
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.len() == 1 && r.len() == 1 {
                    (l.first().unwrap(), r.first().unwrap())
                } else {
                    return None; // Empty or multi-element collections
                }
            }
            (FhirPathValue::Collection(l), other) => {
                if l.len() == 1 {
                    (l.first().unwrap(), other)
                } else {
                    return None;
                }
            }
            (other, FhirPathValue::Collection(r)) => {
                if r.len() == 1 {
                    (other, r.first().unwrap())
                } else {
                    return None;
                }
            }
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => return None,
            _ => (left, right),
        };

        // Compare scalar values
        match (left_val, right_val) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Some(a.cmp(b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Some(a.cmp(b)),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Some(a_decimal.cmp(b)),
                Err(_) => None,
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Some(a.cmp(&b_decimal)),
                Err(_) => None,
            },
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Some(a.cmp(b)),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                // Check if precisions are compatible - if different, comparison is indeterminate
                if a.precision != b.precision {
                    None // Different precision levels make comparison indeterminate
                } else {
                    Some(a.date.cmp(&b.date))
                }
            }
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                // For DateTime comparison with different precisions, check if comparison is indeterminate
                let cmp_result = a.datetime.cmp(&b.datetime);
                if cmp_result == Ordering::Equal {
                    Some(Ordering::Equal) // Same moment in time, regardless of precision
                } else if a.precision == b.precision {
                    Some(cmp_result) // Same precision, definite comparison
                } else {
                    // Different precisions - check if values are indeterminate
                    // If they differ only in precision (e.g., 10:30 vs 10:30:00), it's indeterminate
                    Self::compare_datetime_with_precision_check(a, b)
                }
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                // For Time comparison with different precisions
                let cmp_result = a.time.cmp(&b.time);
                if cmp_result == Ordering::Equal {
                    Some(Ordering::Equal) // Same moment in time, regardless of precision
                } else if a.precision == b.precision {
                    Some(cmp_result) // Same precision, definite comparison
                } else {
                    // Different precisions - check if values are indeterminate
                    Self::compare_time_with_precision_check(a, b)
                }
            }

            // Cross-type date/datetime comparisons
            (FhirPathValue::Date(date), FhirPathValue::DateTime(datetime)) => {
                // Convert date to midnight datetime for comparison
                Self::compare_date_with_datetime(&date.date, &datetime.datetime)
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Date(date)) => {
                // Convert date to midnight datetime for comparison
                Self::compare_date_with_datetime(&date.date, &datetime.datetime)
                    .map(|ord| ord.reverse())
            }

            // Quantity comparison with unit conversion support
            (FhirPathValue::Quantity { value: a_val, unit: a_unit, .. }, 
             FhirPathValue::Quantity { value: b_val, unit: b_unit, .. }) => {
                // For now, simple comparison - units must match exactly
                if a_unit == b_unit {
                    Some(a_val.cmp(b_val))
                } else {
                    None // Different units - incomparable for now
                }
            }

            _ => None, // Incomparable types
        }
    }


    /// Compare DateTime values with precision-aware logic
    fn compare_datetime_with_precision_check(
        a: &PrecisionDateTime,
        b: &PrecisionDateTime,
    ) -> Option<Ordering> {

        // Truncate both datetimes to the lowest precision level
        let min_precision = std::cmp::min(a.precision, b.precision);

        match min_precision {
            TemporalPrecision::Year => {
                let a_year = a.datetime.year();
                let b_year = b.datetime.year();
                Some(a_year.cmp(&b_year))
            }
            TemporalPrecision::Month => {
                let a_month = (a.datetime.year(), a.datetime.month());
                let b_month = (b.datetime.year(), b.datetime.month());
                Some(a_month.cmp(&b_month))
            }
            TemporalPrecision::Day => {
                let a_date = a.datetime.date_naive();
                let b_date = b.datetime.date_naive();
                Some(a_date.cmp(&b_date))
            }
            TemporalPrecision::Hour => {
                let a_hour = (a.datetime.date_naive(), a.datetime.hour());
                let b_hour = (b.datetime.date_naive(), b.datetime.hour());
                Some(a_hour.cmp(&b_hour))
            }
            TemporalPrecision::Minute => {
                let a_min = (
                    a.datetime.date_naive(),
                    a.datetime.hour(),
                    a.datetime.minute(),
                );
                let b_min = (
                    b.datetime.date_naive(),
                    b.datetime.hour(),
                    b.datetime.minute(),
                );
                // Check if they're equal at minute level - if so, comparison is indeterminate
                if a_min == b_min {
                    None // Indeterminate - could be equal or different depending on seconds
                } else {
                    Some(a_min.cmp(&b_min))
                }
            }
            TemporalPrecision::Second => {
                let a_sec = (
                    a.datetime.date_naive(),
                    a.datetime.hour(),
                    a.datetime.minute(),
                    a.datetime.second(),
                );
                let b_sec = (
                    b.datetime.date_naive(),
                    b.datetime.hour(),
                    b.datetime.minute(),
                    b.datetime.second(),
                );
                // Check if they're equal at second level - if so, comparison is indeterminate
                if a_sec == b_sec {
                    None // Indeterminate - could be equal or different depending on milliseconds
                } else {
                    Some(a_sec.cmp(&b_sec))
                }
            }
            TemporalPrecision::Millisecond => {
                // Full precision comparison
                Some(a.datetime.cmp(&b.datetime))
            }
        }
    }

    /// Compare Time values with precision-aware logic
    fn compare_time_with_precision_check(
        a: &PrecisionTime,
        b: &PrecisionTime,
    ) -> Option<Ordering> {

        // Truncate both times to the lowest precision level
        let min_precision = std::cmp::min(a.precision, b.precision);

        match min_precision {
            TemporalPrecision::Hour => {
                let a_hour = a.time.hour();
                let b_hour = b.time.hour();
                Some(a_hour.cmp(&b_hour))
            }
            TemporalPrecision::Minute => {
                let a_min = (a.time.hour(), a.time.minute());
                let b_min = (b.time.hour(), b.time.minute());
                // Check if they're equal at minute level - if so, comparison is indeterminate
                if a_min == b_min {
                    None // Indeterminate - could be equal or different depending on seconds
                } else {
                    Some(a_min.cmp(&b_min))
                }
            }
            TemporalPrecision::Second => {
                let a_sec = (a.time.hour(), a.time.minute(), a.time.second());
                let b_sec = (b.time.hour(), b.time.minute(), b.time.second());
                // Check if they're equal at second level - if so, comparison is indeterminate
                if a_sec == b_sec {
                    None // Indeterminate - could be equal or different depending on milliseconds
                } else {
                    Some(a_sec.cmp(&b_sec))
                }
            }
            TemporalPrecision::Millisecond => {
                // Full precision comparison
                Some(a.time.cmp(&b.time))
            }
            _ => {
                // Default to minute precision for time
                let a_min = (a.time.hour(), a.time.minute());
                let b_min = (b.time.hour(), b.time.minute());
                if a_min == b_min {
                    None
                } else {
                    Some(a_min.cmp(&b_min))
                }
            }
        }
    }

    /// Compare a Date with a DateTime using FHIRPath semantics
    /// Date represents a full day, so comparison depends on whether datetime falls within that day
    fn compare_date_with_datetime(
        date: &chrono::NaiveDate,
        datetime: &chrono::DateTime<chrono::FixedOffset>,
    ) -> Option<Ordering> {
        // Convert date to start and end of day in UTC for comparison
        let date_start = date
            .and_hms_opt(0, 0, 0)?
            .and_local_timezone(chrono::FixedOffset::east_opt(0)?)
            .single()?;
        let date_end = date
            .and_hms_opt(23, 59, 59)?
            .and_local_timezone(chrono::FixedOffset::east_opt(0)?)
            .single()?;

        // FHIRPath comparison semantics for date vs datetime:
        // - If datetime is before the date, return Greater (date > datetime)
        // - If datetime is after the date, return Less (date < datetime)
        // - If datetime is within the date range, comparison is indeterminate (return None)
        if datetime < &date_start {
            Some(Ordering::Greater) // date > datetime
        } else if datetime > &date_end {
            Some(Ordering::Less) // date < datetime
        } else {
            // Datetime falls within the date range - comparison is indeterminate
            // For FHIRPath, this should return None (empty result)
            None
        }
    }
}
