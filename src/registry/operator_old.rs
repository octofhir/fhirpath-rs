//! Operator registry and built-in operators

use crate::registry::signature::OperatorSignature;
use crate::model::TypeInfo;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use thiserror::Error;

mod operators;

/// Result type for operator operations
pub type OperatorResult<T> = Result<T, OperatorError>;

/// Operator evaluation errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OperatorError {
    /// Invalid operand types
    #[error("Operator '{operator}' cannot be applied to types {left_type} and {right_type}")]
    InvalidOperandTypes {
        /// Operator symbol
        operator: String,
        /// Left operand type
        left_type: String,
        /// Right operand type
        right_type: String,
    },

    /// Invalid unary operand type
    #[error("Operator '{operator}' cannot be applied to type {operand_type}")]
    InvalidUnaryOperandType {
        /// Operator symbol
        operator: String,
        /// Operand type
        operand_type: String,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Arithmetic overflow
    #[error("Arithmetic overflow in operation '{operation}'")]
    ArithmeticOverflow {
        /// Operation that caused overflow
        operation: String,
    },

    /// Incompatible units
    #[error("Incompatible units: '{left_unit}' and '{right_unit}'")]
    IncompatibleUnits {
        /// Left operand unit
        left_unit: String,
        /// Right operand unit
        right_unit: String,
    },

    /// Runtime evaluation error
    #[error("Operator '{operator}' evaluation error: {message}")]
    EvaluationError {
        /// Operator symbol
        operator: String,
        /// Error message
        message: String,
    },
}

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    /// Left associative
    Left,
    /// Right associative
    Right,
}

/// Trait for implementing FHIRPath operators
pub trait FhirPathOperator: Send + Sync {
    /// Get the operator symbol
    fn symbol(&self) -> &str;

    /// Get the human-friendly name for the operator (for LSP and documentation)
    fn human_friendly_name(&self) -> &str;

    /// Get the operator precedence (higher = tighter binding)
    fn precedence(&self) -> u8;

    /// Get the associativity
    fn associativity(&self) -> Associativity;

    /// Get the operator signatures
    fn signatures(&self) -> &[OperatorSignature];

    /// Evaluate binary operation
    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue>;

    /// Evaluate unary operation (default implementation returns error)
    fn evaluate_unary(&self, _operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        Err(OperatorError::EvaluationError {
            operator: self.symbol().to_string(),
            message: "Operator does not support unary operations".to_string(),
        })
    }

    /// Check if this is a unary operator
    fn is_unary(&self) -> bool {
        self.signatures().iter().any(|sig| sig.right_type.is_none())
    }

    /// Check if this is a binary operator
    fn is_binary(&self) -> bool {
        self.signatures().iter().any(|sig| sig.right_type.is_some())
    }
}

/// Registry for FHIRPath operators
#[derive(Clone)]
pub struct OperatorRegistry {
    binary_ops: FxHashMap<String, Arc<dyn FhirPathOperator>>,
    unary_ops: FxHashMap<String, Arc<dyn FhirPathOperator>>,
    precedences: FxHashMap<String, u8>,
    associativities: FxHashMap<String, Associativity>,
}

