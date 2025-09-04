//! Math functions implementation for FHIRPath
//!
//! Implements mathematical functions including basic operations (abs, ceiling, floor),
//! advanced functions (sqrt, ln, log, exp, power), and arithmetic operations.

use super::{FunctionRegistry, FunctionCategory, FunctionContext};
use crate::core::{FhirPathValue, Result, error_code::{FP0053}};
use crate::{register_function};
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "abs() can only be called on a single numeric value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Integer(i) => Ok(vec![FhirPathValue::Integer(i.abs())]),
                    FhirPathValue::Decimal(d) => Ok(vec![FhirPathValue::Decimal(d.abs())]),
                    FhirPathValue::Quantity { value, unit, ucum_unit, calendar_unit } => {
                        Ok(vec![FhirPathValue::Quantity {
                            value: value.abs(),
                            unit: unit.clone(),
                            ucum_unit: ucum_unit.clone(),
                            calendar_unit: *calendar_unit,
                        }])
                    }
                    _ => Err(crate::core::FhirPathError::evaluation_error(
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ceiling() can only be called on a single numeric value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Integer(i) => Ok(vec![FhirPathValue::Integer(*i)]),
                    FhirPathValue::Decimal(d) => {
                        let result = d.ceil();
                        // Convert to integer if it fits
                        if let Some(int_value) = result.to_i64() {
                            Ok(vec![FhirPathValue::Integer(int_value)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    FhirPathValue::Quantity { value, .. } => {
                        let result = value.ceil();
                        if let Some(int_value) = result.to_i64() {
                            Ok(vec![FhirPathValue::Integer(int_value)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    _ => Err(crate::core::FhirPathError::evaluation_error(
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "floor() can only be called on a single numeric value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Integer(i) => Ok(vec![FhirPathValue::Integer(*i)]),
                    FhirPathValue::Decimal(d) => {
                        let result = d.floor();
                        if let Some(int_value) = result.to_i64() {
                            Ok(vec![FhirPathValue::Integer(int_value)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    FhirPathValue::Quantity { value, .. } => {
                        let result = value.floor();
                        if let Some(int_value) = result.to_i64() {
                            Ok(vec![FhirPathValue::Integer(int_value)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    _ => Err(crate::core::FhirPathError::evaluation_error(
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "round() can only be called on a single numeric value".to_string()
                    ));
                }

                let precision = if context.arguments.is_empty() {
                    0u32
                } else {
                    match &context.arguments[0] {
                        FhirPathValue::Integer(p) => {
                            if *p < 0 || *p > 28 {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "round() precision must be between 0 and 28".to_string()
                                ));
                            }
                            *p as u32
                        }
                        _ => {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "round() precision must be an integer".to_string()
                            ));
                        }
                    }
                };

                match &context.input[0] {
                    FhirPathValue::Integer(i) => {
                        if precision == 0 {
                            Ok(vec![FhirPathValue::Integer(*i)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(Decimal::from(*i))])
                        }
                    }
                    FhirPathValue::Decimal(d) => {
                        let result = d.round_dp(precision);
                        if precision == 0 && result.fract().is_zero() {
                            if let Some(int_value) = result.to_i64() {
                                Ok(vec![FhirPathValue::Integer(int_value)])
                            } else {
                                Ok(vec![FhirPathValue::Decimal(result)])
                            }
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    FhirPathValue::Quantity { value, unit, ucum_unit, calendar_unit } => {
                        let result = value.round_dp(precision);
                        Ok(vec![FhirPathValue::Quantity {
                            value: result,
                            unit: unit.clone(),
                            ucum_unit: ucum_unit.clone(),
                            calendar_unit: *calendar_unit,
                        }])
                    }
                    _ => Err(crate::core::FhirPathError::evaluation_error(
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "truncate() can only be called on a single numeric value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Integer(i) => Ok(vec![FhirPathValue::Integer(*i)]),
                    FhirPathValue::Decimal(d) => {
                        let result = d.trunc();
                        if let Some(int_value) = result.to_i64() {
                            Ok(vec![FhirPathValue::Integer(int_value)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    FhirPathValue::Quantity { value, .. } => {
                        let result = value.trunc();
                        if let Some(int_value) = result.to_i64() {
                            Ok(vec![FhirPathValue::Integer(int_value)])
                        } else {
                            Ok(vec![FhirPathValue::Decimal(result)])
                        }
                    }
                    _ => Err(crate::core::FhirPathError::evaluation_error(
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "sqrt() can only be called on a single numeric value".to_string()
                    ));
                }

                let value = match &context.input[0] {
                    FhirPathValue::Integer(i) => {
                        if *i < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "sqrt() cannot be called on negative values".to_string()
                            ));
                        }
                        *i as f64
                    }
                    FhirPathValue::Decimal(d) => {
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
                    FhirPathValue::Quantity { value, .. } => {
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
                    _ => {
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

                Ok(vec![FhirPathValue::Decimal(decimal_result)])
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "ln() can only be called on a single numeric value".to_string()
                    ));
                }

                let value = extract_numeric_value(&context.input[0], "ln()")?;

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

                Ok(vec![FhirPathValue::Decimal(decimal_result)])
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
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

                let value = extract_numeric_value(&context.input[0], "log()")?;
                let base = extract_numeric_value(&context.arguments[0], "log() base")?;

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

                Ok(vec![FhirPathValue::Decimal(decimal_result)])
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
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

                let base = extract_numeric_value(&context.input[0], "power()")?;
                let exponent = extract_numeric_value(&context.arguments[0], "power() exponent")?;

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

                Ok(vec![FhirPathValue::Decimal(decimal_result)])
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
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "exp() can only be called on a single numeric value".to_string()
                    ));
                }

                let value = extract_numeric_value(&context.input[0], "exp()")?;

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

                Ok(vec![FhirPathValue::Decimal(decimal_result)])
            }
        )
    }
}

/// Helper function to extract numeric values from FhirPathValue
fn extract_numeric_value(value: &FhirPathValue, context: &str) -> Result<f64> {
    match value {
        FhirPathValue::Integer(i) => Ok(*i as f64),
        FhirPathValue::Decimal(d) => {
            d.to_f64().ok_or_else(|| {
                crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    format!("Invalid decimal value for {}", context)
                )
            })
        }
        FhirPathValue::Quantity { value, .. } => {
            value.to_f64().ok_or_else(|| {
                crate::core::FhirPathError::evaluation_error(
                    FP0053,
                    format!("Invalid quantity value for {}", context)
                )
            })
        }
        _ => Err(crate::core::FhirPathError::evaluation_error(
            FP0053,
            format!("{} can only be called on numeric values", context)
        )),
    }
}

/// Arithmetic operations utilities for FHIRPath expressions
pub struct ArithmeticOperations;

impl ArithmeticOperations {
    pub fn add(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, 
             FhirPathValue::Quantity { value: b, unit: unit_b, ucum_unit: ucum_b, calendar_unit: cal_b }) => {
                if unit_a != unit_b {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot add quantities with different units".to_string()
                    ));
                }
                Ok(FhirPathValue::Quantity {
                    value: a + b,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone().or_else(|| ucum_b.clone()),
                    calendar_unit: cal_a.or(*cal_b),
                })
            }
            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot add values of these types".to_string()
            ))
        }
    }

    pub fn subtract(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, 
             FhirPathValue::Quantity { value: b, unit: unit_b, ucum_unit: ucum_b, calendar_unit: cal_b }) => {
                if unit_a != unit_b {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "Cannot subtract quantities with different units".to_string()
                    ));
                }
                Ok(FhirPathValue::Quantity {
                    value: a - b,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone().or_else(|| ucum_b.clone()),
                    calendar_unit: cal_a.or(*cal_b),
                })
            }
            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot subtract values of these types".to_string()
            ))
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, FhirPathValue::Integer(b)) => {
                let b_decimal = Decimal::from(*b);
                Ok(FhirPathValue::Quantity {
                    value: a * b_decimal,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            (FhirPathValue::Integer(a), FhirPathValue::Quantity { value: b, unit: unit_b, ucum_unit: ucum_b, calendar_unit: cal_b }) => {
                let a_decimal = Decimal::from(*a);
                Ok(FhirPathValue::Quantity {
                    value: a_decimal * b,
                    unit: unit_b.clone(),
                    ucum_unit: ucum_b.clone(),
                    calendar_unit: *cal_b,
                })
            }
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Quantity {
                    value: a * b,
                    unit: unit_a.clone(),
                    ucum_unit: ucum_a.clone(),
                    calendar_unit: *cal_a,
                })
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Quantity { value: b, unit: unit_b, ucum_unit: ucum_b, calendar_unit: cal_b }) => {
                Ok(FhirPathValue::Quantity {
                    value: a * b,
                    unit: unit_b.clone(),
                    ucum_unit: ucum_b.clone(),
                    calendar_unit: *cal_b,
                })
            }
            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot multiply values of these types".to_string()
            ))
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, FhirPathValue::Integer(b)) => {
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, FhirPathValue::Decimal(b)) => {
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
            _ => None // Invalid types for division return empty collection
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
            _ => None // Invalid types for modulo return empty collection
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, FhirPathValue::Integer(b)) => {
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
            (FhirPathValue::Quantity { value: a, unit: unit_a, ucum_unit: ucum_a, calendar_unit: cal_a }, FhirPathValue::Decimal(b)) => {
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
            _ => None // Invalid types for integer division return empty collection
        }
    }
}

// mod math_tests;