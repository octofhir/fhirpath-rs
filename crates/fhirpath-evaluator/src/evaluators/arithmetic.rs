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

//! Arithmetic operations evaluator

use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

/// Specialized evaluator for arithmetic operations
pub struct ArithmeticEvaluator;

impl ArithmeticEvaluator {
    /// Helper to handle collection extraction for binary operations
    fn extract_operands<'a>(
        left: &'a FhirPathValue,
        right: &'a FhirPathValue,
    ) -> (Option<&'a FhirPathValue>, Option<&'a FhirPathValue>) {
        let left_val = match left {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first()
                } else if items.is_empty() {
                    None
                } else {
                    None // Multi-element collections not supported for arithmetic
                }
            }
            FhirPathValue::Empty => None,
            val => Some(val),
        };

        let right_val = match right {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first()
                } else if items.is_empty() {
                    None
                } else {
                    None // Multi-element collections not supported for arithmetic
                }
            }
            FhirPathValue::Empty => None,
            val => Some(val),
        };

        (left_val, right_val)
    }

    /// Evaluate addition operation
    pub async fn evaluate_addition(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (left_val, right_val) {
            (Some(l), Some(r)) => Self::add_values(l, r),
            _ => Ok(FhirPathValue::Empty), // If either operand is empty/invalid, result is empty
        }
    }

    /// Evaluate subtraction operation  
    pub async fn evaluate_subtraction(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (left_val, right_val) {
            (Some(l), Some(r)) => Self::subtract_values(l, r),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate multiplication operation
    pub async fn evaluate_multiplication(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (left_val, right_val) {
            (Some(l), Some(r)) => Self::multiply_values(l, r),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate division operation
    pub async fn evaluate_division(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (left_val, right_val) {
            (Some(l), Some(r)) => Self::divide_values(l, r),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate modulo operation
    pub async fn evaluate_modulo(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (left_val, right_val) {
            (Some(l), Some(r)) => Self::modulo_values(l, r),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate integer division operation
    pub async fn evaluate_integer_division(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (left_val, right_val) {
            (Some(l), Some(r)) => Self::integer_divide_values(l, r),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate unary plus operation
    pub async fn evaluate_unary_plus(operand: &FhirPathValue) -> EvaluationResult<FhirPathValue> {
        let value = match operand {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first().unwrap()
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            val => val,
        };

        // Unary plus just returns the operand for numeric types
        match value {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(value.clone()),
            _ => Err(EvaluationError::TypeError {
                expected: "numeric type".to_string(),
                actual: value.type_name().to_string(),
            }),
        }
    }

    /// Evaluate unary minus operation
    pub async fn evaluate_unary_minus(operand: &FhirPathValue) -> EvaluationResult<FhirPathValue> {
        let value = match operand {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first().unwrap()
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            val => val,
        };

        match value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(-i)),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
            _ => Err(EvaluationError::TypeError {
                expected: "numeric type".to_string(),
                actual: value.type_name().to_string(),
            }),
        }
    }

    // Private helper methods for actual arithmetic operations
    fn add_values(left: &FhirPathValue, right: &FhirPathValue) -> EvaluationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a
                .checked_add(*b)
                .map(FhirPathValue::Integer)
                .ok_or_else(|| EvaluationError::InvalidOperation {
                    message: "Integer overflow in addition".to_string(),
                }),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a + b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal + b)),
                Err(_) => Err(EvaluationError::InvalidOperation {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Ok(FhirPathValue::Decimal(a + b_decimal)),
                Err(_) => Err(EvaluationError::InvalidOperation {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{a}{b}").into()))
            }
            // DateTime + Quantity operations
            (FhirPathValue::DateTime(dt), FhirPathValue::Quantity(q)) => {
                Self::add_quantity_to_datetime(&dt.datetime, q)
            }
            // Date + Quantity operations
            (FhirPathValue::Date(date), FhirPathValue::Quantity(q)) => {
                Self::add_quantity_to_date(&date.date, q)
            }
            // Time + Quantity operations
            (FhirPathValue::Time(time), FhirPathValue::Quantity(q)) => {
                Self::add_quantity_to_time(&time.time, q)
            }
            // JsonValue arithmetic operations - handle Sonic JSON values
            (FhirPathValue::JsonValue(left_json), FhirPathValue::JsonValue(right_json)) => {
                Self::add_json_values(left_json, right_json)
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::Integer(int_val)) => {
                if json_val.is_number() {
                    if let Some(json_int) = json_val.as_i64() {
                        json_int
                            .checked_add(*int_val)
                            .map(FhirPathValue::Integer)
                            .ok_or_else(|| EvaluationError::InvalidOperation {
                                message: "Integer overflow in addition".to_string(),
                            })
                    } else if let Some(json_float) = json_val.as_f64() {
                        match Decimal::try_from(json_float) {
                            Ok(json_decimal) => match Decimal::try_from(*int_val) {
                                Ok(int_decimal) => {
                                    Ok(FhirPathValue::Decimal(json_decimal + int_decimal))
                                }
                                Err(_) => Err(EvaluationError::InvalidOperation {
                                    message: "Cannot convert integer to decimal".to_string(),
                                }),
                            },
                            Err(_) => Err(EvaluationError::InvalidOperation {
                                message: "Cannot convert JsonValue number to decimal".to_string(),
                            }),
                        }
                    } else {
                        Err(EvaluationError::TypeError {
                            expected: "numeric value".to_string(),
                            actual: "non-numeric JsonValue".to_string(),
                        })
                    }
                } else {
                    Err(EvaluationError::TypeError {
                        expected: "numeric JsonValue".to_string(),
                        actual: "non-numeric JsonValue".to_string(),
                    })
                }
            }
            (FhirPathValue::Integer(int_val), FhirPathValue::JsonValue(json_val)) => {
                if json_val.is_number() {
                    if let Some(json_int) = json_val.as_i64() {
                        int_val
                            .checked_add(json_int)
                            .map(FhirPathValue::Integer)
                            .ok_or_else(|| EvaluationError::InvalidOperation {
                                message: "Integer overflow in addition".to_string(),
                            })
                    } else if let Some(json_float) = json_val.as_f64() {
                        match Decimal::try_from(json_float) {
                            Ok(json_decimal) => match Decimal::try_from(*int_val) {
                                Ok(int_decimal) => {
                                    Ok(FhirPathValue::Decimal(int_decimal + json_decimal))
                                }
                                Err(_) => Err(EvaluationError::InvalidOperation {
                                    message: "Cannot convert integer to decimal".to_string(),
                                }),
                            },
                            Err(_) => Err(EvaluationError::InvalidOperation {
                                message: "Cannot convert JsonValue number to decimal".to_string(),
                            }),
                        }
                    } else {
                        Err(EvaluationError::TypeError {
                            expected: "numeric value".to_string(),
                            actual: "non-numeric JsonValue".to_string(),
                        })
                    }
                } else {
                    Err(EvaluationError::TypeError {
                        expected: "numeric JsonValue".to_string(),
                        actual: "non-numeric JsonValue".to_string(),
                    })
                }
            }
            (FhirPathValue::JsonValue(json_val), FhirPathValue::String(string_val)) => {
                if json_val.is_string() {
                    if let Some(json_str) = json_val.as_str() {
                        Ok(FhirPathValue::String(
                            format!("{json_str}{string_val}").into(),
                        ))
                    } else {
                        Err(EvaluationError::TypeError {
                            expected: "string JsonValue".to_string(),
                            actual: "non-string JsonValue".to_string(),
                        })
                    }
                } else {
                    Err(EvaluationError::TypeError {
                        expected: "string JsonValue".to_string(),
                        actual: "non-string JsonValue".to_string(),
                    })
                }
            }
            (FhirPathValue::String(string_val), FhirPathValue::JsonValue(json_val)) => {
                if json_val.is_string() {
                    if let Some(json_str) = json_val.as_str() {
                        Ok(FhirPathValue::String(
                            format!("{string_val}{json_str}").into(),
                        ))
                    } else {
                        Err(EvaluationError::TypeError {
                            expected: "string JsonValue".to_string(),
                            actual: "non-string JsonValue".to_string(),
                        })
                    }
                } else {
                    Err(EvaluationError::TypeError {
                        expected: "string JsonValue".to_string(),
                        actual: "non-string JsonValue".to_string(),
                    })
                }
            }

            // Handle Empty values - any operation with Empty returns Empty per FHIRPath spec
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            // Handle unsupported combinations that should return empty per FHIRPath spec
            (FhirPathValue::Date(_), FhirPathValue::Integer(_)) => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::TypeError {
                expected:
                    "Compatible numeric, string, or DateTime/Date + time-based Quantity types"
                        .to_string(),
                actual: format!("{:?} and {:?}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn subtract_values(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a
                .checked_sub(*b)
                .map(FhirPathValue::Integer)
                .ok_or_else(|| EvaluationError::InvalidOperation {
                    message: "Integer overflow in subtraction".to_string(),
                }),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a - b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal - b)),
                Err(_) => Err(EvaluationError::InvalidOperation {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Ok(FhirPathValue::Decimal(a - b_decimal)),
                Err(_) => Err(EvaluationError::InvalidOperation {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            // Date - Quantity operations (subtract time-based quantities from dates)
            (FhirPathValue::Date(date), FhirPathValue::Quantity(q)) => {
                Self::subtract_quantity_from_date(&date.date, q)
            }
            // DateTime - Quantity operations (subtract time-based quantities from datetime)
            (FhirPathValue::DateTime(dt), FhirPathValue::Quantity(q)) => {
                Self::subtract_quantity_from_datetime(&dt.datetime, q)
            }
            // Time - Quantity operations (subtract time-based quantities from time)
            (FhirPathValue::Time(time), FhirPathValue::Quantity(q)) => {
                Self::subtract_quantity_from_time(&time.time, q)
            }
            // String subtraction is not defined - return empty per FHIRPath spec
            (FhirPathValue::String(_), FhirPathValue::String(_)) => Ok(FhirPathValue::Empty),
            // Handle Empty values - any operation with Empty returns Empty per FHIRPath spec
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::TypeError {
                expected:
                    "Compatible numeric types, or Date/DateTime/Time with time-based Quantity"
                        .to_string(),
                actual: format!("{:?} and {:?}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn multiply_values(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a
                .checked_mul(*b)
                .map(FhirPathValue::Integer)
                .ok_or_else(|| EvaluationError::InvalidOperation {
                    message: "Integer overflow in multiplication".to_string(),
                }),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a * b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal * b)),
                Err(_) => Err(EvaluationError::InvalidOperation {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Ok(FhirPathValue::Decimal(a * b_decimal)),
                Err(_) => Err(EvaluationError::InvalidOperation {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            // Quantity multiplication operations
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                Self::multiply_quantities(a, b)
            }
            // Handle Empty values - any operation with Empty returns Empty per FHIRPath spec
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::TypeError {
                expected: "Compatible numeric types or Quantities".to_string(),
                actual: format!("{:?} and {:?}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn divide_values(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                // Integer division returns decimal in FHIRPath
                match (Decimal::try_from(*a), Decimal::try_from(*b)) {
                    (Ok(a_decimal), Ok(b_decimal)) => {
                        Ok(FhirPathValue::Decimal(a_decimal / b_decimal))
                    }
                    _ => Err(EvaluationError::InvalidOperation {
                        message: "Cannot convert integers to decimal for division".to_string(),
                    }),
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                Ok(FhirPathValue::Decimal(a / b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                match Decimal::try_from(*a) {
                    Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal / b)),
                    Err(_) => Err(EvaluationError::InvalidOperation {
                        message: "Cannot convert integer to decimal".to_string(),
                    }),
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                match Decimal::try_from(*b) {
                    Ok(b_decimal) => Ok(FhirPathValue::Decimal(a / b_decimal)),
                    Err(_) => Err(EvaluationError::InvalidOperation {
                        message: "Cannot convert integer to decimal".to_string(),
                    }),
                }
            }
            // Quantity division operations
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                if b.value.is_zero() {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                Self::divide_quantities(a, b)
            }
            // Handle Empty values - any operation with Empty returns Empty per FHIRPath spec
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::TypeError {
                expected: "Compatible numeric types or Quantities".to_string(),
                actual: format!("{:?} and {:?}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn modulo_values(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Modulo by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                Ok(FhirPathValue::Integer(a % b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    // Modulo by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                Ok(FhirPathValue::Decimal(a % b))
            }
            // Handle Empty values - any operation with Empty returns Empty per FHIRPath spec
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::TypeError {
                expected: "Compatible numeric types".to_string(),
                actual: format!("{:?} and {:?}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn integer_divide_values(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                // Integer division returns integer (truncated division)
                Ok(FhirPathValue::Integer(a / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                // Integer division: divide first, then truncate to integer
                let result = a / b;
                let result_int =
                    result
                        .trunc()
                        .to_i64()
                        .ok_or(EvaluationError::InvalidOperation {
                            message: "Cannot convert division result to integer for div operation"
                                .to_string(),
                        })?;
                Ok(FhirPathValue::Integer(result_int))
            }

            // Mixed integer/decimal cases
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                // Convert integer to decimal for division, then truncate result
                let a_decimal =
                    Decimal::try_from(*a).map_err(|_| EvaluationError::InvalidOperation {
                        message: "Cannot convert integer to decimal for div operation".to_string(),
                    })?;
                let result = a_decimal / b;
                let result_int =
                    result
                        .trunc()
                        .to_i64()
                        .ok_or(EvaluationError::InvalidOperation {
                            message: "Cannot convert division result to integer for div operation"
                                .to_string(),
                        })?;
                Ok(FhirPathValue::Integer(result_int))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Collection(Default::default()));
                }
                // Convert integer to decimal for division, then truncate result
                let b_decimal =
                    Decimal::try_from(*b).map_err(|_| EvaluationError::InvalidOperation {
                        message: "Cannot convert integer to decimal for div operation".to_string(),
                    })?;
                let result = a / b_decimal;
                let result_int =
                    result
                        .trunc()
                        .to_i64()
                        .ok_or(EvaluationError::InvalidOperation {
                            message: "Cannot convert division result to integer for div operation"
                                .to_string(),
                        })?;
                Ok(FhirPathValue::Integer(result_int))
            }

            // Handle Empty values - any operation with Empty returns Empty per FHIRPath spec
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            _ => Err(EvaluationError::TypeError {
                expected: "Compatible numeric types".to_string(),
                actual: format!("{:?} and {:?}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Add a time-based quantity to a DateTime
    fn add_quantity_to_datetime(
        dt: &chrono::DateTime<chrono::FixedOffset>,
        q: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        use chrono::Duration;

        let value = q.value;
        let unit = &q.unit;

        // Convert quantity value to nanoseconds based on unit
        let duration_nanos = match unit.as_deref() {
            Some("s") | Some("second") | Some("seconds") => {
                // Convert seconds to nanoseconds
                let seconds = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (seconds * 1_000_000_000.0) as i64
            }
            Some("ms") | Some("millisecond") | Some("milliseconds") => {
                // Convert milliseconds to nanoseconds
                let millis = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (millis * 1_000_000.0) as i64
            }
            Some("min") | Some("minute") | Some("minutes") => {
                // Convert minutes to nanoseconds
                let minutes = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (minutes * 60.0 * 1_000_000_000.0) as i64
            }
            Some("h") | Some("hour") | Some("hours") => {
                // Convert hours to nanoseconds
                let hours = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (hours * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("d") | Some("day") | Some("days") => {
                // Convert days to nanoseconds
                let days = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (days * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            // Add support for time-based date units
            Some("wk") | Some("week") | Some("weeks") => {
                // Convert weeks to nanoseconds (7 days)
                let weeks = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (weeks * 7.0 * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("mo") | Some("month") | Some("months") => {
                // Convert months to nanoseconds (assume 30.4375 days per month on average)
                let months = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (months * 30.4375 * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("a") | Some("year") | Some("years") => {
                // Convert years to nanoseconds (assume 365.25 days per year)
                let years = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (years * 365.25 * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            _ => {
                return Err(EvaluationError::TypeError {
                    expected: "Time-based quantity unit (s, ms, min, h, d, wk, mo, a)".to_string(),
                    actual: format!("Quantity with unit {unit:?}"),
                });
            }
        };

        // Create duration and add to datetime
        let duration = Duration::nanoseconds(duration_nanos);

        if let Some(new_dt) = dt.checked_add_signed(duration) {
            Ok(FhirPathValue::DateTime(
                octofhir_fhirpath_model::PrecisionDateTime::new(
                    new_dt,
                    octofhir_fhirpath_model::TemporalPrecision::Millisecond, // Default precision for arithmetic
                ),
            ))
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "DateTime arithmetic overflow".to_string(),
            })
        }
    }

    /// Add a time-based quantity to a Date  
    fn add_quantity_to_date(
        date: &chrono::NaiveDate,
        q: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        use chrono::Duration;

        let value = q.value;
        let unit = &q.unit;

        // Convert quantity to days for date arithmetic
        let days = match unit.as_deref() {
            Some("d") | Some("day") | Some("days") => {
                value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?
            }
            Some("wk") | Some("week") | Some("weeks") => {
                let weeks = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                weeks * 7.0
            }
            Some("mo") | Some("month") | Some("months") => {
                let months = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                months * 30.4375 // Average days per month
            }
            Some("a") | Some("year") | Some("years") => {
                let years = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                years * 365.25 // Average days per year
            }
            _ => {
                return Err(EvaluationError::TypeError {
                    expected: "Date-based quantity unit (d, day, days, wk, mo, a)".to_string(),
                    actual: format!("Quantity with unit {unit:?}"),
                });
            }
        };

        let duration = Duration::days(days as i64);

        if let Some(new_date) = date.checked_add_signed(duration) {
            Ok(FhirPathValue::Date(
                octofhir_fhirpath_model::PrecisionDate::new(
                    new_date,
                    octofhir_fhirpath_model::TemporalPrecision::Day, // Default precision for arithmetic
                ),
            ))
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "Date arithmetic overflow".to_string(),
            })
        }
    }

    /// Add a time-based quantity to a Time
    fn add_quantity_to_time(
        time: &chrono::NaiveTime,
        q: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        use chrono::Duration;

        let value = q.value;
        let unit = &q.unit;

        // Convert quantity value to nanoseconds based on unit
        let duration_nanos = match unit.as_deref() {
            Some("s") | Some("second") | Some("seconds") => {
                let seconds = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (seconds * 1_000_000_000.0) as i64
            }
            Some("ms") | Some("millisecond") | Some("milliseconds") => {
                let millis = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (millis * 1_000_000.0) as i64
            }
            Some("min") | Some("minute") | Some("minutes") => {
                let minutes = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (minutes * 60.0 * 1_000_000_000.0) as i64
            }
            Some("h") | Some("hour") | Some("hours") => {
                let hours = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (hours * 3600.0 * 1_000_000_000.0) as i64
            }
            _ => {
                return Err(EvaluationError::TypeError {
                    expected: "Time-based quantity unit (s, ms, min, h, second, millisecond, minute, hour)".to_string(),
                    actual: format!("Quantity with unit {unit:?}"),
                });
            }
        };

        let duration = Duration::nanoseconds(duration_nanos);

        let new_time = time.overflowing_add_signed(duration).0;
        Ok(FhirPathValue::Time(
            octofhir_fhirpath_model::PrecisionTime::new(
                new_time,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond, // Default precision for arithmetic
            ),
        ))
    }

    /// Divide two quantities
    fn divide_quantities(
        left: &octofhir_fhirpath_model::Quantity,
        right: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        // Divide the numeric values
        let result_value = left.value / right.value;

        // Create the resulting unit by combining left unit / right unit
        let result_unit = match (&left.unit, &right.unit) {
            // If same units, they cancel out to dimensionless "1"
            (Some(left_unit), Some(right_unit)) if left_unit == right_unit => Some("1".to_string()),
            // Different units or one is None - create compound unit
            (Some(left_unit), Some(right_unit)) => Some(format!("{left_unit}/{right_unit}")),
            // Left has unit, right is dimensionless
            (Some(left_unit), None) => Some(left_unit.clone()),
            // Left is dimensionless, right has unit - invert right unit
            (None, Some(right_unit)) => Some(format!("1/{right_unit}")),
            // Both dimensionless
            (None, None) => Some("1".to_string()),
        };

        let result_quantity = octofhir_fhirpath_model::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(Arc::new(result_quantity)))
    }

    /// Multiply two quantities
    fn multiply_quantities(
        left: &octofhir_fhirpath_model::Quantity,
        right: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        // Multiply the numeric values
        let result_value = left.value * right.value;

        // Create the resulting unit by combining left unit * right unit
        let result_unit = match (&left.unit, &right.unit) {
            // Both have units - multiply them
            (Some(left_unit), Some(right_unit)) => {
                // Handle special cases for same units that create square units
                if left_unit == right_unit {
                    Some(format!("{left_unit}2"))
                } else {
                    Some(format!("{left_unit}.{right_unit}"))
                }
            }
            // Left has unit, right is dimensionless (or unit "1")
            (Some(left_unit), None) => Some(left_unit.clone()),
            // Left is dimensionless (or unit "1"), right has unit
            (None, Some(right_unit)) => Some(right_unit.clone()),
            // Both dimensionless
            (None, None) => Some("1".to_string()),
        };

        let result_quantity = octofhir_fhirpath_model::Quantity::new(result_value, result_unit);
        Ok(FhirPathValue::Quantity(Arc::new(result_quantity)))
    }

    /// Subtract a time-based quantity from a Date
    fn subtract_quantity_from_date(
        date: &chrono::NaiveDate,
        q: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        use chrono::Duration;

        let value = q.value;
        let unit = &q.unit;

        // Convert quantity to days for date arithmetic
        let days = match unit.as_deref() {
            Some("d") | Some("day") | Some("days") => {
                value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?
            }
            Some("wk") | Some("week") | Some("weeks") => {
                let weeks = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                weeks * 7.0
            }
            Some("mo") | Some("month") | Some("months") => {
                let months = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                months * 30.4375 // Average days per month
            }
            Some("a") | Some("year") | Some("years") => {
                let years = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                years * 365.25 // Average days per year
            }
            _ => {
                // Non-time based quantity with date should return empty
                return Ok(FhirPathValue::Empty);
            }
        };

        let duration = Duration::days(-(days as i64));

        if let Some(new_date) = date.checked_add_signed(duration) {
            Ok(FhirPathValue::Date(
                octofhir_fhirpath_model::PrecisionDate::new(
                    new_date,
                    octofhir_fhirpath_model::TemporalPrecision::Day, // Default precision for arithmetic
                ),
            ))
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "Date arithmetic overflow".to_string(),
            })
        }
    }

    /// Subtract a time-based quantity from a DateTime
    fn subtract_quantity_from_datetime(
        dt: &chrono::DateTime<chrono::FixedOffset>,
        q: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        use chrono::Duration;

        let value = q.value;
        let unit = &q.unit;

        // Convert quantity value to nanoseconds based on unit
        let duration_nanos = match unit.as_deref() {
            Some("s") | Some("second") | Some("seconds") => {
                let seconds = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (seconds * 1_000_000_000.0) as i64
            }
            Some("ms") | Some("millisecond") | Some("milliseconds") => {
                let millis = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (millis * 1_000_000.0) as i64
            }
            Some("min") | Some("minute") | Some("minutes") => {
                let minutes = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (minutes * 60.0 * 1_000_000_000.0) as i64
            }
            Some("h") | Some("hour") | Some("hours") => {
                let hours = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (hours * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("d") | Some("day") | Some("days") => {
                let days = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (days * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("wk") | Some("week") | Some("weeks") => {
                let weeks = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (weeks * 7.0 * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("mo") | Some("month") | Some("months") => {
                let months = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (months * 30.4375 * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            Some("a") | Some("year") | Some("years") => {
                let years = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (years * 365.25 * 24.0 * 3600.0 * 1_000_000_000.0) as i64
            }
            _ => {
                // Non-time based quantity with datetime should return empty
                return Ok(FhirPathValue::Empty);
            }
        };

        // Create negative duration for subtraction
        let duration = Duration::nanoseconds(-duration_nanos);

        if let Some(new_dt) = dt.checked_add_signed(duration) {
            Ok(FhirPathValue::DateTime(
                octofhir_fhirpath_model::PrecisionDateTime::new(
                    new_dt,
                    octofhir_fhirpath_model::TemporalPrecision::Millisecond, // Default precision for arithmetic
                ),
            ))
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "DateTime arithmetic overflow".to_string(),
            })
        }
    }

    /// Subtract a time-based quantity from a Time
    fn subtract_quantity_from_time(
        time: &chrono::NaiveTime,
        q: &octofhir_fhirpath_model::Quantity,
    ) -> EvaluationResult<FhirPathValue> {
        use chrono::Duration;

        let value = q.value;
        let unit = &q.unit;

        // Convert quantity value to nanoseconds based on unit
        let duration_nanos = match unit.as_deref() {
            Some("s") | Some("second") | Some("seconds") => {
                let seconds = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (seconds * 1_000_000_000.0) as i64
            }
            Some("ms") | Some("millisecond") | Some("milliseconds") => {
                let millis = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (millis * 1_000_000.0) as i64
            }
            Some("min") | Some("minute") | Some("minutes") => {
                let minutes = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (minutes * 60.0 * 1_000_000_000.0) as i64
            }
            Some("h") | Some("hour") | Some("hours") => {
                let hours = value
                    .to_f64()
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert quantity value to number".to_string(),
                    })?;
                (hours * 3600.0 * 1_000_000_000.0) as i64
            }
            _ => {
                // Non-time based quantity with time should return empty
                return Ok(FhirPathValue::Empty);
            }
        };

        // Create negative duration for subtraction
        let duration = Duration::nanoseconds(-duration_nanos);

        let new_time = time.overflowing_add_signed(duration).0;
        Ok(FhirPathValue::Time(
            octofhir_fhirpath_model::PrecisionTime::new(
                new_time,
                octofhir_fhirpath_model::TemporalPrecision::Millisecond, // Default precision for arithmetic
            ),
        ))
    }

    /// Helper method to add two JsonValue numbers using Sonic JSON directly
    fn add_json_values(
        left_json: &octofhir_fhirpath_model::JsonValue,
        right_json: &octofhir_fhirpath_model::JsonValue,
    ) -> EvaluationResult<FhirPathValue> {
        // Use Sonic JSON directly for better performance
        if left_json.is_number() && right_json.is_number() {
            // Try integer addition first
            if let (Some(left_int), Some(right_int)) = (left_json.as_i64(), right_json.as_i64()) {
                left_int
                    .checked_add(right_int)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Integer overflow in addition".to_string(),
                    })
            } else {
                // Fall back to decimal addition
                let left_decimal = left_json
                    .as_f64()
                    .and_then(|f| Decimal::try_from(f).ok())
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert left JsonValue to decimal".to_string(),
                    })?;
                let right_decimal = right_json
                    .as_f64()
                    .and_then(|f| Decimal::try_from(f).ok())
                    .ok_or_else(|| EvaluationError::InvalidOperation {
                        message: "Cannot convert right JsonValue to decimal".to_string(),
                    })?;
                Ok(FhirPathValue::Decimal(left_decimal + right_decimal))
            }
        } else if left_json.is_string() && right_json.is_string() {
            if let (Some(left_str), Some(right_str)) = (left_json.as_str(), right_json.as_str()) {
                Ok(FhirPathValue::String(
                    format!("{left_str}{right_str}").into(),
                ))
            } else {
                Err(EvaluationError::TypeError {
                    expected: "valid string JsonValues".to_string(),
                    actual: "invalid string JsonValues".to_string(),
                })
            }
        } else {
            Err(EvaluationError::TypeError {
                expected: "numeric or string JsonValues".to_string(),
                actual: "incompatible JsonValue types".to_string(),
            })
        }
    }
}
