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

use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use std::cmp::Ordering;

/// Specialized evaluator for comparison operations
pub struct ComparisonEvaluator;

impl ComparisonEvaluator {
    /// Helper to handle collection extraction for comparison operations
    fn extract_operands<'a>(
        left: &'a FhirPathValue,
        right: &'a FhirPathValue,
    ) -> (Option<&'a FhirPathValue>, Option<&'a FhirPathValue>) {
        let left_val = match left {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first()
                } else if items.is_empty() {
                    None
                } else {
                    None // Multi-element collections not supported for comparison
                }
            }
            FhirPathValue::Empty => None,
            val => Some(val),
        };

        let right_val = match right {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first()
                } else if items.is_empty() {
                    None
                } else {
                    None // Multi-element collections not supported for comparison
                }
            }
            FhirPathValue::Empty => None,
            val => Some(val),
        };

        (left_val, right_val)
    }

    /// Evaluate equals operation
    pub async fn evaluate_equals(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match Self::compare_equal_with_collections(left, right) {
            Some(result) => Ok(FhirPathValue::Boolean(result)),
            None => Ok(FhirPathValue::Empty), // Empty result per FHIRPath spec
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
                // For non-empty values, use same logic as equality but always return true/false (never empty)
                match Self::compare_equal_with_collections(left, right) {
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
                Some((a - b).abs() < Decimal::new(1, 10)) // Small epsilon for decimal comparison
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Some(a == b),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Some(a == b),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Some(a == b),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Some(a == b),

            // Cross-type numeric equality
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Some((a_decimal - b).abs() < Decimal::new(1, 10)),
                Err(_) => Some(false),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Some((a - b_decimal).abs() < Decimal::new(1, 10)),
                Err(_) => Some(false),
            },

            // Different types are not equal
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
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Some(a.cmp(b)),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Some(a.cmp(b)),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Some(a.cmp(b)),
            _ => None, // Incomparable types
        }
    }
}
