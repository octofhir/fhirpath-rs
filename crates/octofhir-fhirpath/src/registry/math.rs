//! Math functions implementation for FHIRPath
//!
//! Implements mathematical functions including basic operations (abs, ceiling, floor),
//! advanced functions (sqrt, ln, log, exp, power), and arithmetic operations.

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{FhirPathValue, Result, error_code::FP0053};
use crate::register_function;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

impl FunctionRegistry {
    pub fn register_math_functions(&self) -> Result<()> {
        self.register_abs_function()?;
        self.register_ceiling_function()?;
        self.register_floor_function()?;
        self.register_round_function()?;
        self.register_truncate_function()?;
        self.register_sqrt_function()?;
        self.register_ln_function()?;
        self.register_log_function()?;
        self.register_power_function()?;
        self.register_exp_function()?;
        Ok(())
    }

    fn register_abs_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "abs",
            category: FunctionCategory::Math,
            description: "Returns the absolute value of the input",
            parameters: [],
            return_type: "number",
            examples: ["(-5).abs()", "(-3.14).abs()", "5.abs()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "abs() can only be called on a single numeric value".to_string()
                    ));
                }

                match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => Ok(FhirPathValue::Integer(i.abs())),
                    Some(FhirPathValue::Decimal(d)) => Ok(FhirPathValue::Decimal(d.abs())),
                    Some(FhirPathValue::Quantity { value, unit, ucum_unit, calendar_unit }) => {
                        Ok(FhirPathValue::Quantity {
                            value: value.abs(),
                            unit: unit.clone(),
                            ucum_unit: ucum_unit.clone(),
                            calendar_unit: *calendar_unit,
                        })
                    }
                    Some(_) => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "abs() can only be called on numeric values".to_string()
                    )),
                    None => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "abs() can only be called on numeric values".to_string()
                    ))
                }
            }
        )
    }

    fn register_ceiling_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "ceiling",
            category: FunctionCategory::Math,
            description: "Returns the smallest integer greater than or equal to the input",
            parameters: [],
            return_type: "integer",
            examples: ["3.14.ceiling()", "(-2.5).ceiling()", "5.ceiling()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ceiling() can only be called on a single numeric value".to_string()
                    ));
                }

                match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => Ok(FhirPathValue::Integer(*i)),
                    Some(FhirPathValue::Decimal(d)) => {
                        let result = d.ceil();
                        // Convert to integer if it fits
                        if let Some(int_value) = result.to_i64() {
                            Ok(FhirPathValue::Integer(int_value))
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(FhirPathValue::Quantity { value, .. }) => {
                        let result = value.ceil();
                        if let Some(int_value) = result.to_i64() {
                            Ok(FhirPathValue::Integer(int_value))
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(_) => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ceiling() can only be called on numeric values".to_string()
                    )),
                    None => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ceiling() can only be called on numeric values".to_string()
                    ))
                }
            }
        )
    }

    fn register_floor_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "floor",
            category: FunctionCategory::Math,
            description: "Returns the largest integer less than or equal to the input",
            parameters: [],
            return_type: "integer",
            examples: ["3.14.floor()", "(-2.5).floor()", "5.floor()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "floor() can only be called on a single numeric value".to_string()
                    ));
                }

                match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => Ok(FhirPathValue::Integer(*i)),
                    Some(FhirPathValue::Decimal(d)) => {
                        let result = d.floor();
                        if let Some(int_value) = result.to_i64() {
                            Ok(FhirPathValue::Integer(int_value))
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(FhirPathValue::Quantity { value, .. }) => {
                        let result = value.floor();
                        if let Some(int_value) = result.to_i64() {
                            Ok(FhirPathValue::Integer(int_value))
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(_) => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "floor() can only be called on numeric values".to_string()
                    )),
                    None => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "floor() can only be called on numeric values".to_string()
                    ))
                }
            }
        )
    }

    fn register_round_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "round",
            category: FunctionCategory::Math,
            description: "Returns the input rounded to the specified number of decimal places",
            parameters: ["precision": Some("integer".to_string()) => "Number of decimal places (optional, defaults to 0)"],
            return_type: "number",
            examples: ["3.14159.round(2)", "3.6.round()", "(-2.5).round()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "round() can only be called on a single numeric value".to_string()
                    ));
                }

                let precision = if context.arguments.is_empty() {
                    0u32
                } else {
                    match context.arguments.first() {
                        Some(FhirPathValue::Integer(p)) => {
                            if *p < 0 || *p > 28 {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "round() precision must be between 0 and 28".to_string()
                                ));
                            }
                            *p as u32
                        }
                        Some(_) => {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "round() precision must be an integer".to_string()
                            ));
                        }
                        None => {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "round() precision must be an integer".to_string()
                            ));
                        }
                    }
                };

                match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => {
                        if precision == 0 {
                            Ok(FhirPathValue::Integer(*i))
                        } else {
                            Ok(FhirPathValue::Decimal(Decimal::from(*i)))
                        }
                    }
                    Some(FhirPathValue::Decimal(d)) => {
                        let result = d.round_dp(precision);
                        if precision == 0 && result.fract().is_zero() {
                            if let Some(int_value) = result.to_i64() {
                                Ok(FhirPathValue::Integer(int_value))
                            } else {
                                Ok(FhirPathValue::Decimal(result))
                            }
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(FhirPathValue::Quantity { value, unit, ucum_unit, calendar_unit }) => {
                        let result = value.round_dp(precision);
                        Ok(FhirPathValue::Quantity {
                            value: result,
                            unit: unit.clone(),
                            ucum_unit: ucum_unit.clone(),
                            calendar_unit: *calendar_unit,
                        })
                    }
                    Some(_) => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "round() can only be called on numeric values".to_string()
                    )),
                    None => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "round() can only be called on numeric values".to_string()
                    ))
                }
            }
        )
    }

    fn register_truncate_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "truncate",
            category: FunctionCategory::Math,
            description: "Returns the integer part of the input (truncation towards zero)",
            parameters: [],
            return_type: "integer",
            examples: ["3.14.truncate()", "(-2.9).truncate()", "5.truncate()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "truncate() can only be called on a single numeric value".to_string()
                    ));
                }

                match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => Ok(FhirPathValue::Integer(*i)),
                    Some(FhirPathValue::Decimal(d)) => {
                        let result = d.trunc();
                        if let Some(int_value) = result.to_i64() {
                            Ok(FhirPathValue::Integer(int_value))
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(FhirPathValue::Quantity { value, .. }) => {
                        let result = value.trunc();
                        if let Some(int_value) = result.to_i64() {
                            Ok(FhirPathValue::Integer(int_value))
                        } else {
                            Ok(FhirPathValue::Decimal(result))
                        }
                    }
                    Some(_) => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "truncate() can only be called on numeric values".to_string()
                    )),
                    None => Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "truncate() can only be called on numeric values".to_string()
                    ))
                }
            }
        )
    }

    fn register_sqrt_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "sqrt",
            category: FunctionCategory::Math,
            description: "Returns the square root of the input",
            parameters: [],
            return_type: "decimal",
            examples: ["16.sqrt()", "2.25.sqrt()", "9.sqrt()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "sqrt() can only be called on a single numeric value".to_string()
                    ));
                }

                let value = match context.input.first() {
                    Some(FhirPathValue::Integer(i)) => {
                        if *i < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "sqrt() cannot be called on negative values".to_string()
                            ));
                        }
                        *i as f64
                    }
                    Some(FhirPathValue::Decimal(d)) => {
                        let f_val = d.to_f64().ok_or_else(|| {
                            crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "Invalid decimal value for sqrt()".to_string()
                            )
                        })?;
                        if f_val < 0.0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "sqrt() cannot be called on negative values".to_string()
                            ));
                        }
                        f_val
                    }
                    Some(FhirPathValue::Quantity { value, .. }) => {
                        let f_val = value.to_f64().ok_or_else(|| {
                            crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "Invalid quantity value for sqrt()".to_string()
                            )
                        })?;
                        if f_val < 0.0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "sqrt() cannot be called on negative values".to_string()
                            ));
                        }
                        f_val
                    }
                    Some(_) => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "sqrt() can only be called on numeric values".to_string()
                        ));
                    }
                    None => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "sqrt() can only be called on numeric values".to_string()
                        ));
                    }
                };

                let result = value.sqrt();
                let decimal_result = Decimal::try_from(result).map_err(|_| {
                    crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "sqrt() result cannot be represented as decimal".to_string()
                    )
                })?;

                Ok(FhirPathValue::Decimal(decimal_result))
            }
        )
    }

    fn register_ln_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "ln",
            category: FunctionCategory::Math,
            description: "Returns the natural logarithm (base e) of the input",
            parameters: [],
            return_type: "decimal",
            examples: ["2.71828.ln()", "10.ln()", "1.ln()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty(){
                    return Ok(FhirPathValue::Empty)
                }
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ln() can only be called on a single numeric value".to_string()
                    ));
                }

                let value = match context.input.first() {
                    Some(first_value) => extract_numeric_value(first_value, "ln()")?,
                    None => return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ln() can only be called on numeric values".to_string()
                    ))
                };

                if value <= 0.0 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ln() can only be called on positive values".to_string()
                    ));
                }

                let result = value.ln();
                if result.is_nan() || result.is_infinite() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ln() result is not a valid number".to_string()
                    ));
                }

                let decimal_result = Decimal::try_from(result).map_err(|_| {
                    crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ln() result cannot be represented as decimal".to_string()
                    )
                })?;

                Ok(FhirPathValue::Decimal(decimal_result))
            }
        )
    }

    fn register_log_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "log",
            category: FunctionCategory::Math,
            description: "Returns the logarithm of the input to the specified base",
            parameters: ["base": Some("number".to_string()) => "The base for the logarithm"],
            return_type: "decimal",
            examples: ["100.log(10)", "8.log(2)", "27.log(3)"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() can only be called on a single numeric value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() requires exactly one base argument".to_string()
                    ));
                }

                let value = match context.input.first() {
                    Some(first_value) => extract_numeric_value(first_value, "log()")?,
                    None => return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() can only be called on numeric values".to_string()
                    ))
                };
                let base = match context.arguments.first() {
                    Some(first_arg) => extract_numeric_value(first_arg, "log() base")?,
                    None => return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() requires exactly one base argument".to_string()
                    ))
                };

                if value <= 0.0 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() can only be called on positive values".to_string()
                    ));
                }

                if base <= 0.0 || base == 1.0 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() base must be positive and not equal to 1".to_string()
                    ));
                }

                let result = value.log(base);
                if result.is_nan() || result.is_infinite() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() result is not a valid number".to_string()
                    ));
                }

                let decimal_result = Decimal::try_from(result).map_err(|_| {
                    crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "log() result cannot be represented as decimal".to_string()
                    )
                })?;

                Ok(FhirPathValue::Decimal(decimal_result))
            }
        )
    }

    fn register_power_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "power",
            category: FunctionCategory::Math,
            description: "Returns the input raised to the specified power",
            parameters: ["exponent": Some("number".to_string()) => "The exponent"],
            return_type: "decimal",
            examples: ["2.power(3)", "10.power(2)", "16.power(0.5)"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "power() can only be called on a single numeric value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "power() requires exactly one exponent argument".to_string()
                    ));
                }

                let base = match context.input.first() {
                    Some(first_value) => extract_numeric_value(first_value, "power()")?,
                    None => return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "power() can only be called on numeric values".to_string()
                    ))
                };
                let exponent = match context.arguments.first() {
                    Some(first_arg) => extract_numeric_value(first_arg, "power() exponent")?,
                    None => return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "power() requires exactly one exponent argument".to_string()
                    ))
                };

                let result = base.powf(exponent);
                if result.is_nan() || result.is_infinite() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "power() result is not a valid number".to_string()
                    ));
                }

                let decimal_result = Decimal::try_from(result).map_err(|_| {
                    crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "power() result cannot be represented as decimal".to_string()
                    )
                })?;

                Ok(FhirPathValue::Decimal(decimal_result))
            }
        )
    }

    fn register_exp_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "exp",
            category: FunctionCategory::Math,
            description: "Returns e raised to the power of the input",
            parameters: [],
            return_type: "decimal",
            examples: ["1.exp()", "0.exp()", "2.exp()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "exp() can only be called on a single numeric value".to_string()
                    ));
                }

                let value = match context.input.first() {
                    Some(first_value) => extract_numeric_value(first_value, "exp()")?,
                    None => return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "exp() can only be called on numeric values".to_string()
                    ))
                };

                let result = value.exp();
                if result.is_nan() || result.is_infinite() {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "exp() result is not a valid number".to_string()
                    ));
                }

                let decimal_result = Decimal::try_from(result).map_err(|_| {
                    crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "exp() result cannot be represented as decimal".to_string()
                    )
                })?;

                Ok(FhirPathValue::Decimal(decimal_result))
            }
        )
    }
}

