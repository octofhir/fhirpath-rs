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

//! Arithmetic operators for FHIRPath expressions

use super::super::operator::{
    Associativity, FhirPathOperator, OperatorError, OperatorRegistry, OperatorResult,
};
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::signature::OperatorSignature;
use octofhir_ucum;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// Time unit classification for date/datetime arithmetic
#[derive(Debug, Clone, Copy, PartialEq)]
enum TimeUnitType {
    Year,
    Month,
    Week,
    Day,
    Hour,
    Minute,
    Second,
    Millisecond,
}

/// Map FHIRPath calendar units to UCUM equivalents
static CALENDAR_TO_UCUM: &[(&str, &str)] = &[
    ("year", "a"),         // annum
    ("month", "mo"),       // month
    ("week", "wk"),        // week
    ("day", "d"),          // day
    ("hour", "h"),         // hour
    ("minute", "min"),     // minute
    ("second", "s"),       // second
    ("millisecond", "ms"), // millisecond
];

/// Convert FHIRPath calendar unit to UCUM equivalent
pub fn fhirpath_to_ucum(fhirpath_unit: &str) -> Option<&'static str> {
    // Check static mapping first for performance
    CALENDAR_TO_UCUM
        .iter()
        .find(|(fhir, _)| *fhir == fhirpath_unit)
        .map(|(_, ucum)| *ucum)
}

/// Convert UCUM unit to FHIRPath calendar unit equivalent
pub fn ucum_to_fhirpath(ucum_unit: &str) -> Option<&'static str> {
    CALENDAR_TO_UCUM
        .iter()
        .find(|(_, ucum)| *ucum == ucum_unit)
        .map(|(fhir, _)| *fhir)
}

/// Get canonical form of a unit using UCUM library
pub fn get_canonical_unit(unit: &str) -> String {
    // First try FHIRPath calendar mapping
    if let Some(ucum_unit) = fhirpath_to_ucum(unit) {
        return ucum_unit.to_string();
    }

    // Then try UCUM canonicalization
    match octofhir_ucum::canonicalize_expression(unit) {
        Ok(canonical) => canonical.to_string(),
        Err(_) => unit.to_string(), // Fallback to original
    }
}

/// Check if a unit is a valid UCUM unit
pub fn is_valid_ucum_unit(unit: &str) -> bool {
    octofhir_ucum::validate(unit).is_ok()
}

/// Helper function to classify a unit string using UCUM
fn classify_time_unit(unit_str: &str) -> Option<TimeUnitType> {
    // First check exact matches for UCUM standard units
    match unit_str {
        // Supported UCUM units for date arithmetic
        "wk" => return Some(TimeUnitType::Week),
        "d" => return Some(TimeUnitType::Day),
        "h" => return Some(TimeUnitType::Hour),
        "min" => return Some(TimeUnitType::Minute),
        "s" => return Some(TimeUnitType::Second),
        "ms" => return Some(TimeUnitType::Millisecond),
        // Additional UCUM units handled via mapping
        "a" => return Some(TimeUnitType::Year), // annum (year)
        "mo" => return Some(TimeUnitType::Month), // month
        _ => {}
    }

    // Fallback to hardcoded matching for compatibility
    // This ensures we don't break existing functionality while adding UCUM support
    // Also handle quoted strings (strip quotes if present)
    let clean_unit = unit_str.trim_matches('\'').trim_matches('"');
    match clean_unit {
        "year" | "years" => return Some(TimeUnitType::Year),
        "month" | "months" => return Some(TimeUnitType::Month),
        "week" | "weeks" => return Some(TimeUnitType::Week),
        "day" | "days" => return Some(TimeUnitType::Day),
        "hour" | "hours" => return Some(TimeUnitType::Hour),
        "minute" | "minutes" => return Some(TimeUnitType::Minute),
        "second" | "seconds" => return Some(TimeUnitType::Second),
        "millisecond" | "milliseconds" => return Some(TimeUnitType::Millisecond),
        _ => {}
    }

    // Try to parse with UCUM for validation only
    // Since all time units are mutually comparable in UCUM, we can't use comparability
    // to determine the specific unit type. We only use UCUM to validate that it's a valid unit.
    if let Ok(_unit_expr) = octofhir_ucum::parse_expression(unit_str) {
        // If it parses as a valid UCUM unit but we don't recognize it as a time unit,
        // it's probably not a time unit (e.g., "meter", "kg", etc.)
        None
    } else {
        // Invalid UCUM unit
        None
    }
}

