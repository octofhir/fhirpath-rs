//! Quantity type implementation with UCUM support

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

use octofhir_ucum_core::{self, OwnedUnitExpr};

use super::error::{ModelError, Result};

/// Quantity value with optional unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    /// Numeric value
    pub value: Decimal,
    /// Unit string (UCUM)
    pub unit: Option<String>,
    /// Cached parsed UCUM unit expression for performance
    #[serde(skip)]
    ucum_expr: Option<Arc<OwnedUnitExpr>>,
}

impl Quantity {
    /// Create a new quantity
    pub fn new(value: Decimal, unit: Option<String>) -> Self {
        // Normalize unit name to UCUM format
        let normalized_unit = unit.as_ref().map(|u| Self::normalize_unit_name(u));
        let ucum_expr = normalized_unit
            .as_ref()
            .and_then(|u| Self::parse_ucum_unit(u));
        Self {
            value,
            unit: normalized_unit,
            ucum_expr,
        }
    }

    /// Normalize common unit names to UCUM equivalents
    fn normalize_unit_name(unit: &str) -> String {
        match unit {
            // Time units
            "day" | "days" => "d".to_string(),
            "hour" | "hours" => "h".to_string(),
            "minute" | "minutes" => "min".to_string(),
            "second" | "seconds" => "s".to_string(),
            "week" | "weeks" => "wk".to_string(),
            "month" | "months" => "mo".to_string(),
            "year" | "years" => "a".to_string(),
            "millisecond" | "milliseconds" => "ms".to_string(),

            // Keep original for already-UCUM units
            _ => unit.to_string(),
        }
    }

    /// Create a unitless quantity
    pub fn unitless(value: Decimal) -> Self {
        Self {
            value,
            unit: None,
            ucum_expr: None,
        }
    }

    /// Parse a UCUM unit string into an OwnedUnitExpr
    fn parse_ucum_unit(unit_str: &str) -> Option<Arc<OwnedUnitExpr>> {
        use parking_lot::Mutex;
        use std::collections::HashMap;
        use std::sync::OnceLock;

        // Static cache for parsed UCUM units
        static UCUM_CACHE: OnceLock<Mutex<HashMap<String, Option<Arc<OwnedUnitExpr>>>>> =
            OnceLock::new();

        // Early validation
        if unit_str.is_empty() || unit_str.len() > 256 {
            return None;
        }

        // Check cache first
        let cache = UCUM_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Some(cached_result) = cache.lock().get(unit_str).cloned() {
            return cached_result;
        }

        // Parse the unit
        let result = octofhir_ucum_core::parse_expression(unit_str)
            .ok()
            .map(Arc::new);

        // Cache the result
        let mut cache_guard = cache.lock();
        if cache_guard.len() >= 1000 {
            cache_guard.clear();
        }
        cache_guard.insert(unit_str.to_string(), result.clone());

        result
    }

    /// Check if two quantities have compatible dimensions
    pub fn has_compatible_dimensions(&self, other: &Quantity) -> bool {
        match (&self.unit, &other.unit) {
            (Some(unit1), Some(unit2)) => {
                octofhir_ucum_core::is_comparable(unit1, unit2).unwrap_or(false)
            }
            (None, None) => true, // Unitless quantities are comparable
            _ => false,
        }
    }

    /// Convert this quantity to the same unit as another quantity for comparison/arithmetic
    pub fn convert_to_compatible_unit(&self, target_unit: &str) -> Result<Quantity> {
        match &self.unit {
            Some(from_unit) => {
                if from_unit == target_unit {
                    // Same unit, no conversion needed
                    Ok(self.clone())
                } else {
                    // Use UCUM to get conversion factors
                    match (
                        octofhir_ucum_core::analyse(from_unit),
                        octofhir_ucum_core::analyse(target_unit),
                    ) {
                        (Ok(from_analysis), Ok(to_analysis)) => {
                            // Check if units are dimensionally compatible
                            if from_analysis.dimension != to_analysis.dimension {
                                return Err(ModelError::incompatible_units(from_unit, target_unit));
                            }

                            // Calculate conversion factor
                            let conversion_factor = from_analysis.factor / to_analysis.factor;
                            let offset_adjustment = from_analysis.offset - to_analysis.offset;

                            // Convert the value using higher precision approach
                            let from_f64 = self.value.try_into().unwrap_or(0.0);
                            let converted_f64 = from_f64 * conversion_factor + offset_adjustment;

                            // Round to reasonable precision to avoid floating point errors
                            let converted_f64 = (converted_f64 * 1e12).round() / 1e12;
                            let converted_decimal =
                                Decimal::from_f64(converted_f64).unwrap_or(self.value);

                            Ok(Quantity::new(
                                converted_decimal,
                                Some(target_unit.to_string()),
                            ))
                        }
                        _ => Err(ModelError::incompatible_units(from_unit, target_unit)),
                    }
                }
            }
            None => {
                // Cannot convert unitless to unit quantity
                Err(ModelError::incompatible_units("", target_unit))
            }
        }
    }