impl OperatorRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            binary_ops: FxHashMap::default(),
            unary_ops: FxHashMap::default(),
            precedences: FxHashMap::default(),
            associativities: FxHashMap::default(),
        }
    }

    /// Register an operator
    pub fn register<O: FhirPathOperator + 'static>(&mut self, operator: O) {
        let symbol = operator.symbol().to_string();
        let precedence = operator.precedence();
        let associativity = operator.associativity();
        let op_arc = Arc::new(operator);

        if op_arc.is_binary() {
            self.binary_ops.insert(symbol.clone(), op_arc.clone());
        }

        if op_arc.is_unary() {
            self.unary_ops.insert(symbol.clone(), op_arc);
        }

        self.precedences.insert(symbol.clone(), precedence);
        self.associativities.insert(symbol, associativity);
    }

    /// Get a binary operator by symbol
    pub fn get_binary(&self, symbol: &str) -> Option<Arc<dyn FhirPathOperator>> {
        self.binary_ops.get(symbol).cloned()
    }

    /// Get a unary operator by symbol
    pub fn get_unary(&self, symbol: &str) -> Option<Arc<dyn FhirPathOperator>> {
        self.unary_ops.get(symbol).cloned()
    }

    /// Get operator precedence
    pub fn get_precedence(&self, symbol: &str) -> Option<u8> {
        self.precedences.get(symbol).copied()
    }

    /// Get operator associativity
    pub fn get_associativity(&self, symbol: &str) -> Option<Associativity> {
        self.associativities.get(symbol).copied()
    }

    /// Check if a binary operator exists
    pub fn contains_binary(&self, symbol: &str) -> bool {
        self.binary_ops.contains_key(symbol)
    }

    /// Check if a unary operator exists
    pub fn contains_unary(&self, symbol: &str) -> bool {
        self.unary_ops.contains_key(symbol)
    }

    /// Get all registered binary operator symbols
    pub fn binary_operator_symbols(&self) -> Vec<&str> {
        self.binary_ops.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered unary operator symbols
    pub fn unary_operator_symbols(&self) -> Vec<&str> {
        self.unary_ops.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for OperatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all built-in FHIRPath operators
pub fn register_builtin_operators(registry: &mut OperatorRegistry) {
    operators::register_builtin_operators(registry);
}

// Helper function to determine result type for arithmetic operations
fn arithmetic_result_type(left: &TypeInfo, right: &TypeInfo) -> TypeInfo {
    match (left, right) {
        (TypeInfo::Integer, TypeInfo::Integer) => TypeInfo::Integer,
        (TypeInfo::Decimal, _) | (_, TypeInfo::Decimal) => TypeInfo::Decimal,
        (TypeInfo::Quantity, _) | (_, TypeInfo::Quantity) => TypeInfo::Quantity,
        _ => TypeInfo::Any,
    }
}

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

// Arithmetic operators

/// Addition operator (+)
struct AddOperator;

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
            Some(new_date) => Ok(FhirPathValue::Date(new_date)),
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
            Some(new_datetime) => Ok(FhirPathValue::DateTime(new_datetime)),
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
            Some(new_time) => Ok(FhirPathValue::Time(new_time)),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }
}

/// Subtraction operator (-)
struct SubtractOperator;

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
                if a.unit == b.unit {
                    let mut result = a.clone();
                    result.value = a.value - b.value;
                    FhirPathValue::Quantity(result)
                } else {
                    return Err(OperatorError::IncompatibleUnits {
                        left_unit: a.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
                        right_unit: b.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
                    });
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

/// Multiplication operator (*)
struct MultiplyOperator;

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
        q1: &crate::model::quantity::Quantity,
        q2: &crate::model::quantity::Quantity,
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

        let result_quantity = crate::model::quantity::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(result_quantity))
    }
}

/// Division operator (/)
struct DivideOperator;

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
        q1: &crate::model::quantity::Quantity,
        q2: &crate::model::quantity::Quantity,
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

        let result_quantity = crate::model::quantity::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(result_quantity))
    }
}

/// Integer division operator (div)
struct IntegerDivideOperator;

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
struct ModuloOperator;

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
struct PowerOperator;

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

// Comparison operators

/// Equality operator (=)
struct EqualOperator;

impl FhirPathOperator for EqualOperator {
    fn symbol(&self) -> &str {
        "="
    }
    fn human_friendly_name(&self) -> &str {
        "Equality"
    }
    fn precedence(&self) -> u8 {
        6
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "=",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // FHIRPath equality has special semantics:
        // - If both operands are empty, return empty
        // - If one operand is empty and the other is not, return false
        // - If collections have different lengths, return false
        // - Otherwise compare values with type coercion

        // Handle empty cases according to FHIRPath specification
        match (left.is_empty(), right.is_empty()) {
            (true, true) => return Ok(FhirPathValue::Empty),
            (true, false) | (false, true) => return Ok(FhirPathValue::Boolean(false)),
            (false, false) => {} // Continue with normal comparison
        }

        // FHIRPath equality with type coercion support
        let result = match (left, right) {
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::DateTime(r)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let l_datetime = l.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                l_datetime == *r
            }
            (FhirPathValue::DateTime(l), FhirPathValue::Date(r)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let r_datetime = r.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                *l == r_datetime
            }
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,

            // Cross-type numeric comparisons (Integer vs Decimal)
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => Decimal::from(*l) == *r,
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => *l == Decimal::from(*r),

            // Quantity comparisons with unit conversion
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                self.compare_quantities_equal(q1, q2)?
            }

            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                l.len() == r.len()
                    && l.iter()
                        .zip(r.iter())
                        .all(|(a, b)| self.compare_values_equal(a, b).unwrap_or(false))
            }
            _ => false,
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