/// Addition operator (+)
pub struct AddOperator;

impl FhirPathOperator for AddOperator {
    fn symbol(&self) -> &str {
        "+"
    }
    fn human_friendly_name(&self) -> &str {
        "Addition"
    }
    fn precedence(&self) -> u8 {
        6
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "+",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "+",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "+",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "+",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "+",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::String,
                ),
                OperatorSignature::binary(
                    "+",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                ),
                OperatorSignature::binary("+", TypeInfo::Date, TypeInfo::Quantity, TypeInfo::Date),
                OperatorSignature::binary("+", TypeInfo::Date, TypeInfo::Integer, TypeInfo::Date),
                OperatorSignature::binary(
                    "+",
                    TypeInfo::DateTime,
                    TypeInfo::Quantity,
                    TypeInfo::DateTime,
                ),
                OperatorSignature::binary("+", TypeInfo::Time, TypeInfo::Quantity, TypeInfo::Time),
                OperatorSignature::unary("+", TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::unary("+", TypeInfo::Decimal, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                match a.checked_add(*b) {
                    Some(result) => FhirPathValue::Integer(result),
                    None => return Ok(FhirPathValue::Empty), // Overflow returns empty per FHIRPath spec
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => FhirPathValue::Decimal(a + b),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) + b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                FhirPathValue::Decimal(a + rust_decimal::Decimal::from(*b))
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                FhirPathValue::String(format!("{a}{b}").into())
            }
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Try to add quantities using UCUM unit conversion
                match a.add(b) {
                    Ok(result) => FhirPathValue::Quantity(result.into()),
                    Err(_) => {
                        return Err(OperatorError::IncompatibleUnits {
                            left_unit: a.unit.clone().unwrap_or_default(),
                            right_unit: b.unit.clone().unwrap_or_default(),
                        });
                    }
                }
            }
            (FhirPathValue::Date(date), FhirPathValue::Quantity(quantity)) => {
                self.add_date_quantity(date, quantity)?
            }
            (FhirPathValue::Date(date), FhirPathValue::Integer(days)) => {
                // Treat integer as days for date arithmetic
                let new_date = *date + chrono::Duration::days(*days);
                FhirPathValue::Date(new_date)
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Quantity(quantity)) => {
                self.add_datetime_quantity(datetime, quantity)?
            }
            (FhirPathValue::Time(time), FhirPathValue::Quantity(quantity)) => {
                self.add_time_quantity(time, quantity)?
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }

    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match operand {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                Ok(FhirPathValue::collection(vec![operand.clone()]))
            }
            _ => Err(OperatorError::InvalidUnaryOperandType {
                operator: self.symbol().to_string(),
                operand_type: operand.type_name().to_string(),
            }),
        }
    }
}

