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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        match input_item {
            FhirPathValue::String(s) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(s.clone())])),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(i.to_string())])),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(d.to_string())])),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(b.to_string())])),
            FhirPathValue::Date(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(d.to_string())])),
            FhirPathValue::DateTime(dt) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(dt.to_string())])),
            FhirPathValue::Time(t) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(t.to_string())])),
            _ => Ok(FhirPathValue::Empty),
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        match input_item {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
            FhirPathValue::String(s) => {
                // According to FHIRPath spec, strings with decimal points cannot be converted to integers
                if s.contains('.') {
                    Ok(FhirPathValue::Empty)
                } else {
                    match s.trim().parse::<i64>() {
                        Ok(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)])),
                        Err(_) => Ok(FhirPathValue::Empty),
                    }
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(if *b { 1 } else { 0 })])),
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        match input_item {
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(*d)])),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(Decimal::from(*i))])),
            FhirPathValue::String(s) => {
                match Decimal::from_str(s.trim()) {
                    Ok(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(d)])),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(if *b { Decimal::ONE } else { Decimal::ZERO })])),
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        let can_convert = match input_item {
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => !s.contains('.') && s.trim().parse::<i64>().is_ok(),
            FhirPathValue::Boolean(_) => true,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        let can_convert = match input_item {
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => Decimal::from_str(s.trim()).is_ok(),
            FhirPathValue::Boolean(_) => true,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        let can_convert = match input_item {
            FhirPathValue::String(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::Date(_) => true,
            FhirPathValue::DateTime(_) => true,
            FhirPathValue::Time(_) => true,
            FhirPathValue::Quantity(_) => true,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
    }
}

/// toBoolean() function - converts value to boolean
pub struct ToBooleanFunction;

impl FhirPathFunction for ToBooleanFunction {
    fn name(&self) -> &str { "toBoolean" }
    fn human_friendly_name(&self) -> &str { "To Boolean" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toBoolean",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        match input_item {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)])),
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)])),
                    "false" | "f" | "no" | "n" | "0" | "0.0" => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
                    _ => Ok(FhirPathValue::Empty),
                }
            },
            FhirPathValue::Integer(i) => {
                match *i {
                    1 => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)])),
                    0 => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
                    _ => Ok(FhirPathValue::Empty),
                }
            },
            FhirPathValue::Decimal(d) => {
                if *d == Decimal::ONE {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]))
                } else if *d == Decimal::ZERO {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            },
            _ => Ok(FhirPathValue::Empty),
        }
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        let can_convert = match input_item {
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                matches!(lower.as_str(), "true" | "t" | "yes" | "y" | "1" | "1.0" | "false" | "f" | "no" | "n" | "0" | "0.0")
            },
            FhirPathValue::Integer(i) => *i == 0 || *i == 1,
            FhirPathValue::Decimal(d) => *d == Decimal::ZERO || *d == Decimal::ONE,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
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
        
        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };
        
        let type_name = match input_item {
            FhirPathValue::String(_) => "String",
            FhirPathValue::Integer(_) => "Integer",
            FhirPathValue::Decimal(_) => "Decimal",
            FhirPathValue::Boolean(_) => "Boolean",
            FhirPathValue::Date(_) => "Date",
            FhirPathValue::DateTime(_) => "DateTime",
            FhirPathValue::Time(_) => "Time",
            FhirPathValue::Quantity(_) => "Quantity",
            FhirPathValue::Resource(_) => "Resource",
            FhirPathValue::Collection(_) => "Collection",
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::String(type_name.to_string())]))
    }
}