impl EqualOperator {
    /// Compare two values for equality without recursion
    fn compare_values_equal(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<bool> {
        let result = match (left, right) {
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::DateTime(r)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let l_datetime = l.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                l_datetime == *r
            }
            (FhirPathValue::DateTime(l), FhirPathValue::Date(r)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let r_datetime = r.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                *l == r_datetime
            }
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,

            // Cross-type numeric comparisons (Integer vs Decimal)
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => Decimal::from(*l) == *r,
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => *l == Decimal::from(*r),

            // Quantity comparisons with unit conversion
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                self.compare_quantities_equal(q1, q2)?
            }

            // For collections, they are not equal to non-collections
            (FhirPathValue::Collection(_), _) | (_, FhirPathValue::Collection(_)) => false,

            _ => false,
        };
        Ok(result)
    }

    /// Compare two quantities for equality, handling unit conversion
    fn compare_quantities_equal(
        &self,
        q1: &crate::model::Quantity,
        q2: &crate::model::Quantity,
    ) -> OperatorResult<bool> {
        // If units are the same, compare values directly
        if q1.unit == q2.unit {
            return Ok(q1.value == q2.value);
        }

        // Check if quantities have compatible dimensions
        if q1.has_compatible_dimensions(q2) {
            // For now, we'll do a simple comparison
            // TODO: Implement proper unit conversion using UCUM
            // This is a simplified implementation that assumes compatible units are equal
            Ok(q1.value == q2.value)
        } else {
            // If units are not compatible, quantities are not equal
            Ok(false)
        }
    }
}

/// Not equal operator (!=)
struct NotEqualOperator;

impl FhirPathOperator for NotEqualOperator {
    fn symbol(&self) -> &str {
        "!="
    }
    fn human_friendly_name(&self) -> &str {
        "Not Equal"
    }
    fn precedence(&self) -> u8 {
        6
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "!=",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        match EqualOperator.evaluate_binary(left, right)? {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty), // If equal returns empty, != also returns empty
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)), // Handle direct boolean return
            FhirPathValue::Collection(items) if items.len() == 1 => match items.get(0) {
                Some(FhirPathValue::Boolean(b)) => Ok(FhirPathValue::Boolean(!b)),
                _ => Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                }),
            },
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Less than operator (<)
struct LessThanOperator;

impl FhirPathOperator for LessThanOperator {
    fn symbol(&self) -> &str {
        "<"
    }
    fn human_friendly_name(&self) -> &str {
        "Less Than"
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
                    "<",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::Date,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<",
                    TypeInfo::DateTime,
                    TypeInfo::Date,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        if *left == FhirPathValue::Empty || *right == FhirPathValue::Empty {
            return Ok(FhirPathValue::Empty);
        }

        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a < b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a < b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a < b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a < b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let a_datetime = a.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                a_datetime < *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let b_datetime = b.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                *a < b_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a < b,
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            result,
        )]))
    }
}

/// Less than or equal operator (<=)
struct LessThanOrEqualOperator;

impl FhirPathOperator for LessThanOrEqualOperator {
    fn symbol(&self) -> &str {
        "<="
    }
    fn human_friendly_name(&self) -> &str {
        "Less Than or Equal"
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
                    "<=",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<=", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    "<=",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary("<=", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a <= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a <= b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a <= b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a <= b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a <= b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let a_datetime = a.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                a_datetime <= *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let b_datetime = b.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                *a <= b_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a <= b,
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            result,
        )]))
    }
}

/// Greater than operator (>)
struct GreaterThanOperator;

impl FhirPathOperator for GreaterThanOperator {
    fn symbol(&self) -> &str {
        ">"
    }
    fn human_friendly_name(&self) -> &str {
        "Greater Than"
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
                    ">",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    ">",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a > b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a > b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a > b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a > b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a > b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let a_datetime = a.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                a_datetime > *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let b_datetime = b.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                *a > b_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a > b,
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            result,
        )]))
    }
}

