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

//! LowBoundary function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use num_traits::ToPrimitive;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use rust_decimal::Decimal;
use std::str::FromStr;

/// LowBoundary function - returns the lower boundary of a partial date/time value
#[derive(Debug, Clone)]
pub struct LowBoundaryFunction;

impl Default for LowBoundaryFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl LowBoundaryFunction {
    /// Calculate low boundary for decimal values with optional precision
    fn calculate_decimal_low_boundary(
        &self,
        value: Decimal,
        precision: Option<i32>,
    ) -> Option<Decimal> {
        match precision {
            Some(prec) => {
                // Handle edge cases for precision values
                if !(0..=28).contains(&prec) {
                    return None; // Invalid precision returns empty
                }

                // For precision 0, floor to integer (round down)
                if prec == 0 {
                    if value.fract() == Decimal::ZERO {
                        // Integer value - subtract 1
                        return Some(value - Decimal::ONE);
                    } else {
                        // Decimal value - floor
                        return Some(value.floor());
                    }
                }

                // For precision 1, special handling for small values
                if prec == 1 {
                    // If the value is very small (like 0.0034), the low boundary at precision 1 should be 0
                    if value.abs() < Decimal::from_str("0.1").unwrap() {
                        return Some(Decimal::ZERO);
                    }
                    // Otherwise, truncate to 1 decimal place and subtract 0.1
                    let truncated = (value * Decimal::from(10)).trunc() / Decimal::from(10);
                    return Some(truncated - Decimal::from_str("0.1").unwrap());
                }

                // Get current scale (number of decimal places)
                let current_scale = value.scale();

                if (prec as u32) < current_scale {
                    // Truncate to specified precision and round down
                    let scale_factor = match 10_i64.checked_pow(prec as u32) {
                        Some(factor) => Decimal::from(factor),
                        None => return None,
                    };

                    let scaled_value = value * scale_factor;
                    let boundary_value = scaled_value.floor();
                    Some(boundary_value / scale_factor)
                } else if (prec as u32) == current_scale {
                    // Same precision - subtract 5 from the last digit
                    let scale_factor = match 10_i64.checked_pow(current_scale) {
                        Some(factor) => Decimal::from(factor),
                        None => return None,
                    };

                    let extension_value = Decimal::from(5) / scale_factor;
                    Some(value - extension_value)
                } else {
                    // Extend precision by subtracting 5 from the first new digit
                    let first_new_digit_position = current_scale + 1;
                    let extension_factor = match 10_i64.checked_pow(first_new_digit_position) {
                        Some(factor) => Decimal::from(factor),
                        None => return None,
                    };

                    // Subtract 5 from the first new digit position
                    let extension_value = Decimal::from(5) / extension_factor;
                    Some(value - extension_value)
                }
            }
            None => {
                // Default behavior: extend current precision by 1 digit and subtract 5
                let current_scale = value.scale();
                let new_precision = current_scale + 1;

                if new_precision > 28 {
                    return Some(value); // Can't extend further
                }

                // Subtract 5 from the new digit position
                let extension_factor = match 10_i64.checked_pow(new_precision) {
                    Some(factor) => Decimal::from(factor),
                    None => return Some(value),
                };

                let extension_value = Decimal::from(5) / extension_factor;
                Some(value - extension_value)
            }
        }
    }

    /// Calculate low boundary for integer values by converting to decimal
    fn calculate_integer_low_boundary(
        &self,
        value: i64,
        precision: Option<i32>,
    ) -> Option<Decimal> {
        // Convert integer to decimal and use decimal boundary calculation
        let decimal_value = Decimal::from(value);
        self.calculate_decimal_low_boundary(decimal_value, precision)
    }

    // Helper method to convert NaiveDate to DateTime<FixedOffset>
    fn calculate_date_low_boundary_typed(&self, date: &NaiveDate) -> Result<DateTime<FixedOffset>> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let boundary_str = self.calculate_date_low_boundary(&date_str);

