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

//! Precision-aware temporal types for FHIRPath

use crate::core::error_code::{FP0070, FP0071, FP0072, FP0073, FP0075, FP0079, FP0080};
use crate::core::{FhirPathError, Result};
use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Precision levels for temporal values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TemporalPrecision {
    /// Year precision (YYYY)
    Year,
    /// Month precision (YYYY-MM)
    Month,
    /// Day precision (YYYY-MM-DD)
    Day,
    /// Hour precision (YYYY-MM-DDTHH)
    Hour,
    /// Minute precision (YYYY-MM-DDTHH:MM)
    Minute,
    /// Second precision (YYYY-MM-DDTHH:MM:SS)
    Second,
    /// Millisecond precision (YYYY-MM-DDTHH:MM:SS.sss)
    Millisecond,
}

impl TemporalPrecision {
    /// Calculate the number of significant digits for this precision level
    pub fn precision_digits(&self) -> i64 {
        match self {
            Self::Year => 4,         // YYYY
            Self::Month => 6,        // YYYY-MM (ignoring separator)
            Self::Day => 8,          // YYYY-MM-DD (ignoring separators)
            Self::Hour => 10,        // YYYY-MM-DDTHH (ignoring separators)
            Self::Minute => 12,      // YYYY-MM-DDTHH:MM (ignoring separators)
            Self::Second => 14,      // YYYY-MM-DDTHH:MM:SS (ignoring separators)
            Self::Millisecond => 17, // YYYY-MM-DDTHH:MM:SS.sss (ignoring separators)
        }
    }
}

impl fmt::Display for TemporalPrecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Year => write!(f, "year"),
            Self::Month => write!(f, "month"),
            Self::Day => write!(f, "day"),
            Self::Hour => write!(f, "hour"),
            Self::Minute => write!(f, "minute"),
            Self::Second => write!(f, "second"),
            Self::Millisecond => write!(f, "millisecond"),
        }
    }
}

/// A date with precision tracking
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct PrecisionDate {
    /// The date value
    pub date: NaiveDate,
    /// The precision of this date
    pub precision: TemporalPrecision,
}

impl PrecisionDate {
    /// Create a new precision date
    pub fn new(date: NaiveDate, precision: TemporalPrecision) -> Self {
        Self { date, precision }
    }

    /// Create a date with day precision
    pub fn from_date(date: NaiveDate) -> Self {
        Self::new(date, TemporalPrecision::Day)
    }

