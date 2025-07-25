//! Operator registry and built-in operators

use crate::signature::OperatorSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rustc_hash::FxHashMap;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::sync::Arc;
use thiserror::Error;

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
    
    /// Get the operator precedence (higher = tighter binding)
    fn precedence(&self) -> u8;
    
    /// Get the associativity
    fn associativity(&self) -> Associativity;
    
    /// Get the operator signatures
    fn signatures(&self) -> &[OperatorSignature];
    
    /// Evaluate binary operation
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue>;
    
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
    // Arithmetic operators
    registry.register(AddOperator);
    registry.register(SubtractOperator);
    registry.register(MultiplyOperator);
    registry.register(DivideOperator);
    registry.register(IntegerDivideOperator);
    registry.register(ModuloOperator);
    registry.register(PowerOperator);
    
    // Comparison operators
    registry.register(EqualOperator);
    registry.register(NotEqualOperator);
    registry.register(LessThanOperator);
    registry.register(LessThanOrEqualOperator);
    registry.register(GreaterThanOperator);
    registry.register(GreaterThanOrEqualOperator);
    
    // Equivalence operators
    registry.register(EquivalentOperator);
    registry.register(NotEquivalentOperator);
    
    // Logical operators
    registry.register(AndOperator);
    registry.register(OrOperator);
    registry.register(XorOperator);
    registry.register(ImpliesOperator);
    registry.register(NotOperator);
    
    // String operators
    registry.register(ConcatenateOperator);
    
    // Collection operators
    registry.register(UnionOperator);
    registry.register(InOperator);
    registry.register(ContainsOperator);
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

// Arithmetic operators

/// Addition operator (+)
struct AddOperator;