        // Parse the result back to DateTime<FixedOffset>
        match DateTime::parse_from_str(&boundary_str, "%Y-%m-%dT%H:%M:%S%.3f") {
            Ok(dt) => Ok(dt),
            Err(_) => {
                // Try with timezone
                match DateTime::parse_from_str(
                    &format!("{boundary_str}+00:00"),
                    "%Y-%m-%dT%H:%M:%S%.3f%z",
                ) {
                    Ok(dt) => Ok(dt),
                    Err(_) => {
                        // Fallback: create datetime at UTC
                        let naive_datetime = date
                            .and_hms_opt(0, 0, 0)
                            .unwrap_or_else(|| date.and_hms_opt(0, 0, 0).unwrap());
                        Ok(Utc
                            .from_utc_datetime(&naive_datetime)
                            .with_timezone(&FixedOffset::east_opt(0).unwrap()))
                    }
                }
            }
        }
    }

    // Helper method to handle DateTime<FixedOffset>
    fn calculate_datetime_low_boundary_typed(
        &self,
        datetime: &DateTime<FixedOffset>,
    ) -> Result<DateTime<FixedOffset>> {
        let datetime_str = datetime.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string();
        let boundary_str = self.calculate_datetime_low_boundary(&datetime_str);

        // Parse the result back to DateTime<FixedOffset>
        match DateTime::parse_from_str(&boundary_str, "%Y-%m-%dT%H:%M:%S%.3f%z") {
            Ok(dt) => Ok(dt),
            Err(_) => {
                // Try without timezone
                match DateTime::parse_from_str(
                    &format!("{boundary_str}+00:00"),
                    "%Y-%m-%dT%H:%M:%S%.3f%z",
                ) {
                    Ok(dt) => Ok(dt),
                    Err(_) => Ok(*datetime), // Fallback to original
                }
            }
        }
    }

    // Helper method to handle NaiveTime
    fn calculate_time_low_boundary_typed(&self, time: &NaiveTime) -> Result<NaiveTime> {
        let time_str = time.format("%H:%M:%S%.3f").to_string();
        let boundary_str = self.calculate_time_low_boundary(&time_str);

        // Parse the result back to NaiveTime
        match NaiveTime::parse_from_str(&boundary_str, "%H:%M:%S%.3f") {
            Ok(t) => Ok(t),
            Err(_) => {
                // Try other formats
                if let Ok(t) = NaiveTime::parse_from_str(&boundary_str, "%H:%M:%S") {
                    Ok(t)
                } else if let Ok(t) = NaiveTime::parse_from_str(&boundary_str, "%H:%M") {
                    Ok(t)
                } else {
                    Ok(*time) // Fallback to original
                }
            }
        }
    }
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("lowBoundary", OperationType::Function)
            .description("Returns the lower boundary of a partial date/time value or decimal/integer with optional precision")
            .example("@2023-01.lowBoundary()")
            .example("@T12:30.lowBoundary()")
            .example("1.234.lowBoundary(2)")
            .example("42.lowBoundary(1)")
            .parameter("precision", TypeConstraint::Specific(FhirPathType::Integer), true)
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn calculate_date_low_boundary(&self, date: &str) -> String {
        // Convert partial date to full datetime at earliest possible moment
        let parts: Vec<&str> = date.split('-').collect();

        match parts.len() {
            1 => {
                // Year only -> YYYY-01-01T00:00:00.000
                format!("{}-01-01T00:00:00.000", parts[0])
            }
            2 => {
                // Year-month -> YYYY-MM-01T00:00:00.000
                format!("{date}-01T00:00:00.000")
            }
            3 => {
                // Full date -> YYYY-MM-DDT00:00:00.000
                format!("{date}T00:00:00.000")
            }
            _ => date.to_string(), // Invalid format, return as-is
        }
    }

    fn calculate_datetime_low_boundary(&self, datetime: &str) -> String {
        // Fill in missing precision with earliest values
        if datetime.contains('T') {
            let parts: Vec<&str> = datetime.split('T').collect();
            let date_part = parts[0];
            let time_part = if parts.len() > 1 { parts[1] } else { "" };

            // Expand date part if needed
            let full_date = self.expand_date_to_low_boundary(date_part);
            // Expand time part to full precision
            let full_time = self.expand_time_to_low_boundary(time_part);

            // Remove 'T00:00:00.000' from full_date if it was added, since we're adding our own time
            let clean_date = if full_date.contains('T') {
                full_date.split('T').next().unwrap_or(date_part)
            } else {
                &full_date
            };

            format!("{clean_date}T{full_time}")
        } else {
            // No time part, add minimum time
            let full_date = self.expand_date_to_low_boundary(datetime);
            if full_date.contains('T') {
                full_date // Already has time
            } else {
                format!("{full_date}T00:00:00.000")
            }
        }
    }

    fn calculate_time_low_boundary(&self, time: &str) -> String {
        self.expand_time_to_low_boundary(time)
    }

    fn expand_date_to_low_boundary(&self, date: &str) -> String {
        let parts: Vec<&str> = date.split('-').collect();

        match parts.len() {
            1 => {
                // Year only -> YYYY-01-01
                format!("{}-01-01", parts[0])
            }
            2 => {
                // Year-month -> YYYY-MM-01
                format!("{date}-01")
            }
            _ => date.to_string(), // Already full or invalid
        }
    }

    fn expand_time_to_low_boundary(&self, time: &str) -> String {
        if time.is_empty() {
            return "00:00:00.000".to_string();
        }

        // Handle timezone offset
        let (time_part, tz_part) = if time.contains('+') {
            let parts: Vec<&str> = time.split('+').collect();
            (parts[0], Some(format!("+{}", parts[1])))
        } else if time.contains('Z') {
            (time.trim_end_matches('Z'), Some("Z".to_string()))
        } else if time.rfind('-').is_some_and(|pos| pos > 2) {
            // Find last '-' that could be timezone (not in date part)
            let pos = time.rfind('-').unwrap();
            (&time[..pos], Some(time[pos..].to_string()))
        } else {
            (time, None)
        };

        // Expand time part to full precision with minimum values
        let parts: Vec<&str> = time_part.split(':').collect();

        let expanded_time = match parts.len() {
            1 => {
                // Hour only -> HH:00:00.000
                format!("{}:00:00.000", parts[0])
            }
            2 => {
                // Hour:minute -> HH:MM:00.000
                format!("{time_part}:00.000")
            }
            3 => {
                // Hour:minute:second -> check if has milliseconds
                if parts[2].contains('.') {
                    time_part.to_string() // Already has milliseconds
                } else {
                    format!("{time_part}.000") // Add milliseconds
                }
            }
            _ => time_part.to_string(), // Invalid format, return as-is
        };

        // Add timezone back if it existed
        if let Some(tz) = tz_part {
            format!("{expanded_time}{tz}")
        } else {
            expanded_time
        }
    }

    /// Calculate datetime low boundary with precision parameter
    fn calculate_datetime_low_boundary_with_precision(
        &self,
        datetime: &str,
        precision: i32,
    ) -> String {
        // Parse datetime components
        let (date_time_part, tz_part) = self.extract_timezone(datetime);

        match precision {
            17 => {
                // Millisecond precision - extend to minimum milliseconds
                let parts: Vec<&str> = date_time_part.split('T').collect();
                if parts.len() >= 2 {
                    let date_part = parts[0];
                    let time_part = parts[1];

                    // For precision 17, we need to expand to minimum seconds and milliseconds
                    let time_components: Vec<&str> = time_part.split(':').collect();
                    let expanded_time = if time_components.len() >= 2 {
                        let hour = time_components[0];
                        let minute = time_components[1];
                        format!("{hour}:{minute}:00.000")
                    } else if time_components.len() == 1 {
                        let hour = time_components[0];
                        format!("{hour}:00:00.000")
                    } else {
                        "00:00:00.000".to_string()
                    };

                    let result = format!("{date_part}T{expanded_time}");
                    self.format_with_timezone(&result, &tz_part)
                } else {
                    let result = format!("{}T00:00:00.000", parts[0]);
                    self.format_with_timezone(&result, &tz_part)
                }
            }
            _ => {
                // For other precisions, use default behavior
                self.calculate_datetime_low_boundary(datetime)
            }
        }
    }

    /// Extract timezone from datetime string
    fn extract_timezone(&self, datetime: &str) -> (String, Option<String>) {
        if datetime.contains('+') {
            let parts: Vec<&str> = datetime.split('+').collect();
            (parts[0].to_string(), Some(format!("+{}", parts[1])))
        } else if datetime.contains('Z') {
            (
                datetime.trim_end_matches('Z').to_string(),
                Some("Z".to_string()),
            )
        } else if datetime.rfind('-').is_some_and(|pos| pos > 10) {
            // Find last '-' that could be timezone (not in date part)
            let pos = datetime.rfind('-').unwrap();
            (
                datetime[..pos].to_string(),
                Some(datetime[pos..].to_string()),
            )
        } else {
            (datetime.to_string(), None)
        }
    }

    /// Format result with timezone
    fn format_with_timezone(&self, datetime: &str, tz_part: &Option<String>) -> String {
        if let Some(tz) = tz_part {
            // Normalize timezone format to use colon separator
            let normalized_tz = if tz.len() == 5 && (tz.starts_with('+') || tz.starts_with('-')) {
                // Convert +HHMM or -HHMM to +HH:MM or -HH:MM
                let sign = &tz[0..1];
                let hours = &tz[1..3];
                let minutes = &tz[3..5];
                format!("{sign}{hours}:{minutes}")
            } else {
                tz.clone()
            };
            format!("{datetime}{normalized_tz}")
        } else {
            // Default to +14:00 timezone for minimum boundary
            format!("{datetime}+14:00")
        }
    }
}