    /// Create a date with year precision
    pub fn from_year(year: i32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, 1, 1).map(|date| Self::new(date, TemporalPrecision::Year))
    }

    /// Create a date with month precision
    pub fn from_year_month(year: i32, month: u32) -> Option<Self> {
        NaiveDate::from_ymd_opt(year, month, 1)
            .map(|date| Self::new(date, TemporalPrecision::Month))
    }

    /// Parse from ISO 8601 string with automatic precision detection
    pub fn parse(s: &str) -> Option<Self> {
        // YYYY
        if s.len() == 4
            && let Ok(year) = s.parse::<i32>()
        {
            return Self::from_year(year);
        }

        // YYYY-MM
        if s.len() == 7 && s.chars().nth(4) == Some('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2
                && let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>())
            {
                return Self::from_year_month(year, month);
            }
        }

        // YYYY-MM-DD
        if s.len() == 10
            && let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d")
        {
            return Some(Self::from_date(date));
        }

        None
    }

    /// Parse date string with comprehensive validation and proper error codes
    pub fn parse_with_validation(s: &str) -> Result<Self> {
        let trimmed = s.trim();

        // Validate basic format patterns
        if trimmed.is_empty() {
            return Err(FhirPathError::parse_error(
                FP0070,
                "Date string is empty",
                "",
                None,
            ));
        }

        // YYYY
        if trimmed.len() == 4 {
            return Self::parse_year_with_validation(trimmed);
        }

        // YYYY-MM
        if trimmed.len() == 7 && trimmed.chars().nth(4) == Some('-') {
            return Self::parse_year_month_with_validation(trimmed);
        }

        // YYYY-MM-DD
        if trimmed.len() == 10 {
            return Self::parse_date_with_validation(trimmed);
        }

        Err(FhirPathError::parse_error(
            FP0070,
            format!("Invalid date format: '{trimmed}'"),
            trimmed,
            None,
        ))
    }

    /// Parse year string with validation
    fn parse_year_with_validation(s: &str) -> Result<Self> {
        let year = s.parse::<i32>().map_err(|_| {
            FhirPathError::parse_error(FP0073, format!("Invalid year value: '{s}'"), s, None)
        })?;

        // Validate year range
        if !(1900..=2100).contains(&year) {
            return Err(FhirPathError::parse_error(
                FP0073,
                format!("Year {year} is out of valid range (1900-2100)"),
                s,
                None,
            ));
        }

        Self::from_year(year).ok_or_else(|| {
            FhirPathError::parse_error(
                FP0080,
                format!("Failed to create date from year {year}"),
                s,
                None,
            )
        })
    }

    /// Parse year-month string with validation
    fn parse_year_month_with_validation(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(FhirPathError::parse_error(
                FP0070,
                format!("Invalid year-month format: '{s}'"),
                s,
                None,
            ));
        }

        let year = parts[0].parse::<i32>().map_err(|_| {
            FhirPathError::parse_error(
                FP0073,
                format!("Invalid year value: '{}'", parts[0]),
                parts[0],
                None,
            )
        })?;
        let month = parts[1].parse::<u32>().map_err(|_| {
            FhirPathError::parse_error(
                FP0072,
                format!("Invalid month value: '{}'", parts[1]),
                parts[1],
                None,
            )
        })?;

        // Validate ranges
        if !(1900..=2100).contains(&year) {
            return Err(FhirPathError::parse_error(
                FP0073,
                format!("Year {year} is out of valid range (1900-2100)"),
                s,
                None,
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(FhirPathError::parse_error(
                FP0072,
                format!("Month {month} must be between 1 and 12"),
                s,
                None,
            ));
        }

        Self::from_year_month(year, month).ok_or_else(|| {
            FhirPathError::parse_error(
                FP0080,
                format!("Failed to create date from year {year} month {month}"),
                s,
                None,
            )
        })
    }

    /// Parse full date string with validation
    fn parse_date_with_validation(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(FhirPathError::parse_error(
                FP0070,
                format!("Invalid date format: '{s}', expected YYYY-MM-DD"),
                s,
                None,
            ));
        }

        let year = parts[0].parse::<i32>().map_err(|_| {
            FhirPathError::parse_error(
                FP0073,
                format!("Invalid year value: '{}'", parts[0]),
                parts[0],
                None,
            )
        })?;
        let month = parts[1].parse::<u32>().map_err(|_| {
            FhirPathError::parse_error(
                FP0072,
                format!("Invalid month value: '{}'", parts[1]),
                parts[1],
                None,
            )
        })?;
        let day = parts[2].parse::<u32>().map_err(|_| {
            FhirPathError::parse_error(
                FP0071,
                format!("Invalid day value: '{}'", parts[2]),
                parts[2],
                None,
            )
        })?;

        // Validate ranges
        if !(1900..=2100).contains(&year) {
            return Err(FhirPathError::parse_error(
                FP0073,
                format!("Year {year} is out of valid range (1900-2100)"),
                s,
                None,
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(FhirPathError::parse_error(
                FP0072,
                format!("Month {month} must be between 1 and 12"),
                s,
                None,
            ));
        }
        if !(1..=31).contains(&day) {
            return Err(FhirPathError::parse_error(
                FP0071,
                format!("Day {day} must be between 1 and 31"),
                s,
                None,
            ));
        }

        // Special validation for February 29th (leap year check)
        if month == 2 && day == 29 && !Self::is_leap_year(year) {
            return Err(FhirPathError::parse_error(
                FP0079,
                format!("February 29th is not valid in non-leap year {year}"),
                s,
                None,
            ));
        }

        // Validate day for specific months
        if day > Self::days_in_month(year, month) {
            return Err(FhirPathError::parse_error(
                FP0071,
                format!("Day {day} is not valid for month {month} in year {year}"),
                s,
                None,
            ));
        }

        // Use chrono to parse and validate
        NaiveDate::from_ymd_opt(year, month, day)
            .map(Self::from_date)
            .ok_or_else(|| {
                FhirPathError::parse_error(
                    FP0080,
                    format!("Invalid date: {year}-{month:02}-{day:02}"),
                    s,
                    None,
                )
            })
    }

    /// Check if a year is a leap year
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Get number of days in a month for a given year
    fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0, // Invalid month
        }
    }

    /// Convert this precision date to a date range (start, end) representing all possible dates at this precision
    pub fn to_date_range(&self) -> (NaiveDate, NaiveDate) {
        match self.precision {
            TemporalPrecision::Year => {
                let start = NaiveDate::from_ymd_opt(self.date.year(), 1, 1).unwrap();
                let end = NaiveDate::from_ymd_opt(self.date.year(), 12, 31).unwrap();
                (start, end)
            }
            TemporalPrecision::Month => {
                let start =
                    NaiveDate::from_ymd_opt(self.date.year(), self.date.month(), 1).unwrap();
                // Last day of the month
                let next_month = if self.date.month() == 12 {
                    NaiveDate::from_ymd_opt(self.date.year() + 1, 1, 1).unwrap()
                } else {
                    NaiveDate::from_ymd_opt(self.date.year(), self.date.month() + 1, 1).unwrap()
                };
                let end = next_month.pred_opt().unwrap();
                (start, end)
            }
            TemporalPrecision::Day => {
                // For day precision, start and end are the same
                (self.date, self.date)
            }
            _ => {
                // For finer precisions, treat as day precision
                (self.date, self.date)
            }
        }
    }
}

