//! Conversion functions for FHIRPath expressions
//!
//! This module implements type conversion functions like toString(), toInteger(),
//! toDecimal(), toBoolean(), toDate(), toDateTime(), toTime(), and toQuantity().

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::core::error_code::{FP0053, FP0058};
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
use crate::register_function;
use rust_decimal::Decimal;
use std::str::FromStr;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc, TimeZone};

impl FunctionRegistry {
    pub fn register_conversion_functions(&self) -> Result<()> {
        self.register_toString_function()?;
        self.register_toInteger_function()?;
        self.register_toDecimal_function()?;
        self.register_toBoolean_function()?;
        self.register_toDate_function()?;
        self.register_toDateTime_function()?;
        self.register_toTime_function()?;
        self.register_toQuantity_function()?;
        Ok(())
    }

    fn register_toString_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toString",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a string representation",
            parameters: [],
            return_type: "String",
            examples: [
                "123.toString()",
                "true.toString()",
                "Patient.birthDate.toString()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toString() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::String(s) => s.clone(),
                    FhirPathValue::Integer(i) => i.to_string(),
                    FhirPathValue::Decimal(d) => d.to_string(),
                    FhirPathValue::Boolean(b) => b.to_string(),
                    FhirPathValue::Date(d) => d.to_string(),
                    FhirPathValue::DateTime(dt) => dt.to_string(),
                    FhirPathValue::Time(t) => t.to_string(),
                    FhirPathValue::Quantity { value, unit, .. } => {
                        match unit {
                            Some(u) if !u.is_empty() => format!("{} {}", value, u),
                            _ => value.to_string(),
                        }
                    }
                    FhirPathValue::Uri(uri) => uri.clone(),
                    FhirPathValue::Url(url) => url.clone(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to string"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_toInteger_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toInteger",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to an integer",
            parameters: [],
            return_type: "Integer",
            examples: [
                "'123'.toInteger()",
                "123.45.toInteger()",
                "true.toInteger()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toInteger() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::Integer(i) => *i,
                    FhirPathValue::Decimal(d) => {
                        // Truncate decimal to integer
                        let truncated = d.trunc();
                        truncated.to_string().parse::<i64>()
                            .map_err(|_| FhirPathError::evaluation_error(
                                FP0058,
                                "Decimal value too large for integer conversion"
                            ))?
                    }
                    FhirPathValue::String(s) => {
                        s.trim().parse::<i64>()
                            .map_err(|_| FhirPathError::evaluation_error(
                                FP0058,
                                &format!("Cannot convert '{}' to integer", s)
                            ))?
                    }
                    FhirPathValue::Boolean(b) => if *b { 1 } else { 0 },
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to integer"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::integer(result)])
            }
        )
    }

