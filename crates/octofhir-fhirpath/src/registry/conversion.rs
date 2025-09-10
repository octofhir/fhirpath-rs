//! Conversion functions for FHIRPath expressions
//!
//! This module implements type conversion functions like toString(), toInteger(),
//! toDecimal(), toBoolean(), toDate(), toDateTime(), toTime(), and toQuantity().

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::error_code::{FP0053, FP0058};
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
use crate::core::{CalendarUnit, FhirPathError, FhirPathValue, Result};
use crate::register_function;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;

impl FunctionRegistry {
    pub fn register_conversion_functions(&self) -> Result<()> {
        self.register_to_string_function()?;
        self.register_to_integer_function()?;
        self.register_to_decimal_function()?;
        self.register_to_boolean_function()?;
        self.register_to_date_function()?;
        self.register_to_date_time_function()?;
        self.register_to_time_function()?;
        self.register_to_quantity_function()?;

        // Register conversion testing functions
        self.register_converts_to_boolean_function()?;
        self.register_converts_to_integer_function()?;
        self.register_converts_to_decimal_function()?;
        self.register_converts_to_string_function()?;
        self.register_converts_to_date_function()?;
        self.register_converts_to_date_time_function()?;
        self.register_converts_to_time_function()?;
        self.register_converts_to_quantity_function()?;

        Ok(())
    }

    fn register_to_string_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