impl fmt::Display for PrecisionDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Year => write!(f, "{}", self.date.format("%Y")),
            TemporalPrecision::Month => write!(f, "{}", self.date.format("%Y-%m")),
            TemporalPrecision::Day => write!(f, "{}", self.date.format("%Y-%m-%d")),
            _ => write!(f, "{}", self.date.format("%Y-%m-%d")), // Fallback to day format
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for PrecisionDate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;

        // FHIRPath temporal comparison rules:
        // When two dates have different precisions and overlap, comparison is undefined (return None)
        // When they don't overlap, we can compare the ranges

        let self_range = self.to_date_range();
        let other_range = other.to_date_range();

        // Check if ranges overlap
        if self_range.0 <= other_range.1 && other_range.0 <= self_range.1 {
            // Ranges overlap
            if self.precision != other.precision {
                // Different precisions and overlapping - undefined comparison
                return None;
            }
            // Same precision, can compare normally
            return Some(self.date.cmp(&other.date));
        }

        // No overlap, can compare the ranges
        if self_range.1 < other_range.0 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }
}

impl Ord for PrecisionDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // For Ord, we need a total ordering. We'll fall back to comparing date then precision
        // This is only used when partial_cmp returns Some(_)
        match self.date.cmp(&other.date) {
            std::cmp::Ordering::Equal => self.precision.cmp(&other.precision),
            other => other,
        }
    }
}

/// A datetime with precision tracking
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct PrecisionDateTime {
    /// The datetime value (always stored with timezone)
    pub datetime: DateTime<FixedOffset>,
    /// The precision of this datetime
    pub precision: TemporalPrecision,
    /// Whether the original literal included an explicit timezone (Z, +HH:MM, -HH:MM)
    #[serde(default)]
    pub tz_specified: bool,
}

impl PrecisionDateTime {
    /// Create a new precision datetime (assumes timezone was explicitly specified)
    pub fn new(datetime: DateTime<FixedOffset>, precision: TemporalPrecision) -> Self {
        Self {
            datetime,
            precision,
            tz_specified: true,
        }
    }
    /// Create a new precision datetime with explicit timezone presence flag
    pub fn new_with_tz(
        datetime: DateTime<FixedOffset>,
        precision: TemporalPrecision,
        tz_specified: bool,
    ) -> Self {
        Self {
            datetime,
            precision,
            tz_specified,
        }
    }

    /// Create a datetime with full precision
    pub fn from_datetime(datetime: DateTime<FixedOffset>) -> Self {
        Self::new(datetime, TemporalPrecision::Millisecond)
    }

    /// Get the date component
    pub fn date(&self) -> PrecisionDate {
        let naive_date = self.datetime.date_naive();
        let precision = match self.precision {
            TemporalPrecision::Year => TemporalPrecision::Year,
            TemporalPrecision::Month => TemporalPrecision::Month,
            _ => TemporalPrecision::Day,
        };
        PrecisionDate::new(naive_date, precision)
    }

    /// Parse from ISO 8601 datetime string with timezone
    pub fn parse(s: &str) -> Option<Self> {
        // Detect if original string has explicit timezone information
        let has_tz =
            s.ends_with('Z') || s.contains('+') || s.rfind('-').is_some_and(|pos| pos > 10);

        // Support trailing 'Z' (UTC) by normalizing to +00:00
        let s_norm: std::borrow::Cow<str> = if s.ends_with('Z') {
            let mut owned = s.to_string();
            owned.pop();
            owned.push_str("+00:00");
            std::borrow::Cow::Owned(owned)
        } else {
            std::borrow::Cow::Borrowed(s)
        };

        // Try different datetime formats with timezone first
        let tz_formats = [
            "%Y-%m-%dT%H:%M:%S%:z",     // YYYY-MM-DDTHH:MM:SS+TZ
            "%Y-%m-%dT%H:%M:%S%.3f%:z", // YYYY-MM-DDTHH:MM:SS.sss+TZ
            "%Y-%m-%dT%H:%M%:z",        // YYYY-MM-DDTHH:MM+TZ
            "%Y-%m-%dT%H%:z",           // YYYY-MM-DDTHH+TZ
        ];

        for (i, format) in tz_formats.iter().enumerate() {
            if let Ok(dt) = DateTime::parse_from_str(&s_norm, format) {
                let precision = match i {
                    0 => TemporalPrecision::Second,
                    1 => TemporalPrecision::Millisecond,
                    2 => TemporalPrecision::Minute,
                    3 => TemporalPrecision::Hour,
                    _ => TemporalPrecision::Second,
                };
                return Some(Self::new_with_tz(dt, precision, has_tz));
            }
        }

        // Fallback: accept datetimes without timezone by assuming UTC offset
        let naive_formats = [
            "%Y-%m-%dT%H:%M:%S",     // YYYY-MM-DDTHH:MM:SS
            "%Y-%m-%dT%H:%M:%S%.3f", // YYYY-MM-DDTHH:MM:SS.sss
            "%Y-%m-%dT%H:%M",        // YYYY-MM-DDTHH:MM
            "%Y-%m-%dT%H",           // YYYY-MM-DDTHH
        ];

        for (i, format) in naive_formats.iter().enumerate() {
            if let Ok(ndt) = NaiveDateTime::parse_from_str(&s_norm, format) {
                let offset = FixedOffset::east_opt(0)?;
                let dt: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(ndt, offset);
                let precision = match i {
                    0 => TemporalPrecision::Second,
                    1 => TemporalPrecision::Millisecond,
                    2 => TemporalPrecision::Minute,
                    3 => TemporalPrecision::Hour,
                    _ => TemporalPrecision::Second,
                };
                return Some(Self::new_with_tz(dt, precision, has_tz));
            }
        }

        None
    }

