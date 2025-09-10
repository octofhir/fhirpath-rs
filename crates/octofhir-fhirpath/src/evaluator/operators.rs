//! Operator evaluation implementation for FHIRPath binary and unary operations
//!
//! This module implements the OperatorEvaluator trait which handles:
//! - Binary operations (arithmetic, comparison, logical)
//! - Unary operations (negation, not)
//! - Type casting operations (as operator)
//! - Type checking operations (is operator)
//! - FHIRPath-specific comparison semantics

use chrono::Timelike;
use octofhir_ucum::{analyse as ucum_analyse, divide_by as ucum_divide, multiply as ucum_multiply};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::cmp::Ordering;

use crate::{
    ast::{BinaryOperator, UnaryOperator},
    core::types::Collection,
    core::{FhirPathError, FhirPathValue, Result, error_code::*},
    evaluator::traits::OperatorEvaluator,
    registry::datetime_utils::{DateTimeDuration, DateTimeUtils},
};

/// Convert UCUM dot notation with negative powers to slash notation
/// e.g., "kg.m-1" -> "kg/m", "m.s-2" -> "m/s2", "kg.m2.s-3" -> "kg.m2/s3"
fn convert_dot_notation_to_slash(unit: &str) -> String {
    // Handle simple case: kg.m-1 -> kg/m
    if unit.contains('.') && unit.contains('-') {
        let parts: Vec<&str> = unit.split('.').collect();
        let mut numerator_parts = Vec::new();
        let mut denominator_parts = Vec::new();

        for part in parts {
            if let Some(neg_pos) = part.find('-') {
                // This part has a negative power - goes to denominator
                let base_unit = &part[..neg_pos];
                let power_str = &part[neg_pos + 1..];

                if power_str == "1" {
                    denominator_parts.push(base_unit.to_string());
                } else {
                    denominator_parts.push(format!("{}{}", base_unit, power_str));
                }
            } else {
                // This part has positive power - goes to numerator
                numerator_parts.push(part.to_string());
            }
        }

        if !denominator_parts.is_empty() {
            let numerator = numerator_parts.join(".");
            let denominator = denominator_parts.join(".");
            return format!("{}/{}", numerator, denominator);
        }
    }

    // No negative powers found, return as-is
    unit.to_string()
}

/// Implementation of OperatorEvaluator for FHIRPath operations
pub struct OperatorEvaluatorImpl;

impl OperatorEvaluatorImpl {
    /// Create a new standard operator evaluator instance
    pub fn new() -> Self {
        Self
    }

