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

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, NaiveDateTime};
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
        if s.len() == 4 {
            if let Ok(year) = s.parse::<i32>() {
                return Self::from_year(year);
            }
        }
        
        // YYYY-MM
        if s.len() == 7 && s.chars().nth(4) == Some('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                    return Self::from_year_month(year, month);
                }
            }
        }
        
        // YYYY-MM-DD
        if s.len() == 10 {
            if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                return Some(Self::from_date(date));
            }
        }
        
        None
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

/// A datetime with precision tracking
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PrecisionDateTime {
    /// The datetime value (always stored with timezone)
    pub datetime: DateTime<FixedOffset>,
    /// The precision of this datetime
    pub precision: TemporalPrecision,
}

impl PrecisionDateTime {
    /// Create a new precision datetime
    pub fn new(datetime: DateTime<FixedOffset>, precision: TemporalPrecision) -> Self {
        Self { datetime, precision }
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
            "%Y-%m-%dT%H:%M:%S%:z",         // YYYY-MM-DDTHH:MM:SS+TZ
            "%Y-%m-%dT%H:%M:%S%.3f%:z",     // YYYY-MM-DDTHH:MM:SS.sss+TZ
            "%Y-%m-%dT%H:%M%:z",            // YYYY-MM-DDTHH:MM+TZ
            "%Y-%m-%dT%H%:z",               // YYYY-MM-DDTHH+TZ
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
                return Some(Self::new(dt, precision));
            }
        }

        // Fallback: accept datetimes without timezone by assuming UTC offset
        let naive_formats = [
            "%Y-%m-%dT%H:%M:%S",         // YYYY-MM-DDTHH:MM:SS
            "%Y-%m-%dT%H:%M:%S%.3f",     // YYYY-MM-DDTHH:MM:SS.sss
            "%Y-%m-%dT%H:%M",            // YYYY-MM-DDTHH:MM
            "%Y-%m-%dT%H",               // YYYY-MM-DDTHH
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
                return Some(Self::new(dt, precision));
            }
        }

        None
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

/// A time with precision tracking
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

    /// Parse from time string with automatic precision detection
    pub fn parse(s: &str) -> Option<Self> {
        // HH
        if s.len() == 2 {
            if let Ok(hour) = s.parse::<u32>() {
                if let Some(time) = NaiveTime::from_hms_opt(hour, 0, 0) {
                    return Some(Self::new(time, TemporalPrecision::Hour));
                }
            }
        }
        
        // HH:MM
        if s.len() == 5 && s.chars().nth(2) == Some(':') {
            if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M") {
                return Some(Self::new(time, TemporalPrecision::Minute));
            }
        }
        
        // HH:MM:SS
        if s.len() == 8 {
            if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S") {
                return Some(Self::new(time, TemporalPrecision::Second));
            }
        }
        
        // HH:MM:SS.sss
        if s.len() == 12 && s.chars().nth(8) == Some('.') {
            if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S%.3f") {
                return Some(Self::new(time, TemporalPrecision::Millisecond));
            }
        }
        
        None
    }
}

impl fmt::Display for PrecisionTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Hour => write!(f, "{}", self.time.format("%H")),
            TemporalPrecision::Minute => write!(f, "{}", self.time.format("%H:%M")),
            TemporalPrecision::Second => write!(f, "{}", self.time.format("%H:%M:%S")),
            TemporalPrecision::Millisecond => write!(f, "{}", self.time.format("%H:%M:%S%.3f")),
            _ => write!(f, "{}", self.time.format("%H:%M:%S")), // Fallback
        }
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