    /// Parse datetime string with comprehensive validation and proper error codes
    pub fn parse_with_validation(s: &str) -> Result<Self> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(FhirPathError::parse_error(
                FP0075,
                "DateTime string is empty",
                "",
                None,
            ));
        }

        // Detect if original string has explicit timezone information
        let has_tz = trimmed.ends_with('Z')
            || trimmed.contains('+')
            || trimmed.rfind('-').is_some_and(|pos| pos > 10);

        // Support trailing 'Z' (UTC) by normalizing to +00:00
        let s_norm: std::borrow::Cow<str> = if trimmed.ends_with('Z') {
            let mut owned = trimmed.to_string();
            owned.pop();
            owned.push_str("+00:00");
            std::borrow::Cow::Owned(owned)
        } else {
            std::borrow::Cow::Borrowed(trimmed)
        };

        // Try different datetime formats with timezone first
        let tz_formats = [
            ("%Y-%m-%dT%H:%M:%S%:z", TemporalPrecision::Second), // YYYY-MM-DDTHH:MM:SS+TZ
            ("%Y-%m-%dT%H:%M:%S%.3f%:z", TemporalPrecision::Millisecond), // YYYY-MM-DDTHH:MM:SS.sss+TZ
            ("%Y-%m-%dT%H:%M%:z", TemporalPrecision::Minute),             // YYYY-MM-DDTHH:MM+TZ
            ("%Y-%m-%dT%H%:z", TemporalPrecision::Hour),                  // YYYY-MM-DDTHH+TZ
        ];

        for (format, precision) in &tz_formats {
            if let Ok(dt) = DateTime::parse_from_str(&s_norm, format) {
                return Ok(Self::new_with_tz(dt, *precision, has_tz));
            }
        }

        // Fallback: accept datetimes without timezone by assuming UTC
        let naive_formats = [
            ("%Y-%m-%dT%H:%M:%S", TemporalPrecision::Second), // YYYY-MM-DDTHH:MM:SS
            ("%Y-%m-%dT%H:%M:%S%.3f", TemporalPrecision::Millisecond), // YYYY-MM-DDTHH:MM:SS.sss
            ("%Y-%m-%dT%H:%M", TemporalPrecision::Minute),    // YYYY-MM-DDTHH:MM
            ("%Y-%m-%dT%H", TemporalPrecision::Hour),         // YYYY-MM-DDTHH
        ];

        for (format, precision) in &naive_formats {
            if let Ok(ndt) = NaiveDateTime::parse_from_str(&s_norm, format)
                && let Some(offset) = FixedOffset::east_opt(0)
            {
                let dt = DateTime::from_naive_utc_and_offset(ndt, offset);
                // Additional validation can go here if needed
                return Ok(Self::new_with_tz(dt, *precision, has_tz));
            }
        }

        Err(FhirPathError::parse_error(
            FP0075,
            format!(
                "Invalid datetime format: '{trimmed}'. Expected ISO 8601 format like YYYY-MM-DDTHH:MM:SSZ"
            ),
            trimmed,
            None,
        ))
    }

    /// Convert to chrono `DateTime<Utc>`
    pub fn to_chrono_datetime(&self) -> Result<DateTime<Utc>> {
        Ok(self.datetime.with_timezone(&Utc))
    }

    /// Create from chrono DateTime with specified precision
    pub fn from_chrono_datetime(dt: &DateTime<Utc>, precision: TemporalPrecision) -> Self {
        let fixed_offset_dt = dt.with_timezone(&FixedOffset::east_opt(0).unwrap());
        Self::new(fixed_offset_dt, precision)
    }

    /// Convert this precision datetime to a datetime range (start, end) representing all possible datetimes at this precision
    pub fn to_datetime_range(&self) -> (DateTime<FixedOffset>, DateTime<FixedOffset>) {
        use chrono::Timelike;

        match self.precision {
            TemporalPrecision::Year => {
                let start = self
                    .datetime
                    .with_month(1)
                    .unwrap()
                    .with_day(1)
                    .unwrap()
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                let end = self
                    .datetime
                    .with_month(12)
                    .unwrap()
                    .with_day(31)
                    .unwrap()
                    .with_hour(23)
                    .unwrap()
                    .with_minute(59)
                    .unwrap()
                    .with_second(59)
                    .unwrap()
                    .with_nanosecond(999_999_999)
                    .unwrap();
                (start, end)
            }
            TemporalPrecision::Month => {
                let start = self
                    .datetime
                    .with_day(1)
                    .unwrap()
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                // Last day of the month
                let next_month = if self.datetime.month() == 12 {
                    self.datetime
                        .with_year(self.datetime.year() + 1)
                        .unwrap()
                        .with_month(1)
                        .unwrap()
                } else {
                    self.datetime.with_month(self.datetime.month() + 1).unwrap()
                };
                let end = next_month
                    .with_day(1)
                    .unwrap()
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
                    - chrono::Duration::nanoseconds(1);
                (start, end)
            }
            TemporalPrecision::Day => {
                let start = self
                    .datetime
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                let end = self
                    .datetime
                    .with_hour(23)
                    .unwrap()
                    .with_minute(59)
                    .unwrap()
                    .with_second(59)
                    .unwrap()
                    .with_nanosecond(999_999_999)
                    .unwrap();
                (start, end)
            }
            TemporalPrecision::Hour => {
                let start = self
                    .datetime
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                let end = self
                    .datetime
                    .with_minute(59)
                    .unwrap()
                    .with_second(59)
                    .unwrap()
                    .with_nanosecond(999_999_999)
                    .unwrap();
                (start, end)
            }
            TemporalPrecision::Minute => {
                let start = self
                    .datetime
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                let end = self
                    .datetime
                    .with_second(59)
                    .unwrap()
                    .with_nanosecond(999_999_999)
                    .unwrap();
                (start, end)
            }
            TemporalPrecision::Second => {
                let start = self.datetime.with_nanosecond(0).unwrap();
                let end = self.datetime.with_nanosecond(999_999_999).unwrap();
                (start, end)
            }
            TemporalPrecision::Millisecond => {
                // For millisecond precision, start and end are very close
                (self.datetime, self.datetime)
            }
        }
    }
}