    /// Convert quantity to base units for comparison using UCUM
    fn normalize_quantity(&self, value: &Decimal, unit: &Option<String>) -> Result<Decimal> {
        match unit {
            Some(unit_str) => {
                // Handle dimensionless unit explicitly
                if unit_str == "1" {
                    return Ok(*value);
                }

                // Handle calendar units separately (they're not in UCUM)
                match unit_str.as_str() {
                    // Calendar units (variable length)
                    "day" | "days" => Ok(*value), // days are base unit
                    "week" | "weeks" => Ok(*value * Decimal::from(7)), // convert to days
                    "year" | "years" => Ok(*value * Decimal::from(365)), // approximate
                    "month" | "months" => Ok(*value * Decimal::from(30)), // approximate

                    // UCUM time units (fixed definitions, separate from calendar units)
                    "d" => Ok(*value), // UCUM day (24 hours exactly)
                    "wk" => Ok(*value * Decimal::from(7)), // UCUM week (7 days exactly)
                    "a" => Ok(*value * Decimal::from(365)), // UCUM year (365.25 days exactly)
                    "mo" => Ok(*value * Decimal::from(30)), // UCUM month (30 days exactly)
                    _ => {
                        // Convert dot notation to slash notation for UCUM compatibility
                        let normalized_unit = unit_str; // TODO: normalize format

                        // Try UCUM conversion
                        match ucum_analyse(&normalized_unit) {
                            Ok(analysis) => {
                                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                                    eprintln!(
                                        "🔍 UCUM NORMALIZE SUCCESS: {} -> factor: {} -> {} * {} = {}",
                                        unit_str,
                                        analysis.factor,
                                        value,
                                        analysis.factor,
                                        value.to_f64().unwrap_or(0.0) * analysis.factor
                                    );
                                }
                                let factor = Decimal::try_from(analysis.factor).map_err(|_| {
                                    FhirPathError::evaluation_error(
                                        FP0052,
                                        format!(
                                            "Cannot convert UCUM factor to decimal: {}",
                                            analysis.factor
                                        ),
                                    )
                                })?;
                                Ok(*value * factor)
                            }
                            Err(e) => {
                                if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                                    eprintln!(
                                        "🔍 UCUM NORMALIZE FALLBACK: {} failed analysis -> error: {:?}, returning original value: {}",
                                        unit_str, e, value
                                    );
                                }
                                // Unit not found in UCUM, return as-is
                                Ok(*value)
                            }
                        }
                    }
                }
            }
            None => Ok(*value), // Dimensionless quantity (will be treated as '1' in comparisons)
        }
    }

    /// Check if two quantities are comparable (same dimension) using UCUM
    fn quantities_comparable(&self, unit1: &Option<String>, unit2: &Option<String>) -> bool {
        if std::env::var("FHIRPATH_DEBUG").is_ok() {
            eprintln!("🔍 quantities_comparable({:?}, {:?})", unit1, unit2);
        }
        match (unit1, unit2) {
            (None, None) => true, // Both dimensionless
            // Handle dimensionless vs '1' unit (UCUM dimensionless symbol)
            (None, Some(u)) | (Some(u), None) if u == "1" => true,
            (Some(u1), Some(u2)) => {
                // Calendar units and UCUM units are NOT interchangeable
                let is_calendar_unit = |u: &str| {
                    matches!(
                        u,
                        "day" | "days" | "week" | "weeks" | "year" | "years" | "month" | "months"
                    )
                };
                let is_ucum_time_unit = |u: &str| matches!(u, "d" | "wk" | "a" | "mo");

                // Same type of units can be compared
                if (is_calendar_unit(u1) && is_calendar_unit(u2))
                    || (is_ucum_time_unit(u1) && is_ucum_time_unit(u2))
                {
                    return true;
                }

                // Handle specific calendar-UCUM time unit conversions that are equivalent
                if (is_calendar_unit(u1) && is_ucum_time_unit(u2))
                    || (is_ucum_time_unit(u1) && is_calendar_unit(u2))
                {
                    // Allow specific equivalent conversions
                    let calendar_ucum_equivalent = |cal: &str, ucum: &str| -> bool {
                        matches!(
                            (cal, ucum),
                            ("day" | "days", "d") |
                            ("week" | "weeks", "wk") |
                            ("year" | "years", "a") |
                            ("month" | "months", "mo") |
                            // Handle reverse mappings
                            ("d", "day" | "days") |
                            ("wk", "week" | "weeks") |
                            ("a", "year" | "years") |
                            ("mo", "month" | "months") |
                            // Special cross-equivalencies: days can be compared to week
                            ("days", "wk") | ("day", "wk") |
                            ("wk", "days") | ("wk", "day")
                        )
                    };
                    
                    if (is_calendar_unit(u1) && calendar_ucum_equivalent(u1, u2))
                        || (is_calendar_unit(u2) && calendar_ucum_equivalent(u2, u1))
                    {
                        return true; // Allow equivalent calendar-UCUM conversions
                    } else {
                        return false; // Still reject non-equivalent calendar-UCUM combinations
                    }
                }

                // Try UCUM dimension comparison with normalized unit formats
                let norm_u1 = u1; // TODO: normalize format
                let norm_u2 = u2; // TODO: normalize format

                match (ucum_analyse(&norm_u1), ucum_analyse(&norm_u2)) {
                    (Ok(a1), Ok(a2)) => {
                        // Compare dimensions - this handles kg.m-1 vs g/m equivalence
                        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                            eprintln!(
                                "🔍 UCUM SUCCESS: {} -> dimension: {} vs {} -> dimension: {}",
                                u1, a1.dimension, u2, a2.dimension
                            );
                        }
                        a1.dimension == a2.dimension
                    }
                    // If one unit can't be analyzed, try string equality as fallback
                    _ => {
                        if std::env::var("FHIRPATH_DEBUG_PERF").is_ok() {
                            eprintln!(
                                "🔍 UCUM FALLBACK: {} vs {} -> string equality: {}",
                                u1,
                                u2,
                                u1 == u2
                            );
                        }
                        u1 == u2
                    }
                }
            }
            _ => false, // One dimensionless, one with units (except '1')
        }
    }

    /// Check if two normalized quantities are approximately equal (for ~ operator)
    fn quantities_equivalent(&self, val1: Decimal, val2: Decimal) -> bool {
        // FHIRPath equivalence uses 1% tolerance for quantities
        let tolerance = Decimal::try_from(0.01).unwrap_or_default();
        let diff = if val1 > val2 {
            val1 - val2
        } else {
            val2 - val1
        };
        let max_val = if val1 > val2 { val1 } else { val2 };

        if max_val == Decimal::ZERO {
            diff <= tolerance // Absolute tolerance for values near zero
        } else {
            diff <= max_val * tolerance // Relative tolerance
        }
    }

    /// Check if two decimals are equivalent based on precision (per FHIRPath spec)
    fn decimals_equivalent(&self, val1: &Decimal, val2: &Decimal) -> bool {
        // For decimal equivalence in FHIRPath, compare values based on the precision of the least precise operand
        // This means 0.6666... ~ 0.67 should be true when compared at precision of 0.67 (2 decimal places)

        // Get the scale (number of decimal places) of each value
        let scale1 = val1.scale();
        let scale2 = val2.scale();

        // Use the minimum scale (least precise) for comparison
        let min_scale = scale1.min(scale2);

        // Round both values to the minimum scale and compare
        let rounded1 = val1.round_dp(min_scale);
        let rounded2 = val2.round_dp(min_scale);

        rounded1 == rounded2
    }

    /// Perform equality comparison with FHIRPath semantics
    fn equals(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Per FHIRPath specification: empty collections are not comparable
        let left_empty = matches!(left, FhirPathValue::Empty)
            || matches!(left, FhirPathValue::Collection(c) if c.is_empty());
        let right_empty = matches!(right, FhirPathValue::Empty)
            || matches!(right, FhirPathValue::Collection(c) if c.is_empty());

        if left_empty || right_empty {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            // Boolean comparisons
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a == b,

            // Numeric comparisons with type coercion
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a == b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a == b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => &Decimal::from(*a) == b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a == &Decimal::from(*b),

            // String comparisons
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a == b,
            (FhirPathValue::Uri(a), FhirPathValue::Uri(b)) => a == b,
            (FhirPathValue::Url(a), FhirPathValue::Url(b)) => a == b,

            // Cross-string type comparisons
            (FhirPathValue::String(a), FhirPathValue::Uri(b)) => a == b,
            (FhirPathValue::Uri(a), FhirPathValue::String(b)) => a == b,
            (FhirPathValue::String(a), FhirPathValue::Url(b)) => a == b,
            (FhirPathValue::Url(a), FhirPathValue::String(b)) => a == b,
            (FhirPathValue::Uri(a), FhirPathValue::Url(b)) => a == b,
            (FhirPathValue::Url(a), FhirPathValue::Uri(b)) => a == b,

            // Temporal comparisons - use partial_cmp to handle precision normalization
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => {
                match a.partial_cmp(b) {
                    Some(std::cmp::Ordering::Equal) => true,
                    Some(_) => false,
                    None => return Ok(FhirPathValue::Empty), // Incomparable precisions
                }
            }
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                match a.partial_cmp(b) {
                    Some(std::cmp::Ordering::Equal) => true,
                    Some(_) => false,
                    None => return Ok(FhirPathValue::Empty), // Incomparable precisions
                }
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => {
                match a.partial_cmp(b) {
                    Some(std::cmp::Ordering::Equal) => true,
                    Some(_) => false,
                    None => return Ok(FhirPathValue::Empty), // Incomparable precisions
                }
            }

            // Cross-temporal type comparisons are not comparable
            (FhirPathValue::Date(_), FhirPathValue::DateTime(_)) => {
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Date(_)) => {
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::Date(_), FhirPathValue::Time(_)) => {
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::Time(_), FhirPathValue::Date(_)) => {
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Time(_)) => {
                return Ok(FhirPathValue::Empty);
            }
            (FhirPathValue::Time(_), FhirPathValue::DateTime(_)) => {
                return Ok(FhirPathValue::Empty);
            }

            // Quantity comparisons with unit conversion using UCUM
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                if self.quantities_comparable(u1, u2) {
                    // Normalize both quantities and compare
                    match (
                        self.normalize_quantity(v1, u1),
                        self.normalize_quantity(v2, u2),
                    ) {
                        (Ok(norm1), Ok(norm2)) => norm1 == norm2,
                        _ => false, // Normalization failed
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Incomparable quantities return empty
                }
            }

            // ID comparisons
            (FhirPathValue::Id(a), FhirPathValue::Id(b)) => a == b,

            // Resource and JsonValue comparisons - check JSON equality
            (FhirPathValue::Resource(a), FhirPathValue::Resource(b)) => **a == **b,
            (FhirPathValue::JsonValue(a), FhirPathValue::JsonValue(b)) => **a == **b,
            (FhirPathValue::Resource(a), FhirPathValue::JsonValue(b)) => **a == **b,
            (FhirPathValue::JsonValue(a), FhirPathValue::Resource(b)) => **a == **b,

            // Collection comparisons (element-wise)
            (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) => {
                if a.len() != b.len() {
                    false
                } else {
                    a.iter()
                        .zip(b.iter())
                        .all(|(x, y)| match self.equals(x, y) {
                            Ok(FhirPathValue::Boolean(result)) => result,
                            _ => false,
                        })
                }
            }

            // Single value vs collection comparison
            (single, FhirPathValue::Collection(coll))
            | (FhirPathValue::Collection(coll), single) => {
                if coll.len() == 1 {
                    match self.equals(single, &coll[0]) {
                        Ok(FhirPathValue::Boolean(result)) => result,
                        _ => false,
                    }
                } else {
                    false
                }
            }

            // Different types are not equal
            _ => false,
        };

        Ok(FhirPathValue::Boolean(result))
    }

    /// Perform inequality comparison
    fn not_equals(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match self.equals(left, right)? {
            FhirPathValue::Boolean(result) => Ok(FhirPathValue::Boolean(!result)),
            other => Ok(other), // If equals returns empty, not_equals should also return empty
        }
    }

    /// Perform less than comparison
    fn less_than(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        let result = match (left, right) {
            // Numeric comparisons
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a < b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(*a) < *b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => *a < Decimal::from(*b),

            // String comparisons
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a < b,
            (FhirPathValue::Uri(a), FhirPathValue::Uri(b)) => a < b,
            (FhirPathValue::Url(a), FhirPathValue::Url(b)) => a < b,

            // Temporal comparisons
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => match a.partial_cmp(b) {
                Some(Ordering::Less) => true,
                Some(_) => false,
                None => return Ok(FhirPathValue::Empty), // Different precisions or overlap
            },
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => match a.partial_cmp(b) {
                Some(Ordering::Less) => true,
                Some(_) => false,
                None => return Ok(FhirPathValue::Empty), // Different precisions or overlap
            },
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => match a.partial_cmp(b) {
                Some(Ordering::Less) => true,
                Some(_) => false,
                None => return Ok(FhirPathValue::Empty), // Different precisions or overlap
            },
            // String to Date comparison (implicit conversion per FHIRPath specification)
            (FhirPathValue::String(s), FhirPathValue::Date(date)) => {
                if let Ok(parsed_date) = crate::registry::FunctionRegistry::parse_date_string(s) {
                    match parsed_date.partial_cmp(date) {
                        Some(Ordering::Less) => true,
                        _ => false,
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Cannot parse string as date
                }
            }
            (FhirPathValue::Date(date), FhirPathValue::String(s)) => {
                if let Ok(parsed_date) = crate::registry::FunctionRegistry::parse_date_string(s) {
                    match date.partial_cmp(&parsed_date) {
                        Some(Ordering::Less) => true,
                        _ => false,
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Cannot parse string as date
                }
            }
            // String to DateTime comparison (implicit conversion per FHIRPath specification)
            (FhirPathValue::String(s), FhirPathValue::DateTime(datetime)) => {
                if let Ok(parsed_datetime) =
                    crate::registry::FunctionRegistry::parse_datetime_string(s)
                {
                    match parsed_datetime.partial_cmp(datetime) {
                        Some(Ordering::Less) => true,
                        _ => false,
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Cannot parse string as datetime
                }
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::String(s)) => {
                if let Ok(parsed_datetime) =
                    crate::registry::FunctionRegistry::parse_datetime_string(s)
                {
                    match datetime.partial_cmp(&parsed_datetime) {
                        Some(Ordering::Less) => true,
                        _ => false,
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Cannot parse string as datetime
                }
            }
            // Date vs DateTime comparison (convert Date to DateTime at start of day)
            (FhirPathValue::Date(date), FhirPathValue::DateTime(datetime)) => {
                // Convert Date to DateTime at start of day for comparison
                use chrono::{NaiveTime, TimeZone};
                let date_as_datetime = datetime
                    .datetime
                    .timezone()
                    .from_local_datetime(
                        &date
                            .date
                            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                    )
                    .single();
                if let Some(converted_datetime) = date_as_datetime {
                    let precision_datetime = crate::core::temporal::PrecisionDateTime::new(
                        converted_datetime,
                        date.precision,
                    );
                    match precision_datetime.partial_cmp(datetime) {
                        Some(Ordering::Less) => true,
                        Some(_) => false,
                        None => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Date(date)) => {
                // Convert Date to DateTime at start of day for comparison
                use chrono::{NaiveTime, TimeZone};
                let date_as_datetime = datetime
                    .datetime
                    .timezone()
                    .from_local_datetime(
                        &date
                            .date
                            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                    )
                    .single();
                if let Some(converted_datetime) = date_as_datetime {
                    let precision_datetime = crate::core::temporal::PrecisionDateTime::new(
                        converted_datetime,
                        date.precision,
                    );
                    match datetime.partial_cmp(&precision_datetime) {
                        Some(Ordering::Less) => true,
                        Some(_) => false,
                        None => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }

            // Quantity comparisons with unit conversion using UCUM
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                if self.quantities_comparable(u1, u2) {
                    // Normalize both quantities and compare
                    match (
                        self.normalize_quantity(v1, u1),
                        self.normalize_quantity(v2, u2),
                    ) {
                        (Ok(norm1), Ok(norm2)) => match norm1.partial_cmp(&norm2) {
                            Some(Ordering::Less) => true,
                            _ => false,
                        },
                        _ => return Ok(FhirPathValue::Empty), // Normalization failed
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Different dimensions are incomparable
                }
            }

            // Empty values are not comparable
            _ if matches!(left, FhirPathValue::Empty) || matches!(right, FhirPathValue::Empty) => {
                return Ok(FhirPathValue::Empty);
            }

            // Other types are not comparable
            _ => return Ok(FhirPathValue::Empty),
        };

        Ok(FhirPathValue::Boolean(result))
    }

    /// Perform greater than comparison
    fn greater_than(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // A > B is equivalent to B < A
        self.less_than(right, left)
    }

    /// Perform less than or equal comparison
    fn less_than_or_equal(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<FhirPathValue> {
        match (self.less_than(left, right)?, self.equals(left, right)?) {
            (FhirPathValue::Boolean(lt), FhirPathValue::Boolean(eq)) => {
                Ok(FhirPathValue::Boolean(lt || eq))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Perform greater than or equal comparison
    fn greater_than_or_equal(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<FhirPathValue> {
        match (self.greater_than(left, right)?, self.equals(left, right)?) {
            (FhirPathValue::Boolean(gt), FhirPathValue::Boolean(eq)) => {
                Ok(FhirPathValue::Boolean(gt || eq))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Perform arithmetic addition
    fn add(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            // Empty collection handling - operations with empty collections return empty
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),

            // Numeric addition
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a + b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a + b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(Decimal::from(*a) + b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Decimal(a + Decimal::from(*b)))
            }

            // String concatenation - empty collections return empty
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{}{}", a, b)))
            }

            // Date/DateTime/Time + Quantity (temporal arithmetic)
            (
                date_time @ (FhirPathValue::Date(_) | FhirPathValue::DateTime(_)),
                FhirPathValue::Quantity { value, unit, .. },
            ) => self.add_temporal_quantity(date_time, value, unit),
            (FhirPathValue::Time(time), FhirPathValue::Quantity { value, unit, .. }) => {
                self.add_time_quantity(time, value, unit)
            }

            // Quantity addition (only if compatible units)
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                if left.is_quantity_compatible(right) {
                    Ok(FhirPathValue::quantity(v1 + v2, u1.clone()))
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0052,
                        format!(
                            "Cannot add quantities with incompatible units: {:?} and {:?}",
                            u1, u2
                        ),
                    ))
                }
            }

            // Collection handling - single item + collection or collection + item should return empty
            (_, FhirPathValue::Collection(_)) | (FhirPathValue::Collection(_), _) => {
                Ok(FhirPathValue::Empty)
            }

            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!("Cannot add {} and {}", left.type_name(), right.type_name()),
            )),
        }
    }

    /// Perform arithmetic subtraction
    fn subtract(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            // Empty collection handling - operations with empty collections return empty
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),

            // Numeric subtraction
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a - b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a - b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(Decimal::from(*a) - b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Decimal(a - Decimal::from(*b)))
            }

            // Date/DateTime - Quantity (temporal arithmetic)
            (
                date_time @ (FhirPathValue::Date(_) | FhirPathValue::DateTime(_)),
                FhirPathValue::Quantity { value, unit, .. },
            ) => self.subtract_temporal_quantity(date_time, value, unit),
            (FhirPathValue::Time(time), FhirPathValue::Quantity { value, unit, .. }) => {
                self.subtract_time_quantity(time, value, unit)
            }

            // Quantity subtraction (only if compatible units)
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                if left.is_quantity_compatible(right) {
                    Ok(FhirPathValue::quantity(v1 - v2, u1.clone()))
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0052,
                        format!(
                            "Cannot subtract quantities with incompatible units: {:?} and {:?}",
                            u1, u2
                        ),
                    ))
                }
            }

            // Collection handling - single item - collection or collection - item should return empty
            (_, FhirPathValue::Collection(_)) | (FhirPathValue::Collection(_), _) => {
                Ok(FhirPathValue::Empty)
            }

            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!(
                    "Cannot subtract {} from {}",
                    right.type_name(),
                    left.type_name()
                ),
            )),
        }
    }

    /// Perform arithmetic multiplication
    fn multiply(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath specification: if either operand is empty, the result is empty
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match (left, right) {
            // Numeric multiplication
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a * b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a * b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(Decimal::from(*a) * b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Decimal(a * Decimal::from(*b)))
            }

            // Quantity multiplication using UCUM library
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                match (u1, u2) {
                    (Some(unit1), Some(unit2)) => {
                        // Use UCUM library for proper unit multiplication
                        match ucum_multiply(
                            v1.to_f64().unwrap_or(0.0),
                            unit1,
                            v2.to_f64().unwrap_or(0.0),
                            unit2,
                        ) {
                            Ok(result) => {
                                let result_value =
                                    Decimal::try_from(result.value).unwrap_or(Decimal::ZERO);
                                Ok(FhirPathValue::quantity(result_value, Some(result.unit)))
                            }
                            Err(_) => {
                                // Fallback to simple concatenation if UCUM fails
                                let result_unit = Some(format!("{}.{}", unit1, unit2));
                                Ok(FhirPathValue::quantity(v1 * v2, result_unit))
                            }
                        }
                    }
                    (Some(unit), None) => {
                        // Unit1 * unitless = unit1
                        Ok(FhirPathValue::quantity(v1 * v2, Some(unit.clone())))
                    }
                    (None, Some(unit)) => {
                        // Unitless * unit2 = unit2
                        Ok(FhirPathValue::quantity(v1 * v2, Some(unit.clone())))
                    }
                    (None, None) => {
                        // Unitless * unitless = unitless
                        Ok(FhirPathValue::quantity(v1 * v2, None))
                    }
                }
            }

            // Scalar multiplication of quantities
            (FhirPathValue::Quantity { value, unit, .. }, FhirPathValue::Integer(scalar))
            | (FhirPathValue::Integer(scalar), FhirPathValue::Quantity { value, unit, .. }) => Ok(
                FhirPathValue::quantity(value * Decimal::from(*scalar), unit.clone()),
            ),
            (FhirPathValue::Quantity { value, unit, .. }, FhirPathValue::Decimal(scalar))
            | (FhirPathValue::Decimal(scalar), FhirPathValue::Quantity { value, unit, .. }) => {
                Ok(FhirPathValue::quantity(value * scalar, unit.clone()))
            }

            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!(
                    "Cannot multiply {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            )),
        }
    }

    /// Perform arithmetic division
    fn divide(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Empty collection handling - operations with empty collections return empty
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match (left, right) {
            // Numeric division - division by zero returns empty
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Decimal(
                    Decimal::from(*a) / Decimal::from(*b),
                ))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Decimal(a / b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Decimal(Decimal::from(*a) / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Decimal(a / Decimal::from(*b)))
            }

            // Quantity division using UCUM library
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                if v2.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }

                match (u1, u2) {
                    (Some(unit1), Some(unit2)) => {
                        // Calculate numeric result and construct unit string directly
                        let result_value = v1 / v2;

                        // Construct result unit: unit1 / unit2
                        let result_unit = if unit1 == unit2 {
                            None // Same units cancel out
                        } else {
                            Some(format!("{}/{}", unit1, unit2))
                        };

                        // Use UCUM only for validation that the operation makes sense
                        match ucum_divide(
                            v1.to_f64().unwrap_or(0.0),
                            unit1,
                            v2.to_f64().unwrap_or(0.0),
                            unit2,
                        ) {
                            Ok(_) => {
                                // UCUM validation passed, use our result
                                Ok(FhirPathValue::quantity(result_value, result_unit))
                            }
                            Err(_) => {
                                // UCUM validation failed, but still return basic result
                                Ok(FhirPathValue::quantity(result_value, result_unit))
                            }
                        }
                    }
                    (Some(unit), None) => {
                        // unit1 / unitless = unit1
                        Ok(FhirPathValue::quantity(v1 / v2, Some(unit.clone())))
                    }
                    (None, Some(unit)) => {
                        // unitless / unit2 = 1/unit2 (reciprocal unit)
                        let result_unit = Some(format!("/{}", unit));
                        Ok(FhirPathValue::quantity(v1 / v2, result_unit))
                    }
                    (None, None) => {
                        // unitless / unitless = unitless
                        Ok(FhirPathValue::quantity(v1 / v2, None))
                    }
                }
            }

            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!(
                    "Cannot divide {} by {}",
                    left.type_name(),
                    right.type_name()
                ),
            )),
        }
    }

    /// Perform modulo operation
    fn modulo(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath specification: if either operand is empty, the result is empty
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Integer(a % b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Decimal(a % b))
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!(
                    "Cannot perform modulo on {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            )),
        }
    }

    /// Perform logical AND with FHIRPath three-valued logic
    fn and(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath AND truth table:
        // true  AND true  → true
        // true  AND false → false
        // false AND true  → false
        // false AND false → false
        // true  AND {}    → {}
        // false AND {}    → false (short-circuit)
        // {}    AND true  → {}
        // {}    AND false → false (short-circuit)
        // {}    AND {}    → {}
        // Collections: empty collection = {}, non-empty collection = true

        // Convert operands to boolean context
        let left_bool = self.to_boolean_context(left);
        let right_bool = self.to_boolean_context(right);

        match (left_bool, right_bool) {
            // If either operand is false, result is false (short-circuit)
            (Some(false), _) => Ok(FhirPathValue::Boolean(false)),
            (_, Some(false)) => Ok(FhirPathValue::Boolean(false)),

            // Both are true
            (Some(true), Some(true)) => Ok(FhirPathValue::Boolean(true)),

            // One is true, other is empty → empty
            (Some(true), None) => Ok(FhirPathValue::Empty),
            (None, Some(true)) => Ok(FhirPathValue::Empty),

            // Both are empty → empty
            (None, None) => Ok(FhirPathValue::Empty),
        }
    }

    /// Convert a FhirPathValue to boolean context for logical operations
    /// Returns Some(bool) for boolean values, None for empty/collection-empty
    fn to_boolean_context(&self, value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Empty => None,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    None // Empty collection treated as empty
                } else {
                    Some(true) // Non-empty collection treated as true
                }
            }
            _ => Some(true), // Other non-empty values treated as true
        }
    }

    /// Perform logical OR with FHIRPath three-valued logic
    fn or(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath OR truth table:
        // true  OR true  → true
        // true  OR false → true
        // false OR true  → true
        // false OR false → false
        // true  OR {}    → true (short-circuit)
        // false OR {}    → {}
        // {}    OR true  → true (short-circuit)
        // {}    OR false → {}
        // {}    OR {}    → {}
        // Collections: empty collection = {}, non-empty collection = true

        // Convert operands to boolean context
        let left_bool = self.to_boolean_context(left);
        let right_bool = self.to_boolean_context(right);

        match (left_bool, right_bool) {
            // If either operand is true, result is true (short-circuit)
            (Some(true), _) => Ok(FhirPathValue::Boolean(true)),
            (_, Some(true)) => Ok(FhirPathValue::Boolean(true)),

            // Both are false
            (Some(false), Some(false)) => Ok(FhirPathValue::Boolean(false)),

            // One is false, other is empty → empty
            (Some(false), None) => Ok(FhirPathValue::Empty),
            (None, Some(false)) => Ok(FhirPathValue::Empty),

            // Both are empty → empty
            (None, None) => Ok(FhirPathValue::Empty),
        }
    }

    /// Perform logical XOR
    fn xor(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Convert operands to boolean context
        let left_bool = self.to_boolean_context(left);
        let right_bool = self.to_boolean_context(right);

        match (left_bool, right_bool) {
            // If either operand is empty, result is empty
            (None, _) | (_, None) => Ok(FhirPathValue::Empty),

            // Both are boolean - perform XOR
            (Some(a), Some(b)) => Ok(FhirPathValue::Boolean(a ^ b)),
        }
    }

    /// Add a quantity (duration) to a Date or DateTime
    fn add_temporal_quantity(
        &self,
        temporal: &FhirPathValue,
        value: &Decimal,
        unit: &Option<String>,
    ) -> Result<FhirPathValue> {
        let unit_str = unit.as_deref().unwrap_or("");
        let duration = DateTimeDuration::from_quantity(value, unit_str)?;

        match temporal {
            FhirPathValue::Date(_date) => {
                let datetime_utc = DateTimeUtils::to_datetime(temporal)?;
                let result_datetime = duration.add_to_datetime(datetime_utc)?;

                // Convert back to date (keep only date part)
                let result_date = result_datetime.date_naive();
                let precision_date = crate::core::temporal::PrecisionDate::from_date(result_date);
                Ok(FhirPathValue::Date(precision_date))
            }
            FhirPathValue::DateTime(datetime) => {
                let datetime_utc = DateTimeUtils::to_datetime(temporal)?;
                let result_datetime = duration.add_to_datetime(datetime_utc)?;

                // Preserve original timezone offset
                let result_with_tz = result_datetime.with_timezone(&datetime.datetime.timezone());
                let precision_datetime = crate::core::temporal::PrecisionDateTime::new(
                    result_with_tz,
                    datetime.precision,
                );
                Ok(FhirPathValue::DateTime(precision_datetime))
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!("Cannot add quantity to {}", temporal.type_name()),
            )),
        }
    }

    /// Add a quantity (duration) to a Time
    fn add_time_quantity(
        &self,
        time: &crate::core::temporal::PrecisionTime,
        value: &Decimal,
        unit: &Option<String>,
    ) -> Result<FhirPathValue> {
        let unit_str = unit.as_deref().unwrap_or("");
        let duration = DateTimeDuration::from_quantity(value, unit_str)?;

        // For time arithmetic, we only support hour/minute/second/millisecond units
        if duration.years != 0 || duration.months != 0 || duration.days != 0 {
            return Err(FhirPathError::evaluation_error(
                FP0052,
                "Cannot add days/months/years to time - only hours/minutes/seconds/milliseconds supported",
            ));
        }

        let total_seconds = duration.hours * 3600 + duration.minutes * 60 + duration.seconds;
        let total_milliseconds = total_seconds * 1000 + duration.milliseconds;

        // Convert time to total milliseconds since midnight
        let current_ms = (time.time.hour() as i64 * 3600
            + time.time.minute() as i64 * 60
            + time.time.second() as i64)
            * 1000
            + time.time.nanosecond() as i64 / 1_000_000;

        let result_ms = (current_ms + total_milliseconds) % (24 * 60 * 60 * 1000); // Wrap around 24 hours
        let result_ms = if result_ms < 0 {
            result_ms + 24 * 60 * 60 * 1000
        } else {
            result_ms
        };

        let hours = (result_ms / (60 * 60 * 1000)) as u32;
        let minutes = ((result_ms % (60 * 60 * 1000)) / (60 * 1000)) as u32;
        let seconds = ((result_ms % (60 * 1000)) / 1000) as u32;
        let milliseconds = (result_ms % 1000) as u32;

        let result_time =
            chrono::NaiveTime::from_hms_milli_opt(hours, minutes, seconds, milliseconds)
                .ok_or_else(|| {
                    FhirPathError::evaluation_error(FP0052, "Invalid time after arithmetic")
                })?;

        let precision_time = crate::core::temporal::PrecisionTime::from_time_with_precision(
            result_time,
            time.precision,
        );
        Ok(FhirPathValue::Time(precision_time))
    }

    /// Subtract a quantity (duration) from a Date or DateTime
    fn subtract_temporal_quantity(
        &self,
        temporal: &FhirPathValue,
        value: &Decimal,
        unit: &Option<String>,
    ) -> Result<FhirPathValue> {
        let unit_str = unit.as_deref().unwrap_or("");
        let duration = DateTimeDuration::from_quantity(value, unit_str)?;

        match temporal {
            FhirPathValue::Date(_date) => {
                let datetime_utc = DateTimeUtils::to_datetime(temporal)?;
                let result_datetime = duration.subtract_from_datetime(datetime_utc)?;

                // Convert back to date (keep only date part)
                let result_date = result_datetime.date_naive();
                let precision_date = crate::core::temporal::PrecisionDate::from_date(result_date);
                Ok(FhirPathValue::Date(precision_date))
            }
            FhirPathValue::DateTime(datetime) => {
                let datetime_utc = DateTimeUtils::to_datetime(temporal)?;
                let result_datetime = duration.subtract_from_datetime(datetime_utc)?;

                // Preserve original timezone offset
                let result_with_tz = result_datetime.with_timezone(&datetime.datetime.timezone());
                let precision_datetime = crate::core::temporal::PrecisionDateTime::new(
                    result_with_tz,
                    datetime.precision,
                );
                Ok(FhirPathValue::DateTime(precision_datetime))
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!("Cannot subtract quantity from {}", temporal.type_name()),
            )),
        }
    }

    /// Subtract a quantity (duration) from a Time
    fn subtract_time_quantity(
        &self,
        time: &crate::core::temporal::PrecisionTime,
        value: &Decimal,
        unit: &Option<String>,
    ) -> Result<FhirPathValue> {
        // Subtraction is just addition with negative value
        let negative_value = -value;
        self.add_time_quantity(time, &negative_value, unit)
    }

    /// Perform set membership test (in operator)
    fn contains(&self, collection: &FhirPathValue, item: &FhirPathValue) -> Result<FhirPathValue> {
        // Validate FHIRPath 'in' operator constraints:
        // Left operand (item) must be a single value (not a collection)
        // Right operand (collection) should typically be a collection or single value
        if let FhirPathValue::Collection(item_collection) = item {
            if !item_collection.is_empty() {
                return Err(FhirPathError::evaluation_error(
                    crate::core::FP0052,
                    "Left operand of 'in' operator must be a single value, not a collection"
                        .to_string(),
                ));
            }
        }

        // Per FHIRPath specification: if BOTH operands are empty, the result is empty
        // If only the item is empty, the result is empty
        // If only the collection is empty, the result is false
        let collection_is_empty = matches!(collection, FhirPathValue::Empty)
            || matches!(collection, FhirPathValue::Collection(c) if c.is_empty());
        let item_is_empty = matches!(item, FhirPathValue::Empty)
            || matches!(item, FhirPathValue::Collection(c) if c.is_empty());

        if item_is_empty {
            return Ok(FhirPathValue::Empty);
        }

        if collection_is_empty {
            return Ok(FhirPathValue::Boolean(false));
        }

        match collection {
            FhirPathValue::Collection(items) => {
                for collection_item in items.iter() {
                    match self.equals(collection_item, item)? {
                        FhirPathValue::Boolean(true) => return Ok(FhirPathValue::Boolean(true)),
                        _ => continue,
                    }
                }
                Ok(FhirPathValue::Boolean(false))
            }
            single_value => {
                // For single values, check if they're equal
                self.equals(single_value, item)
            }
        }
    }

    /// Perform integer division (div operator)
    fn integer_divide(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Empty collection handling - operations with empty collections return empty
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Integer(a / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Integer(
                    (a / b).trunc().to_i64().unwrap_or(0),
                ))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Integer(
                    (Decimal::from(*a) / b).trunc().to_i64().unwrap_or(0),
                ))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                Ok(FhirPathValue::Integer(
                    (a / Decimal::from(*b)).trunc().to_i64().unwrap_or(0),
                ))
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0052,
                format!(
                    "Cannot perform integer division on {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            )),
        }
    }

    fn equivalent(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Equivalent (~) is similar to equals (=) but with tolerance for quantities and different null/empty handling
        match (left, right) {
            // Both empty are equivalent
            (FhirPathValue::Empty, FhirPathValue::Empty) => Ok(FhirPathValue::Boolean(true)),
            // If either is empty but not both, they're not equivalent
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                Ok(FhirPathValue::Boolean(false))
            }

            // String equivalence is case-insensitive (per FHIRPath spec)
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::Uri(a), FhirPathValue::Uri(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::Url(a), FhirPathValue::Url(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }

            // Cross-string type equivalence is also case-insensitive
            (FhirPathValue::String(a), FhirPathValue::Uri(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::Uri(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::String(a), FhirPathValue::Url(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::Url(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::Uri(a), FhirPathValue::Url(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }
            (FhirPathValue::Url(a), FhirPathValue::Uri(b)) => {
                Ok(FhirPathValue::Boolean(a.to_lowercase() == b.to_lowercase()))
            }

            // Special handling for quantities with tolerance (per FHIRPath spec)
            (
                FhirPathValue::Quantity {
                    value: v1,
                    unit: u1,
                    ..
                },
                FhirPathValue::Quantity {
                    value: v2,
                    unit: u2,
                    ..
                },
            ) => {
                if self.quantities_comparable(u1, u2) {
                    // Normalize both quantities and compare with 1% tolerance
                    match (
                        self.normalize_quantity(v1, u1),
                        self.normalize_quantity(v2, u2),
                    ) {
                        (Ok(norm1), Ok(norm2)) => {
                            return Ok(FhirPathValue::Boolean(
                                self.quantities_equivalent(norm1, norm2),
                            ));
                        }
                        _ => return Ok(FhirPathValue::Boolean(false)), // Normalization failed
                    }
                } else {
                    return Ok(FhirPathValue::Boolean(false)); // Different dimensions are not equivalent
                }
            }

            // Collection equivalence (element-wise, order-independent per FHIRPath spec)
            (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) => {
                if a.len() != b.len() {
                    Ok(FhirPathValue::Boolean(false))
                } else {
                    // For equivalence, collections are equivalent if they contain the same elements (order independent)
                    let mut all_equivalent = true;
                    'outer: for item_a in a.iter() {
                        let mut found_equivalent = false;
                        for item_b in b.iter() {
                            match self.equivalent(item_a, item_b)? {
                                FhirPathValue::Boolean(true) => {
                                    found_equivalent = true;
                                    break;
                                }
                                _ => continue,
                            }
                        }
                        if !found_equivalent {
                            all_equivalent = false;
                            break 'outer;
                        }
                    }
                    Ok(FhirPathValue::Boolean(all_equivalent))
                }
            }

            // Single value vs collection equivalence
            (single, FhirPathValue::Collection(coll))
            | (FhirPathValue::Collection(coll), single) => {
                if coll.len() == 1 {
                    self.equivalent(single, &coll[0])
                } else {
                    Ok(FhirPathValue::Boolean(false))
                }
            }

            // Resource and JsonValue comparisons - check JSON equality
            (FhirPathValue::Resource(a), FhirPathValue::Resource(b)) => {
                Ok(FhirPathValue::Boolean(**a == **b))
            }
            (FhirPathValue::JsonValue(a), FhirPathValue::JsonValue(b)) => {
                Ok(FhirPathValue::Boolean(**a == **b))
            }
            (FhirPathValue::Resource(a), FhirPathValue::JsonValue(b)) => {
                Ok(FhirPathValue::Boolean(**a == **b))
            }
            (FhirPathValue::JsonValue(a), FhirPathValue::Resource(b)) => {
                Ok(FhirPathValue::Boolean(**a == **b))
            }

            // Decimal equivalence based on precision (per FHIRPath spec)
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Boolean(self.decimals_equivalent(a, b)))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Ok(FhirPathValue::Boolean(
                self.decimals_equivalent(&Decimal::from(*a), b),
            )),
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => Ok(FhirPathValue::Boolean(
                self.decimals_equivalent(a, &Decimal::from(*b)),
            )),

            // DateTime equivalence - for equivalence operator (~), different temporal types should return false
            // rather than empty (unlike equals operator which returns empty for incomparable values)
            (FhirPathValue::Date(_), FhirPathValue::DateTime(_)) => {
                Ok(FhirPathValue::Boolean(false))
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Date(_)) => {
                Ok(FhirPathValue::Boolean(false))
            }
            (FhirPathValue::Time(_), FhirPathValue::DateTime(_)) => {
                Ok(FhirPathValue::Boolean(false))
            }
            (FhirPathValue::DateTime(_), FhirPathValue::Time(_)) => {
                Ok(FhirPathValue::Boolean(false))
            }
            (FhirPathValue::Date(_), FhirPathValue::Time(_)) => {
                Ok(FhirPathValue::Boolean(false))
            }
            (FhirPathValue::Time(_), FhirPathValue::Date(_)) => {
                Ok(FhirPathValue::Boolean(false))
            }
            
            // For datetime-datetime comparisons with timezone specification differences,
            // equivalence should return false instead of empty
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => {
                if a.tz_specified != b.tz_specified {
                    Ok(FhirPathValue::Boolean(false))
                } else {
                    // Same timezone specification - delegate to equals logic
                    self.equals(left, right)
                }
            }

            // Otherwise delegate to equals logic for other types
            _ => self.equals(left, right),
        }
    }

    fn not_equivalent(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match self.equivalent(left, right)? {
            FhirPathValue::Boolean(result) => Ok(FhirPathValue::Boolean(!result)),
            _ => Ok(FhirPathValue::Boolean(true)), // If equivalent fails, assume not equivalent
        }
    }

    fn implies(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath implies: false implies anything = true, true implies false = false, true implies true = true
        // Collections: empty collection = {}, non-empty collection = true
        // IMPLIES is equivalent to (not A) or B

        // Convert operands to boolean context
        let left_bool = self.to_boolean_context(left);
        let right_bool = self.to_boolean_context(right);

        match (left_bool, right_bool) {
            // Special cases for IMPLIES with empty (based on (not A) or B logic)
            (Some(false), None) => Ok(FhirPathValue::Boolean(true)), // false implies {} = true
            (None, Some(true)) => Ok(FhirPathValue::Boolean(true)),  // {} implies true = true
            (None, Some(false)) => Ok(FhirPathValue::Empty),         // {} implies false = {}
            (Some(true), None) => Ok(FhirPathValue::Empty),          // true implies {} = {}
            (None, None) => Ok(FhirPathValue::Empty),                // {} implies {} = {}

            // Both are boolean - A implies B is equivalent to (not A) or B
            (Some(a), Some(b)) => Ok(FhirPathValue::Boolean(!a || b)),
        }
    }

    fn concatenate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath concatenate (&) - concatenates strings
        // Non-empty collections cannot be concatenated and should produce an error
        // Empty collections are treated as empty strings
        match (left, right) {
            // Non-empty collections cannot be concatenated
            (FhirPathValue::Collection(items), _) if !items.is_empty() => {
                Err(FhirPathError::evaluation_error(
                    FP0052,
                    "Cannot concatenate non-empty collections with other values".to_string(),
                ))
            }
            (_, FhirPathValue::Collection(items)) if !items.is_empty() => {
                Err(FhirPathError::evaluation_error(
                    FP0052,
                    "Cannot concatenate non-empty collections with other values".to_string(),
                ))
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{}{}", a, b)))
            }
            // Convert other types to string then concatenate (including empty collections and Empty)
            _ => {
                let left_str = left.to_string().unwrap_or_else(|_| "".to_string());
                let right_str = right.to_string().unwrap_or_else(|_| "".to_string());
                Ok(FhirPathValue::String(format!("{}{}", left_str, right_str)))
            }
        }
    }

    fn union(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // FHIRPath union (|) - combines two collections removing duplicates
        let mut result_items = Vec::new();

        // Add items from left
        match left {
            FhirPathValue::Empty => {}
            FhirPathValue::Collection(items) => {
                result_items.extend(items.iter().cloned());
            }
            single => {
                result_items.push(single.clone());
            }
        }

        // Add items from right (avoiding duplicates)
        match right {
            FhirPathValue::Empty => {}
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    let mut found = false;
                    for existing in &result_items {
                        if let Ok(FhirPathValue::Boolean(true)) = self.equals(existing, item) {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        result_items.push(item.clone());
                    }
                }
            }
            single => {
                let mut found = false;
                for existing in &result_items {
                    if let Ok(FhirPathValue::Boolean(true)) = self.equals(existing, single) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    result_items.push(single.clone());
                }
            }
        }

        // Return appropriate result
        match result_items.len() {
            0 => Ok(FhirPathValue::Empty),
            1 => Ok(result_items.into_iter().next().unwrap()),
            _ => Ok(FhirPathValue::Collection(Collection::from_values(
                result_items,
            ))),
        }
    }

    fn is_type(&self, value: &FhirPathValue, type_name: &FhirPathValue) -> Result<FhirPathValue> {
        // Extract type name from right operand
        let target_type = match type_name {
            FhirPathValue::String(s) => s,
            FhirPathValue::Empty => {
                // Special case: When the type name expression evaluates to empty,
                // it might be a namespace type reference like System.Integer or FHIR.code
                // For now, return false to indicate the type check failed rather than erroring
                return Ok(FhirPathValue::Boolean(false));
            }
            FhirPathValue::Collection(coll) if coll.is_empty() => {
                // Also handle empty collections (same as Empty)
                return Ok(FhirPathValue::Boolean(false));
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    FP0052,
                    "Type name must be a string".to_string(),
                ));
            }
        };

        Ok(FhirPathValue::Boolean(self.is_of_type(value, target_type)))
    }

    fn as_type(&self, value: &FhirPathValue, type_name: &FhirPathValue) -> Result<FhirPathValue> {
        // Extract type name from right operand
        let target_type = match type_name {
            FhirPathValue::String(s) => s,
            FhirPathValue::Empty => {
                // Special case: When the type name expression evaluates to empty,
                // it might be a namespace type reference like System.Integer or FHIR.code
                // For now, return empty to indicate the cast failed rather than erroring
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(coll) if coll.is_empty() => {
                // Also handle empty collections (same as Empty)
                return Ok(FhirPathValue::Empty);
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    FP0052,
                    "Type name must be a string".to_string(),
                ));
            }
        };

        self.cast_to_type(value, target_type)
    }
}

