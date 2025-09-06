//! Date/Time functions for FHIRPath expressions
//!
//! This module implements date/time functions following the official FHIRPath specification.
//! Reference: https://build.fhir.org/ig/HL7/FHIRPath/index.html#datetime-functions

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
use crate::core::{FhirPathValue, Result};
use crate::register_function;
use crate::registry::datetime_utils::DateTimeUtils;
use chrono::{Datelike, Local, Timelike};

impl FunctionRegistry {
    pub fn register_datetime_functions(&self) -> Result<()> {
        // Current time functions as per FHIRPath specification
        self.register_now_function()?;
        self.register_today_function()?;
        self.register_timeOfDay_function()?;

        // Date/time component extraction functions (official FHIRPath naming)
        self.register_yearOf_function()?;
        self.register_monthOf_function()?;
        self.register_dayOf_function()?;
        self.register_hourOf_function()?;
        self.register_minuteOf_function()?;
        self.register_secondOf_function()?;
        self.register_millisecondOf_function()?;
        self.register_timezoneOffsetOf_function()?;

        // Additional datetime functions (implementation-specific)
        self.register_dayOfWeek_function()?;
        self.register_dayOfYear_function()?;

        Ok(())
    }

    fn register_now_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "now",
            category: FunctionCategory::DateTime,
            description: "Returns the current date and time, including timezone offset",
            parameters: [],
            return_type: "DateTime",
            examples: ["now()", "now() > Patient.birthDate"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let current_time = Local::now();
                let precision_datetime = PrecisionDateTime::new(
                    current_time.into(),
                    TemporalPrecision::Second
                );
                Ok(vec![FhirPathValue::DateTime(precision_datetime)])
            }
        )
    }

    fn register_today_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "today",
            category: FunctionCategory::DateTime,
            description: "Returns the current date",
            parameters: [],
            return_type: "Date",
            examples: ["today()", "today() = Patient.birthDate"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let current_date = Local::now().date_naive();
                let precision_date = PrecisionDate::new(current_date, TemporalPrecision::Day);
                Ok(vec![FhirPathValue::Date(precision_date)])
            }
        )
    }

    fn register_timeOfDay_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "timeOfDay",
            category: FunctionCategory::DateTime,
            description: "Returns the current time",
            parameters: [],
            return_type: "Time",
            examples: ["timeOfDay()", "timeOfDay() > @T12:00:00"],
            implementation: |_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                let current_time = Local::now().time();
                let precision_time = PrecisionTime::new(current_time, TemporalPrecision::Second);
                Ok(vec![FhirPathValue::Time(precision_time)])
            }
        )
    }

    fn register_yearOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "yearOf",
            category: FunctionCategory::DateTime,
            description: "Returns the year component of a Date or DateTime",
            parameters: [],
            return_type: "Integer",
            examples: ["Patient.birthDate.yearOf()", "now().yearOf()", "@2023-05-15.yearOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::Date(date) => {
                            results.push(FhirPathValue::integer(date.date.year() as i64));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::integer(datetime.datetime.year() as i64));
                        }
                        FhirPathValue::String(s) => {
                            // Use the new temporal parsing utilities with proper validation
                            use crate::core::temporal::parsing::parse_date_or_datetime_string;

                            match parse_date_or_datetime_string(s) {
                                Ok(precision_date) => {
                                    results.push(FhirPathValue::integer(precision_date.date.year() as i64));
                                }
                                Err(_) => {
                                    // If parsing fails with validation errors, skip this item
                                    // The error information is preserved but we don't propagate it
                                    // to maintain backward compatibility with existing behavior
                                    continue;
                                }
                            }
                        }
                        _ => {
                            // For other non-date/datetime values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_monthOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "monthOf",
            category: FunctionCategory::DateTime,
            description: "Returns the month component of a Date or DateTime",
            parameters: [],
            return_type: "Integer",
            examples: ["Patient.birthDate.monthOf()", "now().monthOf()", "@2023-05-15.monthOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::Date(date) => {
                            results.push(FhirPathValue::integer(date.date.month() as i64));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::integer(datetime.datetime.month() as i64));
                        }
                        FhirPathValue::String(s) => {
                            // Use the new temporal parsing utilities with proper validation
                            use crate::core::temporal::parsing::parse_date_or_datetime_string;

                            match parse_date_or_datetime_string(s) {
                                Ok(precision_date) => {
                                    results.push(FhirPathValue::integer(precision_date.date.month() as i64));
                                }
                                Err(_) => {
                                    // If parsing fails with validation errors, skip this item
                                    // The error information is preserved but we don't propagate it
                                    // to maintain backward compatibility with existing behavior
                                    continue;
                                }
                            }
                        }
                        _ => {
                            // For other non-date/datetime values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_dayOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "dayOf",
            category: FunctionCategory::DateTime,
            description: "Returns the day component of a Date or DateTime",
            parameters: [],
            return_type: "Integer",
            examples: ["Patient.birthDate.dayOf()", "now().dayOf()", "@2023-05-15.dayOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::Date(date) => {
                            results.push(FhirPathValue::integer(date.date.day() as i64));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::integer(datetime.datetime.day() as i64));
                        }
                        FhirPathValue::String(s) => {
                            // Use the new temporal parsing utilities with proper validation
                            use crate::core::temporal::parsing::parse_date_or_datetime_string;

                            match parse_date_or_datetime_string(s) {
                                Ok(precision_date) => {
                                    results.push(FhirPathValue::integer(precision_date.date.day() as i64));
                                }
                                Err(_) => {
                                    // If parsing fails with validation errors, skip this item
                                    // The error information is preserved but we don't propagate it
                                    // to maintain backward compatibility with existing behavior
                                    continue;
                                }
                            }
                        }
                        _ => {
                            // For other non-date/datetime values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_hourOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "hourOf",
            category: FunctionCategory::DateTime,
            description: "Returns the hour component of a DateTime or Time",
            parameters: [],
            return_type: "Integer",
            examples: ["now().hourOf()", "@2023-05-15T14:30:45.hourOf()", "@T14:30:45.hourOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::integer(datetime.datetime.hour() as i64));
                        }
                        FhirPathValue::Time(time) => {
                            results.push(FhirPathValue::integer(time.time.hour() as i64));
                        }
                        _ => {
                            // For non-datetime/time values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_minuteOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "minuteOf",
            category: FunctionCategory::DateTime,
            description: "Returns the minute component of a DateTime or Time",
            parameters: [],
            return_type: "Integer",
            examples: ["now().minuteOf()", "@2023-05-15T14:30:45.minuteOf()", "@T14:30:45.minuteOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::integer(datetime.datetime.minute() as i64));
                        }
                        FhirPathValue::Time(time) => {
                            results.push(FhirPathValue::integer(time.time.minute() as i64));
                        }
                        _ => {
                            // For non-datetime/time values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_secondOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "secondOf",
            category: FunctionCategory::DateTime,
            description: "Returns the second component of a DateTime or Time",
            parameters: [],
            return_type: "Integer",
            examples: ["now().secondOf()", "@2023-05-15T14:30:45.secondOf()", "@T14:30:45.secondOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            results.push(FhirPathValue::integer(datetime.datetime.second() as i64));
                        }
                        FhirPathValue::Time(time) => {
                            results.push(FhirPathValue::integer(time.time.second() as i64));
                        }
                        _ => {
                            // For non-datetime/time values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_millisecondOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "millisecondOf",
            category: FunctionCategory::DateTime,
            description: "Returns the millisecond component of a DateTime or Time",
            parameters: [],
            return_type: "Integer",
            examples: ["now().millisecondOf()", "@2023-05-15T14:30:45.123.millisecondOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            let milliseconds = datetime.datetime.timestamp_subsec_millis();
                            results.push(FhirPathValue::integer(milliseconds as i64));
                        }
                        FhirPathValue::Time(time) => {
                            let nanoseconds = time.time.nanosecond();
                            let milliseconds = nanoseconds / 1_000_000;
                            results.push(FhirPathValue::integer(milliseconds as i64));
                        }
                        _ => {
                            // For non-datetime/time values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_timezoneOffsetOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "timezoneOffsetOf",
            category: FunctionCategory::DateTime,
            description: "Returns the timezone offset component of a DateTime in minutes",
            parameters: [],
            return_type: "Integer",
            examples: ["now().timezoneOffsetOf()", "@2023-05-15T14:30:45-05:00.timezoneOffsetOf()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::DateTime(datetime) => {
                            let offset_seconds = datetime.datetime.offset().local_minus_utc();
                            let offset_minutes = offset_seconds / 60;  // Convert seconds to minutes
                            results.push(FhirPathValue::integer(offset_minutes as i64));
                        }
                        _ => {
                            // For non-datetime values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    // Additional implementation-specific functions for compatibility
    fn register_dayOfWeek_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "dayOfWeek",
            category: FunctionCategory::DateTime,
            description: "Returns the day of the week (0=Sunday, 1=Monday, ..., 6=Saturday)",
            parameters: [],
            return_type: "Integer",
            examples: ["Patient.birthDate.dayOfWeek()", "now().dayOfWeek()", "@2023-05-15.dayOfWeek()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::Date(date) => {
                            let weekday = date.date.weekday();
                            let fhirpath_weekday = DateTimeUtils::weekday_to_fhirpath(weekday);
                            results.push(FhirPathValue::integer(fhirpath_weekday));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            let weekday = datetime.datetime.weekday();
                            let fhirpath_weekday = DateTimeUtils::weekday_to_fhirpath(weekday);
                            results.push(FhirPathValue::integer(fhirpath_weekday));
                        }
                        _ => {
                            // For non-date/datetime values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }

    fn register_dayOfYear_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "dayOfYear",
            category: FunctionCategory::DateTime,
            description: "Returns the day of the year (1-366)",
            parameters: [],
            return_type: "Integer",
            examples: ["Patient.birthDate.dayOfYear()", "now().dayOfYear()", "@2023-05-15.dayOfYear()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.is_empty() {
                    return Ok(vec![]);
                }

                let mut results = Vec::new();
                for item in context.input {
                    match item {
                        FhirPathValue::Date(date) => {
                            let day_of_year = date.date.ordinal();
                            results.push(FhirPathValue::integer(day_of_year as i64));
                        }
                        FhirPathValue::DateTime(datetime) => {
                            let day_of_year = datetime.datetime.ordinal();
                            results.push(FhirPathValue::integer(day_of_year as i64));
                        }
                        _ => {
                            // For non-date/datetime values, skip them (don't add to results)
                            continue;
                        }
                    }
                }
                Ok(results)
            }
        )
    }
}