impl fmt::Display for PrecisionDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Year => write!(f, "{}", self.datetime.format("%Y")),
            TemporalPrecision::Month => write!(f, "{}", self.datetime.format("%Y-%m")),
            TemporalPrecision::Day => write!(f, "{}", self.datetime.format("%Y-%m-%d")),
            TemporalPrecision::Hour => write!(f, "{}", self.datetime.format("%Y-%m-%dT%H%:z")),
            TemporalPrecision::Minute => {
                write!(f, "{}", self.datetime.format("%Y-%m-%dT%H:%M%:z"))
            }
            TemporalPrecision::Second => {
                write!(f, "{}", self.datetime.format("%Y-%m-%dT%H:%M:%S%:z"))
            }
            TemporalPrecision::Millisecond => {
                write!(f, "{}", self.datetime.format("%Y-%m-%dT%H:%M:%S%.3f%:z"))
            }
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for PrecisionDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Different timezone specification → indeterminate
        if self.tz_specified != other.tz_specified {
            return None;
        }

        // Same precision → compare exact instants
        if self.precision == other.precision {
            return Some(self.datetime.cmp(&other.datetime));
        }

        // Special case: Second and Millisecond are considered the same precision per FHIR spec
        if matches!(
            (self.precision, other.precision),
            (TemporalPrecision::Second, TemporalPrecision::Millisecond)
                | (TemporalPrecision::Millisecond, TemporalPrecision::Second)
        ) {
            return Some(self.datetime.cmp(&other.datetime));
        }

        // Range-based comparison for differing precisions per FHIRPath
        let (self_start, self_end) = self.to_datetime_range();
        let (other_start, other_end) = other.to_datetime_range();

        if self_end < other_start {
            return Some(std::cmp::Ordering::Less);
        }
        if self_start > other_end {
            return Some(std::cmp::Ordering::Greater);
        }

        // Overlapping ranges → indeterminate
        None
    }
}

impl Ord for PrecisionDateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // For Ord, we need a total ordering. We'll fall back to comparing datetime then precision
        match self.datetime.cmp(&other.datetime) {
            std::cmp::Ordering::Equal => self.precision.cmp(&other.precision),
            other => other,
        }
    }
}

/// A time with precision tracking
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct PrecisionTime {
    /// The time value
    pub time: NaiveTime,
    /// The precision of this time
    pub precision: TemporalPrecision,
}

impl PrecisionTime {
    /// Create a new precision time
    pub fn new(time: NaiveTime, precision: TemporalPrecision) -> Self {
        Self { time, precision }
    }

    /// Create a time with second precision
    pub fn from_time(time: NaiveTime) -> Self {
        Self::new(time, TemporalPrecision::Second)
    }

    /// Create a time with specified precision
    pub fn from_time_with_precision(time: NaiveTime, precision: TemporalPrecision) -> Self {
        Self::new(time, precision)
    }