/// Greater than or equal operator (>=)
struct GreaterThanOrEqualOperator;

impl FhirPathOperator for GreaterThanOrEqualOperator {
    fn symbol(&self) -> &str {
        ">="
    }
    fn human_friendly_name(&self) -> &str {
        "Greater Than or Equal"
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
                    ">=",
                    TypeInfo::Integer,
                    TypeInfo::Integer,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::Decimal,
                    TypeInfo::Decimal,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::String,
                    TypeInfo::String,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">=", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(
                    ">=",
                    TypeInfo::DateTime,
                    TypeInfo::DateTime,
                    TypeInfo::Boolean,
                ),
                OperatorSignature::binary(">=", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a >= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a >= b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a >= b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a >= b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a >= b,
            (FhirPathValue::Date(a), FhirPathValue::DateTime(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let a_datetime = a.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                a_datetime >= *b
            }
            (FhirPathValue::DateTime(a), FhirPathValue::Date(b)) => {
                // Convert Date to DateTime for comparison (using UTC timezone and 00:00:00 time)
                let b_datetime = b.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset();
                *a >= b_datetime
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a >= b,
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                });
            }
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            result,
        )]))
    }
}

// Equivalence operators

/// Equivalence operator (~)
struct EquivalentOperator;

impl FhirPathOperator for EquivalentOperator {
    fn symbol(&self) -> &str {
        "~"
    }
    fn human_friendly_name(&self) -> &str {
        "Equivalent"
    }
    fn precedence(&self) -> u8 {
        6
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "~",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // TODO: Implement proper equivalence logic (case-insensitive strings, etc.)
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            left == right,
        )]))
    }
}

/// Not equivalent operator (!~)
struct NotEquivalentOperator;

impl FhirPathOperator for NotEquivalentOperator {
    fn symbol(&self) -> &str {
        "!~"
    }
    fn human_friendly_name(&self) -> &str {
        "Not Equivalent"
    }
    fn precedence(&self) -> u8 {
        6
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "!~",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // TODO: Implement proper equivalence logic
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            left != right,
        )]))
    }
}

// Logical operators

/// Logical AND operator
struct AndOperator;

impl FhirPathOperator for AndOperator {
    fn symbol(&self) -> &str {
        "and"
    }
    fn human_friendly_name(&self) -> &str {
        "Logical And"
    }
    fn precedence(&self) -> u8 {
        2
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "and",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // FHIRPath logical AND semantics:
        // - true and true = true
        // - true and false = false
        // - false and true = false
        // - false and false = false
        // - true and empty = empty
        // - false and empty = false
        // - empty and true = empty
        // - empty and false = false
        // - empty and empty = empty

        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(*a && *b)]),
            ),
            // If left is false, result is always false (short-circuit)
            (FhirPathValue::Boolean(false), _) if right.is_empty() => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]),
            ),
            // If right is false, result is always false (short-circuit)
            (_, FhirPathValue::Boolean(false)) if left.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(false)],
            )),
            // If either operand is empty (and the other is not false), result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical OR operator
struct OrOperator;

impl FhirPathOperator for OrOperator {
    fn symbol(&self) -> &str {
        "or"
    }
    fn human_friendly_name(&self) -> &str {
        "Logical Or"
    }
    fn precedence(&self) -> u8 {
        1
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "or",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // FHIRPath logical OR semantics:
        // - true or true = true
        // - true or false = true
        // - false or true = true
        // - false or false = false
        // - true or empty = true
        // - false or empty = empty
        // - empty or true = true
        // - empty or false = empty
        // - empty or empty = empty

        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(*a || *b)]),
            ),
            // If left is true, result is always true (short-circuit)
            (FhirPathValue::Boolean(true), _) if right.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(true)],
            )),
            // If right is true, result is always true (short-circuit)
            (_, FhirPathValue::Boolean(true)) if left.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(true)],
            )),
            // If either operand is empty (and the other is not true), result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical XOR operator
struct XorOperator;

impl FhirPathOperator for XorOperator {
    fn symbol(&self) -> &str {
        "xor"
    }
    fn human_friendly_name(&self) -> &str {
        "Exclusive Or"
    }
    fn precedence(&self) -> u8 {
        1
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "xor",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(*a ^ *b)]),
            ),
            // If either operand is empty, result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical IMPLIES operator