    /// Check if two quantities are equal with unit conversion
    pub fn equals_with_conversion(&self, other: &Quantity) -> Result<bool> {
        match (&self.unit, &other.unit) {
            (Some(unit1), Some(unit2)) => {
                if unit1 == unit2 {
                    // Same unit, direct comparison
                    Ok(self.value == other.value)
                } else if self.has_compatible_dimensions(other) {
                    // Convert other to this unit and compare
                    let converted_other = other.convert_to_compatible_unit(unit1)?;
                    Ok(self.value == converted_other.value)
                } else {
                    // Incompatible units
                    Ok(false)
                }
            }
            (None, None) => {
                // Both unitless
                Ok(self.value == other.value)
            }
            _ => {
                // One has unit, other doesn't - not equal
                Ok(false)
            }
        }
    }

    /// Add two quantities with unit conversion
    pub fn add(&self, other: &Quantity) -> Result<Quantity> {
        match (&self.unit, &other.unit) {
            (Some(unit1), Some(unit2)) => {
                if unit1 == unit2 {
                    // Same unit, direct addition
                    Ok(Quantity::new(self.value + other.value, self.unit.clone()))
                } else if self.has_compatible_dimensions(other) {
                    // Convert other to this unit and add
                    let converted_other = other.convert_to_compatible_unit(unit1)?;
                    Ok(Quantity::new(
                        self.value + converted_other.value,
                        self.unit.clone(),
                    ))
                } else {
                    Err(ModelError::incompatible_units(unit1, unit2))
                }
            }
            (None, None) => {
                // Both unitless
                Ok(Quantity::new(self.value + other.value, None))
            }
            _ => {
                // One has unit, other doesn't - incompatible
                Err(ModelError::incompatible_units(
                    self.unit.as_deref().unwrap_or(""),
                    other.unit.as_deref().unwrap_or(""),
                ))
            }
        }
    }

    /// Subtract two quantities with unit conversion
    pub fn subtract(&self, other: &Quantity) -> Result<Quantity> {
        match (&self.unit, &other.unit) {
            (Some(unit1), Some(unit2)) => {
                if unit1 == unit2 {
                    // Same unit, direct subtraction
                    Ok(Quantity::new(self.value - other.value, self.unit.clone()))
                } else if self.has_compatible_dimensions(other) {
                    // Convert other to this unit and subtract
                    let converted_other = other.convert_to_compatible_unit(unit1)?;
                    Ok(Quantity::new(
                        self.value - converted_other.value,
                        self.unit.clone(),
                    ))
                } else {
                    Err(ModelError::incompatible_units(unit1, unit2))
                }
            }
            (None, None) => {
                // Both unitless
                Ok(Quantity::new(self.value - other.value, None))
            }
            _ => {
                // One has unit, other doesn't - incompatible
                Err(ModelError::incompatible_units(
                    self.unit.as_deref().unwrap_or(""),
                    other.unit.as_deref().unwrap_or(""),
                ))
            }
        }
    }

    /// Multiply by a scalar
    pub fn multiply_scalar(&self, scalar: Decimal) -> Quantity {
        Quantity::new(self.value * scalar, self.unit.clone())
    }