impl FhirPathOperator for AddOperator {
    fn symbol(&self) -> &str { "+" }
    fn precedence(&self) -> u8 { 6 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("+", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("+", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("+", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("+", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("+", TypeInfo::String, TypeInfo::String, TypeInfo::String),
                OperatorSignature::binary("+", TypeInfo::Quantity, TypeInfo::Quantity, TypeInfo::Quantity),
                OperatorSignature::binary("+", TypeInfo::Date, TypeInfo::Quantity, TypeInfo::Date),
                OperatorSignature::binary("+", TypeInfo::DateTime, TypeInfo::Quantity, TypeInfo::DateTime),
                OperatorSignature::binary("+", TypeInfo::Time, TypeInfo::Quantity, TypeInfo::Time),
                OperatorSignature::unary("+", TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::unary("+", TypeInfo::Decimal, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_add(*b)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| OperatorError::ArithmeticOverflow {
                        operation: format!("{} + {}", a, b),
                    })?
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                FhirPathValue::Decimal(a + b)
            }
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
                if a.unit == b.unit {
                    let mut result = a.clone();
                    result.value = a.value + b.value;
                    FhirPathValue::Quantity(result)
                } else {
                    return Err(OperatorError::IncompatibleUnits {
                        left_unit: a.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
                        right_unit: b.unit.as_ref().map(|u| u.clone()).unwrap_or_default(),
                    });
                }
            }
            (FhirPathValue::Date(date), FhirPathValue::Quantity(quantity)) => {
                self.add_date_quantity(date, quantity)?
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Quantity(quantity)) => {
                self.add_datetime_quantity(datetime, quantity)?
            }
            (FhirPathValue::Time(time), FhirPathValue::Quantity(quantity)) => {
                self.add_time_quantity(time, quantity)?
            }
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
    
    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match operand {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(FhirPathValue::collection(vec![operand.clone()])),
            _ => Err(OperatorError::InvalidUnaryOperandType {
                operator: self.symbol().to_string(),
                operand_type: operand.type_name().to_string(),
            }),
        }
    }
}

impl AddOperator {
    /// Add a quantity to a date
    fn add_date_quantity(&self, date: &chrono::NaiveDate, quantity: &fhirpath_model::quantity::Quantity) -> OperatorResult<FhirPathValue> {
        use chrono::Datelike;
        
        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value.to_i64().unwrap_or(0);
        
        let result_date = match unit {
            "year" | "years" | "a" => {
                date.with_year((date.year() + amount as i32).max(1))
            }
            "month" | "months" | "mo" => {
                let total_months = date.year() as i64 * 12 + date.month() as i64 - 1 + amount;
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;
                date.with_year(new_year.max(1)).and_then(|d| d.with_month(new_month))
            }
            "week" | "weeks" | "wk" => {
                Some(*date + chrono::Duration::weeks(amount))
            }
            "day" | "days" | "d" => {
                Some(*date + chrono::Duration::days(amount))
            }
            _ => None, // Invalid unit for date arithmetic
        };
        
        match result_date {
            Some(new_date) => Ok(FhirPathValue::Date(new_date)),
            None => Ok(FhirPathValue::Empty), // Invalid operation returns empty
        }
    }
    
    /// Add a quantity to a datetime
    fn add_datetime_quantity(&self, datetime: &chrono::DateTime<chrono::Utc>, quantity: &fhirpath_model::quantity::Quantity) -> OperatorResult<FhirPathValue> {
        use chrono::Datelike;
        
        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;
        
        let result_datetime = match unit {
            "year" | "years" | "a" => {
                let new_year = (datetime.year() as i64 + amount.to_i64().unwrap_or(0)) as i32;
                datetime.with_year(new_year.max(1))
            }
            "month" | "months" | "mo" => {
                let total_months = datetime.year() as i64 * 12 + datetime.month() as i64 - 1 + amount.to_i64().unwrap_or(0);
                let new_year = (total_months / 12) as i32;
                let new_month = (total_months % 12 + 1) as u32;
                datetime.with_year(new_year.max(1)).and_then(|d| d.with_month(new_month))
            }
            "week" | "weeks" | "wk" => {
                Some(*datetime + chrono::Duration::weeks(amount.to_i64().unwrap_or(0)))
            }
            "day" | "days" | "d" => {
                Some(*datetime + chrono::Duration::days(amount.to_i64().unwrap_or(0)))
            }
            "hour" | "hours" | "h" => {
                Some(*datetime + chrono::Duration::hours(amount.to_i64().unwrap_or(0)))
            }
            "minute" | "minutes" | "min" => {
                Some(*datetime + chrono::Duration::minutes(amount.to_i64().unwrap_or(0)))
            }
            "second" | "seconds" | "s" => {
                let seconds = amount.to_i64().unwrap_or(0);
                let millis = ((amount - rust_decimal::Decimal::from(seconds)) * rust_decimal::Decimal::from(1000)).to_i64().unwrap_or(0);
                Some(*datetime + chrono::Duration::seconds(seconds) + chrono::Duration::milliseconds(millis))
            }
            "millisecond" | "milliseconds" | "ms" => {
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
    fn add_time_quantity(&self, time: &chrono::NaiveTime, quantity: &fhirpath_model::quantity::Quantity) -> OperatorResult<FhirPathValue> {
        let unit = quantity.unit.as_deref().unwrap_or("");
        let amount = quantity.value;
        
        let result_time = match unit {
            "hour" | "hours" | "h" => {
                let hours = amount.to_i64().unwrap_or(0) % 24; // Handle wrap-around
                Some(time.overflowing_add_signed(chrono::Duration::hours(hours)).0)
            }
            "minute" | "minutes" | "min" => {
                Some(time.overflowing_add_signed(chrono::Duration::minutes(amount.to_i64().unwrap_or(0))).0)
            }
            "second" | "seconds" | "s" => {
                let seconds = amount.to_i64().unwrap_or(0);
                let millis = ((amount - rust_decimal::Decimal::from(seconds)) * rust_decimal::Decimal::from(1000)).to_i64().unwrap_or(0);
                Some(time.overflowing_add_signed(chrono::Duration::seconds(seconds) + chrono::Duration::milliseconds(millis)).0)
            }
            "millisecond" | "milliseconds" | "ms" => {
                Some(time.overflowing_add_signed(chrono::Duration::milliseconds(amount.to_i64().unwrap_or(0))).0)
            }
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
    fn symbol(&self) -> &str { "-" }
    fn precedence(&self) -> u8 { 6 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("-", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("-", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("-", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("-", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("-", TypeInfo::Quantity, TypeInfo::Quantity, TypeInfo::Quantity),
                OperatorSignature::unary("-", TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::unary("-", TypeInfo::Decimal, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_sub(*b)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| OperatorError::ArithmeticOverflow {
                        operation: format!("{} - {}", a, b),
                    })?
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                FhirPathValue::Decimal(a - b)
            }
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
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
    
    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match operand {
            FhirPathValue::Integer(n) => FhirPathValue::Integer(-n),
            FhirPathValue::Decimal(d) => FhirPathValue::Decimal(-d),
            _ => return Err(OperatorError::InvalidUnaryOperandType {
                operator: self.symbol().to_string(),
                operand_type: operand.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
}

/// Multiplication operator (*)
struct MultiplyOperator;

impl FhirPathOperator for MultiplyOperator {
    fn symbol(&self) -> &str { "*" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("*", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("*", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("*", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("*", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("*", TypeInfo::Quantity, TypeInfo::Integer, TypeInfo::Quantity),
                OperatorSignature::binary("*", TypeInfo::Quantity, TypeInfo::Decimal, TypeInfo::Quantity),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_mul(*b)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| OperatorError::ArithmeticOverflow {
                        operation: format!("{} * {}", a, b),
                    })?
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                FhirPathValue::Decimal(a * b)
            }
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
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
}

/// Division operator (/)
struct DivideOperator;

impl FhirPathOperator for DivideOperator {
    fn symbol(&self) -> &str { "/" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("/", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("/", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("/", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("/", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("/", TypeInfo::Quantity, TypeInfo::Integer, TypeInfo::Quantity),
                OperatorSignature::binary("/", TypeInfo::Quantity, TypeInfo::Decimal, TypeInfo::Quantity),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                let a_dec = rust_decimal::Decimal::from(*a);
                let b_dec = rust_decimal::Decimal::from(*b);
                FhirPathValue::Decimal(a_dec / b_dec)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                FhirPathValue::Decimal(a / b)
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                FhirPathValue::Decimal(rust_decimal::Decimal::from(*a) / b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                FhirPathValue::Decimal(a / rust_decimal::Decimal::from(*b))
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(n)) => {
                if *n == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                let mut result = q.clone();
                result.value = q.value / rust_decimal::Decimal::from(*n);
                FhirPathValue::Quantity(result)
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(d)) => {
                if d.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                let mut result = q.clone();
                result.value = q.value / d;
                FhirPathValue::Quantity(result)
            }
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
}

/// Integer division operator (div)
struct IntegerDivideOperator;

impl FhirPathOperator for IntegerDivideOperator {
    fn symbol(&self) -> &str { "div" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("div", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("div", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Integer),
                OperatorSignature::binary("div", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Integer),
                OperatorSignature::binary("div", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Integer),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                FhirPathValue::Integer(a / b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                // Convert to integer result (truncate)
                let result = (a / b).trunc();
                FhirPathValue::Integer(result.to_i64().unwrap_or(0))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                let a_dec = rust_decimal::Decimal::from(*a);
                let result = (a_dec / b).trunc();
                FhirPathValue::Integer(result.to_i64().unwrap_or(0))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                let b_dec = rust_decimal::Decimal::from(*b);
                let result = (a / b_dec).trunc();
                FhirPathValue::Integer(result.to_i64().unwrap_or(0))
            }
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
}

/// Modulo operator (mod)
struct ModuloOperator;

impl FhirPathOperator for ModuloOperator {
    fn symbol(&self) -> &str { "mod" }
    fn precedence(&self) -> u8 { 7 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("mod", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("mod", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("mod", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("mod", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Handle empty operands per FHIRPath specification
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                FhirPathValue::Integer(a % b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                FhirPathValue::Decimal(a % b)
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OperatorError::DivisionByZero);
                }
                let a_dec = rust_decimal::Decimal::from(*a);
                FhirPathValue::Decimal(a_dec % b)
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Err(OperatorError::DivisionByZero);
                }
                let b_dec = rust_decimal::Decimal::from(*b);
                FhirPathValue::Decimal(a % b_dec)
            }
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
}

/// Power operator (**)
struct PowerOperator;

impl FhirPathOperator for PowerOperator {
    fn symbol(&self) -> &str { "**" }
    fn precedence(&self) -> u8 { 8 }  // Higher than multiplication
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("**", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("**", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("**", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("**", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
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
                    FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default())
                } else if *exp > 32 {
                    // Large exponents use f64 to avoid overflow
                    let base_f64 = *base as f64;
                    let exp_f64 = *exp as f64;
                    let result = base_f64.powf(exp_f64);
                    FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default())
                } else {
                    // For small positive exponents, try to keep as integer
                    match base.checked_pow(*exp as u32) {
                        Some(result) => FhirPathValue::Integer(result),
                        None => {
                            // Overflow, convert to decimal
                            let base_f64 = *base as f64;
                            let exp_f64 = *exp as f64;
                            let result = base_f64.powf(exp_f64);
                            FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default())
                        }
                    }
                }
            }
            (FhirPathValue::Decimal(base), FhirPathValue::Integer(exp)) => {
                let base_f64 = base.to_f64().unwrap_or(0.0);
                let exp_f64 = *exp as f64;
                let result = base_f64.powf(exp_f64);
                FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default())
            }
            (FhirPathValue::Integer(base), FhirPathValue::Decimal(exp)) => {
                let base_f64 = *base as f64;
                let exp_f64 = exp.to_f64().unwrap_or(0.0);
                let result = base_f64.powf(exp_f64);
                FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default())
            }
            (FhirPathValue::Decimal(base), FhirPathValue::Decimal(exp)) => {
                let base_f64 = base.to_f64().unwrap_or(0.0);
                let exp_f64 = exp.to_f64().unwrap_or(0.0);
                let result = base_f64.powf(exp_f64);
                FhirPathValue::Decimal(rust_decimal::Decimal::from_f64(result).unwrap_or_default())
            }
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
    }
}

// Comparison operators

/// Equality operator (=)
struct EqualOperator;

impl FhirPathOperator for EqualOperator {
    fn symbol(&self) -> &str { "=" }
    fn precedence(&self) -> u8 { 3 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("=", TypeInfo::Any, TypeInfo::Any, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // FHIRPath equality has special semantics:
        // - If either operand is empty, return empty
        // - If collections have different lengths, return false
        // - Otherwise compare values
        
        // Check if either operand is empty
        if left.is_empty() || right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        // FHIRPath equality is strict - types must match
        let result = match (left, right) {
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l == r,
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,
            (FhirPathValue::Quantity(q1), FhirPathValue::Quantity(q2)) => {
                q1.value == q2.value && q1.unit == q2.unit
            }
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| {
                    match self.evaluate_binary(a, b) {
                        Ok(FhirPathValue::Collection(result)) if result.len() == 1 => {
                            matches!(result.get(0), Some(FhirPathValue::Boolean(true)))
                        }
                        Ok(FhirPathValue::Empty) => false, // Empty means not equal
                        _ => false
                    }
                })
            }
            _ => false,
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

/// Not equal operator (!=)
struct NotEqualOperator;

impl FhirPathOperator for NotEqualOperator {
    fn symbol(&self) -> &str { "!=" }
    fn precedence(&self) -> u8 { 3 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("!=", TypeInfo::Any, TypeInfo::Any, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match EqualOperator.evaluate_binary(left, right)? {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty), // If equal returns empty, != also returns empty
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.get(0) {
                    Some(FhirPathValue::Boolean(b)) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)])),
                    _ => unreachable!("Equal operator should always return boolean"),
                }
            }
            _ => unreachable!("Equal operator should return empty or collection with one boolean"),
        }
    }
}

/// Less than operator (<)
struct LessThanOperator;

impl FhirPathOperator for LessThanOperator {
    fn symbol(&self) -> &str { "<" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("<", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Boolean),
                OperatorSignature::binary("<", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Boolean),
                OperatorSignature::binary("<", TypeInfo::String, TypeInfo::String, TypeInfo::Boolean),
                OperatorSignature::binary("<", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary("<", TypeInfo::DateTime, TypeInfo::DateTime, TypeInfo::Boolean),
                OperatorSignature::binary("<", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a < b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a < b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a < b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a < b,
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a < b,
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

/// Less than or equal operator (<=)
struct LessThanOrEqualOperator;

impl FhirPathOperator for LessThanOrEqualOperator {
    fn symbol(&self) -> &str { "<=" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("<=", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Boolean),
                OperatorSignature::binary("<=", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Boolean),
                OperatorSignature::binary("<=", TypeInfo::String, TypeInfo::String, TypeInfo::Boolean),
                OperatorSignature::binary("<=", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary("<=", TypeInfo::DateTime, TypeInfo::DateTime, TypeInfo::Boolean),
                OperatorSignature::binary("<=", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a <= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a <= b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a <= b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a <= b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a <= b,
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a <= b,
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

/// Greater than operator (>)
struct GreaterThanOperator;

impl FhirPathOperator for GreaterThanOperator {
    fn symbol(&self) -> &str { ">" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(">", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Boolean),
                OperatorSignature::binary(">", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Boolean),
                OperatorSignature::binary(">", TypeInfo::String, TypeInfo::String, TypeInfo::Boolean),
                OperatorSignature::binary(">", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(">", TypeInfo::DateTime, TypeInfo::DateTime, TypeInfo::Boolean),
                OperatorSignature::binary(">", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a > b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a > b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a > b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a > b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a > b,
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a > b,
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

/// Greater than or equal operator (>=)
struct GreaterThanOrEqualOperator;

impl FhirPathOperator for GreaterThanOrEqualOperator {
    fn symbol(&self) -> &str { ">=" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary(">=", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Boolean),
                OperatorSignature::binary(">=", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Boolean),
                OperatorSignature::binary(">=", TypeInfo::String, TypeInfo::String, TypeInfo::Boolean),
                OperatorSignature::binary(">=", TypeInfo::Date, TypeInfo::Date, TypeInfo::Boolean),
                OperatorSignature::binary(">=", TypeInfo::DateTime, TypeInfo::DateTime, TypeInfo::Boolean),
                OperatorSignature::binary(">=", TypeInfo::Time, TypeInfo::Time, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a >= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a >= b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a >= b,
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a >= b,
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a >= b,
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a >= b,
            _ => return Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

// Equivalence operators

/// Equivalence operator (~)
struct EquivalentOperator;

impl FhirPathOperator for EquivalentOperator {
    fn symbol(&self) -> &str { "~" }
    fn precedence(&self) -> u8 { 3 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("~", TypeInfo::Any, TypeInfo::Any, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // TODO: Implement proper equivalence logic (case-insensitive strings, etc.)
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(left == right)]))
    }
}

/// Not equivalent operator (!~)
struct NotEquivalentOperator;

impl FhirPathOperator for NotEquivalentOperator {
    fn symbol(&self) -> &str { "!~" }
    fn precedence(&self) -> u8 { 3 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("!~", TypeInfo::Any, TypeInfo::Any, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // TODO: Implement proper equivalence logic
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(left != right)]))
    }
}

// Logical operators

/// Logical AND operator
struct AndOperator;

impl FhirPathOperator for AndOperator {
    fn symbol(&self) -> &str { "and" }
    fn precedence(&self) -> u8 { 2 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("and", TypeInfo::Boolean, TypeInfo::Boolean, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*a && *b)]))
            }
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
    fn symbol(&self) -> &str { "or" }
    fn precedence(&self) -> u8 { 1 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("or", TypeInfo::Boolean, TypeInfo::Boolean, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*a || *b)]))
            }
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
    fn symbol(&self) -> &str { "xor" }
    fn precedence(&self) -> u8 { 1 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("xor", TypeInfo::Boolean, TypeInfo::Boolean, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*a ^ *b)]))
            }
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
    fn symbol(&self) -> &str { "implies" }
    fn precedence(&self) -> u8 { 1 }
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("implies", TypeInfo::Boolean, TypeInfo::Boolean, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                // A implies B is equivalent to (not A) or B
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!*a || *b)]))
            }
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
    fn symbol(&self) -> &str { "not" }
    fn precedence(&self) -> u8 { 8 }
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::unary("not", TypeInfo::Boolean, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, _left: &FhirPathValue, _right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        Err(OperatorError::EvaluationError {
            operator: self.symbol().to_string(),
            message: "NOT is a unary operator".to_string(),
        })
    }
    
    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match operand {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!*b)])),
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
    fn symbol(&self) -> &str { "&" }
    fn precedence(&self) -> u8 { 5 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("&", TypeInfo::String, TypeInfo::String, TypeInfo::String),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let left_str = left.to_string_value().unwrap_or_default();
        let right_str = right.to_string_value().unwrap_or_default();
        Ok(FhirPathValue::collection(vec![FhirPathValue::String(left_str + &right_str)]))
    }
}

// Collection operators

/// Union operator (|)
struct UnionOperator;

impl FhirPathOperator for UnionOperator {
    fn symbol(&self) -> &str { "|" }
    fn precedence(&self) -> u8 { 5 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("|", TypeInfo::Any, TypeInfo::Any, TypeInfo::Collection(Box::new(TypeInfo::Any))),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        let mut result = left.clone().to_collection();
        result.extend(right.clone().to_collection());
        Ok(FhirPathValue::Collection(result.into()))
    }
}

/// In operator
struct InOperator;

impl FhirPathOperator for InOperator {
    fn symbol(&self) -> &str { "in" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("in", TypeInfo::Any, TypeInfo::Collection(Box::new(TypeInfo::Any)), TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Per FHIRPath spec for 'in' operator:
        // - If left operand is empty, return empty
        // - If right operand is empty, return [false]
        // - If left operand is multi-item collection, special logic applies
        
        if left.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        if right.is_empty() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
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
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(right_collection.contains(single_item))]))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

/// Contains operator for collections
struct ContainsOperator;

impl FhirPathOperator for ContainsOperator {
    fn symbol(&self) -> &str { "contains" }
    fn precedence(&self) -> u8 { 4 }
    fn associativity(&self) -> Associativity { Associativity::Left }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("contains", TypeInfo::Collection(Box::new(TypeInfo::Any)), TypeInfo::Any, TypeInfo::Boolean),
            ]
        });
        &SIGS
    }
    
    fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        // Per FHIRPath spec for 'contains' operator:
        // - If both operands are empty, return empty
        // - If left operand is empty (but right is not), return [false]
        // - If right operand is empty (but left is not), return empty
        
        if left.is_empty() && right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        if left.is_empty() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        }
        
        if right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let left_collection = left.clone().to_collection();
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(left_collection.contains(right))]))
    }
}