impl Default for OperatorEvaluatorImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl OperatorEvaluator for OperatorEvaluatorImpl {
    fn evaluate_binary_op(
        &self,
        left: &FhirPathValue,
        operator: &BinaryOperator,
        right: &FhirPathValue,
    ) -> Result<FhirPathValue> {
        match operator {
            BinaryOperator::Equal => self.equals(left, right),
            BinaryOperator::NotEqual => self.not_equals(left, right),
            BinaryOperator::LessThan => self.less_than(left, right),
            BinaryOperator::LessThanOrEqual => self.less_than_or_equal(left, right),
            BinaryOperator::GreaterThan => self.greater_than(left, right),
            BinaryOperator::GreaterThanOrEqual => self.greater_than_or_equal(left, right),
            BinaryOperator::Add => self.add(left, right),
            BinaryOperator::Subtract => self.subtract(left, right),
            BinaryOperator::Multiply => self.multiply(left, right),
            BinaryOperator::Divide => self.divide(left, right),
            BinaryOperator::Modulo => self.modulo(left, right),
            BinaryOperator::And => self.and(left, right),
            BinaryOperator::Or => self.or(left, right),
            BinaryOperator::Xor => self.xor(left, right),
            BinaryOperator::In => self.contains(right, left), // Note: reversed order for 'in'
            BinaryOperator::Contains => self.contains(left, right),
            BinaryOperator::IntegerDivide => self.integer_divide(left, right),
            BinaryOperator::Equivalent => self.equivalent(left, right),
            BinaryOperator::NotEquivalent => self.not_equivalent(left, right),
            BinaryOperator::Implies => self.implies(left, right),
            BinaryOperator::Concatenate => self.concatenate(left, right),
            BinaryOperator::Union => self.union(left, right),
            BinaryOperator::Is => self.is_type(left, right),
            BinaryOperator::As => self.as_type(left, right),
        }
    }