    /// Parse from time string with automatic precision detection
    pub fn parse(s: &str) -> Option<Self> {
        // HH
        if s.len() == 2
            && let Ok(hour) = s.parse::<u32>()
            && let Some(time) = NaiveTime::from_hms_opt(hour, 0, 0)
        {
            return Some(Self::new(time, TemporalPrecision::Hour));
        }

        // HH:MM
        if s.len() == 5
            && s.chars().nth(2) == Some(':')
            && let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M")
        {
            return Some(Self::new(time, TemporalPrecision::Minute));
        }

        // HH:MM:SS
        if s.len() == 8
            && let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S")
        {
            return Some(Self::new(time, TemporalPrecision::Second));
        }

        // HH:MM:SS.sss
        if s.len() == 12
            && s.chars().nth(8) == Some('.')
            && let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S%.3f")
        {
            return Some(Self::new(time, TemporalPrecision::Millisecond));
        }

        None
    }

    /// Convert this precision time to a time range (start, end) representing all possible times at this precision
    pub fn to_time_range(&self) -> (NaiveTime, NaiveTime) {
        match self.precision {
            TemporalPrecision::Hour => {
                let start = self
                    .time
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                let end = self
                    .time
                    .with_minute(59)
                    .unwrap()
                    .with_second(59)
                    .unwrap()
                    .with_nanosecond(999_999_999)
                    .unwrap();
                (start, end)
            }
            TemporalPrecision::Minute => {
                let start = self
                    .time
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap();
                let end = self
                    .time
                    .with_second(59)
                    .unwrap()
                    .with_nanosecond(999_999_999)
                    .unwrap();
                (start, end)
            }
            TemporalPrecision::Second => {
                let start = self.time.with_nanosecond(0).unwrap();
                let end = self.time.with_nanosecond(999_999_999).unwrap();
                (start, end)
            }
            TemporalPrecision::Millisecond => {
                // For millisecond precision, start and end are very close
                (self.time, self.time)
            }
            _ => {
                // For other precisions, treat as second precision
                let start = self.time.with_nanosecond(0).unwrap();
                let end = self.time.with_nanosecond(999_999_999).unwrap();
                (start, end)
            }
        }
    }
}

impl fmt::Display for PrecisionTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Hour => write!(f, "T{}", self.time.format("%H")),
            TemporalPrecision::Minute => write!(f, "T{}", self.time.format("%H:%M")),
            TemporalPrecision::Second => write!(f, "T{}", self.time.format("%H:%M:%S")),
            TemporalPrecision::Millisecond => write!(f, "T{}", self.time.format("%H:%M:%S%.3f")),
            _ => write!(f, "T{}", self.time.format("%H:%M:%S")), // Fallback
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for PrecisionTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Same precision - can compare normally
        if self.precision == other.precision {
            return Some(self.time.cmp(&other.time));
        }

        // Special case: Second and Millisecond are considered the same precision per FHIR spec
        if matches!(
            (self.precision, other.precision),
            (TemporalPrecision::Second, TemporalPrecision::Millisecond)
                | (TemporalPrecision::Millisecond, TemporalPrecision::Second)
        ) {
            return Some(self.time.cmp(&other.time));
        }

        // Different precisions - return None (indeterminate)
        None
    }
}

impl Ord for PrecisionTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // For Ord, we need a total ordering. We'll fall back to comparing time then precision
        match self.time.cmp(&other.time) {
            std::cmp::Ordering::Equal => self.precision.cmp(&other.precision),
            other => other,
        }
    }
}

/// Utility functions for parsing temporal strings from FHIR resources
pub mod parsing {
    use super::{FP0070, FP0075, FhirPathError, PrecisionDate, PrecisionDateTime, Result};

    /// Parse a date or datetime string and return the date component with validation
    /// This is the main function used by yearOf/monthOf/dayOf functions
    pub fn parse_date_or_datetime_string(s: &str) -> Result<PrecisionDate> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(FhirPathError::parse_error(
                FP0070,
                "Date/DateTime string is empty",
                "",
                None,
            ));
        }

        // Check if this looks like a datetime string (contains 'T')
        if trimmed.contains('T') {
            // Try parsing as datetime first, then extract date component
            match PrecisionDateTime::parse_with_validation(trimmed) {
                Ok(datetime) => Ok(datetime.date()),
                Err(_) => {
                    // If datetime parsing failed, it might be a malformed datetime
                    Err(FhirPathError::parse_error(
                        FP0075,
                        format!("Invalid datetime format: '{trimmed}'"),
                        trimmed,
                        None,
                    ))
                }
            }
        } else {
            // Try parsing as pure date
            PrecisionDate::parse_with_validation(trimmed)
        }
    }

    /// Parse a datetime string and return the full datetime with validation
    pub fn parse_datetime_string(s: &str) -> Result<PrecisionDateTime> {
        PrecisionDateTime::parse_with_validation(s)
    }

    /// Parse a date string and return the date with validation
    pub fn parse_date_string(s: &str) -> Result<PrecisionDate> {
        PrecisionDate::parse_with_validation(s)
    }
}

