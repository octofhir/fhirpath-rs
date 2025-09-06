//! Conversion validation and safety utilities for FHIRPath
//!
//! This module provides utilities for checking if values can be safely converted
//! between different types, and for performing safe conversions with detailed
//! error reporting.

use crate::core::error_code::FP0058;
use crate::core::{FhirPathError, FhirPathValue, Result};
use rust_decimal::Decimal;

/// Utilities for safe type conversions
pub struct ConversionUtils;

impl ConversionUtils {
    /// Check if a string can be safely converted to integer
    pub fn can_convert_to_integer(s: &str) -> bool {
        s.trim().parse::<i64>().is_ok()
    }

    /// Check if a string can be safely converted to decimal
    pub fn can_convert_to_decimal(s: &str) -> bool {
        s.trim().parse::<Decimal>().is_ok()
    }

    /// Check if a string can be safely converted to boolean
    pub fn can_convert_to_boolean(s: &str) -> bool {
        matches!(
            s.to_lowercase().trim(),
            "true" | "t" | "yes" | "y" | "1" | "false" | "f" | "no" | "n" | "0"
        )
    }

    /// Check if integer conversion would overflow
    pub fn would_integer_overflow(d: &Decimal) -> bool {
        let truncated = d.trunc();
        truncated.to_string().parse::<i64>().is_err()
    }

