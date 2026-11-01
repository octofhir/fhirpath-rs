//! Utility functions for quantity operations and unit conversions
//!
//! This module provides helper functions for working with quantities in FHIRPath,
//! including unit conversions using the UCUM library.

use crate::core::{FhirPathValue, types::CalendarUnit};
use octofhir_ucum::{EvalResult, evaluate, parse_expression};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::borrow::Cow;
use std::sync::Arc;

/// Parse a string into a Quantity according to FHIRPath rules used by convertsToQuantity()/toQuantity().
///
/// Supported forms:
/// - `"<number>"` → unitless quantity
/// - `"<number> '<ucum>'"` → UCUM unit in single quotes
/// - `"<number> <calendar-word>"` → calendar unit (e.g., day, week, month, year, hour, minute, second, millisecond)
///
/// Notably, unquoted UCUM abbreviations like "wk" or "mo" are NOT accepted as calendar words.
pub fn parse_string_to_quantity_value(s: &str) -> Option<FhirPathValue> {
    let trimmed = s.trim();

    // 1) Plain number → unitless quantity
    if let Some(dec) = crate::core::value::utils::decimal_from_text(trimmed) {
        return Some(FhirPathValue::quantity_with_components(
            dec,
            Some("1".to_string()),
            None,
            None,
        ));
    }

    // 2) Split into two parts (number and unit)
    let mut parts = trimmed.split_whitespace();
    let num = parts.next()?;
    let unit_part = parts.next()?;
    // There should be exactly 2 parts
    if parts.next().is_some() {
        return None;
    }

    // Parse number first
    let dec = crate::core::value::utils::decimal_from_text(num)?;

    // Quoted UCUM: '<unit>'
    if unit_part.len() >= 2 && unit_part.starts_with('\'') && unit_part.ends_with('\'') {
        let inner = &unit_part[1..unit_part.len() - 1];
        return Some(FhirPathValue::quoted_quantity(dec, Some(inner.to_string())));
    }

    // Calendar words (full words only; no UCUM abbreviations like wk, mo, a)
    let unit_lc = unit_part.to_ascii_lowercase();
    // Accept singular/plural full words only
    let accepted = [
        "millisecond",
        "milliseconds",
        "second",
        "seconds",
        "minute",
        "minutes",
        "hour",
        "hours",
        "day",
        "days",
        "week",
        "weeks",
        "month",
        "months",
        "year",
        "years",
    ];

    if accepted.contains(&unit_lc.as_str())
        && let Some(cal) = CalendarUnit::from_str(&unit_lc)
    {
        return Some(FhirPathValue::calendar_quantity(dec, cal));
    }

    None
}

fn normalize_ucum_unit(unit: &str) -> Cow<'_, str> {
    match unit.trim() {
        "°C" => Cow::Borrowed("Cel"),
        "°F" => Cow::Borrowed("[degF]"),
        _ => Cow::Borrowed(unit),
    }
}

/// Result of a quantity conversion operation
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub value: Decimal,
    pub unit: Option<String>,
    pub ucum_unit: Option<Arc<octofhir_ucum::UnitRecord>>,
    pub calendar_unit: Option<CalendarUnit>,
}

/// Error type for quantity operations
#[derive(Debug, Clone)]
pub enum QuantityError {
    /// Units are not compatible for conversion
    IncompatibleUnits(String, String),
    /// Failed to parse unit expression
    ParseError(String),
    /// Failed to evaluate unit expression
    EvaluationError(String),
    /// Invalid quantity data
    InvalidQuantity(String),
}

impl std::fmt::Display for QuantityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantityError::IncompatibleUnits(from, to) => {
                write!(
                    f,
                    "Cannot convert from '{from}' to '{to}': incompatible units"
                )
            }
            QuantityError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            QuantityError::EvaluationError(msg) => write!(f, "Evaluation error: {msg}"),
            QuantityError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {msg}"),
        }
    }
}

impl std::error::Error for QuantityError {}

