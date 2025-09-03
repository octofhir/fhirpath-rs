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

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
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