struct ImpliesOperator;

impl FhirPathOperator for ImpliesOperator {
    fn symbol(&self) -> &str {
        "implies"
    }
    fn human_friendly_name(&self) -> &str {
        "Implies"
    }
    fn precedence(&self) -> u8 {
        1
    }
    fn associativity(&self) -> Associativity {
        Associativity::Right
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "implies",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                // A implies B is equivalent to (not A) or B
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    !*a || *b,
                )]))
            }
            // false implies empty = true (because false implies anything is true)
            (FhirPathValue::Boolean(false), _) if right.is_empty() => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]),
            ),
            // empty implies true = true (because empty is considered false-like, so !empty || true = true)
            (_, FhirPathValue::Boolean(true)) if left.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(true)],
            )),
            // If either operand is empty (and not handled above), result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical NOT operator
struct NotOperator;

impl FhirPathOperator for NotOperator {
    fn symbol(&self) -> &str {
        "not"
    }
    fn human_friendly_name(&self) -> &str {
        "Logical Not"
    }
    fn precedence(&self) -> u8 {
        8
    }
    fn associativity(&self) -> Associativity {
        Associativity::Right
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::unary(
                "not",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        _left: &FhirPathValue,
        _right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        Err(OperatorError::EvaluationError {
            operator: self.symbol().to_string(),
            message: "NOT is a unary operator".to_string(),
        })
    }

    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match operand {
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!*b)]))
            }
            _ => Err(OperatorError::InvalidUnaryOperandType {
                operator: self.symbol().to_string(),
                operand_type: operand.type_name().to_string(),
            }),
        }
    }
}

// String operators

/// String concatenation operator (&)
struct ConcatenateOperator;

impl FhirPathOperator for ConcatenateOperator {
    fn symbol(&self) -> &str {
        "&"
    }
    fn human_friendly_name(&self) -> &str {
        "Concatenate"
    }
    fn precedence(&self) -> u8 {
        5
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "&",
                TypeInfo::String,
                TypeInfo::String,
                TypeInfo::String,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let left_str = left.to_string_value().unwrap_or_default();
        let right_str = right.to_string_value().unwrap_or_default();
        Ok(FhirPathValue::collection(vec![FhirPathValue::String(
            left_str + &right_str,
        )]))
    }
}

// Collection operators

/// Union operator (|)
struct UnionOperator;

impl FhirPathOperator for UnionOperator {
    fn symbol(&self) -> &str {
        "|"
    }
    fn human_friendly_name(&self) -> &str {
        "Union"
    }
    fn precedence(&self) -> u8 {
        5
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "|",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let mut result = left.clone().to_collection();
        result.extend(right.clone().to_collection());
        Ok(FhirPathValue::Collection(result.into()))
    }
}

/// In operator
struct InOperator;

impl FhirPathOperator for InOperator {
    fn symbol(&self) -> &str {
        "in"
    }
    fn human_friendly_name(&self) -> &str {
        "In"
    }
    fn precedence(&self) -> u8 {
        4
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "in",
                TypeInfo::Any,
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Per FHIRPath spec for 'in' operator:
        // - If left operand is empty, return empty
        // - If right operand is empty, return [false]
        // - If left operand is multi-item collection, special logic applies

        if left.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if right.is_empty() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                false,
            )]));
        }

        // For multi-item left operand, each item is tested individually
        let left_collection = left.clone().to_collection();
        let right_collection = right.clone().to_collection();

        // If left has multiple items, return empty (based on test testIn5)
        if left_collection.len() > 1 {
            return Ok(FhirPathValue::Empty);
        }

        // Single item test
        if let Some(single_item) = left_collection.first() {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                right_collection.contains(single_item),
            )]))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

/// Contains operator for collections
struct ContainsOperator;

impl FhirPathOperator for ContainsOperator {
    fn symbol(&self) -> &str {
        "contains"
    }
    fn human_friendly_name(&self) -> &str {
        "Contains"
    }
    fn precedence(&self) -> u8 {
        4
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }

    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "contains",
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Per FHIRPath spec for 'contains' operator:
        // - If both operands are empty, return empty
        // - If left operand is empty (but right is not), return [false]
        // - If right operand is empty (but left is not), return empty

