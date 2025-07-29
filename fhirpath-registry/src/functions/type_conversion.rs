//! Type conversion functions

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// toString() function - converts value to string
pub struct ToStringFunction;

impl FhirPathFunction for ToStringFunction {
    fn name(&self) -> &str {
        "toString"
    }
    fn human_friendly_name(&self) -> &str {
        "To String"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toString", vec![], TypeInfo::String)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::String(s) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                s.clone(),
            )])),
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    i.to_string(),
                )]))
            }
            FhirPathValue::Decimal(d) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    d.to_string(),
                )]))
            }
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    b.to_string(),
                )]))
            }
            FhirPathValue::Date(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                d.to_string(),
            )])),
            FhirPathValue::DateTime(dt) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                    dt.to_string(),
                )]))
            }
            FhirPathValue::Time(t) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(
                t.to_string(),
            )])),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// toInteger() function - converts value to integer
pub struct ToIntegerFunction;

impl FhirPathFunction for ToIntegerFunction {
    fn name(&self) -> &str {
        "toInteger"
    }
    fn human_friendly_name(&self) -> &str {
        "To Integer"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toInteger", vec![], TypeInfo::Integer)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)]))
            }
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
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(
                    if *b { 1 } else { 0 },
                )]))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// toDecimal() function - converts value to decimal
pub struct ToDecimalFunction;

impl FhirPathFunction for ToDecimalFunction {
    fn name(&self) -> &str {
        "toDecimal"
    }
    fn human_friendly_name(&self) -> &str {
        "To Decimal"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toDecimal", vec![], TypeInfo::Decimal)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Decimal(d) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(*d)]))
            }
            FhirPathValue::Integer(i) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                    Decimal::from(*i),
                )]))
            }
            FhirPathValue::String(s) => match Decimal::from_str(s.trim()) {
                Ok(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(d)])),
                Err(_) => Ok(FhirPathValue::Empty),
            },
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                    if *b { Decimal::ONE } else { Decimal::ZERO },
                )]))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// convertsToInteger() function - checks if value can be converted to integer
pub struct ConvertsToIntegerFunction;

impl FhirPathFunction for ConvertsToIntegerFunction {
    fn name(&self) -> &str {
        "convertsToInteger"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Integer"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToInteger", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let can_convert = match input_item {
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => !s.contains('.') && s.trim().parse::<i64>().is_ok(),
            FhirPathValue::Boolean(_) => true,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}

/// convertsToDecimal() function - checks if value can be converted to decimal
pub struct ConvertsToDecimalFunction;

impl FhirPathFunction for ConvertsToDecimalFunction {
    fn name(&self) -> &str {
        "convertsToDecimal"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Decimal"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToDecimal", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
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
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}

/// convertsToString() function - checks if value can be converted to string
pub struct ConvertsToStringFunction;

impl FhirPathFunction for ConvertsToStringFunction {
    fn name(&self) -> &str {
        "convertsToString"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To String"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToString", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
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
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}

/// toBoolean() function - converts value to boolean
pub struct ToBooleanFunction;

impl FhirPathFunction for ToBooleanFunction {
    fn name(&self) -> &str {
        "toBoolean"
    }
    fn human_friendly_name(&self) -> &str {
        "To Boolean"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toBoolean", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)]))
            }
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => Ok(FhirPathValue::collection(
                        vec![FhirPathValue::Boolean(true)],
                    )),
                    "false" | "f" | "no" | "n" | "0" | "0.0" => Ok(FhirPathValue::collection(
                        vec![FhirPathValue::Boolean(false)],
                    )),
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Integer(i) => match *i {
                1 => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )])),
                0 => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    false,
                )])),
                _ => Ok(FhirPathValue::Empty),
            },
            FhirPathValue::Decimal(d) => {
                if *d == Decimal::ONE {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        true,
                    )]))
                } else if *d == Decimal::ZERO {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        false,
                    )]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// convertsToBoolean() function - checks if value can be converted to boolean
pub struct ConvertsToBooleanFunction;

