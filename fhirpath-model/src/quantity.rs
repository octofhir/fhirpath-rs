//! Quantity type implementation with UCUM support

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

use octofhir_ucum_core::{self, OwnedUnitExpr};

use crate::error::{ModelError, Result};

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
        let ucum_expr = unit.as_ref().and_then(|u| Self::parse_ucum_unit(u));
        Self {
            value,
            unit,
            ucum_expr,
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

    /// Add two quantities
    pub fn add(&self, other: &Quantity) -> Result<Quantity> {
        if !self.has_compatible_dimensions(other) {
            return Err(ModelError::incompatible_units(
                self.unit.as_deref().unwrap_or(""),
                other.unit.as_deref().unwrap_or(""),
            ));
        }

        Ok(Quantity::new(self.value + other.value, self.unit.clone()))
    }

    /// Subtract two quantities
    pub fn subtract(&self, other: &Quantity) -> Result<Quantity> {
        if !self.has_compatible_dimensions(other) {
            return Err(ModelError::incompatible_units(
                self.unit.as_deref().unwrap_or(""),
                other.unit.as_deref().unwrap_or(""),
            ));
        }

        Ok(Quantity::new(self.value - other.value, self.unit.clone()))
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
}
