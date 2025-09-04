//! Utility functions and types for date/time operations in FHIRPath
//!
//! This module provides utilities for date/time arithmetic, duration parsing,
//! and common temporal operations for healthcare data processing.

use crate::core::{FhirPathValue, FhirPathError, Result};
use crate::core::error_code::{FP0052, FP0058};
use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
use chrono::{DateTime, Utc, Datelike, Timelike, Duration, NaiveDate, TimeZone};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::str::FromStr;

/// Utility functions for date/time operations
pub struct DateTimeUtils;

impl DateTimeUtils {
    /// Convert any date/datetime value to a DateTime<Utc>
    pub fn to_datetime(value: &FhirPathValue) -> Result<DateTime<Utc>> {
        match value {
            FhirPathValue::DateTime(dt) => Ok(dt.datetime.with_timezone(&Utc)),
            FhirPathValue::Date(date) => {
                let naive_datetime = date.date.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| FhirPathError::evaluation_error(
                        FP0058,
                        "Invalid date for datetime conversion"
                    ))?;
                Ok(DateTime::from_naive_utc_and_offset(naive_datetime, Utc))
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0058,
                "Value cannot be converted to datetime"
            ))
        }
    }

    /// Calculate the difference between two datetimes in the specified unit
    pub fn calculate_difference(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        unit: &str,
    ) -> Result<i64> {
        let duration = end.signed_duration_since(start);

        match unit.to_lowercase().as_str() {
            "seconds" | "second" => Ok(duration.num_seconds()),
            "minutes" | "minute" => Ok(duration.num_minutes()),
            "hours" | "hour" => Ok(duration.num_hours()),
            "days" | "day" => Ok(duration.num_days()),
            "weeks" | "week" => Ok(duration.num_weeks()),
            "months" | "month" => {
                // Approximate month calculation
                let years_diff = end.year() - start.year();
                let months_diff = end.month() as i32 - start.month() as i32;
                let total_months = years_diff * 12 + months_diff;
                
                // Adjust for partial months based on day
                let adjusted_months = if end.day() < start.day() {
                    total_months - 1
                } else {
                    total_months
                };
                
                Ok(adjusted_months as i64)
            }
            "years" | "year" => {
                let mut years_diff = end.year() - start.year();
                
                // Adjust for partial years
                if end.month() < start.month() || 
                   (end.month() == start.month() && end.day() < start.day()) {
                    years_diff -= 1;
                }
                
                Ok(years_diff as i64)
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0058,
                &format!("Unsupported time unit: '{}'", unit)
            ))
        }
    }

    /// Check if a year is a leap year
    pub fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Get the number of days in a month
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if Self::is_leap_year(year) { 29 } else { 28 },
            _ => 0,
        }
    }

    /// Convert weekday to FHIRPath convention (0=Sunday, 1=Monday, etc.)
    pub fn weekday_to_fhirpath(weekday: chrono::Weekday) -> i64 {
        match weekday {
            chrono::Weekday::Sun => 0,
            chrono::Weekday::Mon => 1,
            chrono::Weekday::Tue => 2,
            chrono::Weekday::Wed => 3,
            chrono::Weekday::Thu => 4,
            chrono::Weekday::Fri => 5,
            chrono::Weekday::Sat => 6,
        }
    }
}

/// Duration handling for date/time arithmetic
#[derive(Debug, Clone)]
pub struct DateTimeDuration {
    pub years: i32,
    pub months: i32,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

impl DateTimeDuration {
    /// Parse a duration from a quantity value
    pub fn from_quantity(value: &Decimal, unit: &str) -> Result<Self> {
        let amount = value.to_i64().ok_or_else(|| {
            FhirPathError::evaluation_error(FP0058, "Duration value too large")
        })?;

        match unit.to_lowercase().as_str() {
            "year" | "years" | "yr" => Ok(Self {
                years: amount as i32,
                months: 0,
                days: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }),
            "month" | "months" | "mo" => Ok(Self {
                years: 0,
                months: amount as i32,
                days: 0,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }),
            "week" | "weeks" | "wk" => Ok(Self {
                years: 0,
                months: 0,
                days: amount * 7,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }),
            "day" | "days" | "d" => Ok(Self {
                years: 0,
                months: 0,
                days: amount,
                hours: 0,
                minutes: 0,
                seconds: 0,
            }),
            "hour" | "hours" | "hr" | "h" => Ok(Self {
                years: 0,
                months: 0,
                days: 0,
                hours: amount,
                minutes: 0,
                seconds: 0,
            }),
            "minute" | "minutes" | "min" => Ok(Self {
                years: 0,
                months: 0,
                days: 0,
                hours: 0,
                minutes: amount,
                seconds: 0,
            }),
            "second" | "seconds" | "sec" | "s" => Ok(Self {
                years: 0,
                months: 0,
                days: 0,
                hours: 0,
                minutes: 0,
                seconds: amount,
            }),
            _ => Err(FhirPathError::evaluation_error(
                FP0058,
                &format!("Unsupported duration unit: '{}'", unit)
            ))
        }
    }

