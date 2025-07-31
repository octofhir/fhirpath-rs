//! Arithmetic operators for FHIRPath expressions

use crate::operator::{FhirPathOperator, OperatorError, OperatorRegistry, OperatorResult, Associativity};
use crate::signature::OperatorSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use octofhir_ucum_core;
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

/// Helper function to classify a unit string using UCUM
fn classify_time_unit(unit_str: &str) -> Option<TimeUnitType> {
    // First check exact matches for UCUM standard units
    // Note: According to FHIRPath specification, 'mo' (month) and 'a' (year)
    // are not supported for date arithmetic and should return empty
    match unit_str {
        // Unsupported UCUM units for date arithmetic - return None to indicate empty result
        "a" | "mo" => return None,
        // Supported UCUM units for date arithmetic
        "wk" => return Some(TimeUnitType::Week),
        "d" => return Some(TimeUnitType::Day),
        "h" => return Some(TimeUnitType::Hour),
        "min" => return Some(TimeUnitType::Minute),
        "s" => return Some(TimeUnitType::Second),
        "ms" => return Some(TimeUnitType::Millisecond),
        _ => {}
    }

    // Fallback to hardcoded matching for compatibility
    // This ensures we don't break existing functionality while adding UCUM support
    match unit_str {
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
    if let Ok(_unit_expr) = octofhir_ucum_core::parse_expression(unit_str) {
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
                FhirPathValue::String(format!("{}{}", a, b))
            }
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Try to add quantities using UCUM unit conversion
                match a.add(b) {
                    Ok(result) => FhirPathValue::Quantity(result),
                    Err(_) => {
                        return Err(OperatorError::IncompatibleUnits {
                            left_unit: a.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
                            right_unit: b.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
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

        Ok(FhirPathValue::collection(vec![result]))
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
        quantity: &fhirpath_model::quantity::Quantity,
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
            Some(new_date) => Ok(FhirPathValue::Date(new_date)),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Add a quantity to a datetime
    fn add_datetime_quantity(
        &self,
        datetime: &chrono::DateTime<chrono::FixedOffset>,
        quantity: &fhirpath_model::quantity::Quantity,
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
            Some(new_datetime) => Ok(FhirPathValue::DateTime(new_datetime)),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Add a quantity to a time
    fn add_time_quantity(
        &self,
        time: &chrono::NaiveTime,
        quantity: &fhirpath_model::quantity::Quantity,
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
            Some(new_time) => Ok(FhirPathValue::Time(new_time)),
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
                    Ok(result) => FhirPathValue::Quantity(result),
                    Err(_) => {
                        return Err(OperatorError::IncompatibleUnits {
                            left_unit: a.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
                            right_unit: b.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
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

        Ok(FhirPathValue::collection(vec![result]))
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

        Ok(FhirPathValue::collection(vec![result]))
    }
}

impl SubtractOperator {
    /// Subtract a quantity from a date
    fn subtract_date_quantity(
        &self,
        date: &chrono::NaiveDate,
        quantity: &fhirpath_model::quantity::Quantity,
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
            Some(new_date) => Ok(FhirPathValue::Date(new_date)),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Subtract a quantity from a datetime
    fn subtract_datetime_quantity(
        &self,
        datetime: &chrono::DateTime<chrono::FixedOffset>,
        quantity: &fhirpath_model::quantity::Quantity,
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
                let total_months = datetime.year() as i64 * 12 + datetime.month() as i64 - 1
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
            Some(new_datetime) => Ok(FhirPathValue::DateTime(new_datetime)),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }

    /// Subtract a quantity from a time
    fn subtract_time_quantity(
        &self,
        time: &chrono::NaiveTime,
        quantity: &fhirpath_model::quantity::Quantity,
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
            Some(new_time) => Ok(FhirPathValue::Time(new_time)),
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
                let mut result = q.clone();
                result.value = q.value * rust_decimal::Decimal::from(*n);
                FhirPathValue::Quantity(result)
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(d)) => {
                let mut result = q.clone();
                result.value = q.value * d;
                FhirPathValue::Quantity(result)
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

        Ok(FhirPathValue::collection(vec![result]))
    }
}

impl MultiplyOperator {
    /// Multiply two quantities with UCUM unit multiplication
    fn multiply_quantities(
        &self,
        q1: &fhirpath_model::quantity::Quantity,
        q2: &fhirpath_model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        let result_value = q1.value * q2.value;

        let result_unit = match (&q1.unit, &q2.unit) {
            (Some(unit1), Some(unit2)) => {
                // Use UCUM to multiply the units
                match octofhir_ucum_core::unit_multiply(unit1, unit2) {
                    Ok(result) => Some(result.expression),
                    Err(_) => {
                        // Fallback: basic unit multiplication
                        if unit1 == "1" || unit1.is_empty() {
                            Some(unit2.clone())
                        } else if unit2 == "1" || unit2.is_empty() {
                            Some(unit1.clone())
                        } else {
                            Some(format!("{}.{}", unit1, unit2))
                        }
                    }
                }
            }
            (Some(unit), None) | (None, Some(unit)) => Some(unit.clone()),
            (None, None) => None,
        };

        let result_quantity = fhirpath_model::quantity::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(result_quantity))
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
                let mut result = q.clone();
                result.value = q.value / rust_decimal::Decimal::from(*n);
                FhirPathValue::Quantity(result)
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(d)) => {
                if d.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                let mut result = q.clone();
                result.value = q.value / d;
                FhirPathValue::Quantity(result)
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

        Ok(FhirPathValue::collection(vec![result]))
    }
}

impl DivideOperator {
    /// Divide two quantities with UCUM unit division
    fn divide_quantities(
        &self,
        q1: &fhirpath_model::quantity::Quantity,
        q2: &fhirpath_model::quantity::Quantity,
    ) -> OperatorResult<FhirPathValue> {
        let result_value = q1.value / q2.value;

        let result_unit = match (&q1.unit, &q2.unit) {
            (Some(unit1), Some(unit2)) => {
                // Use UCUM to divide the units
                match octofhir_ucum_core::unit_divide(unit1, unit2) {
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
                            Some(format!("{}/{}", unit1, unit2))
                        }
                    }
                }
            }
            (Some(unit), None) => Some(unit.clone()),
            (None, Some(unit)) => Some(format!("1/{}", unit)),
            (None, None) => None,
        };

        let result_quantity = fhirpath_model::quantity::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(result_quantity))
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

        Ok(FhirPathValue::collection(vec![result]))
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

        Ok(FhirPathValue::collection(vec![result]))
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

        Ok(FhirPathValue::collection(vec![result]))
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