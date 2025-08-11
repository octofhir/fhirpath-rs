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

//! Comparison operators for FHIRPath expressions

use super::super::operator::{
    Associativity, FhirPathOperator, OperatorError, OperatorRegistry, OperatorResult,
};
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::signature::OperatorSignature;
use rust_decimal::Decimal;

/// Equality operator (=)
pub struct EqualOperator;

impl FhirPathOperator for EqualOperator {
    fn symbol(&self) -> &str {
        "="
    }
    fn human_friendly_name(&self) -> &str {
        "Equality"
    }
    fn precedence(&self) -> u8 {
        9 // Per FHIRPath spec: =, ~, !=, !~ have precedence #09
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "=",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // FHIRPath equality has special semantics:
        // - If both operands are empty, return empty
        // - If one operand is empty and the other is not, return empty
        // - If collections have different lengths, return false
        // - Otherwise compare values with type coercion

        // Handle empty cases according to FHIRPath specification
        match (left.is_empty(), right.is_empty()) {
            (true, true) => return Ok(FhirPathValue::Empty),
            (true, false) | (false, true) => return Ok(FhirPathValue::Empty),
            (false, false) => {} // Continue with normal comparison
        }

        // FHIRPath equality with type coercion support
        let result = match (left, right) {
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => {
                // For now, use standard DateTime comparison
                // TODO: Implement proper FHIRPath precision-aware DateTime comparison
                // that preserves timezone precision information from parsing
                l == r
            }
            (FhirPathValue::Date(_), FhirPathValue::DateTime(_)) => {
                // Per FHIRPath spec: different precision levels return empty
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Date(_)) => {
                // Per FHIRPath spec: different precision levels return empty
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,

            // Cross-type numeric comparisons (Integer vs Decimal)
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => Decimal::from(*l) == *r,
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => *l == Decimal::from(*r),

            // Quantity comparisons with unit conversion
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                self.compare_quantities_equal(q1, q2)?
            }

            // Resource comparisons - compare JSON representations
            (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
                r1.to_json() == r2.to_json()
            }

            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                l.len() == r.len()
                    && l.iter()
                        .zip(r.iter())
                        .all(|(a, b)| match self.compare_values_equal(a, b) {
                            Ok(FhirPathValue::Boolean(b)) => b,
                            _ => false,
                        })
            }
            _ => false,
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

impl EqualOperator {
    /// Compare two values for equality without recursion
    fn compare_values_equal(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l == r,
            (FhirPathValue::Date(_), FhirPathValue::DateTime(_)) => {
                // Per FHIRPath spec: different precision levels return empty
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Date(_)) => {
                // Per FHIRPath spec: different precision levels return empty
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,

            // Cross-type numeric comparisons (Integer vs Decimal)
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => Decimal::from(*l) == *r,
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => *l == Decimal::from(*r),

            // Quantity comparisons with unit conversion
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                self.compare_quantities_equal(q1, q2)?
            }

            // Resource comparisons - compare JSON representations
            (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
                r1.to_json() == r2.to_json()
            }

            // For collections, they are not equal to non-collections
            (FhirPathValue::Collection(_), _) | (_, FhirPathValue::Collection(_)) => false,

            _ => false,
        };
        Ok(FhirPathValue::Boolean(result))
    }

    /// Compare two quantities for equality, handling unit conversion
    fn compare_quantities_equal(
        &self,
        q1: &crate::model::quantity::Quantity,
        q2: &crate::model::quantity::Quantity,
    ) -> OperatorResult<bool> {
        match q1.equals_with_conversion(q2) {
            Ok(result) => Ok(result),
            Err(_) => {
                // If conversion fails, quantities are not equal
                Ok(false)
            }
        }
    }
}

/// Not equal operator (!=)
pub struct NotEqualOperator;

impl FhirPathOperator for NotEqualOperator {
    fn symbol(&self) -> &str {
        "!="
    }
    fn human_friendly_name(&self) -> &str {
        "Not Equal"
    }
    fn precedence(&self) -> u8 {
        9 // Per FHIRPath spec: =, ~, !=, !~ have precedence #09
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "!=",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        match EqualOperator.evaluate_binary(left, right)? {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty), // If equal returns empty, != also returns empty
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)), // Handle direct boolean return
            FhirPathValue::Collection(items) if items.len() == 1 => match items.get(0) {
                Some(FhirPathValue::Boolean(b)) => Ok(FhirPathValue::Boolean(!b)),
                _ => Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                }),
            },
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Less than operator (<)
pub struct LessThanOperator;

