//! Utility functions for quantity operations and unit conversions
//!
//! This module provides helper functions for working with quantities in FHIRPath,
//! including unit conversions using the UCUM library.

use crate::core::{FhirPathValue, types::CalendarUnit};
use octofhir_ucum::{parse_expression, evaluate_owned, precision::to_f64};
use rust_decimal::Decimal;
use std::sync::Arc;

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
                write!(f, "Cannot convert from '{}' to '{}': incompatible units", from, to)
            }
            QuantityError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            QuantityError::EvaluationError(msg) => write!(f, "Evaluation error: {}", msg),
            QuantityError::InvalidQuantity(msg) => write!(f, "Invalid quantity: {}", msg),
        }
    }
}

impl std::error::Error for QuantityError {}

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
        (None, None) => return Ok(Some(left_value.cmp(&right_value))),

        // One has unit, other doesn't - not comparable
        (Some(_), None) | (None, Some(_)) => return Ok(None),

        // Both have same unit - compare values directly
        (Some(lu), Some(ru)) if lu == ru => {
            return Ok(Some(left_value.cmp(&right_value)));
        }

        // Different units - need conversion
        (Some(lu), Some(ru)) => {
            // First check if they are calendar units
            if let (Some(left_cal), Some(right_cal)) = (left_calendar_unit, right_calendar_unit) {
                return compare_calendar_quantities_ordering(left_value, *left_cal, right_value, *right_cal);
            }

            // Try UCUM conversion
            return convert_and_compare_ucum_ordering(left_value, lu, right_value, ru);
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
    // Handle simple cases first
    match (left_unit, right_unit) {
        // Both have no units - compare values directly
        (None, None) => return Ok((left_value - right_value).abs() < Decimal::new(1, 10)),

        // One has unit, other doesn't - not equivalent
        (Some(_), None) | (None, Some(_)) => return Ok(false),

        // Both have same unit - compare values directly
        (Some(lu), Some(ru)) if lu == ru => {
            return Ok((left_value - right_value).abs() < Decimal::new(1, 10));
        }

        // Different units - need conversion
        (Some(lu), Some(ru)) => {
            // First check if they are calendar units
            if let (Some(left_cal), Some(right_cal)) = (left_calendar_unit, right_calendar_unit) {
                return compare_calendar_quantities(left_value, *left_cal, right_value, *right_cal);
            }

            // Try UCUM conversion for equivalence
            return convert_and_compare_ucum_equivalence(left_value, lu, right_value, ru);
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
    // Handle simple cases first
    match (left_unit, right_unit) {
        // Both have no units - compare values directly
        (None, None) => return Ok((left_value - right_value).abs() < Decimal::new(1, 10)),

        // One has unit, other doesn't - not equal
        (Some(_), None) | (None, Some(_)) => return Ok(false),

        // Both have same unit - compare values directly
        (Some(lu), Some(ru)) if lu == ru => {
            return Ok((left_value - right_value).abs() < Decimal::new(1, 10));
        }

        // Different units - need conversion
        (Some(lu), Some(ru)) => {
            // First check if they are calendar units
            if let (Some(left_cal), Some(right_cal)) = (left_calendar_unit, right_calendar_unit) {
                return compare_calendar_quantities(left_value, *left_cal, right_value, *right_cal);
            }

            // Try UCUM conversion for equality (exact)
            return convert_and_compare_ucum(left_value, lu, right_value, ru);
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
    if let Some(from_cal) = from_calendar_unit {
        if let Some(to_cal) = CalendarUnit::from_str(to_unit) {
            let converted_value = convert_calendar_units(value, *from_cal, to_cal)?;
            return Ok(ConversionResult {
                value: converted_value,
                unit: Some(to_unit.to_string()),
                ucum_unit: None,
                calendar_unit: Some(to_cal),
            });
        }
    }

    // Handle UCUM conversions
    if let Some(from_unit_str) = from_unit {
        return convert_ucum_quantity(value, from_unit_str, to_unit);
    }

    Err(QuantityError::InvalidQuantity("Cannot convert unitless quantity to a specific unit".to_string()))
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

/// Convert and compare two UCUM quantities for ordering
fn convert_and_compare_ucum_ordering(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<Option<std::cmp::Ordering>, QuantityError> {
    // Parse both units
    let left_expr = parse_expression(left_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", left_unit, e))
    })?;

    let right_expr = parse_expression(right_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", right_unit, e))
    })?;

    // Evaluate both units
    let left_eval = evaluate_owned(&left_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", left_unit, e))
    })?;

    let right_eval = evaluate_owned(&right_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", right_unit, e))
    })?;

    // Check if dimensions are compatible
    if left_eval.dim != right_eval.dim {
        return Ok(None);
    }

    // Convert both to canonical units and compare
    let left_canonical = to_f64(left_value * Decimal::try_from(to_f64(left_eval.factor)).unwrap_or_default());
    let right_canonical = to_f64(right_value * Decimal::try_from(to_f64(right_eval.factor)).unwrap_or_default());

    Ok(Some(left_canonical.partial_cmp(&right_canonical).unwrap_or(std::cmp::Ordering::Equal)))
}

/// Convert and compare two UCUM quantities for equivalence (with tolerance)
fn convert_and_compare_ucum_equivalence(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<bool, QuantityError> {
    // Parse both units
    let left_expr = parse_expression(left_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", left_unit, e))
    })?;

    let right_expr = parse_expression(right_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", right_unit, e))
    })?;

    // Evaluate both units
    let left_eval = evaluate_owned(&left_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", left_unit, e))
    })?;

    let right_eval = evaluate_owned(&right_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", right_unit, e))
    })?;

    // Check if dimensions are compatible
    if left_eval.dim != right_eval.dim {
        return Ok(false);
    }

    // Convert both to canonical units and compare
    let left_canonical = to_f64(left_value * Decimal::try_from(to_f64(left_eval.factor)).unwrap_or_default());
    let right_canonical = to_f64(right_value * Decimal::try_from(to_f64(right_eval.factor)).unwrap_or_default());

    // Use a more lenient tolerance for equivalence to account for measurement precision
    // This allows for small differences that might occur in real-world measurements
    let tolerance = (left_canonical.abs().max(right_canonical.abs())) * 0.01; // 1% tolerance
    let min_tolerance = 1e-10; // Minimum absolute tolerance
    let effective_tolerance = tolerance.max(min_tolerance);

    Ok((left_canonical - right_canonical).abs() < effective_tolerance)
}