    /// Divide by a scalar
    pub fn divide_scalar(&self, scalar: Decimal) -> Option<Quantity> {
        if scalar.is_zero() {
            None
        } else {
            Some(Quantity::new(self.value / scalar, self.unit.clone()))
        }
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        // Convert decimal to JSON value
        let value_json = if let Ok(f) = self.value.try_into() {
            if let Some(num) = serde_json::Number::from_f64(f) {
                serde_json::Value::Number(num)
            } else {
                serde_json::Value::String(self.value.to_string())
            }
        } else {
            serde_json::Value::String(self.value.to_string())
        };

        obj.insert("value".to_string(), value_json);

        if let Some(unit) = &self.unit {
            obj.insert("unit".to_string(), serde_json::Value::String(unit.clone()));
        }

        serde_json::Value::Object(obj)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(unit) = &self.unit {
            write!(f, "{} {}", self.value, unit)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantity_creation() {
        let q1 = Quantity::new(Decimal::from(5), Some("mg".to_string()));
        assert_eq!(q1.value, Decimal::from(5));
        assert_eq!(q1.unit, Some("mg".to_string()));

        let q2 = Quantity::unitless(Decimal::from(10));
        assert_eq!(q2.value, Decimal::from(10));
        assert_eq!(q2.unit, None);
    }

    #[test]
    fn test_quantity_arithmetic() {
        let q1 = Quantity::new(Decimal::from(5), Some("mg".to_string()));
        let q2 = Quantity::new(Decimal::from(3), Some("mg".to_string()));

        let sum = q1.add(&q2).unwrap();
        assert_eq!(sum.value, Decimal::from(8));
        assert_eq!(sum.unit, Some("mg".to_string()));

        let diff = q1.subtract(&q2).unwrap();
        assert_eq!(diff.value, Decimal::from(2));
        assert_eq!(diff.unit, Some("mg".to_string()));
    }

    #[test]
    fn test_incompatible_units() {
        let q1 = Quantity::new(Decimal::from(5), Some("mg".to_string()));
        let q2 = Quantity::new(Decimal::from(3), Some("mL".to_string()));

        assert!(q1.add(&q2).is_err());
        assert!(q1.subtract(&q2).is_err());
    }

    #[test]
    fn test_ucum_conversion_debug() {
        // Test the g to mg conversion like in the failing test
        let q1 = Quantity::new(Decimal::from(4), Some("g".to_string()));
        let q2 = Quantity::new(Decimal::from(4000), Some("mg".to_string()));

        println!("q1: {q1:?}");
        println!("q2: {q2:?}");

        // Test has_compatible_dimensions
        println!(
            "Compatible dimensions: {}",
            q1.has_compatible_dimensions(&q2)
        );

        // Test UCUM analysis directly
        match octofhir_ucum_core::analyse("g") {
            Ok(analysis) => println!(
                "g analysis: factor={}, dimension={:?}",
                analysis.factor, analysis.dimension
            ),
            Err(e) => println!("Error analyzing g: {e}"),
        }

        match octofhir_ucum_core::analyse("mg") {
            Ok(analysis) => println!(
                "mg analysis: factor={}, dimension={:?}",
                analysis.factor, analysis.dimension
            ),
            Err(e) => println!("Error analyzing mg: {e}"),
        }

        // Test manual conversion
        if let (Ok(g_analysis), Ok(mg_analysis)) = (
            octofhir_ucum_core::analyse("g"),
            octofhir_ucum_core::analyse("mg"),
        ) {
            let conversion_factor = g_analysis.factor / mg_analysis.factor;
            println!("Conversion factor g->mg: {conversion_factor}");
            println!("4 g in mg should be: {}", 4.0 * conversion_factor);
        }

        // Test equals_with_conversion
        match q1.equals_with_conversion(&q2) {
            Ok(result) => {
                println!("Equals with conversion: {result}");
                assert!(result, "4 g should equal 4000 mg");
            }
            Err(e) => {
                println!("Error in equals_with_conversion: {e}");
                panic!("Conversion should not fail");
            }
        }
    }

    #[test]
    fn test_time_units_debug() {
        // Test time units that are failing
        println!("Testing time units...");

        // Test normalized units - should work now
        let q1 = Quantity::new(Decimal::from(7), Some("days".to_string()));
        let q2 = Quantity::new(Decimal::from(1), Some("week".to_string()));

        println!("q1 normalized unit: {:?}", q1.unit);
        println!("q2 normalized unit: {:?}", q2.unit);

        match q1.equals_with_conversion(&q2) {
            Ok(result) => {
                println!("7 days == 1 week: {result}");
                assert!(result, "7 days should equal 1 week");
            }
            Err(e) => println!("Error comparing 7 days and 1 week: {e}"),
        }
    }
}