        if left.is_empty() && right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if left.is_empty() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                false,
            )]));
        }

        if right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let left_collection = left.clone().to_collection();
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            left_collection.contains(right),
        )]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
    use crate::model::quantity::Quantity;
    use rust_decimal::Decimal;

    #[test]
    fn test_classify_time_unit_ucum_integration() {
        // Test UCUM standard units
        assert_eq!(classify_time_unit("a"), Some(TimeUnitType::Year));
        assert_eq!(classify_time_unit("mo"), Some(TimeUnitType::Month));
        assert_eq!(classify_time_unit("wk"), Some(TimeUnitType::Week));
        assert_eq!(classify_time_unit("d"), Some(TimeUnitType::Day));
        assert_eq!(classify_time_unit("h"), Some(TimeUnitType::Hour));
        assert_eq!(classify_time_unit("min"), Some(TimeUnitType::Minute));
        assert_eq!(classify_time_unit("s"), Some(TimeUnitType::Second));
        assert_eq!(classify_time_unit("ms"), Some(TimeUnitType::Millisecond));

        // Test fallback to hardcoded matching for backward compatibility
        assert_eq!(classify_time_unit("year"), Some(TimeUnitType::Year));
        assert_eq!(classify_time_unit("years"), Some(TimeUnitType::Year));
        assert_eq!(classify_time_unit("month"), Some(TimeUnitType::Month));
        assert_eq!(classify_time_unit("months"), Some(TimeUnitType::Month));

        // Test invalid units (non-time units should return None)
        assert_eq!(classify_time_unit("meter"), None);
        assert_eq!(classify_time_unit("kg"), None);
        assert_eq!(classify_time_unit("invalid"), None);
    }

    #[test]
    fn test_add_date_quantity_with_ucum() {
        let add_op = AddOperator;
        let test_date = NaiveDate::from_ymd_opt(2023, 6, 15).unwrap();
        let date_value = FhirPathValue::Date(test_date);

        // Test with UCUM standard units
        let quantity_year = Quantity::new(Decimal::from(1), Some("a".to_string()));
        let result = add_op.evaluate_binary(&date_value, &FhirPathValue::Quantity(quantity_year));
        assert!(result.is_ok());

        let quantity_month = Quantity::new(Decimal::from(3), Some("mo".to_string()));
        let result = add_op.evaluate_binary(&date_value, &FhirPathValue::Quantity(quantity_month));
        assert!(result.is_ok());

        // Test with traditional units (backward compatibility)
        let quantity_days = Quantity::new(Decimal::from(10), Some("days".to_string()));
        let result = add_op.evaluate_binary(&date_value, &FhirPathValue::Quantity(quantity_days));
        assert!(result.is_ok());

        // Test with invalid units
        let quantity_invalid = Quantity::new(Decimal::from(1), Some("meter".to_string()));
        let result =
            add_op.evaluate_binary(&date_value, &FhirPathValue::Quantity(quantity_invalid));
        assert!(result.is_ok());
        // Should return Collection([Empty]) for invalid units
        if let Ok(FhirPathValue::Collection(ref items)) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Empty));
        } else {
            panic!(
                "Expected Collection([Empty]) result for invalid unit, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn test_add_datetime_quantity_with_ucum() {
        let add_op = AddOperator;
        let test_datetime = FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2023, 6, 15, 14, 30, 0)
            .unwrap();
        let datetime_value = FhirPathValue::DateTime(test_datetime);

        // Test with UCUM standard units
        let quantity_hour = Quantity::new(Decimal::from(2), Some("h".to_string()));
        let result =
            add_op.evaluate_binary(&datetime_value, &FhirPathValue::Quantity(quantity_hour));
        assert!(result.is_ok());

        let quantity_minute = Quantity::new(Decimal::from(30), Some("min".to_string()));
        let result =
            add_op.evaluate_binary(&datetime_value, &FhirPathValue::Quantity(quantity_minute));
        assert!(result.is_ok());

        // Test with traditional units (backward compatibility)
        let quantity_seconds = Quantity::new(Decimal::from(45), Some("seconds".to_string()));
        let result =
            add_op.evaluate_binary(&datetime_value, &FhirPathValue::Quantity(quantity_seconds));
        assert!(result.is_ok());
    }
}
