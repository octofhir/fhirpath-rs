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

    /// Get precision for time components
    pub fn time_precision_digits(&self) -> i64 {
        match self {
            Self::Hour => 2,        // HH
            Self::Minute => 4,      // HH:MM (ignoring separator)
            Self::Second => 6,      // HH:MM:SS (ignoring separators)
            Self::Millisecond => 9, // HH:MM:SS.sss (ignoring separators)
            _ => 2,                 // Default to hour precision for time
        }
    }
}

/// Precision-aware date type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PrecisionDate {
    /// The actual date value
    pub date: NaiveDate,
    /// The precision of the original input
    pub precision: TemporalPrecision,
}

impl PrecisionDate {
    pub fn new(date: NaiveDate, precision: TemporalPrecision) -> Self {
        Self { date, precision }
    }

    /// Get the precision as number of digits
    pub fn precision_digits(&self) -> i64 {
        self.precision.precision_digits()
    }
}

impl fmt::Display for PrecisionDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Year => write!(f, "{}", self.date.format("%Y")),
            TemporalPrecision::Month => write!(f, "{}", self.date.format("%Y-%m")),
            TemporalPrecision::Day => write!(f, "{}", self.date.format("%Y-%m-%d")),
            _ => write!(f, "{}", self.date.format("%Y-%m-%d")), // Default to day precision
        }
    }
}

/// Precision-aware datetime type  
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PrecisionDateTime {
    /// The actual datetime value
    pub datetime: DateTime<FixedOffset>,
    /// The precision of the original input
    pub precision: TemporalPrecision,
}

impl PrecisionDateTime {
    pub fn new(datetime: DateTime<FixedOffset>, precision: TemporalPrecision) -> Self {
        Self {
            datetime,
            precision,
        }
    }

    /// Get the precision as number of digits
    pub fn precision_digits(&self) -> i64 {
        self.precision.precision_digits()
    }
}

impl fmt::Display for PrecisionDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Year => write!(f, "{}", self.datetime.format("%Y")),
            TemporalPrecision::Month => write!(f, "{}", self.datetime.format("%Y-%m")),
            TemporalPrecision::Day => write!(f, "{}", self.datetime.format("%Y-%m-%d")),
            TemporalPrecision::Hour => write!(f, "{}", self.datetime.format("%Y-%m-%dT%H%z")),
            TemporalPrecision::Minute => write!(f, "{}", self.datetime.format("%Y-%m-%dT%H:%M%z")),
            TemporalPrecision::Second => {
                write!(f, "{}", self.datetime.format("%Y-%m-%dT%H:%M:%S%z"))
            }
            TemporalPrecision::Millisecond => {
                write!(f, "{}", self.datetime.format("%Y-%m-%dT%H:%M:%S%.3f%z"))
            }
        }
    }
}

/// Precision-aware time type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PrecisionTime {
    /// The actual time value
    pub time: NaiveTime,
    /// The precision of the original input
    pub precision: TemporalPrecision,
}

impl PrecisionTime {
    pub fn new(time: NaiveTime, precision: TemporalPrecision) -> Self {
        Self { time, precision }
    }

    /// Get the precision as number of digits for time
    pub fn precision_digits(&self) -> i64 {
        self.precision.time_precision_digits()
    }
}

impl fmt::Display for PrecisionTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.precision {
            TemporalPrecision::Hour => write!(f, "{}", self.time.format("%H")),
            TemporalPrecision::Minute => write!(f, "{}", self.time.format("%H:%M")),
            TemporalPrecision::Second => write!(f, "{}", self.time.format("%H:%M:%S")),
            TemporalPrecision::Millisecond => write!(f, "{}", self.time.format("%H:%M:%S%.3f")),
            _ => write!(f, "{}", self.time.format("%H:%M")), // Default to minute precision
        }
    }
}