/// Check if two unit strings are equivalent
/// Handles common cases where unit and code fields might differ but represent the same unit
fn units_are_equivalent(unit1: &str, unit2: &str) -> bool {
    // Direct match (already checked above, but for completeness)
    if unit1 == unit2 {
        return true;
    }

    // Common pound equivalences in FHIR/UCUM
    let pound_variants = ["lb", "lbs", "[lb_av]", "lb_av"];
    let is_pound1 = pound_variants.contains(&unit1);
    let is_pound2 = pound_variants.contains(&unit2);

    if is_pound1 && is_pound2 {
        return true;
    }

    // Add other common equivalences as needed
    // kg/kilogram, g/gram, m/meter, etc.
    let kg_variants = ["kg", "kilogram"];
    let is_kg1 = kg_variants.contains(&unit1);
    let is_kg2 = kg_variants.contains(&unit2);

    if is_kg1 && is_kg2 {
        return true;
    }

    false
}

/// Map UCUM time units to FHIRPath calendar units for equivalence checking.
/// Per FHIRPath spec: `1 year` is equivalent to `1 'a'`, `1 month` is equivalent to `1 'mo'`, etc.
/// Note: These are equivalent (~) but not equal (=) because UCUM uses fixed durations
/// (e.g., 'a' = 365.25 days) while FHIRPath calendar units are variable length.
fn ucum_to_calendar_unit(ucum_unit: &str) -> Option<CalendarUnit> {
    match ucum_unit {
        "a" => Some(CalendarUnit::Year),
        "mo" => Some(CalendarUnit::Month),
        "wk" => Some(CalendarUnit::Week),
        "d" => Some(CalendarUnit::Day),
        "h" => Some(CalendarUnit::Hour),
        "min" => Some(CalendarUnit::Minute),
        "s" => Some(CalendarUnit::Second),
        "ms" => Some(CalendarUnit::Millisecond),
        _ => None,
    }
}

/// Map FHIRPath calendar unit to its equivalent UCUM unit
#[allow(dead_code)]
fn calendar_to_ucum_unit(cal_unit: CalendarUnit) -> &'static str {
    match cal_unit {
        CalendarUnit::Year => "a",
        CalendarUnit::Month => "mo",
        CalendarUnit::Week => "wk",
        CalendarUnit::Day => "d",
        CalendarUnit::Hour => "h",
        CalendarUnit::Minute => "min",
        CalendarUnit::Second => "s",
        CalendarUnit::Millisecond => "ms",
    }
}

/// Compare two quantities and return an Ordering (for <, >, <= comparisons)
pub fn compare_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<Option<std::cmp::Ordering>, QuantityError> {
    // Handle simple cases first
    match (left_unit, right_unit) {
        // Both have no units - compare values directly
        (None, None) => Ok(Some(left_value.cmp(&right_value))),

        // One has unit, other doesn't - not comparable
        (Some(_), None) | (None, Some(_)) => Ok(None),

        // Both have same unit - compare values directly
        (Some(lu), Some(ru)) if lu == ru => Ok(Some(left_value.cmp(&right_value))),

        // Handle common unit equivalences that might not match exactly
        (Some(lu), Some(ru)) if units_are_equivalent(lu, ru) => {
            Ok(Some(left_value.cmp(&right_value)))
        }

        // Different units - need conversion
        (Some(lu), Some(ru)) => {
            // First check if they are calendar units
            if let (Some(left_cal), Some(right_cal)) = (left_calendar_unit, right_calendar_unit) {
                return compare_calendar_quantities_ordering(
                    left_value,
                    *left_cal,
                    right_value,
                    *right_cal,
                );
            }

            // Try UCUM conversion
            match convert_and_compare_ucum_ordering(left_value, lu, right_value, ru) {
                Ok(result) => Ok(result),
                Err(_) => {
                    // UCUM conversion failed, but if units are exactly the same string,
                    // we can still compare them directly as a fallback
                    if lu == ru {
                        return Ok(Some(left_value.cmp(&right_value)));
                    }
                    // If units are different and UCUM failed, return not comparable
                    Ok(None)
                }
            }
        }
    }
}