// FHIR-compliant equality implementations
// According to FHIR spec: "seconds and milliseconds are considered a single precision using a decimal, with decimal equality semantics"

impl PartialEq for PrecisionDate {
    fn eq(&self, other: &Self) -> bool {
        // For dates, precision must match exactly (no special second/millisecond rule for dates)
        if self.precision != other.precision {
            return false;
        }
        self.date == other.date
    }
}

impl PartialEq for PrecisionDateTime {
    fn eq(&self, other: &Self) -> bool {
        // Check if precisions are compatible according to FHIR spec
        let precision_compatible = match (self.precision, other.precision) {
            // Exact match
            (a, b) if a == b => true,
            // Second and Millisecond are considered the same precision with decimal semantics
            (TemporalPrecision::Second, TemporalPrecision::Millisecond) => true,
            (TemporalPrecision::Millisecond, TemporalPrecision::Second) => true,
            _ => false,
        };

        if !precision_compatible {
            return false;
        }

        // Compare the datetime values considering precision
        match (self.precision, other.precision) {
            // Both have same precision
            (a, b) if a == b => self.datetime == other.datetime,
            // Second vs Millisecond: use decimal semantics
            (TemporalPrecision::Second, TemporalPrecision::Millisecond)
            | (TemporalPrecision::Millisecond, TemporalPrecision::Second) => {
                // Compare truncated to seconds precision
                self.datetime.timestamp() == other.datetime.timestamp()
                    && self.datetime.timestamp_subsec_millis()
                        == other.datetime.timestamp_subsec_millis()
            }
            _ => false,
        }
    }
}

impl PartialEq for PrecisionTime {
    fn eq(&self, other: &Self) -> bool {
        // Check if precisions are compatible according to FHIR spec
        let precision_compatible = match (self.precision, other.precision) {
            // Exact match
            (a, b) if a == b => true,
            // Second and Millisecond are considered the same precision with decimal semantics
            (TemporalPrecision::Second, TemporalPrecision::Millisecond) => true,
            (TemporalPrecision::Millisecond, TemporalPrecision::Second) => true,
            _ => false,
        };

        if !precision_compatible {
            return false;
        }

        // Compare the time values considering precision
        match (self.precision, other.precision) {
            // Both have same precision
            (a, b) if a == b => self.time == other.time,
            // Second vs Millisecond: use decimal semantics
            (TemporalPrecision::Second, TemporalPrecision::Millisecond)
            | (TemporalPrecision::Millisecond, TemporalPrecision::Second) => {
                // Compare hours, minutes, and seconds
                self.time.hour() == other.time.hour() &&
                self.time.minute() == other.time.minute() &&
                self.time.second() == other.time.second() &&
                // For second/millisecond compatibility, compare nanoseconds as milliseconds
                (self.time.nanosecond() / 1_000_000) == (other.time.nanosecond() / 1_000_000)
            }
            _ => false,
        }
    }
}

/// A duration expressed in calendar units
///
/// This represents durations in terms of calendar units (year, month, day, etc.)
/// rather than fixed time intervals. This is important for accurate temporal
/// arithmetic as calendar months and years have variable lengths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarDuration {
    /// The magnitude of the duration
    pub value: i64,
    /// The calendar unit
    pub unit: crate::core::CalendarUnit,
}

impl CalendarDuration {
    /// Create a new calendar duration
    pub fn new(value: i64, unit: crate::core::CalendarUnit) -> Self {
        Self { value, unit }
    }