    /// Safe string to integer conversion with validation
    pub fn safe_string_to_integer(s: &str) -> Result<i64> {
        let trimmed = s.trim();

        // Check for obvious invalid formats
        if trimmed.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0058,
                "Cannot convert empty string to integer",
            ));
        }

        if trimmed.contains('.') {
            return Err(FhirPathError::evaluation_error(
                FP0058,
                "Cannot convert decimal string to integer without explicit conversion",
            ));
        }

        trimmed.parse::<i64>().map_err(|_| {
            FhirPathError::evaluation_error(FP0058, &format!("Cannot convert '{}' to integer", s))
        })
    }

    /// Safe decimal conversion with precision validation
    pub fn safe_string_to_decimal(s: &str) -> Result<Decimal> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0058,
                "Cannot convert empty string to decimal",
            ));
        }

        trimmed.parse::<Decimal>().map_err(|_| {
            FhirPathError::evaluation_error(FP0058, &format!("Cannot convert '{}' to decimal", s))
        })
    }

    /// Format conversion error with helpful context
    pub fn format_conversion_error(
        from_type: &str,
        to_type: &str,
        value: &str,
        reason: Option<&str>,
    ) -> String {
        match reason {
            Some(r) => format!(
                "Cannot convert {} '{}' to {}: {}",
                from_type, value, to_type, r
            ),
            None => format!("Cannot convert {} '{}' to {}", from_type, value, to_type),
        }
    }

    /// Check if a value can be converted to the target type
    pub fn can_convert_to_type(value: &FhirPathValue, target_type: &str) -> bool {
        match (value, target_type) {
            (FhirPathValue::String(s), "Integer") => Self::can_convert_to_integer(s),
            (FhirPathValue::String(s), "Decimal") => Self::can_convert_to_decimal(s),
            (FhirPathValue::String(s), "Boolean") => Self::can_convert_to_boolean(s),
            (FhirPathValue::Integer(_), "Decimal") => true,
            (FhirPathValue::Integer(_), "String") => true,
            (FhirPathValue::Integer(_), "Boolean") => true,
            (FhirPathValue::Decimal(d), "Integer") => !Self::would_integer_overflow(d),
            (FhirPathValue::Decimal(_), "String") => true,
            (FhirPathValue::Boolean(_), "String") => true,
            (FhirPathValue::Boolean(_), "Integer") => true,
            (FhirPathValue::Boolean(_), "Decimal") => true,
            (FhirPathValue::Date(_), "String") => true,
            (FhirPathValue::DateTime(_), "String") => true,
            (FhirPathValue::Time(_), "String") => true,
            (FhirPathValue::DateTime(_), "Date") => true,
            (FhirPathValue::DateTime(_), "Time") => true,
            (FhirPathValue::Date(_), "DateTime") => true,
            (FhirPathValue::String(_), "Date") => true, // Might fail at runtime
            (FhirPathValue::String(_), "DateTime") => true, // Might fail at runtime
            (FhirPathValue::String(_), "Time") => true, // Might fail at runtime
            (FhirPathValue::String(_), "Quantity") => true, // Might fail at runtime
            (FhirPathValue::Integer(_), "Quantity") => true,
            (FhirPathValue::Decimal(_), "Quantity") => true,
            (FhirPathValue::Quantity { .. }, "String") => true,
            (FhirPathValue::Quantity { .. }, "Quantity") => true,
            (_, "String") => true, // Most values can be converted to string
            _ => false,
        }
    }

    /// Get the FHIRPath type name for a value
    pub fn get_value_type_name(value: &FhirPathValue) -> &'static str {
        value.type_name()
    }

    /// Validate conversion compatibility between two types
    pub fn validate_conversion(from_type: &str, to_type: &str) -> bool {
        match (from_type, to_type) {
            // Numeric conversions
            ("Integer", "Decimal") => true,
            ("Decimal", "Integer") => true, // May truncate
            ("Boolean", "Integer") => true,
            ("Boolean", "Decimal") => true,
            ("Integer", "Boolean") => true,
            ("Decimal", "Boolean") => true,

            // String conversions (may fail at runtime)
            ("String", "Integer") => true,
            ("String", "Decimal") => true,
            ("String", "Boolean") => true,
            ("String", "Date") => true,
            ("String", "DateTime") => true,
            ("String", "Time") => true,
            ("String", "Quantity") => true,

            // Temporal conversions
            ("DateTime", "Date") => true,
            ("DateTime", "Time") => true,
            ("Date", "DateTime") => true,

            // Quantity conversions
            ("Integer", "Quantity") => true,
            ("Decimal", "Quantity") => true,
            ("Quantity", "String") => true,
            // Same type conversions
            (from, to) if from == to => true,
            // Any type can convert to string
            (_, "String") => true,

            _ => false,
        }
    }

    /// Check if conversion might lose information
    pub fn conversion_loses_information(from_type: &str, to_type: &str) -> bool {
        matches!(
            (from_type, to_type),
            ("Decimal", "Integer") |  // Truncation
            ("DateTime", "Date") |    // Lose time information
            ("DateTime", "Time") |    // Lose date information
            ("Quantity", "Integer") | // Lose unit information
            ("Quantity", "Decimal") // Lose unit information
        )
    }

    /// Get a human-readable description of what information might be lost
    pub fn describe_information_loss(from_type: &str, to_type: &str) -> Option<&'static str> {
        match (from_type, to_type) {
            ("Decimal", "Integer") => Some("Decimal places will be truncated"),
            ("DateTime", "Date") => Some("Time information will be lost"),
            ("DateTime", "Time") => Some("Date information will be lost"),
            ("Quantity", "Integer") | ("Quantity", "Decimal") => {
                Some("Unit information will be lost")
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_convert_to_integer() {
        assert!(ConversionUtils::can_convert_to_integer("123"));
        assert!(ConversionUtils::can_convert_to_integer("-456"));
        assert!(ConversionUtils::can_convert_to_integer("0"));
        assert!(!ConversionUtils::can_convert_to_integer("12.3"));
        assert!(!ConversionUtils::can_convert_to_integer("not_a_number"));
        assert!(!ConversionUtils::can_convert_to_integer(""));
    }

    #[test]
    fn test_can_convert_to_boolean() {
        assert!(ConversionUtils::can_convert_to_boolean("true"));
        assert!(ConversionUtils::can_convert_to_boolean("FALSE"));
        assert!(ConversionUtils::can_convert_to_boolean("1"));
        assert!(ConversionUtils::can_convert_to_boolean("0"));
        assert!(ConversionUtils::can_convert_to_boolean("yes"));
        assert!(!ConversionUtils::can_convert_to_boolean("maybe"));
        assert!(!ConversionUtils::can_convert_to_boolean("2"));
    }

    #[test]
    fn test_safe_string_to_integer() {
        assert_eq!(ConversionUtils::safe_string_to_integer("123").unwrap(), 123);
        assert_eq!(
            ConversionUtils::safe_string_to_integer("-456").unwrap(),
            -456
        );

        assert!(ConversionUtils::safe_string_to_integer("12.3").is_err());
        assert!(ConversionUtils::safe_string_to_integer("").is_err());
        assert!(ConversionUtils::safe_string_to_integer("not_a_number").is_err());
    }

    #[test]
    fn test_conversion_compatibility() {
        assert!(ConversionUtils::validate_conversion("Integer", "String"));
        assert!(ConversionUtils::validate_conversion("Integer", "Decimal"));
        assert!(ConversionUtils::validate_conversion("String", "Integer"));
        assert!(ConversionUtils::validate_conversion("DateTime", "Date"));
        assert!(!ConversionUtils::validate_conversion("String", "Resource"));
    }

    #[test]
    fn test_information_loss() {
        assert!(ConversionUtils::conversion_loses_information(
            "Decimal", "Integer"
        ));
        assert!(ConversionUtils::conversion_loses_information(
            "DateTime", "Date"
        ));
        assert!(!ConversionUtils::conversion_loses_information(
            "Integer", "Decimal"
        ));
        assert!(!ConversionUtils::conversion_loses_information(
            "String", "String"
        ));
    }

    #[test]
    fn test_describe_information_loss() {
        assert!(ConversionUtils::describe_information_loss("Decimal", "Integer").is_some());
        assert!(ConversionUtils::describe_information_loss("DateTime", "Date").is_some());
        assert!(ConversionUtils::describe_information_loss("Integer", "Decimal").is_none());
    }

    #[test]
    fn test_integration_fhir_data_conversions() {
        // Test with realistic FHIR data patterns

        // Test Patient age conversion (often comes as string in FHIR)
        let patient_age = FhirPathValue::string("65".to_string());
        assert!(ConversionUtils::can_convert_to_type(
            &patient_age,
            "Integer"
        ));
        assert_eq!(ConversionUtils::safe_string_to_integer("65").unwrap(), 65);

        // Test Observation value conversion
        let observation_value = FhirPathValue::string("98.6".to_string());
        assert!(ConversionUtils::can_convert_to_type(
            &observation_value,
            "Decimal"
        ));
        assert_eq!(
            ConversionUtils::safe_string_to_decimal("98.6").unwrap(),
            Decimal::from_str("98.6").unwrap()
        );

        // Test boolean flags (active/inactive status)
        let patient_active = FhirPathValue::string("true".to_string());
        assert!(ConversionUtils::can_convert_to_type(
            &patient_active,
            "Boolean"
        ));

        // Test quantity conversion (weight, height, etc.)
        let weight_value = FhirPathValue::decimal(Decimal::from_str("70.5").unwrap());
        assert!(ConversionUtils::can_convert_to_type(
            &weight_value,
            "String"
        ));
        assert!(ConversionUtils::can_convert_to_type(
            &weight_value,
            "Quantity"
        ));

        // Test temporal data (birthDate, dates)
        let birth_date = FhirPathValue::string("1958-03-15".to_string());
        assert!(ConversionUtils::can_convert_to_type(&birth_date, "Date"));

        // Test diagnostic code conversions (often need string representation)
        let diagnostic_code = FhirPathValue::string("I10".to_string());
        assert!(ConversionUtils::can_convert_to_type(
            &diagnostic_code,
            "String"
        ));
    }

    #[test]
    fn test_fhir_conversion_warnings() {
        // Test scenarios where FHIR data conversion might lose information

        // DateTime to Date (lose time information)
        assert!(ConversionUtils::conversion_loses_information(
            "DateTime", "Date"
        ));
        assert_eq!(
            ConversionUtils::describe_information_loss("DateTime", "Date"),
            Some("Time information will be lost")
        );

        // Decimal to Integer (lose precision) - common for measurements
        assert!(ConversionUtils::conversion_loses_information(
            "Decimal", "Integer"
        ));
        assert_eq!(
            ConversionUtils::describe_information_loss("Decimal", "Integer"),
            Some("Decimal places will be truncated")
        );

        // Quantity to numeric types (lose units) - critical for medical data
        assert!(ConversionUtils::conversion_loses_information(
            "Quantity", "Decimal"
        ));
        assert_eq!(
            ConversionUtils::describe_information_loss("Quantity", "Decimal"),
            Some("Unit information will be lost")
        );
    }

    #[test]
    fn test_medical_data_edge_cases() {
        // Test edge cases common in medical data

        // Very precise measurements (lab results)
        let lab_result = "0.0001234";
        assert!(ConversionUtils::can_convert_to_decimal(lab_result));

        // Age ranges and special values
        assert!(ConversionUtils::can_convert_to_integer("0")); // newborn
        assert!(ConversionUtils::can_convert_to_integer("120")); // very elderly
        assert!(!ConversionUtils::can_convert_to_integer("unknown")); // missing data

        // Medical boolean flags with various formats
        assert!(ConversionUtils::can_convert_to_boolean("yes"));
        assert!(ConversionUtils::can_convert_to_boolean("no"));
        assert!(ConversionUtils::can_convert_to_boolean("Y"));
        assert!(ConversionUtils::can_convert_to_boolean("N"));

        // Error handling for malformed medical data
        let malformed_result = ConversionUtils::safe_string_to_integer("N/A");
        assert!(malformed_result.is_err());

        let error_message = ConversionUtils::format_conversion_error(
            "String",
            "Integer",
            "N/A",
            Some("Invalid medical data format"),
        );
        assert!(error_message.contains("Cannot convert String 'N/A' to Integer"));
        assert!(error_message.contains("Invalid medical data format"));
    }
}