impl FhirPathFunction for ConvertsToBooleanFunction {
    fn name(&self) -> &str {
        "convertsToBoolean"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Boolean"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToBoolean", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let can_convert = match input_item {
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::String(s) => {
                let lower = s.to_lowercase();
                matches!(
                    lower.as_str(),
                    "true"
                        | "t"
                        | "yes"
                        | "y"
                        | "1"
                        | "1.0"
                        | "false"
                        | "f"
                        | "no"
                        | "n"
                        | "0"
                        | "0.0"
                )
            }
            FhirPathValue::Integer(i) => *i == 0 || *i == 1,
            FhirPathValue::Decimal(d) => *d == Decimal::ZERO || *d == Decimal::ONE,
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}

/// type() function - returns the type of the value
pub struct TypeFunction;

impl FhirPathFunction for TypeFunction {
    fn name(&self) -> &str {
        "type"
    }
    fn human_friendly_name(&self) -> &str {
        "Type"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "type",
                vec![],
                TypeInfo::Any, // Returns a TypeInfo object
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let type_info = match input_item {
            FhirPathValue::String(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "String".to_string(),
            },
            FhirPathValue::Integer(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Integer".to_string(),
            },
            FhirPathValue::Decimal(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Decimal".to_string(),
            },
            FhirPathValue::Boolean(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Boolean".to_string(),
            },
            FhirPathValue::Date(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Date".to_string(),
            },
            FhirPathValue::DateTime(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "DateTime".to_string(),
            },
            FhirPathValue::Time(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Time".to_string(),
            },
            FhirPathValue::Quantity(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Quantity".to_string(),
            },
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, determine the appropriate type
                let resource_type = resource.resource_type();

                // Check if this is a FHIR primitive type by examining the value
                if let Some(json_value) = resource.as_json().as_bool() {
                    // Boolean primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "boolean".to_string(),
                    }
                } else if let Some(json_value) = resource.as_json().as_str() {
                    // String-based FHIR primitive
                    // Check if it looks like a UUID or URI
                    let fhir_type = if json_value.starts_with("urn:uuid:") {
                        "uuid"
                    } else if json_value.starts_with("http://")
                        || json_value.starts_with("https://")
                        || json_value.starts_with("urn:")
                    {
                        "uri"
                    } else {
                        "string"
                    };
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: fhir_type.to_string(),
                    }
                } else if let Some(json_value) = resource.as_json().as_i64() {
                    // Integer primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "integer".to_string(),
                    }
                } else if let Some(json_value) = resource.as_json().as_f64() {
                    // Decimal primitive in FHIR context
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "decimal".to_string(),
                    }
                } else if resource_type.is_some() {
                    // This is a complex FHIR resource with a resourceType
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: resource_type.unwrap().to_string(),
                    }
                } else {
                    // Unknown resource type
                    FhirPathValue::TypeInfoObject {
                        namespace: "FHIR".to_string(),
                        name: "Unknown".to_string(),
                    }
                }
            }
            FhirPathValue::Collection(_) => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "Collection".to_string(),
            },
            FhirPathValue::TypeInfoObject { .. } => FhirPathValue::TypeInfoObject {
                namespace: "System".to_string(),
                name: "TypeInfo".to_string(),
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };
        Ok(FhirPathValue::collection(vec![type_info]))
    }
}

/// convertsToDate() function - checks if value can be converted to date
pub struct ConvertsToDateFunction;

impl FhirPathFunction for ConvertsToDateFunction {
    fn name(&self) -> &str {
        "convertsToDate"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Date"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToDate", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let can_convert = match input_item {
            FhirPathValue::Date(_) => true,
            FhirPathValue::String(s) => {
                // Check if string matches valid date formats: YYYY, YYYY-MM, YYYY-MM-DD
                let date_regex = regex::Regex::new(r"^\d{4}(-\d{2}(-\d{2})?)?$").unwrap();
                date_regex.is_match(s)
            }
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}

/// convertsToDateTime() function - checks if value can be converted to datetime
pub struct ConvertsToDateTimeFunction;

impl FhirPathFunction for ConvertsToDateTimeFunction {
    fn name(&self) -> &str {
        "convertsToDateTime"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To DateTime"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToDateTime", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let can_convert = match input_item {
            FhirPathValue::DateTime(_) => true,
            FhirPathValue::Date(_) => true, // Date can be converted to DateTime
            FhirPathValue::String(s) => {
                // Check if string matches valid datetime formats
                let datetime_regex = regex::Regex::new(r"^\d{4}(-\d{2}(-\d{2}(T\d{2}(:\d{2}(:\d{2}(\.\d{3})?)?)?(Z|[+-]\d{2}:\d{2})?)?)?)?$").unwrap();
                datetime_regex.is_match(s)
            }
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}

/// convertsToTime() function - checks if value can be converted to time
pub struct ConvertsToTimeFunction;

impl FhirPathFunction for ConvertsToTimeFunction {
    fn name(&self) -> &str {
        "convertsToTime"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Time"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToTime", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        let can_convert = match input_item {
            FhirPathValue::Time(_) => true,
            FhirPathValue::String(s) => {
                // Check if string matches valid time formats: HH, HH:MM, HH:MM:SS, HH:MM:SS.mmm
                let time_regex = regex::Regex::new(r"^\d{2}(:\d{2}(:\d{2}(\.\d{3})?)?)?$").unwrap();
                time_regex.is_match(s)
            }
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}