    /// Create a duration from a string like "5 days" or "2 years"
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!(
                    "Invalid calendar duration format: '{s}'. Expected format: '<number> <unit>'"
                ),
            ));
        }

        let value = parts[0].parse::<i64>().map_err(|_| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("Invalid number in calendar duration: '{}'", parts[0]),
            )
        })?;

        let unit = crate::core::CalendarUnit::from_str(parts[1]).ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("Invalid calendar unit: '{}'", parts[1]),
            )
        })?;

        Ok(Self::new(value, unit))
    }

    /// Convert to total milliseconds if the unit has a fixed duration
    pub fn to_milliseconds(&self) -> Option<i64> {
        self.unit.to_milliseconds().map(|ms| self.value * ms)
    }

    /// Check if this duration is zero
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// Get the absolute value of this duration
    pub fn abs(&self) -> Self {
        Self {
            value: self.value.abs(),
            unit: self.unit,
        }
    }

    /// Negate this duration
    pub fn negate(&self) -> Self {
        Self {
            value: -self.value,
            unit: self.unit,
        }
    }

    /// Add calendar duration to a precision date
    pub fn add_to_date(&self, date: &PrecisionDate) -> Result<PrecisionDate> {
        use crate::core::CalendarUnit;
        use chrono::{Datelike, NaiveDate};

        match self.unit {
            CalendarUnit::Year => {
                let new_year = date.date.year() + self.value as i32;
                let new_date = date.date.with_year(new_year).ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0076,
                        format!("Invalid date after adding {} years", self.value),
                    )
                })?;
                Ok(PrecisionDate {
                    date: new_date,
                    precision: date.precision,
                })
            }
            CalendarUnit::Month => {
                let total_months =
                    date.date.year() as i64 * 12 + date.date.month() as i64 - 1 + self.value;
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;

                let new_date = NaiveDate::from_ymd_opt(new_year, new_month, date.date.day())
                    .or_else(|| {
                        // Handle month-end overflow (e.g., Jan 31 + 1 month = Feb 28/29)
                        let last_day_of_month = NaiveDate::from_ymd_opt(new_year, new_month + 1, 1)
                            .unwrap_or_else(|| NaiveDate::from_ymd_opt(new_year + 1, 1, 1).unwrap())
                            .pred_opt()
                            .unwrap();
                        Some(last_day_of_month)
                    })
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0076,
                            format!("Invalid date after adding {} months", self.value),
                        )
                    })?;
                Ok(PrecisionDate {
                    date: new_date,
                    precision: date.precision,
                })
            }
            CalendarUnit::Week => {
                let new_date = date.date + chrono::Duration::weeks(self.value);
                Ok(PrecisionDate {
                    date: new_date,
                    precision: date.precision,
                })
            }
            CalendarUnit::Day => {
                let new_date = date.date + chrono::Duration::days(self.value);
                Ok(PrecisionDate {
                    date: new_date,
                    precision: date.precision,
                })
            }
            _ => {
                // For time units, convert to milliseconds if possible and add as duration
                if let Some(ms) = self.to_milliseconds() {
                    let new_date = date.date + chrono::Duration::milliseconds(ms);
                    Ok(PrecisionDate {
                        date: new_date,
                        precision: date.precision,
                    })
                } else {
                    Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        format!("Cannot add {} {} to date", self.value, self.unit),
                    ))
                }
            }
        }
    }

    /// Add calendar duration to a precision datetime
    pub fn add_to_datetime(&self, datetime: &PrecisionDateTime) -> Result<PrecisionDateTime> {
        use crate::core::CalendarUnit;

        match self.unit {
            CalendarUnit::Year | CalendarUnit::Month => {
                // Use date addition for year/month units - extract naive date for calendar arithmetic
                let naive_date = datetime.datetime.date_naive();
                let date_part = PrecisionDate {
                    date: naive_date,
                    precision: datetime.precision,
                };
                let new_date = self.add_to_date(&date_part)?;

                // Reconstruct datetime with the same time part and timezone
                let time_part = datetime.datetime.time();
                let naive_datetime = new_date.date.and_time(time_part);
                let new_datetime = datetime
                    .datetime
                    .timezone()
                    .from_local_datetime(&naive_datetime)
                    .single()
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0076,
                            "Cannot create datetime after calendar arithmetic".to_string(),
                        )
                    })?;

                Ok(PrecisionDateTime {
                    datetime: new_datetime,
                    precision: datetime.precision,
                    tz_specified: datetime.tz_specified,
                })
            }
            _ => {
                // For other units, convert to milliseconds and add as duration
                if let Some(ms) = self.to_milliseconds() {
                    let new_datetime = datetime.datetime + chrono::Duration::milliseconds(ms);
                    Ok(PrecisionDateTime {
                        datetime: new_datetime,
                        precision: datetime.precision,
                        tz_specified: datetime.tz_specified,
                    })
                } else {
                    Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0053,
                        format!("Cannot add {} {} to datetime", self.value, self.unit),
                    ))
                }
            }
        }
    }
}

impl fmt::Display for CalendarDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_date_parsing() {
        assert_eq!(
            PrecisionDate::parse("2023").unwrap().precision,
            TemporalPrecision::Year
        );
        assert_eq!(
            PrecisionDate::parse("2023-12").unwrap().precision,
            TemporalPrecision::Month
        );
        assert_eq!(
            PrecisionDate::parse("2023-12-25").unwrap().precision,
            TemporalPrecision::Day
        );
    }

    #[test]
    fn test_precision_time_parsing() {
        assert_eq!(
            PrecisionTime::parse("14").unwrap().precision,
            TemporalPrecision::Hour
        );
        assert_eq!(
            PrecisionTime::parse("14:30").unwrap().precision,
            TemporalPrecision::Minute
        );
        assert_eq!(
            PrecisionTime::parse("14:30:45").unwrap().precision,
            TemporalPrecision::Second
        );
        assert_eq!(
            PrecisionTime::parse("14:30:45.123").unwrap().precision,
            TemporalPrecision::Millisecond
        );
    }

    #[test]
    fn test_precision_digits() {
        assert_eq!(TemporalPrecision::Year.precision_digits(), 4);
        assert_eq!(TemporalPrecision::Month.precision_digits(), 6);
        assert_eq!(TemporalPrecision::Day.precision_digits(), 8);
        assert_eq!(TemporalPrecision::Second.precision_digits(), 14);
        assert_eq!(TemporalPrecision::Millisecond.precision_digits(), 17);
    }
}
