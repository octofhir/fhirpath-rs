//! Literal value types for FHIRPath expressions
//!
//! This module defines all the literal value types that can appear directly
//! in FHIRPath expressions, with proper parsing and validation.

use std::fmt;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::NaiveDate;

use crate::core::{FhirPathError, FP0006, FP0001};
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};

/// Literal values that can appear directly in FHIRPath expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    /// String literal (e.g., 'hello', "world")
    String(String),
    
    /// Integer literal (e.g., 42, -17)
    Integer(i64),
    
    /// Decimal literal (e.g., 3.14, -0.5)
    Decimal(Decimal),
    
    /// Boolean literal (true, false)
    Boolean(bool),
    
    /// Date literal (e.g., @2023-12-25)
    Date(PrecisionDate),
    
    /// DateTime literal (e.g., @2023-12-25T10:30:00Z)
    DateTime(PrecisionDateTime),
    
    /// Time literal (e.g., @T10:30:00)
    Time(PrecisionTime),
    
    /// Quantity literal (e.g., 5 'mg', 10.5 'kg')
    Quantity {
        value: Decimal,
        unit: Option<String>,
    },
}

impl LiteralValue {
    /// Parse a string literal, handling escape sequences
    pub fn parse_string(input: &str) -> Result<Self, FhirPathError> {
        // Handle escape sequences in string literals
        let unescaped = input
            .replace("\\'", "'")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
            .replace("\\r", "\r")
            .replace("\\n", "\n")
            .replace("\\t", "\t");
        
        Ok(Self::String(unescaped))
    }

    /// Parse an integer literal
    pub fn parse_integer(input: &str) -> Result<Self, FhirPathError> {
        input.parse::<i64>()
            .map(Self::Integer)
            .map_err(|_| FhirPathError::parse_error(
                FP0006,
                format!("Invalid integer literal: {}", input),
                input.to_string(),
                None,
            ))
    }

    /// Parse a decimal literal
    pub fn parse_decimal(input: &str) -> Result<Self, FhirPathError> {
        input.parse::<Decimal>()
            .map(Self::Decimal)
            .map_err(|_| FhirPathError::parse_error(
                FP0006,
                format!("Invalid decimal literal: {}", input),
                input.to_string(),
                None,
            ))
    }

    /// Parse a date literal from @YYYY-MM-DD format
    pub fn parse_date(input: &str) -> Result<Self, FhirPathError> {
        if !input.starts_with('@') {
            return Err(FhirPathError::parse_error(
                FP0001,
                "Date literals must start with @",
                input.to_string(),
                None,
            ));
        }

        let date_str = &input[1..]; // Remove @ prefix
        
        if let Some(date) = PrecisionDate::parse(date_str) {
            Ok(Self::Date(date))
        } else {
            Err(FhirPathError::parse_error(
                FP0001,
                format!("Invalid date literal: {}", input),
                input.to_string(),
                None,
            ))
        }
    }

    /// Parse a datetime literal from @YYYY-MM-DDTHH:MM:SS format
    pub fn parse_datetime(input: &str) -> Result<Self, FhirPathError> {
        if !input.starts_with('@') {
            return Err(FhirPathError::parse_error(
                FP0001,
                "DateTime literals must start with @",
                input.to_string(),
                None,
            ));
        }

        let datetime_str = &input[1..]; // Remove @ prefix
        
        if let Some(datetime) = PrecisionDateTime::parse(datetime_str) {
            Ok(Self::DateTime(datetime))
        } else {
            Err(FhirPathError::parse_error(
                FP0001,
                format!("Invalid datetime literal: {}", input),
                input.to_string(),
                None,
            ))
        }
    }

    /// Parse a time literal from @Thh:mm:ss format
    pub fn parse_time(input: &str) -> Result<Self, FhirPathError> {
        if !input.starts_with("@T") {
            return Err(FhirPathError::parse_error(
                FP0001,
                "Time literals must start with @T",
                input.to_string(),
                None,
            ));
        }

        let time_str = &input[2..]; // Remove @T prefix
        
        if let Some(time) = PrecisionTime::parse(time_str) {
            Ok(Self::Time(time))
        } else {
            Err(FhirPathError::parse_error(
                FP0001,
                format!("Invalid time literal: {}", input),
                input.to_string(),
                None,
            ))
        }
    }

    /// Parse a quantity literal (number followed by unit in quotes)
    pub fn parse_quantity(value_str: &str, unit_str: Option<&str>) -> Result<Self, FhirPathError> {
        let value = value_str.parse::<Decimal>()
            .map_err(|_| FhirPathError::parse_error(
                FP0006,
                format!("Invalid quantity value: {}", value_str),
                value_str.to_string(),
                None,
            ))?;

        let unit = unit_str.map(|u| {
            // Remove quotes from unit if present
            let clean_unit = if (u.starts_with('\'') && u.ends_with('\'')) || (u.starts_with('"') && u.ends_with('"')) {
                &u[1..u.len()-1]
            } else {
                u
            };
            
            // Normalize common unit names to UCUM codes
            let normalized_unit = match clean_unit {
                "day" | "days" => "d",
                "week" | "weeks" => "wk", 
                "month" | "months" => "mo",
                "year" | "years" => "a",
                "hour" | "hours" => "h",
                "minute" | "minutes" => "min",
                "second" | "seconds" => "s",
                "millisecond" | "milliseconds" => "ms",
                "gram" | "grams" => "g",
                "kilogram" | "kilograms" => "kg",
                "meter" | "meters" | "metre" | "metres" => "m",
                "centimeter" | "centimeters" | "centimetre" | "centimetres" => "cm",
                "millimeter" | "millimeters" | "millimetre" | "millimetres" => "mm",
                "inch" | "inches" => "[in_i]",
                "foot" | "feet" => "[ft_i]",
                _ => clean_unit, // Keep original for valid UCUM codes
            };
            
            normalized_unit.to_string()
        });

        Ok(Self::Quantity { value, unit })
    }

