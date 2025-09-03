//! toLong() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// toLong(): Converts input to Long (64-bit integer) where possible
pub struct ToLongFunction;

impl SyncOperation for ToLongFunction {
    fn name(&self) -> &'static str {
        "toLong"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toLong",
            parameters: vec![],
            return_type: ValueType::Integer, // Long maps to Integer in our type system
            variadic: false,
            category: FunctionCategory::Scalar,
            cardinality_requirement: CardinalityRequirement::AcceptsBoth,
        };
        &SIGNATURE
    }

    fn execute(
        &self,
        _args: &[FhirPathValue],
        context: &crate::traits::EvaluationContext,
    ) -> Result<FhirPathValue> {
        convert_to_long(&context.input)
    }
}

fn convert_to_long(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already an integer (which is i64 in our implementation)
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),

        // Decimal can be converted if it's a whole number within i64 range
        FhirPathValue::Decimal(d) => {
            if d.fract().is_zero() {
                match d.to_i64() {
                    Some(i) => {
                        // Ensure it's within i64 range
                        if (i64::MIN..=i64::MAX).contains(&i) {
                            Ok(FhirPathValue::Integer(i))
                        } else {
                            Err(FhirPathError::ConversionError {
                                from: format!("Decimal value {d} is out of Long range"),
                                to: "Long".to_string(),
                            })
                        }
                    }
                    None => Err(FhirPathError::ConversionError {
                        from: format!("Cannot convert decimal {d} to Long"),
                        to: "Long".to_string(),
                    }),
                }
            } else {
                Err(FhirPathError::ConversionError {
                    from: format!("Cannot convert decimal {d} to Long (has fractional part)"),
                    to: "Long".to_string(),
                })
            }
        }

        // String conversion with proper long parsing
        FhirPathValue::String(s) => match s.trim().parse::<i64>() {
            Ok(i) => Ok(FhirPathValue::Integer(i)),
            Err(_) => Err(FhirPathError::ConversionError {
                from: format!("Cannot convert string '{s}' to Long"),
                to: "Long".to_string(),
            }),
        },

        // Boolean conversion (true = 1, false = 0)
        FhirPathValue::Boolean(b) => {
            let i = if *b { 1i64 } else { 0i64 };
            Ok(FhirPathValue::Integer(i))
        }

        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![]))
            } else if c.len() == 1 {
                convert_to_long(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![]))
            }
        }

        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: format!("Cannot convert {} to Long", value.type_name()),
            to: "Long".to_string(),
        }),
    }
}