/// Convert and compare two UCUM quantities
fn convert_and_compare_ucum(
    left_value: Decimal,
    left_unit: &str,
    right_value: Decimal,
    right_unit: &str,
) -> Result<bool, QuantityError> {
    // Parse both units
    let left_expr = parse_expression(left_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", left_unit, e))
    })?;

    let right_expr = parse_expression(right_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", right_unit, e))
    })?;

    // Evaluate both units
    let left_eval = evaluate_owned(&left_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", left_unit, e))
    })?;

    let right_eval = evaluate_owned(&right_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", right_unit, e))
    })?;

    // Check if dimensions are compatible
    if left_eval.dim != right_eval.dim {
        return Ok(false);
    }

    // Convert both to canonical units and compare
    let left_canonical = to_f64(left_value * Decimal::try_from(to_f64(left_eval.factor)).unwrap_or_default());
    let right_canonical = to_f64(right_value * Decimal::try_from(to_f64(right_eval.factor)).unwrap_or_default());

    Ok((left_canonical - right_canonical).abs() < 1e-10)
}

/// Check if two UCUM units are comparable (same dimension)
pub fn are_ucum_units_comparable(left_unit: &str, right_unit: &str) -> Result<bool, QuantityError> {
    // If units are identical, they're comparable
    if left_unit == right_unit {
        return Ok(true);
    }

    // Parse both units
    let left_expr = parse_expression(left_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", left_unit, e))
    })?;

    let right_expr = parse_expression(right_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", right_unit, e))
    })?;

    // Evaluate both units
    let left_eval = evaluate_owned(&left_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", left_unit, e))
    })?;

    let right_eval = evaluate_owned(&right_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", right_unit, e))
    })?;

    // Units are comparable if they have the same dimension
    Ok(left_eval.dim == right_eval.dim)
}

