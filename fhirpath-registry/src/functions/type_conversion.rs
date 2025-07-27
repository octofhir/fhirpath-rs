//! Type conversion functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// toString() function - converts value to string
pub struct ToStringFunction;

impl FhirPathFunction for ToStringFunction {
    fn name(&self) -> &str { "toString" }
    fn human_friendly_name(&self) -> &str { "To String" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toString",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string())),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::String(d.to_string())),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string())),
            FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.to_string())),
            FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.to_string())),
            FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.to_string())),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Cannot convert this type to string".to_string(),
            }),
        }
    }
}

/// toInteger() function - converts value to integer
pub struct ToIntegerFunction;

impl FhirPathFunction for ToIntegerFunction {
    fn name(&self) -> &str { "toInteger" }
    fn human_friendly_name(&self) -> &str { "To Integer" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toInteger",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Integer(d.trunc().to_i64().unwrap_or(0))),
            FhirPathValue::String(s) => {
                match s.parse::<i64>() {
                    Ok(i) => Ok(FhirPathValue::Integer(i)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Integer(if *b { 1 } else { 0 })),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// toDecimal() function - converts value to decimal
pub struct ToDecimalFunction;

impl FhirPathFunction for ToDecimalFunction {
    fn name(&self) -> &str { "toDecimal" }
    fn human_friendly_name(&self) -> &str { "To Decimal" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toDecimal",
                vec![],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(Decimal::from(*i))),
            FhirPathValue::String(s) => {
                match Decimal::from_str(s) {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Decimal(if *b { Decimal::ONE } else { Decimal::ZERO })),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// convertsToInteger() function - checks if value can be converted to integer
pub struct ConvertsToIntegerFunction;

impl FhirPathFunction for ConvertsToIntegerFunction {
    fn name(&self) -> &str { "convertsToInteger" }
    fn human_friendly_name(&self) -> &str { "Converts To Integer" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToInteger",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let can_convert = match &context.input {
            FhirPathValue::Integer(_) => true,
            FhirPathValue::Decimal(d) => d.fract().is_zero(),
            FhirPathValue::String(s) => s.parse::<i64>().is_ok(),
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::Empty => false,
            _ => false,
        };
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

/// convertsToDecimal() function - checks if value can be converted to decimal
pub struct ConvertsToDecimalFunction;

impl FhirPathFunction for ConvertsToDecimalFunction {
    fn name(&self) -> &str { "convertsToDecimal" }
    fn human_friendly_name(&self) -> &str { "Converts To Decimal" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToDecimal",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let can_convert = match &context.input {
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => Decimal::from_str(s).is_ok(),
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::Empty => false,
            _ => false,
        };
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

/// convertsToString() function - checks if value can be converted to string
pub struct ConvertsToStringFunction;

impl FhirPathFunction for ConvertsToStringFunction {
    fn name(&self) -> &str { "convertsToString" }
    fn human_friendly_name(&self) -> &str { "Converts To String" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToString",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let can_convert = match &context.input {
            FhirPathValue::String(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::Date(_) => true,
            FhirPathValue::DateTime(_) => true,
            FhirPathValue::Time(_) => true,
            FhirPathValue::Empty => false,
            _ => false,
        };
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

/// convertsToBoolean() function - checks if value can be converted to boolean
pub struct ConvertsToBooleanFunction;

impl FhirPathFunction for ConvertsToBooleanFunction {
    fn name(&self) -> &str { "convertsToBoolean" }
    fn human_friendly_name(&self) -> &str { "Converts To Boolean" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToBoolean",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let can_convert = match &context.input {
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::String(s) => s == "true" || s == "false",
            FhirPathValue::Integer(i) => *i == 0 || *i == 1,
            FhirPathValue::Empty => false,
            _ => false,
        };
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

/// type() function - returns the type of the value
pub struct TypeFunction;

impl FhirPathFunction for TypeFunction {
    fn name(&self) -> &str { "type" }
    fn human_friendly_name(&self) -> &str { "Type" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "type",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let type_name = match &context.input {
            FhirPathValue::String(_) => "String",
            FhirPathValue::Integer(_) => "Integer",
            FhirPathValue::Decimal(_) => "Decimal",
            FhirPathValue::Boolean(_) => "Boolean",
            FhirPathValue::Date(_) => "Date",
            FhirPathValue::DateTime(_) => "DateTime",
            FhirPathValue::Time(_) => "Time",
            FhirPathValue::Quantity(_) => "Quantity",
            FhirPathValue::Collection(_) => "Collection",
            FhirPathValue::Resource(_) => "Resource",
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };
        Ok(FhirPathValue::String(type_name.to_string()))
    }
}