impl AddOperator {
    /// Add a quantity to a date
    fn add_date_quantity(
        &self,
        date: &chrono::NaiveDate,
        quantity: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        use chrono::Datelike;

        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;

        let result_date = match classify_time_unit(unit) {
            Some(TimeUnitType::Year) => {
                let years = amount.to_i64().unwrap_or(0);
                date.with_year((date.year() + years as i32).max(1))
            }
            Some(TimeUnitType::Month) => {
                let months = amount.to_i64().unwrap_or(0);
                let total_months = date.year() as i64 * 12 + date.month() as i64 - 1 + months;
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;
                date.with_year(new_year.max(1))
                    .and_then(|d| d.with_month(new_month))
            }
            Some(TimeUnitType::Week) => {
                // Handle fractional weeks by converting to days
                let total_days = amount * rust_decimal::Decimal::from(7);
                let days = total_days.to_i64().unwrap_or(0);
                Some(*date + chrono::Duration::days(days))
            }
            Some(TimeUnitType::Day) => {
                // Handle fractional days by truncating to whole days
                // FHIRPath spec: fractional days are truncated for date arithmetic
                let days = amount.to_i64().unwrap_or(0);
                Some(*date + chrono::Duration::days(days))
            }
            _ => None, // Invalid unit for date arithmetic
        };

        match result_date {
            Some(new_date) => Ok(FhirPathValue::String(
                format!("@{}", new_date.format("%Y-%m-%d")).into(),
            )),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Add a quantity to a datetime
    fn add_datetime_quantity(
        &self,
        datetime: &chrono::DateTime<chrono::FixedOffset>,
        quantity: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        use chrono::Datelike;

        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;

        let result_datetime = match classify_time_unit(unit) {
            Some(TimeUnitType::Year) => {
                let new_year = (datetime.year() as i64 + amount.to_i64().unwrap_or(0)) as i32;
                datetime.with_year(new_year.max(1))
            }
            Some(TimeUnitType::Month) => {
                let total_months = datetime.year() as i64 * 12 + datetime.month() as i64 - 1
                    + amount.to_i64().unwrap_or(0);
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;
                datetime
                    .with_year(new_year.max(1))
                    .and_then(|d| d.with_month(new_month))
            }
            Some(TimeUnitType::Week) => {
                // Handle fractional weeks by converting to total seconds
                let total_seconds = amount * rust_decimal::Decimal::from(7 * 24 * 3600);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime + chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Day) => {
                // Handle fractional days by converting to total seconds
                let total_seconds = amount * rust_decimal::Decimal::from(24 * 3600);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime + chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Hour) => {
                // Handle fractional hours by converting to total seconds
                let total_seconds = amount * rust_decimal::Decimal::from(3600);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime + chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Minute) => {
                // Handle fractional minutes by converting to total seconds
                let total_seconds = amount * rust_decimal::Decimal::from(60);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime + chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Second) => {
                let seconds = amount.to_i64().unwrap_or(0);
                let millis = ((amount - rust_decimal::Decimal::from(seconds))
                    * rust_decimal::Decimal::from(1000))
                .to_i64()
                .unwrap_or(0);
                Some(
                    *datetime
                        + chrono::Duration::seconds(seconds)
                        + chrono::Duration::milliseconds(millis),
                )
            }
            Some(TimeUnitType::Millisecond) => {
                Some(*datetime + chrono::Duration::milliseconds(amount.to_i64().unwrap_or(0)))
            }
            _ => None, // Invalid unit
        };

        match result_datetime {
            Some(new_datetime) => Ok(FhirPathValue::String(
                format!("@{}", new_datetime.format("%Y-%m-%dT%H:%M:%S%.3f%:z")).into(),
            )),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Add a quantity to a time
    fn add_time_quantity(
        &self,
        time: &chrono::NaiveTime,
        quantity: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;

        let result_time = match classify_time_unit(unit) {
            Some(TimeUnitType::Hour) => {
                let hours = amount.to_i64().unwrap_or(0) % 24; // Handle wrap-around
                Some(
                    time.overflowing_add_signed(chrono::Duration::hours(hours))
                        .0,
                )
            }
            Some(TimeUnitType::Minute) => Some(
                time.overflowing_add_signed(chrono::Duration::minutes(
                    amount.to_i64().unwrap_or(0),
                ))
                .0,
            ),
            Some(TimeUnitType::Second) => {
                let seconds = amount.to_i64().unwrap_or(0);
                let millis = ((amount - rust_decimal::Decimal::from(seconds))
                    * rust_decimal::Decimal::from(1000))
                .to_i64()
                .unwrap_or(0);
                Some(
                    time.overflowing_add_signed(
                        chrono::Duration::seconds(seconds) + chrono::Duration::milliseconds(millis),
                    )
                    .0,
                )
            }
            Some(TimeUnitType::Millisecond) => Some(
                time.overflowing_add_signed(chrono::Duration::milliseconds(
                    amount.to_i64().unwrap_or(0),
                ))
                .0,
            ),
            _ => None, // Invalid unit for time arithmetic
        };

        match result_time {
            Some(new_time) => Ok(FhirPathValue::String(
                format!("@T{}", new_time.format("%H:%M:%S")).into(),
            )),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }
}

/// Subtraction operator (-)
pub struct SubtractOperator;

impl FhirPathOperator for SubtractOperator {
    fn symbol(&self) -> &str {
        "-"
    }
    fn human_friendly_name(&self) -> &str {
        "Subtraction"
    }
    fn precedence(&self) -> u8 {
        6
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "-",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "-",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "-",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "-",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "-",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                ),
                OperatorSignature::binary("-", TypeInfo::Date, TypeInfo::Quantity, TypeInfo::Date),
                OperatorSignature::binary(
                    "-",
                    TypeInfo::DateTime,
                    TypeInfo::Quantity,
                    TypeInfo::DateTime,
                ),
                OperatorSignature::binary("-", TypeInfo::Time, TypeInfo::Quantity, TypeInfo::Time),
                OperatorSignature::unary("-", TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::unary("-", TypeInfo::Decimal, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                match a.checked_sub(*b) {
                    Some(result) => FhirPathValue::Integer(result),
                    None => return Ok(FhirPathValue::Empty), // Overflow returns empty per FHIRPath spec
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => FhirPathValue::Decimal(a - b),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) - b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                FhirPathValue::Decimal(a - rust_decimal::Decimal::from(*b))
            }
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Try to subtract quantities using UCUM unit conversion
                match a.subtract(b) {
                    Ok(result) => FhirPathValue::Quantity(result.into()),
                    Err(_) => {
                        return Err(OperatorError::IncompatibleUnits {
                            left_unit: a.unit.clone().unwrap_or_default(),
                            right_unit: b.unit.clone().unwrap_or_default(),
                        });
                    }
                }
            }
            (FhirPathValue::Date(date), FhirPathValue::Quantity(quantity)) => {
                self.subtract_date_quantity(date, quantity)?
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Quantity(quantity)) => {
                self.subtract_datetime_quantity(datetime, quantity)?
            }
            (FhirPathValue::Time(time), FhirPathValue::Quantity(quantity)) => {
                self.subtract_time_quantity(time, quantity)?
            }
            (FhirPathValue::String(_), FhirPathValue::String(_)) => {
                // String subtraction returns empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }

    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match operand {
            FhirPathValue::Integer(n) => FhirPathValue::Integer(-n),
            FhirPathValue::Decimal(d) => FhirPathValue::Decimal(-d),
            _ => {
                return Err(OperatorError::InvalidUnaryOperandType {
                    operator: self.symbol().to_string(),
                    operand_type: operand.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }
}

impl SubtractOperator {
    /// Subtract a quantity from a date
    fn subtract_date_quantity(
        &self,
        date: &chrono::NaiveDate,
        quantity: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        use chrono::Datelike;

        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;

        let result_date = match classify_time_unit(unit) {
            Some(TimeUnitType::Year) => {
                let years = amount.to_i64().unwrap_or(0);
                date.with_year((date.year() - years as i32).max(1))
            }
            Some(TimeUnitType::Month) => {
                let months = amount.to_i64().unwrap_or(0);
                let total_months = date.year() as i64 * 12 + date.month() as i64 - 1 - months;
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;
                date.with_year(new_year.max(1))
                    .and_then(|d| d.with_month(new_month))
            }
            Some(TimeUnitType::Week) => {
                let total_days = amount * rust_decimal::Decimal::from(7);
                let days = total_days.to_i64().unwrap_or(0);
                Some(*date - chrono::Duration::days(days))
            }
            Some(TimeUnitType::Day) => {
                let days = amount.to_i64().unwrap_or(0);
                Some(*date - chrono::Duration::days(days))
            }
            _ => None, // Invalid unit for date arithmetic
        };

        match result_date {
            Some(new_date) => Ok(FhirPathValue::String(
                format!("@{}", new_date.format("%Y-%m-%d")).into(),
            )),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Subtract a quantity from a datetime
    fn subtract_datetime_quantity(
        &self,
        datetime: &chrono::DateTime<chrono::FixedOffset>,
        quantity: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        use chrono::Datelike;

        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;

        let result_datetime = match classify_time_unit(unit) {
            Some(TimeUnitType::Year) => {
                let new_year = (datetime.year() as i64 - amount.to_i64().unwrap_or(0)) as i32;
                datetime.with_year(new_year.max(1))
            }
            Some(TimeUnitType::Month) => {
                let total_months = datetime.year() as i64 * 12 + datetime.month() as i64
                    - 1
                    - amount.to_i64().unwrap_or(0);
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;
                datetime
                    .with_year(new_year.max(1))
                    .and_then(|d| d.with_month(new_month))
            }
            Some(TimeUnitType::Week) => {
                let total_seconds = amount * rust_decimal::Decimal::from(7 * 24 * 3600);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime - chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Day) => {
                let total_seconds = amount * rust_decimal::Decimal::from(24 * 3600);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime - chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Hour) => {
                let total_seconds = amount * rust_decimal::Decimal::from(3600);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime - chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Minute) => {
                let total_seconds = amount * rust_decimal::Decimal::from(60);
                let seconds = total_seconds.to_i64().unwrap_or(0);
                Some(*datetime - chrono::Duration::seconds(seconds))
            }
            Some(TimeUnitType::Second) => {
                let seconds = amount.to_i64().unwrap_or(0);
                let millis = ((amount - rust_decimal::Decimal::from(seconds))
                    * rust_decimal::Decimal::from(1000))
                .to_i64()
                .unwrap_or(0);
                Some(
                    *datetime
                        - chrono::Duration::seconds(seconds)
                        - chrono::Duration::milliseconds(millis),
                )
            }
            Some(TimeUnitType::Millisecond) => {
                Some(*datetime - chrono::Duration::milliseconds(amount.to_i64().unwrap_or(0)))
            }
            _ => None, // Invalid unit
        };

        match result_datetime {
            Some(new_datetime) => Ok(FhirPathValue::String(
                format!("@{}", new_datetime.format("%Y-%m-%dT%H:%M:%S%.3f%:z")).into(),
            )),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Subtract a quantity from a time
    fn subtract_time_quantity(
        &self,
        time: &chrono::NaiveTime,
        quantity: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;

        let result_time = match classify_time_unit(unit) {
            Some(TimeUnitType::Hour) => {
                let hours = amount.to_i64().unwrap_or(0);
                Some(
                    time.overflowing_sub_signed(chrono::Duration::hours(hours))
                        .0,
                )
            }
            Some(TimeUnitType::Minute) => Some(
                time.overflowing_sub_signed(chrono::Duration::minutes(
                    amount.to_i64().unwrap_or(0),
                ))
                .0,
            ),
            Some(TimeUnitType::Second) => {
                let seconds = amount.to_i64().unwrap_or(0);
                let millis = ((amount - rust_decimal::Decimal::from(seconds))
                    * rust_decimal::Decimal::from(1000))
                .to_i64()
                .unwrap_or(0);
                Some(
                    time.overflowing_sub_signed(
                        chrono::Duration::seconds(seconds) + chrono::Duration::milliseconds(millis),
                    )
                    .0,
                )
            }
            Some(TimeUnitType::Millisecond) => Some(
                time.overflowing_sub_signed(chrono::Duration::milliseconds(
                    amount.to_i64().unwrap_or(0),
                ))
                .0,
            ),
            _ => None, // Invalid unit for time arithmetic
        };

        match result_time {
            Some(new_time) => Ok(FhirPathValue::String(
                format!("@T{}", new_time.format("%H:%M:%S")).into(),
            )),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }
}

/// Multiplication operator (*)
pub struct MultiplyOperator;

impl FhirPathOperator for MultiplyOperator {
    fn symbol(&self) -> &str {
        "*"
    }
    fn human_friendly_name(&self) -> &str {
        "Multiplication"
    }
    fn precedence(&self) -> u8 {
        7
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Quantity,
                    TypeInfo::Integer,
                    TypeInfo::Quantity,
                ),
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Quantity,
                    TypeInfo::Decimal,
                    TypeInfo::Quantity,
                ),
                OperatorSignature::binary(
                    "*",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                match a.checked_mul(*b) {
                    Some(result) => FhirPathValue::Integer(result),
                    None => return Ok(FhirPathValue::Empty), // Overflow returns empty per FHIRPath spec
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => FhirPathValue::Decimal(a * b),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) * b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                FhirPathValue::Decimal(a * rust_decimal::Decimal::from(*b))
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(n)) => {
                let result = crate::model::Quantity::new(
                    q.value * rust_decimal::Decimal::from(*n),
                    q.unit.clone(),
                );
                FhirPathValue::Quantity(result.into())
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(d)) => {
                let result = crate::model::Quantity::new(q.value * d, q.unit.clone());
                FhirPathValue::Quantity(result.into())
            }
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                // Multiply two quantities with UCUM unit multiplication
                self.multiply_quantities(q1, q2)?
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }
}

impl MultiplyOperator {
    /// Multiply two quantities with UCUM unit multiplication
    fn multiply_quantities(
        &self,
        q1: &crate::model::quantity::Quantity,
        q2: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        let result_value = q1.value * q2.value;

        let result_unit = match (&q1.unit, &q2.unit) {
            (Some(unit1), Some(unit2)) => {
                // Use UCUM to multiply the units
                match octofhir_ucum::unit_multiply(unit1, unit2) {
                    Ok(result) => Some(result.expression),
                    Err(_) => {
                        // Fallback: basic unit multiplication
                        if unit1 == "1" || unit1.is_empty() {
                            Some(unit2.clone())
                        } else if unit2 == "1" || unit2.is_empty() {
                            Some(unit1.clone())
                        } else {
                            Some(format!("{unit1}.{unit2}"))
                        }
                    }
                }
            }
            (Some(unit), None) | (None, Some(unit)) => Some(unit.clone()),
            (None, None) => None,
        };

        let result_quantity = crate::model::quantity::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(result_quantity.into()))
    }
}

/// Division operator (/)
pub struct DivideOperator;

impl FhirPathOperator for DivideOperator {
    fn symbol(&self) -> &str {
        "/"
    }
    fn human_friendly_name(&self) -> &str {
        "Division"
    }
    fn precedence(&self) -> u8 {
        7
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Quantity,
                    TypeInfo::Integer,
                    TypeInfo::Quantity,
                ),
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Quantity,
                    TypeInfo::Decimal,
                    TypeInfo::Quantity,
                ),
                OperatorSignature::binary(
                    "/",
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                    TypeInfo::Quantity,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                let a_dec = rust_decimal::Decimal::from(*a);
                let b_dec = rust_decimal::Decimal::from(*b);
                FhirPathValue::Decimal(a_dec / b_dec)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                FhirPathValue::Decimal(a / b)
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) / b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                FhirPathValue::Decimal(a / rust_decimal::Decimal::from(*b))
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(n)) => {
                if *n == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                let result = crate::model::Quantity::new(
                    q.value / rust_decimal::Decimal::from(*n),
                    q.unit.clone(),
                );
                FhirPathValue::Quantity(result.into())
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(d)) => {
                if d.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                let result = crate::model::Quantity::new(q.value / d, q.unit.clone());
                FhirPathValue::Quantity(result.into())
            }
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                if q2.value.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                // Divide two quantities with UCUM unit division
                self.divide_quantities(q1, q2)?
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }
}

impl DivideOperator {
    /// Divide two quantities with UCUM unit division
    fn divide_quantities(
        &self,
        q1: &crate::model::quantity::Quantity,
        q2: &crate::model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        let result_value = q1.value / q2.value;

        let result_unit = match (&q1.unit, &q2.unit) {
            (Some(unit1), Some(unit2)) => {
                // Use UCUM to divide the units
                match octofhir_ucum::unit_divide(unit1, unit2) {
                    Ok(result) => {
                        // Handle the special case where units cancel out to dimensionless "1"
                        if result.expression == "1" || result.expression.is_empty() {
                            None
                        } else {
                            Some(result.expression)
                        }
                    }
                    Err(_) => {
                        // Fallback: basic unit division
                        if unit1 == unit2 {
                            None // Same units cancel out
                        } else {
                            Some(format!("{unit1}/{unit2}"))
                        }
                    }
                }
            }
            (Some(unit), None) => Some(unit.clone()),
            (None, Some(unit)) => Some(format!("1/{unit}")),
            (None, None) => None,
        };

        let result_quantity = crate::model::quantity::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(result_quantity.into()))
    }
}

/// Integer division operator (div)
pub struct IntegerDivideOperator;

impl FhirPathOperator for IntegerDivideOperator {
    fn symbol(&self) -> &str {
        "div"
    }
    fn human_friendly_name(&self) -> &str {
        "Integer Division"
    }
    fn precedence(&self) -> u8 {
        7
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "div",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "div",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "div",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "div",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                FhirPathValue::Integer(a / b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                // Convert to integer result (truncate)
                let result = (a / b).trunc();
                FhirPathValue::Integer(result.to_i64().unwrap_or(0))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                let a_dec = rust_decimal::Decimal::from(*a);
                let result = (a_dec / b).trunc();
                FhirPathValue::Integer(result.to_i64().unwrap_or(0))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                let b_dec = rust_decimal::Decimal::from(*b);
                let result = (a / b_dec).trunc();
                FhirPathValue::Integer(result.to_i64().unwrap_or(0))
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }
}

/// Modulo operator (mod)
pub struct ModuloOperator;

impl FhirPathOperator for ModuloOperator {
    fn symbol(&self) -> &str {
        "mod"
    }
    fn human_friendly_name(&self) -> &str {
        "Modulo"
    }
    fn precedence(&self) -> u8 {
        7
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "mod",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "mod",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "mod",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "mod",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                FhirPathValue::Integer(a % b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                FhirPathValue::Decimal(a % b)
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                let a_dec = rust_decimal::Decimal::from(*a);
                FhirPathValue::Decimal(a_dec % b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Ok(FhirPathValue::Empty);
                }
                let b_dec = rust_decimal::Decimal::from(*b);
                FhirPathValue::Decimal(a % b_dec)
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }
}

/// Power operator (**)
pub struct PowerOperator;

impl FhirPathOperator for PowerOperator {
    fn symbol(&self) -> &str {
        "**"
    }
    fn human_friendly_name(&self) -> &str {
        "Power"
    }
    fn precedence(&self) -> u8 {
        8
    } // Higher than multiplication
    fn associativity(&self) -> Associativity {
        Associativity::Right
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(
                    "**",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                ),
                OperatorSignature::binary(
                    "**",
                    TypeInfo::Decimal,
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "**",
                    TypeInfo::Integer,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
                OperatorSignature::binary(
                    "**",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                ),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(base), FhirPathValue::Integer(exp)) => {
                if *exp < 0 {
                    // Negative exponent results in decimal using f64 for power calculation
                    let base_f64 = *base as f64;
                    let exp_f64 = *exp as f64;
                    let result = base_f64.powf(exp_f64);
                    if result.is_nan() || result.is_infinite() {
                        return Ok(FhirPathValue::Empty);
                    }
                    match rust_decimal::Decimal::from_f64(result) {
                        Some(decimal) => FhirPathValue::Decimal(decimal),
                        None => return Ok(FhirPathValue::Empty),
                    }
                } else if *exp > 32 {
                    // Large exponents use f64 to avoid overflow
                    let base_f64 = *base as f64;
                    let exp_f64 = *exp as f64;
                    let result = base_f64.powf(exp_f64);
                    if result.is_nan() || result.is_infinite() {
                        return Ok(FhirPathValue::Empty);
                    }
                    match rust_decimal::Decimal::from_f64(result) {
                        Some(decimal) => FhirPathValue::Decimal(decimal),
                        None => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    // For small positive exponents, try to keep as integer
                    match base.checked_pow(*exp as u32) {
                        Some(result) => FhirPathValue::Integer(result),
                        None => {
                            // Overflow, convert to decimal
                            let base_f64 = *base as f64;
                            let exp_f64 = *exp as f64;
                            let result = base_f64.powf(exp_f64);
                            if result.is_nan() || result.is_infinite() {
                                return Ok(FhirPathValue::Empty);
                            }
                            match rust_decimal::Decimal::from_f64(result) {
                                Some(decimal) => FhirPathValue::Decimal(decimal),
                                None => return Ok(FhirPathValue::Empty),
                            }
                        }
                    }
                }
            }
            (FhirPathValue::Decimal(base), FhirPathValue::Integer(exp)) => {
                let base_f64 = base.to_f64().unwrap_or(0.0);
                let exp_f64 = *exp as f64;
                let result = base_f64.powf(exp_f64);
                if result.is_nan() || result.is_infinite() {
                    return Ok(FhirPathValue::Empty);
                }
                match rust_decimal::Decimal::from_f64(result) {
                    Some(decimal) => FhirPathValue::Decimal(decimal),
                    None => return Ok(FhirPathValue::Empty),
                }
            }
            (FhirPathValue::Integer(base), FhirPathValue::Decimal(exp)) => {
                let base_f64 = *base as f64;
                let exp_f64 = exp.to_f64().unwrap_or(0.0);
                let result = base_f64.powf(exp_f64);
                if result.is_nan() || result.is_infinite() {
                    return Ok(FhirPathValue::Empty);
                }
                match rust_decimal::Decimal::from_f64(result) {
                    Some(decimal) => FhirPathValue::Decimal(decimal),
                    None => return Ok(FhirPathValue::Empty),
                }
            }
            (FhirPathValue::Decimal(base), FhirPathValue::Decimal(exp)) => {
                let base_f64 = base.to_f64().unwrap_or(0.0);
                let exp_f64 = exp.to_f64().unwrap_or(0.0);
                let result = base_f64.powf(exp_f64);
                if result.is_nan() || result.is_infinite() {
                    return Ok(FhirPathValue::Empty);
                }
                match rust_decimal::Decimal::from_f64(result) {
                    Some(decimal) => FhirPathValue::Decimal(decimal),
                    None => return Ok(FhirPathValue::Empty),
                }
            }
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(result)
    }
}

/// Register all arithmetic operators
pub fn register_arithmetic_operators(registry: &mut OperatorRegistry) {
    registry.register(AddOperator);
    registry.register(SubtractOperator);
    registry.register(MultiplyOperator);
    registry.register(DivideOperator);
    registry.register(IntegerDivideOperator);
    registry.register(ModuloOperator);
    registry.register(PowerOperator);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fhirpath_to_ucum_mapping() {
        assert_eq!(fhirpath_to_ucum("year"), Some("a"));
        assert_eq!(fhirpath_to_ucum("month"), Some("mo"));
        assert_eq!(fhirpath_to_ucum("week"), Some("wk"));
        assert_eq!(fhirpath_to_ucum("day"), Some("d"));
        assert_eq!(fhirpath_to_ucum("hour"), Some("h"));
        assert_eq!(fhirpath_to_ucum("minute"), Some("min"));
        assert_eq!(fhirpath_to_ucum("second"), Some("s"));
        assert_eq!(fhirpath_to_ucum("millisecond"), Some("ms"));
        assert_eq!(fhirpath_to_ucum("invalid"), None);
    }

    #[test]
    fn test_ucum_to_fhirpath_mapping() {
        assert_eq!(ucum_to_fhirpath("a"), Some("year"));
        assert_eq!(ucum_to_fhirpath("mo"), Some("month"));
        assert_eq!(ucum_to_fhirpath("wk"), Some("week"));
        assert_eq!(ucum_to_fhirpath("d"), Some("day"));
        assert_eq!(ucum_to_fhirpath("h"), Some("hour"));
        assert_eq!(ucum_to_fhirpath("min"), Some("minute"));
        assert_eq!(ucum_to_fhirpath("s"), Some("second"));
        assert_eq!(ucum_to_fhirpath("ms"), Some("millisecond"));
        assert_eq!(ucum_to_fhirpath("invalid"), None);
    }

    #[test]
    fn test_classify_time_unit_ucum() {
        // Test UCUM units
        assert_eq!(classify_time_unit("a"), Some(TimeUnitType::Year));
        assert_eq!(classify_time_unit("mo"), Some(TimeUnitType::Month));
        assert_eq!(classify_time_unit("wk"), Some(TimeUnitType::Week));
        assert_eq!(classify_time_unit("d"), Some(TimeUnitType::Day));
        assert_eq!(classify_time_unit("h"), Some(TimeUnitType::Hour));
        assert_eq!(classify_time_unit("min"), Some(TimeUnitType::Minute));
        assert_eq!(classify_time_unit("s"), Some(TimeUnitType::Second));
        assert_eq!(classify_time_unit("ms"), Some(TimeUnitType::Millisecond));
    }

    #[test]
    fn test_classify_time_unit_fhirpath() {
        // Test FHIRPath calendar units
        assert_eq!(classify_time_unit("year"), Some(TimeUnitType::Year));
        assert_eq!(classify_time_unit("years"), Some(TimeUnitType::Year));
        assert_eq!(classify_time_unit("month"), Some(TimeUnitType::Month));
        assert_eq!(classify_time_unit("months"), Some(TimeUnitType::Month));
        assert_eq!(classify_time_unit("week"), Some(TimeUnitType::Week));
        assert_eq!(classify_time_unit("weeks"), Some(TimeUnitType::Week));
        assert_eq!(classify_time_unit("day"), Some(TimeUnitType::Day));
        assert_eq!(classify_time_unit("days"), Some(TimeUnitType::Day));
        assert_eq!(classify_time_unit("hour"), Some(TimeUnitType::Hour));
        assert_eq!(classify_time_unit("hours"), Some(TimeUnitType::Hour));
        assert_eq!(classify_time_unit("minute"), Some(TimeUnitType::Minute));
        assert_eq!(classify_time_unit("minutes"), Some(TimeUnitType::Minute));
        assert_eq!(classify_time_unit("second"), Some(TimeUnitType::Second));
        assert_eq!(classify_time_unit("seconds"), Some(TimeUnitType::Second));
        assert_eq!(
            classify_time_unit("millisecond"),
            Some(TimeUnitType::Millisecond)
        );
        assert_eq!(
            classify_time_unit("milliseconds"),
            Some(TimeUnitType::Millisecond)
        );
    }
}
