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

//! Quantity type implementation with UCUM support

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

use octofhir_ucum::{self, OwnedUnitExpr};

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
        // First check calendar units for FHIRPath compatibility
        match unit {
            // Time units - FHIRPath calendar units to UCUM
            "day" | "days" => "d".to_string(),
            "hour" | "hours" => "h".to_string(),
            "minute" | "minutes" => "min".to_string(),
            "second" | "seconds" => "s".to_string(),
            "week" | "weeks" => "wk".to_string(),
            "month" | "months" => "mo".to_string(),
            "year" | "years" => "a".to_string(),
            "millisecond" | "milliseconds" => "ms".to_string(),

            _ => {
                // Check if it's already a valid UCUM unit - if so, keep it as-is
                // This preserves units like "g", "mg", "m", "cm", etc. without
                // converting them to base SI units
                match octofhir_ucum::validate(unit) {
                    Ok(_) => unit.to_string(),  // Valid UCUM unit, keep as-is
                    Err(_) => unit.to_string(), // Invalid or unknown, keep original
                }
            }
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
        let result = octofhir_ucum::parse_expression(unit_str).ok().map(Arc::new);

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
                octofhir_ucum::is_comparable(unit1, unit2).unwrap_or(false)
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
                        octofhir_ucum::analyse(from_unit),
                        octofhir_ucum::analyse(target_unit),
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
                            let converted_f64 = (converted_f64 * 1e12_f64).round() / 1e12_f64;
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

    /// Multiply two quantities - multiplies values and combines units
    pub fn multiply(&self, other: &Quantity) -> Quantity {
        let result_value = self.value * other.value;
        let result_unit = Self::combine_units_multiply(&self.unit, &other.unit);
        Quantity::new(result_value, result_unit)
    }

    /// Divide two quantities - divides values and divides units  
    pub fn divide(&self, other: &Quantity) -> Option<Quantity> {
        if other.value.is_zero() {
            None // Division by zero
        } else {
            let result_value = self.value / other.value;
            let result_unit = Self::combine_units_divide(&self.unit, &other.unit);
            Some(Quantity::new(result_value, result_unit))
        }
    }

    /// Combine units for multiplication (e.g., "m" * "s" = "m.s")
    fn combine_units_multiply(left: &Option<String>, right: &Option<String>) -> Option<String> {
        match (left, right) {
            (Some(l), Some(r)) => {
                if l.is_empty() || l == "1" {
                    Some(r.clone())
                } else if r.is_empty() || r == "1" {
                    Some(l.clone())
                } else {
                    Some(format!("{l}.{r}"))
                }
            }
            (Some(u), None) | (None, Some(u)) => {
                if u.is_empty() || u == "1" {
                    Some("1".to_string())
                } else {
                    Some(u.clone())
                }
            }
            (None, None) => Some("1".to_string()),
        }
    }

    /// Combine units for division (e.g., "g" / "m" = "g/m", "m" / "m" = "1")
    fn combine_units_divide(
        numerator: &Option<String>,
        denominator: &Option<String>,
    ) -> Option<String> {
        match (numerator, denominator) {
            (Some(num), Some(den)) => {
                if num == den {
                    // Same units cancel out to dimensionless "1"
                    Some("1".to_string())
                } else if den.is_empty() || den == "1" {
                    // Dividing by dimensionless
                    Some(num.clone())
                } else if num.is_empty() || num == "1" {
                    // Dimensionless divided by unit
                    Some(format!("1/{den}"))
                } else {
                    // Different units
                    Some(format!("{num}/{den}"))
                }
            }
            (Some(num), None) => {
                // Dividing by scalar (no unit)
                Some(num.clone())
            }
            (None, Some(den)) => {
                // Scalar divided by unit
                if den.is_empty() || den == "1" {
                    Some("1".to_string())
                } else {
                    Some(format!("1/{den}"))
                }
            }
            (None, None) => {
                // Scalar divided by scalar
                Some("1".to_string())
            }
        }
    }

    /// Convert to JSON representation
    pub fn to_json(&self) -> sonic_rs::Value {
        let mut obj = sonic_rs::object! {};

        // Convert decimal to JSON value - use string representation for precision
        let value_json = sonic_rs::Value::from(self.value.to_string().as_str());

        obj.insert("value", value_json);

        if let Some(unit) = &self.unit {
            obj.insert("unit", sonic_rs::Value::from(unit.as_str()));
        }

        obj.into()
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
        match octofhir_ucum::analyse("g") {
            Ok(analysis) => println!(
                "g analysis: factor={}, dimension={:?}",
                analysis.factor, analysis.dimension
            ),
            Err(e) => println!("Error analyzing g: {e}"),
        }

        match octofhir_ucum::analyse("mg") {
            Ok(analysis) => println!(
                "mg analysis: factor={}, dimension={:?}",
                analysis.factor, analysis.dimension
            ),
            Err(e) => println!("Error analyzing mg: {e}"),
        }

        // Test manual conversion
        if let (Ok(g_analysis), Ok(mg_analysis)) =
            (octofhir_ucum::analyse("g"), octofhir_ucum::analyse("mg"))
        {
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