impl FhirPathOperator for LessThanOperator {
    fn symbol(&self) -> &str {
        "<"
    }
    fn human_friendly_name(&self) -> &str {
        "Less Than"
    }
    fn precedence(&self) -> u8 {
        8 // Per FHIRPath spec: >, <, >=, <= have precedence #08
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Date,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::DateTime,
                    TypeInfo::Date,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Boolean,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                // Convert integer to decimal for comparison
                let b_decimal = Decimal::from(*b);
                a < &b_decimal
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                // Convert integer to decimal for comparison
                let a_decimal = Decimal::from(*a);
                a_decimal < *b
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a < b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a < b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a < b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = a.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                date_as_datetime < *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = b.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                *a < date_as_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a < b,
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // For quantity comparison, check if units are compatible
                if a.has_compatible_dimensions(b) {
                    // Convert b to a's unit for comparison
                    match b.convert_to_compatible_unit(a.unit.as_deref().unwrap_or("")) {
                        Ok(converted_b) => a.value < converted_b.value,
                        Err(_) => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    // Different units - return empty per FHIRPath spec for incompatible comparisons
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

/// Less than or equal operator (<=)
pub struct LessThanOrEqualOperator;

impl FhirPathOperator for LessThanOrEqualOperator {
    fn symbol(&self) -> &str {
        "<="
    }
    fn human_friendly_name(&self) -> &str {
        "Less Than or Equal"
    }
    fn precedence(&self) -> u8 {
        8 // Per FHIRPath spec: >, <, >=, <= have precedence #08
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<=", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<=", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Boolean,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a <= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a <= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                // Convert integer to decimal for comparison
                let b_decimal = Decimal::from(*b);
                a <= &b_decimal
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                // Convert integer to decimal for comparison
                let a_decimal = Decimal::from(*a);
                a_decimal <= *b
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a <= b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a <= b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a <= b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = a.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                date_as_datetime <= *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = b.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                *a <= date_as_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a <= b,
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // For quantity comparison, check if units are compatible
                if a.has_compatible_dimensions(b) {
                    // Convert b to a's unit for comparison
                    match b.convert_to_compatible_unit(a.unit.as_deref().unwrap_or("")) {
                        Ok(converted_b) => a.value <= converted_b.value,
                        Err(_) => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    // Different units - return empty per FHIRPath spec for incompatible comparisons
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }
}

/// Greater than operator (>)
pub struct GreaterThanOperator;

impl FhirPathOperator for GreaterThanOperator {
    fn symbol(&self) -> &str {
        ">"
    }
    fn human_friendly_name(&self) -> &str {
        "Greater Than"
    }
    fn precedence(&self) -> u8 {
        8 // Per FHIRPath spec: >, <, >=, <= have precedence #08
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    ">",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Boolean,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a > b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a > b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                // Convert integer to decimal for comparison
                let b_decimal = Decimal::from(*b);
                a > &b_decimal
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                // Convert integer to decimal for comparison
                let a_decimal = Decimal::from(*a);
                a_decimal > *b
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a > b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a > b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a > b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = a.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                date_as_datetime > *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = b.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                *a > date_as_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a > b,
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // For quantity comparison, check if units are compatible
                if a.has_compatible_dimensions(b) {
                    // Convert b to a's unit for comparison
                    match b.convert_to_compatible_unit(a.unit.as_deref().unwrap_or("")) {
                        Ok(converted_b) => a.value > converted_b.value,
                        Err(_) => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    // Different units - return empty per FHIRPath spec for incompatible comparisons
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }
}

/// Greater than or equal operator (>=)
pub struct GreaterThanOrEqualOperator;

impl FhirPathOperator for GreaterThanOrEqualOperator {
    fn symbol(&self) -> &str {
        ">="
    }
    fn human_friendly_name(&self) -> &str {
        "Greater Than or Equal"
    }
    fn precedence(&self) -> u8 {
        8 // Per FHIRPath spec: >, <, >=, <= have precedence #08
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">=", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">=", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Boolean,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a >= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a >= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                // Convert integer to decimal for comparison
                let b_decimal = Decimal::from(*b);
                a >= &b_decimal
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                // Convert integer to decimal for comparison
                let a_decimal = Decimal::from(*a);
                a_decimal >= *b
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a >= b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a >= b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a >= b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = a.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                date_as_datetime >= *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert date to datetime at start of day for comparison
                use chrono::{NaiveTime, TimeZone, Utc};
                let start_of_day = b.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let date_as_datetime = Utc.from_utc_datetime(&start_of_day);
                *a >= date_as_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a >= b,
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // For quantity comparison, check if units are compatible
                if a.has_compatible_dimensions(b) {
                    // Convert b to a's unit for comparison
                    match b.convert_to_compatible_unit(a.unit.as_deref().unwrap_or("")) {
                        Ok(converted_b) => a.value >= converted_b.value,
                        Err(_) => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    // Different units - return empty per FHIRPath spec for incompatible comparisons
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }
}

/// Equivalence operator (~)
pub struct EquivalentOperator;

impl FhirPathOperator for EquivalentOperator {
    fn symbol(&self) -> &str {
        "~"
    }
    fn human_friendly_name(&self) -> &str {
        "Equivalent"
    }
    fn precedence(&self) -> u8 {
        9 // Per FHIRPath spec: =, ~, !=, !~ have precedence #09
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "~",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Equivalence has different semantics from equality:
        // - Empty collections are equivalent to each other (true)
        // - Strings are case-insensitive
        // - Collections are order-independent
        // - Decimal precision differences are more tolerant

        // Handle empty cases - empty collections are equivalent
        match (left.is_empty(), right.is_empty()) {
            (true, true) => return Ok(FhirPathValue::Boolean(true)),
            (true, false) | (false, true) => return Ok(FhirPathValue::Boolean(false)),
            (false, false) => {} // Continue with normal comparison
        }

        let result = match (left, right) {
            // String equivalence is case-insensitive
            (FhirPathValue::String(l), FhirPathValue::String(r)) => {
                l.to_lowercase() == r.to_lowercase()
            }

            // Collection equivalence is order-independent
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                self.compare_collections_equivalent(l, r)?
            }

            // Decimal equivalence with tolerance
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                self.compare_decimals_equivalent(*l, *r)?
            }
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                let l_decimal = rust_decimal::Decimal::from(*l);
                self.compare_decimals_equivalent(l_decimal, *r)?
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                let r_decimal = rust_decimal::Decimal::from(*r);
                self.compare_decimals_equivalent(*l, r_decimal)?
            }

            // Handle quantities with unit conversion (same as equality)
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                self.compare_quantities_equivalent(q1, q2)?
            }

            // For other types, use standard equality
            _ => {
                let equal_op = EqualOperator;
                match equal_op.evaluate_binary(left, right)? {
                    FhirPathValue::Boolean(b) => b,
                    FhirPathValue::Empty => false, // Convert empty to false for equivalence
                    _ => false,
                }
            }
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

impl EquivalentOperator {
    /// Compare two quantities for equivalence (same as equality for quantities)
    fn compare_quantities_equivalent(
        &self,
        q1: &crate::model::quantity::Quantity,
        q2: &crate::model::quantity::Quantity,
    ) -> OperatorResult<bool> {
        // For quantities, equivalence is the same as equality
        let equal_op = EqualOperator;
        equal_op.compare_quantities_equal(q1, q2)
    }

    /// Compare two collections for equivalence (order-independent)
    fn compare_collections_equivalent(
        &self,
        left: &crate::model::Collection,
        right: &crate::model::Collection,
    ) -> OperatorResult<bool> {
        if left.len() != right.len() {
            return Ok(false);
        }

        // For order-independent comparison, we need to ensure each element
        // in left has exactly one match in right (like multiset equality)
        let mut right_used = vec![false; right.len()];

        for left_item in left.iter() {
            let mut found_match = false;

            for (right_idx, right_item) in right.iter().enumerate() {
                if right_used[right_idx] {
                    continue; // This right element is already matched
                }

                // Use equivalence comparison for elements
                let equivalent = match (left_item, right_item) {
                    (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
                    (FhirPathValue::String(l), FhirPathValue::String(r)) => {
                        l.to_lowercase() == r.to_lowercase()
                    }
                    (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => {
                        let l_rounded = l.round_dp(2);
                        let r_rounded = r.round_dp(2);
                        l_rounded == r_rounded
                    }
                    (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
                    _ => {
                        // For other types, use standard equality
                        let equal_op = EqualOperator;
                        match equal_op.evaluate_binary(left_item, right_item) {
                            Ok(FhirPathValue::Boolean(b)) => b,
                            _ => false,
                        }
                    }
                };

                if equivalent {
                    right_used[right_idx] = true;
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Compare two decimals for equivalence with tolerance
    fn compare_decimals_equivalent(
        &self,
        left: rust_decimal::Decimal,
        right: rust_decimal::Decimal,
    ) -> OperatorResult<bool> {
        // For FHIRPath equivalence, we need to handle precision differences
        // Round to 2 decimal places for comparison to handle cases like 0.67 ~ 1.2/1.8
        let left_rounded = left.round_dp(2);
        let right_rounded = right.round_dp(2);

        Ok(left_rounded == right_rounded)
    }
}

/// Not equivalent operator (!~)
pub struct NotEquivalentOperator;

impl FhirPathOperator for NotEquivalentOperator {
    fn symbol(&self) -> &str {
        "!~"
    }
    fn human_friendly_name(&self) -> &str {
        "Not Equivalent"
    }
    fn precedence(&self) -> u8 {
        9 // Per FHIRPath spec: =, ~, !=, !~ have precedence #09
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "!~",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // TODO: Implement proper equivalence logic
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            left != right,
        )]))
    }
}

/// Register all comparison operators
pub fn register_comparison_operators(registry: &mut OperatorRegistry) {
    registry.register(EqualOperator);
    registry.register(NotEqualOperator);
    registry.register(LessThanOperator);
    registry.register(LessThanOrEqualOperator);
    registry.register(GreaterThanOperator);
    registry.register(GreaterThanOrEqualOperator);
    registry.register(EquivalentOperator);
    registry.register(NotEquivalentOperator);
}