    /// Add this duration to a datetime
    pub fn add_to_datetime(&self, datetime: DateTime<Utc>) -> Result<DateTime<Utc>> {
        let mut result = datetime;

        // Add years and months first (these can change the day due to month lengths)
        if self.years != 0 || self.months != 0 {
            let new_year = result.year() + self.years;
            let total_months = result.month() as i32 + self.months;
            let (final_year, final_month) = if total_months > 12 {
                (new_year + (total_months - 1) / 12, ((total_months - 1) % 12 + 1) as u32)
            } else if total_months < 1 {
                (new_year + (total_months - 12) / 12, (total_months + 11) as u32)
            } else {
                (new_year, total_months as u32)
            };

            // Adjust day if it's beyond the last day of the target month
            let max_day = DateTimeUtils::days_in_month(final_year, final_month);
            let final_day = std::cmp::min(result.day(), max_day);

            result = result
                .with_year(final_year)
                .and_then(|dt| dt.with_month(final_month))
                .and_then(|dt| dt.with_day(final_day))
                .ok_or_else(|| FhirPathError::evaluation_error(
                    FP0052,
                    "Invalid date after year/month arithmetic"
                ))?;
        }

        // Add the remaining duration components
        let duration = Duration::days(self.days) +
                      Duration::hours(self.hours) +
                      Duration::minutes(self.minutes) +
                      Duration::seconds(self.seconds);

        result.checked_add_signed(duration)
            .ok_or_else(|| FhirPathError::evaluation_error(
                FP0052,
                "DateTime overflow in arithmetic operation"
            ))
    }

    /// Subtract this duration from a datetime
    pub fn subtract_from_datetime(&self, datetime: DateTime<Utc>) -> Result<DateTime<Utc>> {
        let negative_duration = Self {
            years: -self.years,
            months: -self.months,
            days: -self.days,
            hours: -self.hours,
            minutes: -self.minutes,
            seconds: -self.seconds,
        };
        
        negative_duration.add_to_datetime(datetime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};
    use std::str::FromStr;

    #[test]
    fn test_is_leap_year() {
        assert!(DateTimeUtils::is_leap_year(2020));
        assert!(!DateTimeUtils::is_leap_year(2021));
        assert!(DateTimeUtils::is_leap_year(2000));
        assert!(!DateTimeUtils::is_leap_year(1900));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(DateTimeUtils::days_in_month(2023, 2), 28);
        assert_eq!(DateTimeUtils::days_in_month(2020, 2), 29);
        assert_eq!(DateTimeUtils::days_in_month(2023, 4), 30);
        assert_eq!(DateTimeUtils::days_in_month(2023, 1), 31);
    }

    #[test]
    fn test_weekday_conversion() {
        assert_eq!(DateTimeUtils::weekday_to_fhirpath(chrono::Weekday::Sun), 0);
        assert_eq!(DateTimeUtils::weekday_to_fhirpath(chrono::Weekday::Mon), 1);
        assert_eq!(DateTimeUtils::weekday_to_fhirpath(chrono::Weekday::Sat), 6);
    }

    #[test]
    fn test_duration_parsing() {
        let duration = DateTimeDuration::from_quantity(
            &Decimal::from(2),
            "months"
        ).unwrap();
        assert_eq!(duration.months, 2);
        assert_eq!(duration.years, 0);

        let duration = DateTimeDuration::from_quantity(
            &Decimal::from(7),
            "days"
        ).unwrap();
        assert_eq!(duration.days, 7);

        // Test invalid unit
        let result = DateTimeDuration::from_quantity(
            &Decimal::from(1),
            "invalid_unit"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_difference() {
        let start = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

        let years_diff = DateTimeUtils::calculate_difference(start, end, "years").unwrap();
        assert_eq!(years_diff, 3);

        let days_diff = DateTimeUtils::calculate_difference(start, end, "days").unwrap();
        assert_eq!(days_diff, 1095); // 3 years * 365 + 1 leap day

        // Test invalid unit
        let result = DateTimeUtils::calculate_difference(start, end, "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_duration_arithmetic() {
        let start = Utc.with_ymd_and_hms(2023, 1, 15, 10, 30, 0).unwrap();
        
        // Test adding 1 month
        let duration = DateTimeDuration::from_quantity(&Decimal::from(1), "month").unwrap();
        let result = duration.add_to_datetime(start).unwrap();
        assert_eq!(result.month(), 2);
        assert_eq!(result.day(), 15);

        // Test adding days
        let duration = DateTimeDuration::from_quantity(&Decimal::from(10), "days").unwrap();
        let result = duration.add_to_datetime(start).unwrap();
        assert_eq!(result.day(), 25);

        // Test subtracting
        let result = duration.subtract_from_datetime(start).unwrap();
        assert_eq!(result.day(), 5);
    }
}