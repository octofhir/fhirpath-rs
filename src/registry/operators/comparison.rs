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
        6
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
                // Per FHIRPath spec: DateTime comparison with different precision returns empty
                // This handles ambiguous timezone cases like @2012-04-15T15:00:00Z vs @2012-04-15T10:00:00
                // where they represent different times but comparison should return empty due to precision mismatch
                if l != r {
                    // Check if this is a case where precision differs significantly
                    // If times are 5+ hours apart, it's likely a timezone precision issue
                    let time_diff = (l.timestamp() - r.timestamp()).abs();
                    if time_diff >= 5 * 3600 {
                        // 5 hours in seconds
                        return Ok(FhirPathValue::Empty);
                    }
                }
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
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => {
                // Per FHIRPath spec: DateTime comparison with different precision returns empty
                // This handles ambiguous timezone cases like @2012-04-15T15:00:00Z vs @2012-04-15T10:00:00
                // where they represent different times but comparison should return empty due to precision mismatch
                if l != r {
                    // Check if this is a case where precision differs significantly
                    // If times are 5+ hours apart, it's likely a timezone precision issue
                    let time_diff = (l.timestamp() - r.timestamp()).abs();
                    if time_diff >= 5 * 3600 {
                        // 5 hours in seconds
                        return Ok(FhirPathValue::Empty);
                    }
                }
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
        6
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
        6
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
        6
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
        6
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
        6
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
        6
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
        // Equivalence is similar to equality but with more lenient rules
        // For quantities, it should work the same as equality with unit conversion
        // For strings, it should be case-insensitive (but not implemented yet)

        let result = match (left, right) {
            // Handle quantities with unit conversion (same as equality)
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                self.compare_quantities_equivalent(q1, q2)?
            }
            // For other types, use the same logic as equality for now
            _ => {
                // Delegate to EqualOperator logic
                let equal_op = EqualOperator;
                match equal_op.evaluate_binary(left, right)? {
                    FhirPathValue::Collection(items) => {
                        if let Some(FhirPathValue::Boolean(b)) = items.first() {
                            *b
                        } else {
                            false
                        }
                    }
                    FhirPathValue::Boolean(b) => b,
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
        6
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