/// Convert a UCUM quantity to a target unit
fn convert_ucum_quantity(
    value: Decimal,
    from_unit: &str,
    to_unit: &str,
) -> Result<ConversionResult, QuantityError> {
    // Parse both units
    let from_expr = parse_expression(from_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", from_unit, e))
    })?;

    let to_expr = parse_expression(to_unit).map_err(|e| {
        QuantityError::ParseError(format!("Failed to parse '{}': {}", to_unit, e))
    })?;

    // Evaluate both units
    let from_eval = evaluate_owned(&from_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", from_unit, e))
    })?;

    let to_eval = evaluate_owned(&to_expr).map_err(|e| {
        QuantityError::EvaluationError(format!("Failed to evaluate '{}': {}", to_unit, e))
    })?;

    // Check if dimensions are compatible
    if from_eval.dim != to_eval.dim {
        return Err(QuantityError::IncompatibleUnits(from_unit.to_string(), to_unit.to_string()));
    }

    // Calculate conversion factor
    let factor = to_f64(from_eval.factor) / to_f64(to_eval.factor);
    let converted_value = value * Decimal::try_from(factor).unwrap_or_default();

    // Try to find the target unit record
    let ucum_unit = octofhir_ucum::find_unit(to_unit).map(|u| Arc::new(u.clone()));

    Ok(ConversionResult {
        value: converted_value,
        unit: Some(to_unit.to_string()),
        ucum_unit,
        calendar_unit: CalendarUnit::from_str(to_unit),
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
    let (result_unit, result_ucum_unit, result_calendar_unit) = match (left_unit, right_unit) {
        (None, None) => (None, None::<Arc<octofhir_ucum::UnitRecord>>, None),
        (Some(l), None) => (Some(l.clone()), None, *left_calendar_unit),
        (None, Some(r)) => (Some(r.clone()), None, *right_calendar_unit),
        (Some(l), Some(r)) => {
            // For now, simple concatenation - TODO: implement proper UCUM unit multiplication
            let combined_unit = format!("{}.{}", l, r);
            (Some(combined_unit), None, None)
        }
    };

    Ok(FhirPathValue::quantity(result_value, result_unit))
}

/// Divide two quantities, handling unit combination
pub fn divide_quantities(
    left_value: Decimal,
    left_unit: &Option<String>,
    left_calendar_unit: &Option<CalendarUnit>,
    right_value: Decimal,
    right_unit: &Option<String>,
    right_calendar_unit: &Option<CalendarUnit>,
) -> Result<FhirPathValue, QuantityError> {
    if right_value == Decimal::ZERO {
        return Err(QuantityError::InvalidQuantity("Division by zero".to_string()));
    }

    let result_value = left_value / right_value;

    // Handle unit combination
    let (result_unit, result_ucum_unit, result_calendar_unit) = match (left_unit, right_unit) {
        (None, None) => (None, None::<Arc<octofhir_ucum::UnitRecord>>, None),
        (Some(l), None) => (Some(l.clone()), None, *left_calendar_unit),
        (None, Some(r)) => (Some(format!("1/{}", r)), None, None),
        (Some(l), Some(r)) => {
            // Handle special cases
            if l == r {
                // Same units cancel out to dimensionless
                (Some("1".to_string()), None, None)
            } else {
                // For now, simple concatenation - TODO: implement proper UCUM unit division
                let combined_unit = format!("{}/{}", l, r);
                (Some(combined_unit), None, None)
            }
        }
    };

    Ok(FhirPathValue::quantity(result_value, result_unit))
}

#[cfg(test)]
mod tests {
    use super::*;

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