    fn evaluate_unary_op(
        &self,
        operator: &UnaryOperator,
        operand: &FhirPathValue,
    ) -> Result<FhirPathValue> {
        match operator {
            UnaryOperator::Positive => match operand {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),
                FhirPathValue::Quantity { .. } => Ok(operand.clone()),
                _ => Err(FhirPathError::evaluation_error(
                    FP0052,
                    format!("Cannot apply unary plus to {}", operand.type_name()),
                )),
            },
            UnaryOperator::Negate => match operand {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(-i)),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
                FhirPathValue::Quantity { value, unit, .. } => {
                    Ok(FhirPathValue::quantity(-value, unit.clone()))
                }
                _ => Err(FhirPathError::evaluation_error(
                    FP0052,
                    format!("Cannot apply unary minus to {}", operand.type_name()),
                )),
            },
            UnaryOperator::Not => match operand {
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
                FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                _ => Err(FhirPathError::evaluation_error(
                    FP0052,
                    format!(
                        "Cannot apply NOT to non-boolean value: {}",
                        operand.type_name()
                    ),
                )),
            },
        }
    }

    fn cast_to_type(&self, value: &FhirPathValue, target_type: &str) -> Result<FhirPathValue> {
        match target_type.to_lowercase().as_str() {
            "boolean" => match value {
                FhirPathValue::Boolean(_) => Ok(value.clone()),
                FhirPathValue::String(s) => match s.to_lowercase().as_str() {
                    "true" => Ok(FhirPathValue::Boolean(true)),
                    "false" => Ok(FhirPathValue::Boolean(false)),
                    _ => Ok(FhirPathValue::Empty),
                },
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Boolean(*i != 0)),
                _ => Ok(FhirPathValue::Empty),
            },
            "integer" => match value {
                FhirPathValue::Integer(_) => Ok(value.clone()),
                FhirPathValue::String(s) => match s.parse::<i64>() {
                    Ok(i) => Ok(FhirPathValue::Integer(i)),
                    Err(_) => Ok(FhirPathValue::Empty),
                },
                FhirPathValue::Decimal(d) => {
                    if d.fract().is_zero() {
                        Ok(FhirPathValue::Integer(d.to_i64().unwrap_or(0)))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Integer(if *b { 1 } else { 0 })),
                _ => Ok(FhirPathValue::Empty),
            },
            "decimal" => match value {
                FhirPathValue::Decimal(_) => Ok(value.clone()),
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(Decimal::from(*i))),
                FhirPathValue::String(s) => match s.parse::<Decimal>() {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                },
                _ => Ok(FhirPathValue::Empty),
            },
            "string" => match value {
                FhirPathValue::String(_) => Ok(value.clone()),
                _ => match value.to_string() {
                    Ok(s) => Ok(FhirPathValue::String(s)),
                    Err(_) => Ok(FhirPathValue::Empty),
                },
            },
            _ => {
                // For unknown types, return empty
                Ok(FhirPathValue::Empty)
            }
        }
    }

    fn is_of_type(&self, value: &FhirPathValue, target_type: &str) -> bool {
        let value_type = value.type_name().to_lowercase();
        let target = target_type.to_lowercase();

        // Direct type match
        if value_type == target {
            return true;
        }

        // Handle type hierarchy and compatibility
        match target.as_str() {
            "system.any" => true, // All values are of type System.Any
            "system.boolean" => matches!(value, FhirPathValue::Boolean(_)),
            "system.integer" => matches!(value, FhirPathValue::Integer(_)),
            "system.decimal" => {
                matches!(value, FhirPathValue::Decimal(_) | FhirPathValue::Integer(_))
            }
            "system.string" => matches!(
                value,
                FhirPathValue::String(_) | FhirPathValue::Uri(_) | FhirPathValue::Url(_)
            ),
            "system.date" => matches!(value, FhirPathValue::Date(_)),
            "system.datetime" => matches!(value, FhirPathValue::DateTime(_)),
            "system.time" => matches!(value, FhirPathValue::Time(_)),
            "system.quantity" => matches!(value, FhirPathValue::Quantity { .. }),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_equality_operations() {
        let evaluator = StandardOperatorEvaluator::new();

        // Boolean equality
        let result = evaluator
            .equals(&FhirPathValue::Boolean(true), &FhirPathValue::Boolean(true))
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = evaluator
            .equals(
                &FhirPathValue::Boolean(true),
                &FhirPathValue::Boolean(false),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Numeric equality with type coercion
        let result = evaluator
            .equals(
                &FhirPathValue::Integer(5),
                &FhirPathValue::Decimal(dec!(5.0)),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // String equality
        let result = evaluator
            .equals(
                &FhirPathValue::String("test".to_string()),
                &FhirPathValue::String("test".to_string()),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Empty comparisons
        let result = evaluator
            .equals(&FhirPathValue::Empty, &FhirPathValue::Empty)
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = evaluator
            .equals(&FhirPathValue::Empty, &FhirPathValue::Integer(5))
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_comparison_operations() {
        let evaluator = StandardOperatorEvaluator::new();

        // Numeric comparisons
        let result = evaluator
            .less_than(&FhirPathValue::Integer(3), &FhirPathValue::Integer(5))
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let result = evaluator
            .greater_than(
                &FhirPathValue::Decimal(dec!(5.5)),
                &FhirPathValue::Integer(5),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // String comparisons
        let result = evaluator
            .less_than(
                &FhirPathValue::String("apple".to_string()),
                &FhirPathValue::String("banana".to_string()),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_arithmetic_operations() {
        let evaluator = StandardOperatorEvaluator::new();

        // Addition
        let result = evaluator
            .add(&FhirPathValue::Integer(3), &FhirPathValue::Integer(5))
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(8));

        let result = evaluator
            .add(
                &FhirPathValue::Decimal(dec!(3.5)),
                &FhirPathValue::Integer(2),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Decimal(dec!(5.5)));

        // String concatenation
        let result = evaluator
            .add(
                &FhirPathValue::String("Hello ".to_string()),
                &FhirPathValue::String("World".to_string()),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::String("Hello World".to_string()));

        // Division by zero
        let result = evaluator.divide(&FhirPathValue::Integer(5), &FhirPathValue::Integer(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_logical_operations() {
        let evaluator = StandardOperatorEvaluator::new();

        // AND operation
        let result = evaluator
            .and(
                &FhirPathValue::Boolean(true),
                &FhirPathValue::Boolean(false),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // OR operation
        let result = evaluator
            .or(
                &FhirPathValue::Boolean(true),
                &FhirPathValue::Boolean(false),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // XOR operation
        let result = evaluator
            .xor(
                &FhirPathValue::Boolean(true),
                &FhirPathValue::Boolean(false),
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_unary_operations() {
        let evaluator = StandardOperatorEvaluator::new();

        // Unary minus
        let result = evaluator
            .evaluate_unary_op(&UnaryOperator::Minus, &FhirPathValue::Integer(5))
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(-5));

        // Logical NOT
        let result = evaluator
            .evaluate_unary_op(&UnaryOperator::Not, &FhirPathValue::Boolean(true))
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_type_casting() {
        let evaluator = StandardOperatorEvaluator::new();

        // String to integer
        let result = evaluator
            .cast_to_type(&FhirPathValue::String("42".to_string()), "integer")
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Boolean to string
        let result = evaluator
            .cast_to_type(&FhirPathValue::Boolean(true), "string")
            .unwrap();
        assert_eq!(result, FhirPathValue::String("true".to_string()));

        // Invalid cast
        let result = evaluator
            .cast_to_type(
                &FhirPathValue::String("not_a_number".to_string()),
                "integer",
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_type_checking() {
        let evaluator = StandardOperatorEvaluator::new();

        // Direct type check
        assert!(evaluator.is_of_type(&FhirPathValue::Integer(42), "Integer"));
        assert!(evaluator.is_of_type(&FhirPathValue::String("test".to_string()), "String"));

        // System type hierarchy
        assert!(evaluator.is_of_type(&FhirPathValue::Integer(42), "System.Any"));
        assert!(evaluator.is_of_type(&FhirPathValue::Integer(42), "System.Decimal")); // Integer is a Decimal

        // Negative cases
        assert!(!evaluator.is_of_type(&FhirPathValue::String("test".to_string()), "Integer"));
    }

    #[test]
    fn test_quantity_operations() {
        let evaluator = StandardOperatorEvaluator::new();

        let qty1 = FhirPathValue::quantity(dec!(5.0), Some("kg".to_string()));
        let qty2 = FhirPathValue::quantity(dec!(3.0), Some("kg".to_string()));

        // Addition of compatible quantities
        let result = evaluator.add(&qty1, &qty2).unwrap();
        match result {
            FhirPathValue::Quantity { value, unit, .. } => {
                assert_eq!(value, dec!(8.0));
                assert_eq!(unit, Some("kg".to_string()));
            }
            _ => panic!("Expected Quantity result"),
        }

        // Multiplication with scalar
        let scalar = FhirPathValue::Integer(2);
        let result = evaluator.multiply(&qty1, &scalar).unwrap();
        match result {
            FhirPathValue::Quantity { value, unit, .. } => {
                assert_eq!(value, dec!(10.0));
                assert_eq!(unit, Some("kg".to_string()));
            }
            _ => panic!("Expected Quantity result"),
        }
    }

    #[test]
    fn test_collection_membership() {
        let evaluator = StandardOperatorEvaluator::new();

        let collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);

        // Item is in collection
        let result = evaluator
            .contains(&collection, &FhirPathValue::Integer(2))
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Item is not in collection
        let result = evaluator
            .contains(&collection, &FhirPathValue::Integer(5))
            .unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
}