/// Check if two quantities are equivalent (same dimension and value after conversion with tolerance)
pub fn are_quantities_equivalent(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<bool, QuantityError> {
    are_quantities_equivalent_with_codes(
        left_value,
        left_unit,
        None, // No code available in legacy signature
        left_calendar_unit,
        right_value,
        right_unit,
        None, // No code available in legacy signature
        right_calendar_unit,
    )
}

/// Check if two quantities are equivalent, with optional UCUM code fields
/// This extended version allows passing the FHIR `code` field which contains
/// the canonical UCUM code (e.g., "a" for year) separate from the display `unit` (e.g., "year")
pub fn are_quantities_equivalent_with_codes(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_code: Option<&str>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_code: Option<&str>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<bool, QuantityError> {
    // Use code if available, otherwise fall back to unit
    let effective_left_unit = left_code
        .map(|s| s.to_string())
        .or_else(|| left_unit.clone());
    let effective_right_unit = right_code
        .map(|s| s.to_string())
        .or_else(|| right_unit.clone());

    // Handle simple cases first
    match (&effective_left_unit, &effective_right_unit) {
        // Both have no units - compare values directly
        (None, None) => Ok((left_value - right_value).abs() < Decimal::new(1, 10)),

        // One has unit, other doesn't - not equivalent
        (Some(_), None) | (None, Some(_)) => Ok(false),

        // Both have same unit - compare values directly
        (Some(lu), Some(ru)) if lu == ru => {
            Ok((left_value - right_value).abs() < Decimal::new(1, 10))
        }

        // Handle common unit equivalences that might not match exactly
        (Some(lu), Some(ru)) if units_are_equivalent(lu, ru) => {
            Ok((left_value - right_value).abs() < Decimal::new(1, 10))
        }

        // Different units - need conversion
        (Some(lu), Some(ru)) => {
            // First check if they are both calendar units
            if let (Some(left_cal), Some(right_cal)) = (left_calendar_unit, right_calendar_unit) {
                return compare_calendar_quantities(left_value, *left_cal, right_value, *right_cal);
            }

            // Handle cross-type comparison: UCUM time unit vs calendar unit
            // Per FHIRPath spec: `1 year` is equivalent to `1 'a'`, etc.
            match (left_calendar_unit, right_calendar_unit) {
                // Left is calendar, right might be UCUM time unit
                (Some(left_cal), None) => {
                    if let Some(right_cal) = ucum_to_calendar_unit(ru) {
                        return compare_calendar_quantities(
                            left_value,
                            *left_cal,
                            right_value,
                            right_cal,
                        );
                    }
                }
                // Right is calendar, left might be UCUM time unit
                (None, Some(right_cal)) => {
                    if let Some(left_cal) = ucum_to_calendar_unit(lu) {
                        return compare_calendar_quantities(
                            left_value,
                            left_cal,
                            right_value,
                            *right_cal,
                        );
                    }
                }
                // Neither is calendar, but both might be UCUM time units that map to calendar
                (None, None) => {
                    if let (Some(left_cal), Some(right_cal)) =
                        (ucum_to_calendar_unit(lu), ucum_to_calendar_unit(ru))
                    {
                        return compare_calendar_quantities(
                            left_value,
                            left_cal,
                            right_value,
                            right_cal,
                        );
                    }
                }
                _ => {}
            }

            // Try UCUM conversion for equivalence
            convert_and_compare_ucum_equivalence(left_value, lu, right_value, ru)
        }
    }
}

/// Check if two quantities are equal (same dimension and exact value after conversion)
pub fn are_quantities_equal(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<bool, QuantityError> {
    are_quantities_equal_with_codes(
        left_value,
        left_unit,
        None,
        left_calendar_unit,
        right_value,
        right_unit,
        None,
        right_calendar_unit,
    )
}

/// Check if two quantities are equal, with optional UCUM code fields
/// This extended version allows passing the FHIR `code` field which contains
/// the canonical UCUM code (e.g., "a" for year) separate from the display `unit` (e.g., "year")
pub fn are_quantities_equal_with_codes(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_code: Option<&str>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_code: Option<&str>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<bool, QuantityError> {
    // Use code if available, otherwise fall back to unit
    let effective_left_unit = left_code
        .map(|s| s.to_string())
        .or_else(|| left_unit.clone());
    let effective_right_unit = right_code
        .map(|s| s.to_string())
        .or_else(|| right_unit.clone());

    // Handle simple cases first
    match (&effective_left_unit, &effective_right_unit) {
        // Both have no units - compare values directly
        (None, None) => Ok((left_value - right_value).abs() < Decimal::new(1, 10)),

        // One has unit, other doesn't - not equal
        (Some(_), None) | (None, Some(_)) => Ok(false),

        // Both have same unit - compare values directly
        (Some(lu), Some(ru)) if lu == ru => {
            Ok((left_value - right_value).abs() < Decimal::new(1, 10))
        }

        // Handle common unit equivalences that might not match exactly
        (Some(lu), Some(ru)) if units_are_equivalent(lu, ru) => {
            Ok((left_value - right_value).abs() < Decimal::new(1, 10))
        }

        // Different units - need conversion
        (Some(lu), Some(ru)) => {
            // First check if they are calendar units
            if let (Some(left_cal), Some(right_cal)) = (left_calendar_unit, right_calendar_unit) {
                return compare_calendar_quantities(left_value, *left_cal, right_value, *right_cal);
            }

            // Try UCUM conversion for equality (exact)
            convert_and_compare_ucum(left_value, lu, right_value, ru)
        }
    }
}

/// Convert a quantity to a target unit
pub fn convert_quantity(
    value: Decimal,
    from_unit: &Option<String>,
    from_calendar_unit: &Option<CalendarUnit>,
    to_unit: &str,
) -> Result<ConversionResult, QuantityError> {
    // Handle calendar unit conversions
    if let Some(from_cal) = from_calendar_unit
        && let Some(to_cal) = CalendarUnit::from_str(to_unit)
    {
        let converted_value = convert_calendar_units(value, *from_cal, to_cal)?;
        return Ok(ConversionResult {
            value: converted_value,
            unit: Some(to_unit.to_string()),
            ucum_unit: None,
            calendar_unit: Some(to_cal),
        });
    }

    // Handle UCUM conversions
    if let Some(from_unit_str) = from_unit {
        return convert_ucum_quantity(value, from_unit_str, to_unit);
    }

    Err(QuantityError::InvalidQuantity(
        "Cannot convert unitless quantity to a specific unit".to_string(),
    ))
}

/// Compare two calendar quantities and return an Ordering
fn compare_calendar_quantities_ordering(
    left_value: Decimal,
    left_unit: CalendarUnit,
    right_value: Decimal,
    right_unit: CalendarUnit,
) -> Result<Option<std::cmp::Ordering>, QuantityError> {
    if left_unit == right_unit {
        Ok(Some(left_value.cmp(&right_value)))
    } else if left_unit.is_compatible_with(&right_unit) {
        // Convert right to left unit and compare
        let converted_right = convert_calendar_units(right_value, right_unit, left_unit)?;
        Ok(Some(left_value.cmp(&converted_right)))
    } else {
        Ok(None)
    }
}

/// Compare two calendar quantities for equivalence
fn compare_calendar_quantities(
    left_value: Decimal,
    left_unit: CalendarUnit,
    right_value: Decimal,
    right_unit: CalendarUnit,
) -> Result<bool, QuantityError> {
    if left_unit == right_unit {
        Ok((left_value - right_value).abs() < Decimal::new(1, 10))
    } else if left_unit.is_compatible_with(&right_unit) {
        // Convert right to left unit and compare
        let converted_right = convert_calendar_units(right_value, right_unit, left_unit)?;
        Ok((left_value - converted_right).abs() < Decimal::new(1, 10))
    } else {
        Ok(false)
    }
}

/// Convert between calendar units
fn convert_calendar_units(
    value: Decimal,
    from_unit: CalendarUnit,
    to_unit: CalendarUnit,
) -> Result<Decimal, QuantityError> {
    if from_unit == to_unit {
        return Ok(value);
    }

    // Get conversion factors in days using approximate_days
    let from_days = Decimal::try_from(from_unit.approximate_days()).map_err(|_| {
        QuantityError::EvaluationError("Failed to convert calendar unit to decimal".to_string())
    })?;

    let to_days = Decimal::try_from(to_unit.approximate_days()).map_err(|_| {
        QuantityError::EvaluationError("Failed to convert calendar unit to decimal".to_string())
    })?;

    // Convert: value * from_days / to_days
    Ok(value * from_days / to_days)
}

/// Parse and evaluate a UCUM unit (after FHIRPath display-unit normalization).
fn evaluate_unit(unit: &str) -> Result<EvalResult, QuantityError> {
    let normalized = normalize_ucum_unit(unit);
    let expr = parse_expression(normalized.as_ref())
        .map_err(|e| QuantityError::ParseError(format!("Failed to parse '{unit}': {e}")))?;
    evaluate(&expr)
        .map_err(|e| QuantityError::EvaluationError(format!("Failed to evaluate '{unit}': {e}")))
}

/// Linear units (no special function, UCUM §21-23) are commensurable when they
/// have the same base-unit dimension and the same arbitrary atoms (§24-25:
/// `[IU]` is commensurable with no other unit).
fn linear_commensurable(left: &EvalResult, right: &EvalResult) -> bool {
    left.dim == right.dim && left.arbitrary == right.arbitrary
}

fn decimal_to_f64(value: Decimal) -> Result<f64, QuantityError> {
    value.to_f64().ok_or_else(|| {
        QuantityError::EvaluationError(format!("Quantity value {value} is out of range"))
    })
}

/// Both quantities expressed on one scale, for comparison: the magnitudes in
/// base units for linear units, or the right value converted into the left
/// unit when either is a special unit (`Cel`, `[degF]`, `[pH]`, ...), whose
/// function is not a factor. `None` when the units are not commensurable.
fn ucum_comparable_values(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<Option<(f64, f64)>, QuantityError> {
    let left_eval = evaluate_unit(left_unit)?;
    let right_eval = evaluate_unit(right_unit)?;

    if left_eval.special.is_none() && right_eval.special.is_none() {
        if !linear_commensurable(&left_eval, &right_eval) {
            return Ok(None);
        }
        let left = decimal_to_f64(left_value)? * left_eval.factor.to_f64();
        let right = decimal_to_f64(right_value)? * right_eval.factor.to_f64();
        return Ok(Some((left, right)));
    }

    let left_normalized = normalize_ucum_unit(left_unit);
    let right_normalized = normalize_ucum_unit(right_unit);
    let comparable =
        octofhir_ucum::is_comparable(left_normalized.as_ref(), right_normalized.as_ref())
            .map_err(|e| QuantityError::EvaluationError(e.to_string()))?;
    if !comparable {
        return Ok(None);
    }
    let right_in_left = octofhir_ucum::convert(
        decimal_to_f64(right_value)?,
        right_normalized.as_ref(),
        left_normalized.as_ref(),
    )
    .map_err(|e| QuantityError::EvaluationError(e.to_string()))?;
    Ok(Some((decimal_to_f64(left_value)?, right_in_left)))
}

/// Convert and compare two UCUM quantities for ordering
fn convert_and_compare_ucum_ordering(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<Option<std::cmp::Ordering>, QuantityError> {
    Ok(
        ucum_comparable_values(left_value, left_unit, right_value, right_unit)?.map(
            |(left, right)| {
                left.partial_cmp(&right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            },
        ),
    )
}

/// Convert and compare two UCUM quantities for equivalence (with tolerance)
fn convert_and_compare_ucum_equivalence(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<bool, QuantityError> {
    let Some((left, right)) =
        ucum_comparable_values(left_value, left_unit, right_value, right_unit)?
    else {
        return Ok(false);
    };
    // Use a more lenient tolerance for equivalence to account for measurement precision
    // This allows for small differences that might occur in real-world measurements
    let tolerance = (left.abs().max(right.abs())) * 0.01; // 1% tolerance
    let min_tolerance = 1e-10; // Minimum absolute tolerance
    Ok((left - right).abs() < tolerance.max(min_tolerance))
}

/// Convert and compare two UCUM quantities
fn convert_and_compare_ucum(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<bool, QuantityError> {
    Ok(
        ucum_comparable_values(left_value, left_unit, right_value, right_unit)?
            .is_some_and(|(left, right)| (left - right).abs() < 1e-10),
    )
}

/// Check if two UCUM units are comparable (commensurable)
pub fn are_ucum_units_comparable(left_unit: &str, right_unit: &str) -> Result<bool, QuantityError> {
    // If units are identical, they're comparable
    if left_unit == right_unit {
        return Ok(true);
    }
    let left_normalized = normalize_ucum_unit(left_unit);
    let right_normalized = normalize_ucum_unit(right_unit);
    // Parse first so a malformed unit is a parse error, not "not comparable".
    evaluate_unit(left_unit)?;
    evaluate_unit(right_unit)?;
    octofhir_ucum::is_comparable(left_normalized.as_ref(), right_normalized.as_ref())
        .map_err(|e| QuantityError::EvaluationError(e.to_string()))
}

/// Convert a UCUM quantity to a target unit
fn convert_ucum_quantity(
    value: Decimal,
    from_unit: &str,
    to_unit: &str,
) -> Result<ConversionResult, QuantityError> {
    let from_eval = evaluate_unit(from_unit)?;
    let to_eval = evaluate_unit(to_unit)?;
    let to_unit_normalized = normalize_ucum_unit(to_unit);
    let to_unit_value = to_unit_normalized.as_ref();

    let incompatible =
        || QuantityError::IncompatibleUnits(from_unit.to_string(), to_unit.to_string());

    let converted_value = if from_eval.special.is_none() && to_eval.special.is_none() {
        if !linear_commensurable(&from_eval, &to_eval) {
            return Err(incompatible());
        }
        // Linear: scale by the ratio of the magnitudes, kept in Decimal so an
        // exact conversion (1 'km' -> 1000 'm') stays exact.
        let ratio = from_eval
            .factor
            .checked_div(to_eval.factor)
            .ok_or_else(incompatible)?
            .to_f64();
        value * Decimal::try_from(ratio).unwrap_or_default()
    } else {
        // Special units convert through their function, not a factor.
        let from_normalized = normalize_ucum_unit(from_unit);
        let converted = octofhir_ucum::convert(
            decimal_to_f64(value)?,
            from_normalized.as_ref(),
            to_unit_value,
        )
        .map_err(|_| incompatible())?;
        Decimal::try_from(converted).map_err(|_| {
            QuantityError::EvaluationError(format!("Converted value {converted} is out of range"))
        })?
    };

    // Try to find the target unit record
    let ucum_unit =
        crate::core::types::ucum_unit_record(to_unit_value).map(|u| Arc::new(u.clone()));

    Ok(ConversionResult {
        value: converted_value,
        unit: Some(to_unit_value.to_string()),
        ucum_unit,
        calendar_unit: CalendarUnit::from_str(to_unit_value),
    })
}

/// Multiply two quantities, handling unit combination
pub fn multiply_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<FhirPathValue, QuantityError> {
    let result_value = left_value * right_value;

    // Handle unit combination
    let (result_unit, _result_ucum_unit, _result_calendar_unit) = match (left_unit, right_unit) {
        (None, None) => (None, None::<Arc<octofhir_ucum::UnitRecord>>, None),
        (Some(l), None) => (Some(l.clone()), None, *left_calendar_unit),
        (None, Some(r)) => (Some(r.clone()), None, *right_calendar_unit),
        (Some(l), Some(r)) => {
            // For now, simple concatenation - TODO: implement proper UCUM unit multiplication
            let combined_unit = format!("{l}.{r}");
            (Some(combined_unit), None, None)
        }
    };

    Ok(FhirPathValue::quantity_with_components(
        result_value,
        result_unit.clone(),
        result_unit,
        None,
    ))
}

/// Divide two quantities, handling unit combination
pub fn divide_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    _right_calendar_unit: &Option<CalendarUnit>,
) -> Result<FhirPathValue, QuantityError> {
    if right_value == Decimal::ZERO {
        return Err(QuantityError::InvalidQuantity(
            "Division by zero".to_string(),
        ));
    }

    let result_value = left_value / right_value;

    // Handle unit combination
    let (result_unit, _result_ucum_unit, _result_calendar_unit) = match (left_unit, right_unit) {
        (None, None) => (None, None::<Arc<octofhir_ucum::UnitRecord>>, None),
        (Some(l), None) => (Some(l.clone()), None, *left_calendar_unit),
        (None, Some(r)) => (Some(format!("1/{r}")), None, None),
        (Some(l), Some(r)) => {
            // Handle special cases
            if l == r {
                // Same units cancel out to dimensionless
                (Some("1".to_string()), None, None)
            } else {
                // For now, simple concatenation - TODO: implement proper UCUM unit division
                let combined_unit = format!("{l}/{r}");
                (Some(combined_unit), None, None)
            }
        }
    };

    Ok(FhirPathValue::quantity_with_components(
        result_value,
        result_unit.clone(),
        result_unit,
        None,
    ))
}

#[cfg(test)]
mod tests {

    #[test]
    fn ucum_linear_conversion_scales_by_the_magnitude_ratio() {
        let mg_to_g = convert_ucum_quantity(Decimal::new(1000, 0), "mg", "g");
        assert_eq!(mg_to_g.expect("mg -> g").value, Decimal::new(1, 0));
        let km_to_m = convert_ucum_quantity(Decimal::new(15, 1), "km", "m");
        assert_eq!(km_to_m.expect("km -> m").value, Decimal::new(1500, 0));
    }

    #[test]
    fn ucum_special_units_convert_through_their_function() {
        // UCUM §21-23: a temperature is not a factor of kelvin.
        let c_to_f = convert_ucum_quantity(Decimal::new(100, 0), "Cel", "[degF]");
        assert_eq!(c_to_f.expect("Cel -> [degF]").value, Decimal::new(212, 0));
    }

    #[test]
    fn ucum_arbitrary_units_are_not_commensurable() {
        // UCUM §24-25: [IU] converts to no other unit.
        assert!(convert_ucum_quantity(Decimal::new(1, 0), "[IU]", "1").is_err());
        assert!(!are_ucum_units_comparable("[IU]", "[arb'U]").unwrap());
    }

    use super::*;

    #[test]
    fn test_ucum_to_calendar_mapping() {
        // Test that UCUM time units map to calendar units
        assert_eq!(ucum_to_calendar_unit("a"), Some(CalendarUnit::Year));
        assert_eq!(ucum_to_calendar_unit("mo"), Some(CalendarUnit::Month));
        assert_eq!(ucum_to_calendar_unit("wk"), Some(CalendarUnit::Week));
        assert_eq!(ucum_to_calendar_unit("d"), Some(CalendarUnit::Day));
        assert_eq!(ucum_to_calendar_unit("h"), Some(CalendarUnit::Hour));
        assert_eq!(ucum_to_calendar_unit("min"), Some(CalendarUnit::Minute));
        assert_eq!(ucum_to_calendar_unit("s"), Some(CalendarUnit::Second));
        assert_eq!(ucum_to_calendar_unit("ms"), Some(CalendarUnit::Millisecond));
        // Non-time units should return None
        assert_eq!(ucum_to_calendar_unit("kg"), None);
        assert_eq!(ucum_to_calendar_unit("m"), None);
    }

    #[test]
    fn test_ucum_a_vs_calendar_year_equivalence() {
        // Per FHIRPath spec: 1 'a' ~ 1 year should be true
        let result = are_quantities_equivalent(
            Decimal::new(1, 0),
            &Some("a".to_string()),
            &None, // UCUM, not calendar
            Decimal::new(1, 0),
            &Some("year".to_string()),
            &Some(CalendarUnit::Year),
        );
        assert!(result.unwrap(), "1 'a' should be equivalent to 1 year");
    }

    #[test]
    fn test_calendar_year_vs_ucum_a_equivalence() {
        // Same test in reverse order
        let result = are_quantities_equivalent(
            Decimal::new(1, 0),
            &Some("year".to_string()),
            &Some(CalendarUnit::Year),
            Decimal::new(1, 0),
            &Some("a".to_string()),
            &None, // UCUM, not calendar
        );
        assert!(result.unwrap(), "1 year should be equivalent to 1 'a'");
    }

    #[test]
    fn test_same_unit_equivalence() {
        let result = are_quantities_equivalent(
            Decimal::new(1000, 0),
            &Some("g".to_string()),
            &None,
            Decimal::new(1000, 0),
            &Some("g".to_string()),
            &None,
        );
        assert!(result.unwrap());
    }

    #[test]
    fn test_ucum_conversion() {
        let result = are_quantities_equivalent(
            Decimal::new(1000, 0),
            &Some("mg".to_string()),
            &None,
            Decimal::new(1, 0),
            &Some("g".to_string()),
            &None,
        );
        assert!(result.unwrap());
    }

    #[test]
    fn test_calendar_unit_conversion() {
        let result = are_quantities_equivalent(
            Decimal::new(7, 0),
            &Some("days".to_string()),
            &Some(CalendarUnit::Day),
            Decimal::new(1, 0),
            &Some("week".to_string()),
            &Some(CalendarUnit::Week),
        );
        assert!(result.unwrap());
    }
}