                // FHIRPath spec: "If the input collection contains multiple items, signal an error"
                if context.input.len() > 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toString() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::String(s)) => s.clone(),
                    Some(FhirPathValue::Integer(i)) => i.to_string(),
                    Some(FhirPathValue::Decimal(d)) => d.to_string(),
                    Some(FhirPathValue::Boolean(b)) => b.to_string(),
                    Some(FhirPathValue::Date(d)) => d.to_string(),
                    Some(FhirPathValue::DateTime(dt)) => dt.to_string(),
                    Some(FhirPathValue::Time(t)) => t.to_string(),
                    Some(FhirPathValue::Quantity { value, unit, calendar_unit, .. }) => {
                        if let Some(cu) = calendar_unit {
                            format!("{} {}", value, cu)
                        } else {
                            match unit {
                                Some(u) if !u.is_empty() => {
                                    // Determine if this is a UCUM unit that should be quoted
                                    let is_ucum_unit = Self::is_ucum_unit(u);
                                    if is_ucum_unit {
                                        format!("{} '{}'", value, u)
                                    } else {
                                        format!("{} {}", value, u)
                                    }
                                },
                                _ => value.to_string(),
                            }
                        }
                    }
                    Some(FhirPathValue::Uri(uri)) => uri.clone(),
                    Some(FhirPathValue::Url(url)) => url.clone(),
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to string"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to string"
                        ));
                    }
                };

                Ok(FhirPathValue::String(result))
            }
        )
    }

    fn register_to_integer_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

                if context.input.len() > 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toInteger() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => *i,
                    Some(FhirPathValue::Decimal(d)) => {
                        // Truncate decimal to integer
                        let truncated = d.trunc();
                        truncated.to_string().parse::<i64>()
                            .map_err(|_| FhirPathError::evaluation_error(
                                FP0058,
                                "Decimal value too large for integer conversion"
                            ))?
                    }
                    Some(FhirPathValue::String(s)) => {
                        let trimmed = s.trim();
                        // Per FHIRPath specification, strings with decimal points should not convert to integers
                        if trimmed.contains('.') {
                            return Ok(FhirPathValue::empty());
                        }
                        match trimmed.parse::<i64>() {
                            Ok(int_val) => int_val,
                            Err(_) => return Ok(FhirPathValue::empty()), // Return empty for failed conversion per FHIRPath specification
                        }
                    }
                    Some(FhirPathValue::Boolean(b)) => if *b { 1 } else { 0 },
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to integer"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to integer"
                        ));
                    }
                };

                Ok(FhirPathValue::integer(result))
            }
        )
    }

    fn register_to_decimal_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

                if context.input.len() > 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toDecimal() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::Decimal(d)) => *d,
                    Some(FhirPathValue::Integer(i)) => Decimal::from(*i),
                    Some(FhirPathValue::String(s)) => {
                        match s.trim().parse::<Decimal>() {
                            Ok(decimal_val) => decimal_val,
                            Err(_) => return Ok(FhirPathValue::empty()), // Return empty for failed conversion per FHIRPath specification
                        }
                    }
                    Some(FhirPathValue::Boolean(b)) => Decimal::from(if *b { 1 } else { 0 }),
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to decimal"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to decimal"
                        ));
                    }
                };

                Ok(FhirPathValue::decimal(result))
            }
        )
    }

    fn register_to_boolean_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toBoolean() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::Boolean(b)) => *b,
                    Some(FhirPathValue::String(s)) => {
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
                    Some(FhirPathValue::Integer(i)) => {
                        if *i == 0 {
                            false
                        } else if *i == 1 {
                            true
                        } else {
                            return Ok(FhirPathValue::Empty)
                        }
                    }
                    Some(FhirPathValue::Decimal(d)) => {
                        if d.is_zero() {
                            false
                        } else if *d == Decimal::ONE {
                            true
                        } else {
                            return Ok(FhirPathValue::Empty)
                        }
                    }
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to boolean"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to boolean"
                        ));
                    }
                };

                Ok(FhirPathValue::boolean(result))
            }
        )
    }

    fn register_to_date_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toDate() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::Date(d)) => d.clone(),
                    Some(FhirPathValue::DateTime(dt)) => {
                        // Extract date part from datetime
                        PrecisionDate::from_date(dt.datetime.naive_utc().date())
                    }
                    Some(FhirPathValue::Empty) => return Ok(FhirPathValue::Empty),
                    Some(FhirPathValue::String(s)) => {
                        return match Self::parse_date_string(s) {
                            Ok(val) => Ok(FhirPathValue::Date(val)),
                            Err(_) => Ok(FhirPathValue::Empty)
                        }
                    }
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "toDate() can only convert strings and datetimes"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to date"
                        ));
                    }
                };

                Ok(FhirPathValue::Date(result))
            }
        )
    }

    fn register_to_date_time_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty(){
                    return Ok(FhirPathValue::Empty)
                }
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toDateTime() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::DateTime(dt)) => dt.clone(),
                    Some(FhirPathValue::Date(d)) => {
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
                    Some(FhirPathValue::Empty) => return Ok(FhirPathValue::Empty),
                    Some(FhirPathValue::String(s)) => {
                        return match Self::parse_datetime_string(s) {
                            Ok(val) => Ok(FhirPathValue::DateTime(val)),
                            Err(_) => Ok(FhirPathValue::Empty)
                        }
                    }
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "toDateTime() can only convert strings and dates"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to datetime"
                        ));
                    }
                };

                Ok(FhirPathValue::DateTime(result))
            }
        )
    }

    fn register_to_time_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toTime() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(FhirPathValue::Time(t)) => t.clone(),
                    Some(FhirPathValue::DateTime(dt)) => {
                        // Extract time part from datetime
                        PrecisionTime::new(
                            dt.datetime.naive_local().time(),
                            TemporalPrecision::Second
                        )
                    }
                    Some(FhirPathValue::Empty) => return Ok(FhirPathValue::Empty),
                    Some(FhirPathValue::String(s)) => {
                        Self::parse_time_string(s)?
                    }
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "toTime() can only convert strings and datetimes"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to time"
                        ));
                    }
                };

                Ok(FhirPathValue::Time(result))
            }
        )
    }

    // Helper functions for parsing
    pub fn parse_date_string(input: &str) -> Result<PrecisionDate> {
        let trimmed = input.trim();

        if let Some(pdt) = PrecisionDateTime::parse(trimmed) {
            return Ok(PrecisionDate::from_date(pdt.datetime.naive_utc().date()));
        }

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
            &format!("Unable to parse '{}' as a date", input),
        ))
    }

    pub fn parse_datetime_string(input: &str) -> Result<PrecisionDateTime> {
        let trimmed = input.trim();
        // Use unified precision-aware parser that supports partial precisions and timezones
        if let Some(pdt) = PrecisionDateTime::parse(trimmed) {
            return Ok(pdt);
        }
        // Fallback: handle partial datetime without timezone explicitly (e.g., YYYY-MM-DDTHH)
        if trimmed.len() == 13 && trimmed.chars().nth(10) == Some('T') {
            let date_part = &trimmed[..10];
            let hour_part = &trimmed[11..13];
            if let Some(pdate) = PrecisionDate::parse(date_part) {
                if let Ok(hour) = hour_part.parse::<u32>() {
                    if let Some(nt) = NaiveTime::from_hms_opt(hour, 0, 0) {
                        let ndt = pdate.date.and_time(nt);
                        let dt = DateTime::from_naive_utc_and_offset(
                            ndt,
                            FixedOffset::east_opt(0).unwrap(),
                        );
                        return Ok(PrecisionDateTime::new(dt, TemporalPrecision::Hour));
                    }
                }
            }
        }
        // Try parsing just as a date and convert to datetime at midnight
        if let Ok(date) = Self::parse_date_string(trimmed) {
            let dt: DateTime<Utc> =
                DateTime::from_naive_utc_and_offset(date.date.and_hms_opt(0, 0, 0).unwrap(), Utc);
            return Ok(PrecisionDateTime::new(
                DateTime::from_naive_utc_and_offset(
                    dt.naive_utc(),
                    FixedOffset::east_opt(0).unwrap(),
                ),
                date.precision,
            ));
        }
        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a datetime", input),
        ))
    }

    pub fn parse_time_string(input: &str) -> Result<PrecisionTime> {
        let trimmed = input.trim();

        // Try standard time formats
        let formats = ["%H:%M:%S", "%H:%M:%S%.3f", "%H:%M"];

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
                TemporalPrecision::Second,
            ));
        }

        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a time", input),
        ))
    }

    fn register_to_quantity_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "toQuantity() can only be called on a single value"
                    ));
                }

                let result = match context.input.first() {
                    Some(quantity @ FhirPathValue::Quantity { .. }) => quantity.clone(),
                    Some(FhirPathValue::String(s)) => {
                        Self::parse_quantity_string(s)?
                    }
                    Some(FhirPathValue::Integer(i)) => {
                        FhirPathValue::Quantity {
                            value: Decimal::from(*i),
                            unit: Some("1".to_string()),
                            ucum_unit: None,
                            calendar_unit: None,
                        }
                    }
                    Some(FhirPathValue::Decimal(d)) => {
                        FhirPathValue::Quantity {
                            value: *d,
                            unit: Some("1".to_string()),
                            ucum_unit: None,
                            calendar_unit: None,
                        }
                    }
                    Some(_) => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert this value type to quantity"
                        ));
                    }
                    None => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            "Cannot convert empty value to quantity"
                        ));
                    }
                };

                Ok(result)
            }
        )
    }

    fn parse_quantity_string(input: &str) -> Result<FhirPathValue> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0058,
                "Cannot parse empty string as quantity",
            ));
        }

        // Try to parse as just a number first
        if let Ok(decimal_value) = Decimal::from_str(trimmed) {
            // Numeric-only strings are treated as UCUM dimensionless with unit '1'
            return Ok(FhirPathValue::Quantity {
                value: decimal_value,
                unit: Some("1".to_string()),
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
                // Handle calendar unit keywords
                let unit_lc = unit_str.to_lowercase();
                let (unit_opt, cal_unit_opt) = match unit_lc.as_str() {
                    "day" | "days" => (Some("d".to_string()), CalendarUnit::from_str("day")),
                    "week" | "weeks" => (Some("wk".to_string()), CalendarUnit::from_str("week")),
                    "month" | "months" => (Some("mo".to_string()), CalendarUnit::from_str("month")),
                    "year" | "years" => (Some("a".to_string()), CalendarUnit::from_str("year")),
                    // Reject bare UCUM abbreviations without quotes for certain units
                    "wk" | "mo" | "a" | "d" => {
                        return Err(FhirPathError::evaluation_error(
                            FP0058,
                            &format!("Unit '{}' must be quoted as a UCUM unit", unit_str),
                        ));
                    }
                    _ => {
                        // Handle quoted UCUM units like 'wk'
                        if (unit_str.starts_with('\'') && unit_str.ends_with('\''))
                            || (unit_str.starts_with('"') && unit_str.ends_with('"'))
                        {
                            let inner = unit_str[1..unit_str.len() - 1].to_string();
                            let cal = match inner.as_str() {
                                "wk" => CalendarUnit::from_str("week"),
                                "mo" => CalendarUnit::from_str("month"),
                                "a" => CalendarUnit::from_str("year"),
                                "d" => CalendarUnit::from_str("day"),
                                _ => None,
                            };
                            (Some(inner), cal)
                        } else {
                            // Generic unit
                            if unit_str.is_empty() {
                                (None, None)
                            } else {
                                (Some(unit_str.to_string()), None)
                            }
                        }
                    }
                };
                return Ok(FhirPathValue::Quantity {
                    value,
                    unit: unit_opt,
                    ucum_unit: None, // Will be populated later by UCUM parsing if needed
                    calendar_unit: cal_unit_opt,
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
                    unit: if unit_part.trim().is_empty() {
                        None
                    } else {
                        Some(unit_part.trim().to_string())
                    },
                    ucum_unit: None,
                    calendar_unit: None,
                });
            }
        }

        Err(FhirPathError::evaluation_error(
            FP0058,
            &format!("Unable to parse '{}' as a quantity", input),
        ))
    }

    fn register_converts_to_boolean_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToBoolean",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a boolean",
            parameters: [],
            return_type: "Boolean",
            examples: ["'true'.convertsToBoolean()", "'false'.convertsToBoolean()", "'invalid'.convertsToBoolean()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::Boolean(_)) => true,
                    Some(FhirPathValue::String(s)) => {
                        matches!(s.to_lowercase().as_str(), "true" | "false")
                    },
                    Some(FhirPathValue::Integer(i)) => *i == 0 || *i == 1,
                    Some(FhirPathValue::Decimal(d)) => *d == Decimal::ZERO || *d == Decimal::ONE,
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_integer_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToInteger",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to an integer",
            parameters: [],
            return_type: "Boolean",
            examples: ["'123'.convertsToInteger()", "'abc'.convertsToInteger()", "true.convertsToInteger()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::Integer(_)) => true,
                    Some(FhirPathValue::String(s)) => s.parse::<i64>().is_ok(),
                    Some(FhirPathValue::Boolean(_b)) => true, // Booleans can be converted to 0/1
                    Some(FhirPathValue::Decimal(d)) => {
                        // Check if decimal is a whole number
                        d.fract() == rust_decimal::Decimal::ZERO
                    },
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_decimal_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToDecimal",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a decimal",
            parameters: [],
            return_type: "Boolean",
            examples: ["'123.45'.convertsToDecimal()", "'abc'.convertsToDecimal()", "123.convertsToDecimal()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::Decimal(_)) => true,
                    Some(FhirPathValue::Integer(_)) => true,
                    Some(FhirPathValue::Boolean(_)) => true,
                    Some(FhirPathValue::String(s)) => s.parse::<rust_decimal::Decimal>().is_ok() || matches!(s.trim(), "true" | "false" | "True" | "False"),
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_string_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToString",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a string",
            parameters: [],
            return_type: "Boolean",
            examples: ["123.convertsToString()", "true.convertsToString()", "{}.convertsToString()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                // Most primitive types can be converted to string
                let can_convert = match context.input.first() {
                    Some(FhirPathValue::String(_)) |
                    Some(FhirPathValue::Integer(_)) |
                    Some(FhirPathValue::Decimal(_)) |
                    Some(FhirPathValue::Boolean(_)) |
                    Some(FhirPathValue::Date(_)) |
                    Some(FhirPathValue::DateTime(_)) |
                    Some(FhirPathValue::Time(_)) |
                    Some(FhirPathValue::Uri(_)) |
                    Some(FhirPathValue::Url(_)) |
                    Some(FhirPathValue::Id(_)) |
                    Some(FhirPathValue::Base64Binary(_)) |
                    Some(FhirPathValue::Quantity { .. }) => true,
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_date_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToDate",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a date",
            parameters: [],
            return_type: "Boolean",
            examples: ["'2023-12-25'.convertsToDate()", "'invalid-date'.convertsToDate()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::Date(_)) => true,
                    Some(FhirPathValue::DateTime(_)) => true,
                    Some(FhirPathValue::String(s)) => {
                        Self::parse_date_string(s).is_ok()
                    },
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_date_time_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToDateTime",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a datetime",
            parameters: [],
            return_type: "Boolean",
            examples: ["'2023-12-25T10:30:00'.convertsToDateTime()", "'invalid'.convertsToDateTime()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::DateTime(_)) => true,
                    Some(FhirPathValue::Date(_)) => true,
                    Some(FhirPathValue::String(s)) => {
                        Self::parse_datetime_string(s).is_ok()
                    },
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_time_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToTime",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a time",
            parameters: [],
            return_type: "Boolean",
            examples: ["'10:30:00'.convertsToTime()", "'invalid'.convertsToTime()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::Time(_)) => true,
                    Some(FhirPathValue::DateTime(_)) => true,
                    Some(FhirPathValue::String(s)) => {
                        Self::parse_time_string(s).is_ok()
                    },
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    fn register_converts_to_quantity_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "convertsToQuantity",
            category: FunctionCategory::Conversion,
            description: "Returns true if the input can be converted to a quantity",
            parameters: [],
            return_type: "Boolean",
            examples: ["'10 kg'.convertsToQuantity()", "'123'.convertsToQuantity()", "'abc'.convertsToQuantity()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Ok(FhirPathValue::Boolean(false));
                }

                let can_convert = match context.input.first() {
                    Some(FhirPathValue::Quantity { .. }) => true,
                    Some(FhirPathValue::Integer(_)) => true,
                    Some(FhirPathValue::Decimal(_)) => true,
                    Some(FhirPathValue::String(s)) => {
                        Self::parse_quantity_string(s).is_ok()
                    },
                    Some(_) => false,
                    None => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
        )
    }

    // Helper function to determine if a unit is a UCUM unit that should be quoted
    fn is_ucum_unit(unit: &str) -> bool {
        // Common UCUM time units that are typically quoted to distinguish from calendar units
        matches!(
            unit,
            "s" | "min" | "h" | "d" | "wk" | "mo" | "a" | 
            "ms" | "us" | "ns" | "ks" | "Ms" | "Gs" |
            // Other common UCUM units
            "m" | "cm" | "mm" | "km" | "g" | "kg" | "mg" | "L" | "mL" | 
            "dL" | "mmHg" | "Pa" | "kPa" | "J" | "cal" | "Cel" | "degF" |
            "%" | "1" | "/min" | "/h" | "/d" | "beats/min" | "breaths/min"
        )
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;
    use crate::core::temporal::{
        PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision,
    };
    use chrono::{NaiveDate, NaiveTime};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_parse_date_string() {
        let date_result = FunctionRegistry::parse_date_string("2023-12-25").unwrap();
        assert_eq!(
            date_result.date,
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()
        );
        assert_eq!(date_result.precision, TemporalPrecision::Day);
    }

    #[test]
    fn test_parse_date_string_year() {
        let date_result = FunctionRegistry::parse_date_string("2023").unwrap();
        assert_eq!(
            date_result.date,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
        );
        assert_eq!(date_result.precision, TemporalPrecision::Year);
    }

    #[test]
    fn test_parse_datetime_string() {
        let datetime_result =
            FunctionRegistry::parse_datetime_string("2023-12-25T10:30:00Z").unwrap();
        assert_eq!(datetime_result.precision, TemporalPrecision::Second);
    }

    #[test]
    fn test_parse_time_string() {
        let time_result = FunctionRegistry::parse_time_string("10:30:00").unwrap();
        assert_eq!(
            time_result.time,
            NaiveTime::from_hms_opt(10, 30, 0).unwrap()
        );
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