    /// Get the type name of this literal
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Integer(_) => "Integer",
            Self::Decimal(_) => "Decimal",
            Self::Boolean(_) => "Boolean",
            Self::Date(_) => "Date",
            Self::DateTime(_) => "DateTime",
            Self::Time(_) => "Time",
            Self::Quantity { .. } => "Quantity",
        }
    }

    /// Convert this literal to a FhirPathValue
    pub fn to_fhir_path_value(&self) -> crate::core::FhirPathValue {
        match self {
            Self::String(s) => crate::core::FhirPathValue::string(s.clone()),
            Self::Integer(i) => crate::core::FhirPathValue::integer(*i),
            Self::Decimal(d) => crate::core::FhirPathValue::decimal(*d),
            Self::Boolean(b) => crate::core::FhirPathValue::boolean(*b),
            Self::Date(d) => crate::core::FhirPathValue::date(d.clone()),
            Self::DateTime(dt) => crate::core::FhirPathValue::datetime(dt.clone()),
            Self::Time(t) => crate::core::FhirPathValue::time(t.clone()),
            Self::Quantity { value, unit } => crate::core::FhirPathValue::quantity(*value, unit.clone()),
        }
    }
}

impl fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => {
                // Escape quotes and special characters for display
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\r', "\\r")
                    .replace('\n', "\\n")
                    .replace('\t', "\\t");
                write!(f, "'{}'", escaped)
            },
            Self::Integer(i) => write!(f, "{}", i),
            Self::Decimal(d) => write!(f, "{}", d),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Date(d) => write!(f, "@{}", d),
            Self::DateTime(dt) => write!(f, "@{}", dt),
            Self::Time(t) => write!(f, "@T{}", t),
            Self::Quantity { value, unit } => {
                if let Some(unit) = unit {
                    write!(f, "{} '{}'", value, unit)
                } else {
                    write!(f, "{}", value)
                }
            },
        }
    }
}

// Convenience constructors
impl LiteralValue {
    /// Create a string literal
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Create an integer literal
    pub fn integer(i: i64) -> Self {
        Self::Integer(i)
    }

    /// Create a decimal literal
    pub fn decimal(d: impl Into<Decimal>) -> Self {
        Self::Decimal(d.into())
    }

    /// Create a boolean literal
    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }

    /// Create a simple date literal (YYYY-MM-DD)
    pub fn date(year: i32, month: u32, day: u32) -> Result<Self, FhirPathError> {
        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| FhirPathError::parse_error(
                FP0001,
                format!("Invalid date: {}-{:02}-{:02}", year, month, day),
                format!("{}-{:02}-{:02}", year, month, day),
                None,
            ))?;
        
        Ok(Self::Date(PrecisionDate::from_date(naive_date)))
    }

    /// Create a quantity literal
    pub fn quantity(value: impl Into<Decimal>, unit: Option<impl Into<String>>) -> Self {
        Self::Quantity {
            value: value.into(),
            unit: unit.map(|u| u.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_parsing() {
        let literal = LiteralValue::parse_string("hello world").unwrap();
        assert_eq!(literal, LiteralValue::string("hello world"));
    }

    #[test]
    fn test_string_escaping() {
        let literal = LiteralValue::parse_string("hello\\nworld").unwrap();
        assert_eq!(literal, LiteralValue::string("hello\nworld"));
    }

    #[test]
    fn test_integer_parsing() {
        let literal = LiteralValue::parse_integer("42").unwrap();
        assert_eq!(literal, LiteralValue::integer(42));

        let literal = LiteralValue::parse_integer("-17").unwrap();
        assert_eq!(literal, LiteralValue::integer(-17));
    }

    #[test]
    fn test_decimal_parsing() {
        let literal = LiteralValue::parse_decimal("3.14").unwrap();
        assert_eq!(literal, LiteralValue::decimal(Decimal::new(314, 2)));
    }

    #[test]
    fn test_date_parsing() {
        let literal = LiteralValue::parse_date("@2023-12-25").unwrap();
        match literal {
            LiteralValue::Date(date) => {
                assert_eq!(date.precision, TemporalPrecision::Day);
            },
            _ => panic!("Expected date literal"),
        }
    }

    #[test]
    fn test_quantity_parsing() {
        let literal = LiteralValue::parse_quantity("5", Some("'mg'")).unwrap();
        match literal {
            LiteralValue::Quantity { value, unit } => {
                assert_eq!(value, Decimal::from(5));
                assert_eq!(unit, Some("mg".to_string()));
            },
            _ => panic!("Expected quantity literal"),
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(LiteralValue::string("test").to_string(), "'test'");
        assert_eq!(LiteralValue::integer(42).to_string(), "42");
        assert_eq!(LiteralValue::boolean(true).to_string(), "true");
    }
}