    fn register_toDecimal_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toDecimal",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a decimal",
            parameters: [],
            return_type: "Decimal",
            examples: [
                "'123.45'.toDecimal()",
                "123.toDecimal()",
                "'3.14159'.toDecimal()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toDecimal() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::Decimal(d) => *d,
                    FhirPathValue::Integer(i) => Decimal::from(*i),
                    FhirPathValue::String(s) => {
                        s.trim().parse::<Decimal>()
                            .map_err(|_| FhirPathError::evaluation_error(
                                FP0058,
                                &format!("Cannot convert '{}' to decimal", s)
                            ))?
                    }
                    FhirPathValue::Boolean(b) => Decimal::from(if *b { 1 } else { 0 }),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to decimal"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::decimal(result)])
            }
        )
    }

    fn register_toBoolean_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toBoolean",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a boolean",
            parameters: [],
            return_type: "Boolean",
            examples: [
                "'true'.toBoolean()",
                "1.toBoolean()",
                "'false'.toBoolean()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toBoolean() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::Boolean(b) => *b,
                    FhirPathValue::String(s) => {
                        match s.to_lowercase().trim() {
                            "true" | "t" | "yes" | "y" | "1" => true,
                            "false" | "f" | "no" | "n" | "0" => false,
                            _ => {
                                return Err(FhirPathError::evaluation_error(
                                    FP0058,
                                    &format!("Cannot convert '{}' to boolean", s)
                                ));
                            }
                        }
                    }
                    FhirPathValue::Integer(i) => *i != 0,
                    FhirPathValue::Decimal(d) => !d.is_zero(),
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to boolean"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::boolean(result)])
            }
        )
    }

    fn register_toDate_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toDate",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a date",
            parameters: [],
            return_type: "Date",
            examples: [
                "'2023-12-25'.toDate()",
                "'2023-12-25T10:30:00Z'.toDate()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toDate() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::Date(d) => d.clone(),
                    FhirPathValue::DateTime(dt) => {
                        // Extract date part from datetime
                        PrecisionDate::from_date(dt.datetime.naive_utc().date())
                    }
                    FhirPathValue::String(s) => {
                        Self::parse_date_string(s)?
                    }
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "toDate() can only convert strings and datetimes"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::Date(result)])
            }
        )
    }

    fn register_toDateTime_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toDateTime",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a datetime",
            parameters: [],
            return_type: "DateTime",
            examples: [
                "'2023-12-25T10:30:00Z'.toDateTime()",
                "'2023-12-25'.toDateTime()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toDateTime() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::DateTime(dt) => dt.clone(),
                    FhirPathValue::Date(d) => {
                        // Convert date to datetime at midnight UTC
                        let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
                            d.date.and_hms_opt(0, 0, 0).unwrap(),
                            Utc
                        );
                        PrecisionDateTime::new(
                            DateTime::from_naive_utc_and_offset(dt.naive_utc(), FixedOffset::east_opt(0).unwrap()),
                            TemporalPrecision::Day
                        )
                    }
                    FhirPathValue::String(s) => {
                        Self::parse_datetime_string(s)?
                    }
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "toDateTime() can only convert strings and dates"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::DateTime(result)])
            }
        )
    }

    fn register_toTime_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toTime",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a time",
            parameters: [],
            return_type: "Time",
            examples: [
                "'10:30:00'.toTime()",
                "'2023-12-25T10:30:00Z'.toTime()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toTime() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::Time(t) => t.clone(),
                    FhirPathValue::DateTime(dt) => {
                        // Extract time part from datetime
                        PrecisionTime::new(
                            dt.datetime.naive_local().time(),
                            TemporalPrecision::Second
                        )
                    }
                    FhirPathValue::String(s) => {
                        Self::parse_time_string(s)?
                    }
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "toTime() can only convert strings and datetimes"
                        ));
                    }
                };

                Ok(vec![FhirPathValue::Time(result)])
            }
        )
    }

    // Helper functions for parsing
    fn parse_date_string(input: &str) -> Result<PrecisionDate> {
        let trimmed = input.trim();
        
        // Try to parse as ISO date
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            return Ok(PrecisionDate::from_date(date));
        }
        
        // Try year-month format
        if let Ok(date) = NaiveDate::parse_from_str(&format!("{}-01", trimmed), "%Y-%m-%d") {
            let mut precision_date = PrecisionDate::from_date(date);
            precision_date.precision = TemporalPrecision::Month;
            return Ok(precision_date);
        }
        
        // Try year format
        if let Ok(year) = trimmed.parse::<i32>() {
            if let Some(precision_date) = PrecisionDate::from_year(year) {
                return Ok(precision_date);
            }
        }

        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a date", input)
        ))
    }

    fn parse_datetime_string(input: &str) -> Result<PrecisionDateTime> {
        let trimmed = input.trim();
        
        // Try RFC3339/ISO 8601 format
        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return Ok(PrecisionDateTime::new(dt, TemporalPrecision::Second));
        }
        
        // Try various ISO formats
        let formats = [
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S%.3fZ",
            "%Y-%m-%dT%H:%M:%S%z",
            "%Y-%m-%dT%H:%M:%S%.3f%z",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
        ];

        for fmt in &formats {
            if let Ok(dt) = DateTime::parse_from_str(trimmed, fmt) {
                return Ok(PrecisionDateTime::new(dt, TemporalPrecision::Second));
            }
        }

        // Try parsing just as a date and convert to datetime
        if let Ok(date) = Self::parse_date_string(trimmed) {
            let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
                date.date.and_hms_opt(0, 0, 0).unwrap(),
                Utc
            );
            return Ok(PrecisionDateTime::new(
                DateTime::from_naive_utc_and_offset(dt.naive_utc(), FixedOffset::east_opt(0).unwrap()),
                date.precision
            ));
        }

        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a datetime", input)
        ))
    }

    fn parse_time_string(input: &str) -> Result<PrecisionTime> {
        let trimmed = input.trim();
        
        // Try standard time formats
        let formats = [
            "%H:%M:%S",
            "%H:%M:%S%.3f",
            "%H:%M",
        ];

        for fmt in &formats {
            if let Ok(time) = NaiveTime::parse_from_str(trimmed, fmt) {
                let precision = if fmt.contains("%.3f") {
                    TemporalPrecision::Millisecond
                } else if fmt.contains(":%S") {
                    TemporalPrecision::Second
                } else {
                    TemporalPrecision::Minute
                };
                return Ok(PrecisionTime::new(time, precision));
            }
        }

        // Try extracting time from datetime string
        if let Ok(datetime) = Self::parse_datetime_string(trimmed) {
            return Ok(PrecisionTime::new(
                datetime.datetime.naive_local().time(),
                TemporalPrecision::Second
            ));
        }

        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a time", input)
        ))
    }

    fn register_toQuantity_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toQuantity",
            category: FunctionCategory::Conversion,
            description: "Converts the input value to a quantity",
            parameters: [],
            return_type: "Quantity",
            examples: [
                "'10 kg'.toQuantity()",
                "42.toQuantity()",
                "'98.6 degF'.toQuantity()"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toQuantity() can only be called on a single value"
                    ));
                }

                let result = match &context.input[0] {
                    FhirPathValue::Quantity { .. } => context.input[0].clone(),
                    FhirPathValue::String(s) => {
                        Self::parse_quantity_string(s)?
                    }
                    FhirPathValue::Integer(i) => {
                        FhirPathValue::Quantity {
                            value: Decimal::from(*i),
                            unit: None,
                            ucum_unit: None,
                            calendar_unit: None,
                        }
                    }
                    FhirPathValue::Decimal(d) => {
                        FhirPathValue::Quantity {
                            value: *d,
                            unit: None,
                            ucum_unit: None,
                            calendar_unit: None,
                        }
                    }
                    _ => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to quantity"
                        ));
                    }
                };

                Ok(vec![result])
            }
        )
    }

    fn parse_quantity_string(input: &str) -> Result<FhirPathValue> {
        let trimmed = input.trim();
        
        if trimmed.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0058,
                "Cannot parse empty string as quantity"
            ));
        }

        // Try to parse as just a number first
        if let Ok(decimal_value) = Decimal::from_str(trimmed) {
            return Ok(FhirPathValue::Quantity {
                value: decimal_value,
                unit: None,
                ucum_unit: None,
                calendar_unit: None,
            });
        }

        // Parse with regex for number + unit
        // Match pattern: optional minus, digits, optional decimal part, optional whitespace, unit
        if let Some(space_pos) = trimmed.find(' ') {
            let value_str = trimmed[..space_pos].trim();
            let unit_str = trimmed[space_pos + 1..].trim();

            if let Ok(value) = Decimal::from_str(value_str) {
                return Ok(FhirPathValue::Quantity {
                    value,
                    unit: if unit_str.is_empty() { None } else { Some(unit_str.to_string()) },
                    ucum_unit: None, // Will be populated later by UCUM parsing if needed
                    calendar_unit: None, // Will be populated for time units like 'year', 'month'
                });
            }
        }

        // Try common patterns like "10kg" (no space)
        let mut chars = trimmed.chars();
        let mut numeric_part = String::new();
        
        // Extract numeric part
        for ch in &mut chars {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
                numeric_part.push(ch);
            } else {
                break;
            }
        }
        
        if !numeric_part.is_empty() {
            let unit_part: String = chars.collect();
            
            if let Ok(value) = Decimal::from_str(&numeric_part) {
                return Ok(FhirPathValue::Quantity {
                    value,
                    unit: if unit_part.trim().is_empty() { None } else { Some(unit_part.trim().to_string()) },
                    ucum_unit: None,
                    calendar_unit: None,
                });
            }
        }

        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a quantity", input)
        ))
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;
    use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
    use chrono::{NaiveDate, NaiveTime};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_parse_date_string() {
        let date_result = FunctionRegistry::parse_date_string("2023-12-25").unwrap();
        assert_eq!(date_result.date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
        assert_eq!(date_result.precision, TemporalPrecision::Day);
    }

    #[test]
    fn test_parse_date_string_year() {
        let date_result = FunctionRegistry::parse_date_string("2023").unwrap();
        assert_eq!(date_result.date, NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        assert_eq!(date_result.precision, TemporalPrecision::Year);
    }

    #[test]
    fn test_parse_datetime_string() {
        let datetime_result = FunctionRegistry::parse_datetime_string("2023-12-25T10:30:00Z").unwrap();
        assert_eq!(datetime_result.precision, TemporalPrecision::Second);
    }

    #[test]
    fn test_parse_time_string() {
        let time_result = FunctionRegistry::parse_time_string("10:30:00").unwrap();
        assert_eq!(time_result.time, NaiveTime::from_hms_opt(10, 30, 0).unwrap());
        assert_eq!(time_result.precision, TemporalPrecision::Second);
    }

    #[test]
    fn test_parse_quantity_string_with_unit() {
        let quantity_result = FunctionRegistry::parse_quantity_string("10 kg").unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = quantity_result {
            assert_eq!(value, Decimal::from(10));
            assert_eq!(unit, Some("kg".to_string()));
        } else {
            panic!("Expected Quantity value");
        }
    }

    #[test]
    fn test_parse_quantity_string_no_space() {
        let quantity_result = FunctionRegistry::parse_quantity_string("10kg").unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = quantity_result {
            assert_eq!(value, Decimal::from(10));
            assert_eq!(unit, Some("kg".to_string()));
        } else {
            panic!("Expected Quantity value");
        }
    }

    #[test]
    fn test_parse_quantity_string_without_unit() {
        let quantity_result = FunctionRegistry::parse_quantity_string("42.5").unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = quantity_result {
            assert_eq!(value, Decimal::from_str("42.5").unwrap());
            assert_eq!(unit, None);
        } else {
            panic!("Expected Quantity value");
        }
    }

    #[test]
    fn test_parse_invalid_date() {
        let result = FunctionRegistry::parse_date_string("not-a-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_quantity() {
        let result = FunctionRegistry::parse_quantity_string("");
        assert!(result.is_err());
    }
}