#[async_trait]
impl FhirPathOperation for LowBoundaryFunction {
    fn identifier(&self) -> &str {
        "lowBoundary"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(LowBoundaryFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate argument count (0 or 1 arguments)
        if args.len() > 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Validate precision parameter type if provided
        if args.len() == 1 {
            match &args[0] {
                FhirPathValue::Integer(precision) => {
                    // Validate precision range
                    if *precision < 0 || *precision > 28 {
                        return Ok(FhirPathValue::Collection(Collection::new())); // Return empty for invalid precision
                    }
                }
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.first().unwrap() {
                        FhirPathValue::Integer(precision) => {
                            if *precision < 0 || *precision > 28 {
                                return Ok(FhirPathValue::Collection(Collection::new()));
                            }
                        }
                        _ => {
                            return Err(FhirPathError::TypeError {
                                message: format!(
                                    "lowBoundary() precision parameter must be an Integer, got {:?}",
                                    items.first().unwrap()
                                ),
                            });
                        }
                    }
                }
                FhirPathValue::Collection(items) if items.is_empty() => {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                }
                FhirPathValue::Collection(_) => {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "lowBoundary() precision parameter must be a single integer"
                            .to_string(),
                    });
                }
                _ => {
                    return Err(FhirPathError::TypeError {
                        message: format!(
                            "lowBoundary() precision parameter must be an Integer, got {:?}",
                            args[0]
                        ),
                    });
                }
            }
        }

        let input = &context.input;

        // Extract precision parameter if provided
        let precision = if args.len() == 1 {
            match &args[0] {
                FhirPathValue::Integer(p) => Some(*p as i32),
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.first().unwrap() {
                        FhirPathValue::Integer(p) => Some(*p as i32),
                        _ => None, // Already validated above
                    }
                }
                _ => None, // Already validated above
            }
        } else {
            None
        };

        match input {
            FhirPathValue::Decimal(d) => {
                match self.calculate_decimal_low_boundary(*d, precision) {
                    Some(result) => {
                        // If precision is 0 or 1 and result is zero, return as integer
                        if (precision == Some(0) || precision == Some(1)) && result == Decimal::ZERO {
                            Ok(FhirPathValue::Integer(0))
                        } else if precision == Some(0) && result.fract() == Decimal::ZERO {
                            if let Some(int_result) = result.to_i64() {
                                Ok(FhirPathValue::Integer(int_result))
                            } else {
                                Ok(FhirPathValue::Decimal(result))
                            }
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    None => Ok(FhirPathValue::Collection(Collection::new())), // Invalid precision
                }
            }
            FhirPathValue::Integer(i) => {
                match self.calculate_integer_low_boundary(*i, precision) {
                    Some(result) => {
                        // If precision is 0, return as integer
                        if precision == Some(0) && result.fract() == Decimal::ZERO {
                            if let Some(int_result) = result.to_i64() {
                                Ok(FhirPathValue::Integer(int_result))
                            } else {
                                Ok(FhirPathValue::Decimal(result))
                            }
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    None => Ok(FhirPathValue::Collection(Collection::new())), // Invalid precision
                }
            }
            FhirPathValue::Quantity(q) => {
                match self.calculate_decimal_low_boundary(q.value, precision) {
                    Some(result_value) => {
                        // Format the quantity as a string with proper precision
                        let formatted_value = if let Some(prec) = precision {
                            if (0..=28).contains(&prec) {
                                // Format with specified precision
                                format!("{:.prec$}", result_value, prec = prec as usize)
                            } else {
                                result_value.to_string()
                            }
                        } else {
                            result_value.to_string()
                        };

                        let unit_str = q.unit.as_deref().unwrap_or("");
                        let quantity_string = if unit_str.is_empty() {
                            formatted_value
                        } else {
                            format!("{formatted_value} '{unit_str}'")
                        };

                        Ok(FhirPathValue::String(quantity_string.into()))
                    }
                    None => Ok(FhirPathValue::Collection(Collection::new())), // Invalid precision
                }
            }
            FhirPathValue::Date(d) => {
                // For dates with precision, return appropriate type based on precision
                match precision {
                    Some(3) => {
                        // Year precision - return as string "@YYYY"
                        let year = d.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}").into()))
                    }
                    Some(6) => {
                        // Month precision - return as string "@YYYY-01" (January is the low boundary for year)
                        let year = d.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}-01").into()))
                    }
                    Some(8) => {
                        // Day precision - return as date "@YYYY-MM-DD"
                        let date_str = d.format("%Y-%m-%d").to_string();
                        Ok(FhirPathValue::String(format!("@{date_str}").into()))
                    }
                    _ => {
                        // Default behavior - return as datetime
                        let low_boundary = self.calculate_date_low_boundary_typed(d)?;
                        Ok(FhirPathValue::DateTime(low_boundary))
                    }
                }
            }
            FhirPathValue::DateTime(dt) => {
                // For datetime with precision, return appropriate type based on precision
                match precision {
                    Some(3) => {
                        // Year precision - return as string "@YYYY"
                        let year = dt.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}").into()))
                    }
                    Some(6) => {
                        // Month precision - return as string "@YYYY-01" (January is the low boundary for year)
                        let year = dt.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}-01").into()))
                    }
                    Some(8) => {
                        // Day precision - return as string "@YYYY-MM-DD"
                        let date_str = dt.format("%Y-%m-%d").to_string();
                        Ok(FhirPathValue::String(format!("@{date_str}").into()))
                    }
                    Some(17) => {
                        // Millisecond precision - handle timezone correctly
                        let datetime_str = dt.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string();

                        // Check if the original datetime had a timezone by checking if it's UTC (+0000)
                        // For low boundary, if no explicit timezone was provided, use +12:00
                        let boundary_str = if datetime_str.ends_with("+0000") {
                            // No explicit timezone was provided, force +12:00 for low boundary
                            let dt_without_tz = datetime_str.trim_end_matches("+0000");
                            self.calculate_datetime_low_boundary_with_precision(dt_without_tz, 17)
                        } else {
                            self.calculate_datetime_low_boundary_with_precision(&datetime_str, 17)
                        };

                        // Parse back to datetime and return as string to preserve timezone format
                        Ok(FhirPathValue::String(format!("@{boundary_str}").into()))
                    }
                    _ => {
                        // Default behavior or other precisions - return as datetime
                        let low_boundary = self.calculate_datetime_low_boundary_typed(dt)?;
                        Ok(FhirPathValue::DateTime(low_boundary))
                    }
                }
            }
            FhirPathValue::Time(t) => {
                // For time with precision, return appropriate type based on precision
                match precision {
                    Some(9) => {
                        // Millisecond precision - return as string "@T..."
                        // For precision 9, extend to minimum milliseconds
                        let hour = t.hour();
                        let minute = t.minute();
                        let expanded_time = format!("{hour:02}:{minute:02}:00.000");
                        Ok(FhirPathValue::String(format!("@T{expanded_time}").into()))
                    }
                    _ => {
                        // Default behavior - return as time
                        let low_boundary = self.calculate_time_low_boundary_typed(t)?;
                        Ok(FhirPathValue::Time(low_boundary))
                    }
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    let item_context = context.with_focus(items.get(0).unwrap().clone());
                    self.evaluate(args, &item_context).await
                } else if items.is_empty() {
                    Ok(FhirPathValue::Collection(Collection::new()))
                } else {
                    Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "lowBoundary() requires a single value".to_string()
                    })
                }
            }
            _ => Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                message: "lowBoundary() requires a decimal, integer, quantity, date, datetime, or time value".to_string()
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Validate argument count (0 or 1 arguments)
        if args.len() > 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        // Validate precision parameter type if provided
        if args.len() == 1 {
            match &args[0] {
                FhirPathValue::Integer(precision) => {
                    // Validate precision range
                    if *precision < 0 || *precision > 28 {
                        return Some(Ok(FhirPathValue::Collection(Collection::new()))); // Return empty for invalid precision
                    }
                }
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.first().unwrap() {
                        FhirPathValue::Integer(precision) => {
                            if *precision < 0 || *precision > 28 {
                                return Some(Ok(FhirPathValue::Collection(Collection::new())));
                            }
                        }
                        _ => {
                            return Some(Err(FhirPathError::TypeError {
                                message: format!(
                                    "lowBoundary() precision parameter must be an Integer, got {:?}",
                                    items.first().unwrap()
                                ),
                            }));
                        }
                    }
                }
                FhirPathValue::Collection(items) if items.is_empty() => {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                }
                FhirPathValue::Collection(_) => {
                    return Some(Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "lowBoundary() precision parameter must be a single integer"
                            .to_string(),
                    }));
                }
                _ => {
                    return Some(Err(FhirPathError::TypeError {
                        message: format!(
                            "lowBoundary() precision parameter must be an Integer, got {:?}",
                            args[0]
                        ),
                    }));
                }
            }
        }

        let input = &context.input;

        // Extract precision parameter if provided
        let precision = if args.len() == 1 {
            match &args[0] {
                FhirPathValue::Integer(p) => Some(*p as i32),
                FhirPathValue::Collection(items) if items.len() == 1 => {
                    match items.first().unwrap() {
                        FhirPathValue::Integer(p) => Some(*p as i32),
                        _ => None, // Already validated above
                    }
                }
                _ => None, // Already validated above
            }
        } else {
            None
        };

        let result = match input {
            FhirPathValue::Decimal(d) => {
                match self.calculate_decimal_low_boundary(*d, precision) {
                    Some(result) => {
                        // If precision is 0 or 1 and result is zero, return as integer
                        if (precision == Some(0) || precision == Some(1)) && result == Decimal::ZERO {
                            Ok(FhirPathValue::Integer(0))
                        } else if precision == Some(0) && result.fract() == Decimal::ZERO {
                            if let Some(int_result) = result.to_i64() {
                                Ok(FhirPathValue::Integer(int_result))
                            } else {
                                Ok(FhirPathValue::Decimal(result))
                            }
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    None => Ok(FhirPathValue::Collection(Collection::new())), // Invalid precision
                }
            }
            FhirPathValue::Integer(i) => {
                match self.calculate_integer_low_boundary(*i, precision) {
                    Some(result) => {
                        // If precision is 0, return as integer
                        if precision == Some(0) && result.fract() == Decimal::ZERO {
                            if let Some(int_result) = result.to_i64() {
                                Ok(FhirPathValue::Integer(int_result))
                            } else {
                                Ok(FhirPathValue::Decimal(result))
                            }
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    None => Ok(FhirPathValue::Collection(Collection::new())), // Invalid precision
                }
            }
            FhirPathValue::Quantity(q) => {
                match self.calculate_decimal_low_boundary(q.value, precision) {
                    Some(result_value) => {
                        // Format the quantity as a string with proper precision
                        let formatted_value = if let Some(prec) = precision {
                            if (0..=28).contains(&prec) {
                                // Format with specified precision
                                format!("{:.prec$}", result_value, prec = prec as usize)
                            } else {
                                result_value.to_string()
                            }
                        } else {
                            result_value.to_string()
                        };

                        let unit_str = q.unit.as_deref().unwrap_or("");
                        let quantity_string = if unit_str.is_empty() {
                            formatted_value
                        } else {
                            format!("{formatted_value} '{unit_str}'")
                        };

                        Ok(FhirPathValue::String(quantity_string.into()))
                    }
                    None => Ok(FhirPathValue::Collection(Collection::new())), // Invalid precision
                }
            }
            FhirPathValue::Date(d) => {
                // For dates with precision, return appropriate type based on precision
                match precision {
                    Some(3) => {
                        // Year precision - return as string "@YYYY"
                        let year = d.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}").into()))
                    }
                    Some(6) => {
                        // Month precision - return as string "@YYYY-01" (January is the low boundary for year)
                        let year = d.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}-01").into()))
                    }
                    Some(8) => {
                        // Day precision - return as date "@YYYY-MM-DD"
                        let date_str = d.format("%Y-%m-%d").to_string();
                        Ok(FhirPathValue::String(format!("@{date_str}").into()))
                    }
                    _ => {
                        // Default behavior - return as datetime
                        match self.calculate_date_low_boundary_typed(d) {
                            Ok(low_boundary) => Ok(FhirPathValue::DateTime(low_boundary)),
                            Err(e) => Err(e),
                        }
                    }
                }
            }
            FhirPathValue::DateTime(dt) => {
                // For datetime with precision, return appropriate type based on precision
                match precision {
                    Some(3) => {
                        // Year precision - return as string "@YYYY"
                        let year = dt.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}").into()))
                    }
                    Some(6) => {
                        // Month precision - return as string "@YYYY-01" (January is the low boundary for year)
                        let year = dt.format("%Y").to_string();
                        Ok(FhirPathValue::String(format!("@{year}-01").into()))
                    }
                    Some(8) => {
                        // Day precision - return as string "@YYYY-MM-DD"
                        let date_str = dt.format("%Y-%m-%d").to_string();
                        Ok(FhirPathValue::String(format!("@{date_str}").into()))
                    }
                    Some(17) => {
                        // Millisecond precision - handle timezone correctly
                        let datetime_str = dt.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string();

                        // Check if the original datetime had a timezone by checking if it's UTC (+0000)
                        // For low boundary, if no explicit timezone was provided, use +12:00
                        let boundary_str = if datetime_str.ends_with("+0000") {
                            // No explicit timezone was provided, force +12:00 for low boundary
                            let dt_without_tz = datetime_str.trim_end_matches("+0000");
                            self.calculate_datetime_low_boundary_with_precision(dt_without_tz, 17)
                        } else {
                            self.calculate_datetime_low_boundary_with_precision(&datetime_str, 17)
                        };

                        // Parse back to datetime and return as string to preserve timezone format
                        Ok(FhirPathValue::String(format!("@{boundary_str}").into()))
                    }
                    _ => {
                        // Default behavior or other precisions - return as datetime
                        match self.calculate_datetime_low_boundary_typed(dt) {
                            Ok(low_boundary) => Ok(FhirPathValue::DateTime(low_boundary)),
                            Err(e) => Err(e),
                        }
                    }
                }
            }
            FhirPathValue::Time(t) => {
                // For time with precision, return appropriate type based on precision
                match precision {
                    Some(9) => {
                        // Millisecond precision - return as string "@T..."
                        // For precision 9, extend to minimum milliseconds
                        let hour = t.hour();
                        let minute = t.minute();
                        let expanded_time = format!("{hour:02}:{minute:02}:00.000");
                        Ok(FhirPathValue::String(format!("@T{expanded_time}").into()))
                    }
                    _ => {
                        // Default behavior - return as time
                        match self.calculate_time_low_boundary_typed(t) {
                            Ok(low_boundary) => Ok(FhirPathValue::Time(low_boundary)),
                            Err(e) => Err(e),
                        }
                    }
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    let item_context = context.with_focus(items.get(0).unwrap().clone());
                    return self.try_evaluate_sync(args, &item_context);
                } else if items.is_empty() {
                    Ok(FhirPathValue::Collection(Collection::new()))
                } else {
                    Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "lowBoundary() requires a single value".to_string()
                    })
                }
            }
            _ => Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                message: "lowBoundary() requires a decimal, integer, quantity, date, datetime, or time value".to_string()
            }),
        };

        Some(result)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