/// Helper function to extract numeric values from FhirPathValue
fn extract_numeric_value(value: &FhirPathValue, context: &str) -> Result<f64> {
    match value {
        FhirPathValue::Integer(i) => Ok(*i as f64),
        FhirPathValue::Decimal(d) => d.to_f64().ok_or_else(|| {
            crate::core::FhirPathError::evaluation_error(
                FP0053,
                format!("Invalid decimal value for {}", context),
            )
        }),
        FhirPathValue::Quantity { value, .. } => value.to_f64().ok_or_else(|| {
            crate::core::FhirPathError::evaluation_error(
                FP0053,
                format!("Invalid quantity value for {}", context),
            )
        }),
        _ => Err(crate::core::FhirPathError::evaluation_error(
            FP0053,
            format!("{} can only be called on numeric values", context),
        )),
    }
}

/// Arithmetic operations utilities for FHIRPath expressions
pub struct ArithmeticOperations;

impl ArithmeticOperations {
    pub fn add(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        use crate::core::types::CalendarUnit;
        use crate::registry::datetime_utils::DateTimeDuration;
        use chrono::{DateTime, NaiveTime, Timelike, Utc};

        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a + b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                let a_decimal = Decimal::from(*a);
                Ok(FhirPathValue::Decimal(a_decimal + b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                let b_decimal = Decimal::from(*b);
                Ok(FhirPathValue::Decimal(a + b_decimal))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a + b))
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{}{}", a, b)))
            }
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Quantity {
                    value: b,
                    unit: unit_b,
                    ucum_unit: ucum_b,
                    calendar_unit: cal_b,
                },
            ) => {
                if unit_a != unit_b {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot add quantities with different units".to_string(),
                    ));
                }
                Ok(FhirPathValue::Quantity {
                    value: a + b,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone().or_else(|| ucum_b.clone()),
                    calendar_unit: cal_a.or(*cal_b),
                })
            }

            // Number + String unit -> convert to Quantity and add
            (FhirPathValue::Integer(val), FhirPathValue::String(unit)) => {
                let decimal_val = Decimal::from(*val);

                // Parse calendar unit
                let calendar_unit = match unit.as_str() {
                    "year" | "years" | "yr" | "a" => Some(CalendarUnit::Year),
                    "month" | "months" | "mo" => Some(CalendarUnit::Month),
                    "week" | "weeks" | "wk" => Some(CalendarUnit::Week),
                    "day" | "days" | "d" => Some(CalendarUnit::Day),
                    "1" => None, // Unitless quantity
                    _ => None,
                };

                Ok(FhirPathValue::Quantity {
                    value: decimal_val,
                    unit: Some(unit.clone()),
                    ucum_unit: None,
                    calendar_unit,
                })
            }
            (FhirPathValue::Decimal(val), FhirPathValue::String(unit)) => {
                // Parse calendar unit
                let calendar_unit = match unit.as_str() {
                    "year" | "years" | "yr" | "a" => Some(CalendarUnit::Year),
                    "month" | "months" | "mo" => Some(CalendarUnit::Month),
                    "week" | "weeks" | "wk" => Some(CalendarUnit::Week),
                    "day" | "days" | "d" => Some(CalendarUnit::Day),
                    "1" => None, // Unitless quantity
                    _ => None,
                };

                Ok(FhirPathValue::Quantity {
                    value: *val,
                    unit: Some(unit.clone()),
                    ucum_unit: None,
                    calendar_unit,
                })
            }

            // Date + Quantity -> date arithmetic
            (
                FhirPathValue::Date(date),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    calendar_unit,
                    ucum_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    // For date arithmetic, accept calendar units, unquoted time units, or valid UCUM time units
                    let duration = if calendar_unit.is_some() {
                        // Valid calendar unit - use DateTimeDuration
                        DateTimeDuration::from_quantity(value, unit_str)?
                    } else if let Some(_ucum) = ucum_unit {
                        // Valid UCUM unit - check if it's a time unit
                        match unit_str.as_str() {
                            "ms" | "s" | "min" | "h" | "d" | "wk" => {
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid UCUM unit for date arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unit '{}' is not valid for date arithmetic", unit_str),
                                ));
                            }
                        }
                    } else {
                        // Check if it's an unquoted time unit (not in CalendarUnit but valid for time)
                        // Check if it's an unquoted time unit (not in CalendarUnit but valid for time)
                        match unit_str.to_lowercase().as_str() {
                            "second" | "seconds" | "minute" | "minutes" | "hour" | "hours"
                            | "day" | "days" | "millisecond" | "milliseconds" => {
                                // Valid unquoted time units - use DateTimeDuration
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid unit for date arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unknown unit '{}' for date arithmetic", unit_str),
                                ));
                            }
                        }
                    };

                    // Convert date to datetime at midnight UTC for arithmetic
                    let naive_datetime = date.date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                        crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "Invalid date for arithmetic".to_string(),
                        )
                    })?;
                    let datetime = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

                    let result_datetime = duration.add_to_datetime(datetime)?;

                    // Convert back to date (preserving precision) and return as string for FHIRPath compliance
                    let result_date = result_datetime.naive_utc().date();
                    let precision_date =
                        crate::core::temporal::PrecisionDate::from_date(result_date);
                    Ok(FhirPathValue::String(precision_date.to_string()))
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot add quantity without unit to date".to_string(),
                    ))
                }
            }

            // DateTime + Quantity -> datetime arithmetic
            (
                FhirPathValue::DateTime(datetime),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    calendar_unit,
                    ucum_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    // For datetime arithmetic, accept calendar units, unquoted time units, or valid UCUM time units
                    let duration = if calendar_unit.is_some() {
                        // Valid calendar unit - use DateTimeDuration
                        DateTimeDuration::from_quantity(value, unit_str)?
                    } else if let Some(_ucum) = ucum_unit {
                        // Valid UCUM unit - check if it's a time unit
                        match unit_str.as_str() {
                            "ms" | "s" | "min" | "h" | "d" | "wk" => {
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid UCUM unit for datetime arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!(
                                        "Unit '{}' is not valid for datetime arithmetic",
                                        unit_str
                                    ),
                                ));
                            }
                        }
                    } else {
                        // Check if it's an unquoted time unit (not in CalendarUnit but valid for time)
                        // Check if it's an unquoted time unit (not in CalendarUnit but valid for time)
                        match unit_str.to_lowercase().as_str() {
                            "second" | "seconds" | "minute" | "minutes" | "hour" | "hours"
                            | "millisecond" | "milliseconds" => {
                                // Valid unquoted time units - use DateTimeDuration
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid unit for datetime arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unknown unit '{}' for datetime arithmetic", unit_str),
                                ));
                            }
                        }
                    };
                    let result_datetime =
                        duration.add_to_datetime(datetime.datetime.with_timezone(&Utc))?;

                    // Preserve original timezone if possible
                    let result_with_tz =
                        result_datetime.with_timezone(&datetime.datetime.timezone());

                    // Return as string for FHIRPath compliance
                    let precision_datetime = crate::core::temporal::PrecisionDateTime::new(
                        result_with_tz,
                        datetime.precision,
                    );
                    Ok(FhirPathValue::String(precision_datetime.to_string()))
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot add quantity without unit to datetime".to_string(),
                    ))
                }
            }

            // Time + Quantity -> time arithmetic with wrap-around
            (
                FhirPathValue::Time(time),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    calendar_unit,
                    ucum_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    // For time arithmetic, only accept calendar units or valid UCUM time units
                    // No calendar units are valid for time, only time-related UCUM units
                    if calendar_unit.is_some() {
                        // Calendar units like year, month don't make sense for time-of-day
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!(
                                "Calendar unit '{}' is not valid for time arithmetic",
                                unit_str
                            ),
                        ));
                    } else if ucum_unit.is_some() {
                        // Valid UCUM unit - check if it's a time unit
                        match unit_str.as_str() {
                            "ms" | "s" | "min" | "h" => {
                                // Valid time units - continue with arithmetic
                            }
                            _ => {
                                // Invalid UCUM unit for time arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unit '{}' is not valid for time arithmetic", unit_str),
                                ));
                            }
                        }
                    } else {
                        // Check unquoted time units (hour, minute, second, etc.)
                        match unit_str.to_lowercase().as_str() {
                            "hour" | "hours" | "minute" | "minutes" | "second" | "seconds"
                            | "millisecond" | "milliseconds" => {
                                // Valid unquoted time units - continue
                            }
                            _ => {
                                // Invalid unit for time arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unknown unit '{}' for time arithmetic", unit_str),
                                ));
                            }
                        }
                    }

                    match unit_str.to_lowercase().as_str() {
                        "hour" | "hours" | "h" => {
                            let hours = value.to_i64().unwrap_or(0) % 24;
                            let new_hour = (time.time.hour() as i64 + hours) % 24;
                            let result_time =
                                time.time.with_hour(new_hour as u32).ok_or_else(|| {
                                    crate::core::FhirPathError::evaluation_error(
                                        FP0053,
                                        "Invalid hour in time arithmetic".to_string(),
                                    )
                                })?;
                            // Return as string for FHIRPath compliance
                            let precision_time = crate::core::temporal::PrecisionTime::new(
                                result_time,
                                time.precision,
                            );
                            Ok(FhirPathValue::String(precision_time.to_string()))
                        }
                        "minute" | "minutes" | "min" => {
                            let total_minutes = time.time.hour() as i64 * 60
                                + time.time.minute() as i64
                                + value.to_i64().unwrap_or(0);
                            let wrapped_minutes = total_minutes % (24 * 60);
                            let new_hour = (wrapped_minutes / 60) as u32;
                            let new_minute = (wrapped_minutes % 60) as u32;
                            let result_time =
                                NaiveTime::from_hms_opt(new_hour, new_minute, time.time.second())
                                    .ok_or_else(|| {
                                    crate::core::FhirPathError::evaluation_error(
                                        FP0053,
                                        "Invalid time in time arithmetic".to_string(),
                                    )
                                })?;
                            // Return as string for FHIRPath compliance
                            let precision_time = crate::core::temporal::PrecisionTime::new(
                                result_time,
                                time.precision,
                            );
                            Ok(FhirPathValue::String(precision_time.to_string()))
                        }
                        "second" | "seconds" | "s" => {
                            let total_seconds = time.time.hour() as i64 * 3600
                                + time.time.minute() as i64 * 60
                                + time.time.second() as i64;
                            let add_seconds = value.to_i64().unwrap_or(0);
                            let result_seconds = (total_seconds + add_seconds) % (24 * 3600);
                            let new_hour = (result_seconds / 3600) as u32;
                            let new_minute = ((result_seconds % 3600) / 60) as u32;
                            let new_second = (result_seconds % 60) as u32;
                            let result_time =
                                NaiveTime::from_hms_opt(new_hour, new_minute, new_second)
                                    .ok_or_else(|| {
                                        crate::core::FhirPathError::evaluation_error(
                                            FP0053,
                                            "Invalid time in time arithmetic".to_string(),
                                        )
                                    })?;
                            // Return as string for FHIRPath compliance
                            let precision_time = crate::core::temporal::PrecisionTime::new(
                                result_time,
                                time.precision,
                            );
                            Ok(FhirPathValue::String(precision_time.to_string()))
                        }
                        _ => Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Unsupported time unit for time arithmetic: '{}'", unit_str),
                        )),
                    }
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot add quantity without unit to time".to_string(),
                    ))
                }
            }

            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot add values of these types".to_string(),
            )),
        }
    }

    pub fn subtract(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        use crate::registry::datetime_utils::DateTimeDuration;
        use chrono::{DateTime, Utc};

        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a - b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                let a_decimal = Decimal::from(*a);
                Ok(FhirPathValue::Decimal(a_decimal - b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                let b_decimal = Decimal::from(*b);
                Ok(FhirPathValue::Decimal(a - b_decimal))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a - b))
            }
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Quantity {
                    value: b,
                    unit: unit_b,
                    ucum_unit: ucum_b,
                    calendar_unit: cal_b,
                },
            ) => {
                if unit_a != unit_b {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot subtract quantities with different units".to_string(),
                    ));
                }
                Ok(FhirPathValue::Quantity {
                    value: a - b,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone().or_else(|| ucum_b.clone()),
                    calendar_unit: cal_a.or(*cal_b),
                })
            }

            // Date - Quantity -> date arithmetic
            (
                FhirPathValue::Date(date),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    calendar_unit,
                    ucum_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    // For date arithmetic, accept calendar units, unquoted time units, or valid UCUM time units
                    let duration = if calendar_unit.is_some() {
                        // Valid calendar unit - use DateTimeDuration
                        DateTimeDuration::from_quantity(value, unit_str)?
                    } else if let Some(_ucum) = ucum_unit {
                        // Valid UCUM unit - check if it's a time unit
                        match unit_str.as_str() {
                            "ms" | "s" | "min" | "h" | "d" | "wk" => {
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid UCUM unit for date arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unit '{}' is not valid for date arithmetic", unit_str),
                                ));
                            }
                        }
                    } else {
                        // Check if it's an unquoted time unit (not in CalendarUnit but valid for time)
                        match unit_str.to_lowercase().as_str() {
                            "second" | "seconds" | "minute" | "minutes" | "hour" | "hours"
                            | "day" | "days" | "millisecond" | "milliseconds" => {
                                // Valid unquoted time units - use DateTimeDuration
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid unit for date arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unknown unit '{}' for date arithmetic", unit_str),
                                ));
                            }
                        }
                    };

                    // Convert date to datetime at midnight UTC for arithmetic
                    let naive_datetime = date.date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                        crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "Invalid date for arithmetic".to_string(),
                        )
                    })?;
                    let datetime = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

                    let result_datetime = duration.subtract_from_datetime(datetime)?;

                    // Convert back to date (preserving precision) and return as string for FHIRPath compliance
                    let result_date = result_datetime.naive_utc().date();
                    let precision_date =
                        crate::core::temporal::PrecisionDate::from_date(result_date);
                    Ok(FhirPathValue::String(precision_date.to_string()))
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot subtract quantity without unit from date".to_string(),
                    ))
                }
            }

            // DateTime - Quantity -> datetime arithmetic
            (
                FhirPathValue::DateTime(datetime),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    calendar_unit,
                    ucum_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    // For datetime arithmetic, accept calendar units, unquoted time units, or valid UCUM time units
                    let duration = if calendar_unit.is_some() {
                        // Valid calendar unit - use DateTimeDuration
                        DateTimeDuration::from_quantity(value, unit_str)?
                    } else if let Some(_ucum) = ucum_unit {
                        // Valid UCUM unit - check if it's a time unit
                        match unit_str.as_str() {
                            "ms" | "s" | "min" | "h" | "d" | "wk" => {
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid UCUM unit for datetime arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!(
                                        "Unit '{}' is not valid for datetime arithmetic",
                                        unit_str
                                    ),
                                ));
                            }
                        }
                    } else {
                        // Check if it's an unquoted time unit (not in CalendarUnit but valid for time)
                        match unit_str.to_lowercase().as_str() {
                            "second" | "seconds" | "minute" | "minutes" | "hour" | "hours"
                            | "day" | "days" | "millisecond" | "milliseconds" => {
                                // Valid unquoted time units - use DateTimeDuration
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid unit for datetime arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unknown unit '{}' for datetime arithmetic", unit_str),
                                ));
                            }
                        }
                    };

                    // Convert PrecisionDateTime to chrono DateTime
                    let chrono_datetime = datetime.to_chrono_datetime()?;

                    let result_datetime = duration.subtract_from_datetime(chrono_datetime)?;

                    // Convert result back to PrecisionDateTime and return as string
                    let result_precision =
                        crate::core::temporal::PrecisionDateTime::from_chrono_datetime(
                            &result_datetime,
                            datetime.precision,
                        );
                    Ok(FhirPathValue::String(result_precision.to_string()))
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot subtract quantity without unit from datetime".to_string(),
                    ))
                }
            }

            // Time - Quantity -> time arithmetic
            (
                FhirPathValue::Time(time),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    calendar_unit,
                    ucum_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    // For time arithmetic, only accept time units
                    let duration = if calendar_unit.is_some() {
                        // Calendar units not valid for time arithmetic
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!(
                                "Calendar unit '{}' is not valid for time arithmetic",
                                unit_str
                            ),
                        ));
                    } else if let Some(_ucum) = ucum_unit {
                        // Valid UCUM unit - check if it's a time unit
                        match unit_str.as_str() {
                            "ms" | "s" | "min" | "h" => {
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid UCUM unit for time arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unit '{}' is not valid for time arithmetic", unit_str),
                                ));
                            }
                        }
                    } else {
                        // Check if it's an unquoted time unit
                        match unit_str.to_lowercase().as_str() {
                            "second" | "seconds" | "minute" | "minutes" | "hour" | "hours"
                            | "millisecond" | "milliseconds" => {
                                DateTimeDuration::from_quantity(value, unit_str)?
                            }
                            _ => {
                                // Invalid unit for time arithmetic
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    format!("Unit '{}' is not valid for time arithmetic", unit_str),
                                ));
                            }
                        }
                    };

                    // Convert PrecisionTime to a base datetime (today at this time) for arithmetic
                    let today = chrono::Utc::now().naive_utc().date();
                    let naive_datetime = today.and_time(time.time);
                    let datetime = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

                    let result_datetime = duration.subtract_from_datetime(datetime)?;

                    // Extract time from result and return as string
                    let result_time = result_datetime.naive_utc().time();
                    let precision_time =
                        crate::core::temporal::PrecisionTime::from_time_with_precision(
                            result_time,
                            time.precision,
                        );
                    Ok(FhirPathValue::String(precision_time.to_string()))
                } else {
                    Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot subtract quantity without unit from time".to_string(),
                    ))
                }
            }

            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot subtract values of these types".to_string(),
            )),
        }
    }

    pub fn multiply(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a * b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                let a_decimal = Decimal::from(*a);
                Ok(FhirPathValue::Decimal(a_decimal * b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                let b_decimal = Decimal::from(*b);
                Ok(FhirPathValue::Decimal(a * b_decimal))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a * b))
            }
            // Quantity multiplication by scalar
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Integer(b),
            ) => {
                let b_decimal = Decimal::from(*b);
                Ok(FhirPathValue::Quantity {
                    value: a * b_decimal,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            (
                FhirPathValue::Integer(a),
                FhirPathValue::Quantity {
                    value: b,
                    unit: unit_b,
                    ucum_unit: ucum_b,
                    calendar_unit: cal_b,
                },
            ) => {
                let a_decimal = Decimal::from(*a);
                Ok(FhirPathValue::Quantity {
                    value: a_decimal * b,
                    unit: unit_b.clone(),
                    ucum_unit: ucum_b.clone(),
                    calendar_unit: *cal_b,
                })
            }
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Decimal(b),
            ) => Ok(FhirPathValue::Quantity {
                value: a * b,
                unit: unit_a.clone(),
                ucum_unit: ucum_a.clone(),
                calendar_unit: *cal_a,
            }),
            (
                FhirPathValue::Decimal(a),
                FhirPathValue::Quantity {
                    value: b,
                    unit: unit_b,
                    ucum_unit: ucum_b,
                    calendar_unit: cal_b,
                },
            ) => Ok(FhirPathValue::Quantity {
                value: a * b,
                unit: unit_b.clone(),
                ucum_unit: ucum_b.clone(),
                calendar_unit: *cal_b,
            }),
            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot multiply values of these types".to_string(),
            )),
        }
    }

    pub fn divide(left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return None; // Division by zero returns empty collection
                }
                // Integer division in FHIRPath returns decimal
                let a_decimal = Decimal::from(*a);
                let b_decimal = Decimal::from(*b);
                Some(FhirPathValue::Decimal(a_decimal / b_decimal))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return None; // Division by zero returns empty collection
                }
                let a_decimal = Decimal::from(*a);
                Some(FhirPathValue::Decimal(a_decimal / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return None; // Division by zero returns empty collection
                }
                let b_decimal = Decimal::from(*b);
                Some(FhirPathValue::Decimal(a / b_decimal))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return None; // Division by zero returns empty collection
                }
                Some(FhirPathValue::Decimal(a / b))
            }
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Integer(b),
            ) => {
                if *b == 0 {
                    return None; // Division by zero returns empty collection
                }
                let b_decimal = Decimal::from(*b);
                Some(FhirPathValue::Quantity {
                    value: a / b_decimal,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Decimal(b),
            ) => {
                if b.is_zero() {
                    return None; // Division by zero returns empty collection
                }
                Some(FhirPathValue::Quantity {
                    value: a / b,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            _ => None, // Invalid types for division return empty collection
        }
    }

    pub fn modulo(left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return None; // Modulo by zero returns empty collection
                }
                Some(FhirPathValue::Integer(a % b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return None; // Modulo by zero returns empty collection
                }
                Some(FhirPathValue::Decimal(a % b))
            }
            _ => None, // Invalid types for modulo return empty collection
        }
    }

    pub fn integer_divide(left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return None; // Division by zero returns empty collection
                }
                // Integer division truncates towards zero
                Some(FhirPathValue::Integer(a / b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return None; // Division by zero returns empty collection
                }
                let a_decimal = Decimal::from(*a);
                let result = a_decimal / b;
                // Convert to integer by truncating towards zero
                if let Some(int_value) = result.trunc().to_i64() {
                    Some(FhirPathValue::Integer(int_value))
                } else {
                    None // Cannot represent as integer returns empty collection
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return None; // Division by zero returns empty collection
                }
                let b_decimal = Decimal::from(*b);
                let result = a / b_decimal;
                // Convert to integer by truncating towards zero
                if let Some(int_value) = result.trunc().to_i64() {
                    Some(FhirPathValue::Integer(int_value))
                } else {
                    None // Cannot represent as integer returns empty collection
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return None; // Division by zero returns empty collection
                }
                let result = a / b;
                // Convert to integer by truncating towards zero
                if let Some(int_value) = result.trunc().to_i64() {
                    Some(FhirPathValue::Integer(int_value))
                } else {
                    None // Cannot represent as integer returns empty collection
                }
            }
            // Quantity integer division by scalar
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Integer(b),
            ) => {
                if *b == 0 {
                    return None; // Division by zero returns empty collection
                }
                let b_decimal = Decimal::from(*b);
                let result = a / b_decimal;
                // Truncate for integer division
                Some(FhirPathValue::Quantity {
                    value: result.trunc(),
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            (
                FhirPathValue::Quantity {
                    value: a,
                    unit: unit_a,
                    ucum_unit: ucum_a,
                    calendar_unit: cal_a,
                },
                FhirPathValue::Decimal(b),
            ) => {
                if b.is_zero() {
                    return None; // Division by zero returns empty collection
                }
                let result = a / b;
                // Truncate for integer division
                Some(FhirPathValue::Quantity {
                    value: result.trunc(),
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            _ => None, // Invalid types for integer division return empty collection
        }
    }

    /// Negate a numeric value (unary minus)
    pub fn negate(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(-i)),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
            FhirPathValue::Quantity {
                value,
                unit,
                ucum_unit,
                calendar_unit,
            } => Ok(FhirPathValue::Quantity {
                value: -value,
                unit: unit.clone(),
                ucum_unit: ucum_unit.clone(),
                calendar_unit: *calendar_unit,
            }),
            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot negate non-numeric values".to_string(),
            )),
        }
    }
}

// mod math_tests;
