//! toInteger() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// toInteger(): Converts input to Integer where possible
pub struct ToIntegerFunction;

impl SyncOperation for ToIntegerFunction {
    fn name(&self) -> &'static str {
        "toInteger"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toInteger",
            parameters: vec![],
            return_type: ValueType::Integer,
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
        convert_to_integer(&context.input)
    }
}

fn convert_to_integer(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already an integer
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),

        // Decimal can be converted if it's a whole number
        FhirPathValue::Decimal(d) => {
            if d.fract().is_zero() {
                match d.to_i64() {
                    Some(i) => Ok(FhirPathValue::Integer(i)),
                    None => Ok(FhirPathValue::Empty), // Return empty for out-of-range decimals
                }
            } else {
                Ok(FhirPathValue::Empty) // Return empty for non-whole decimals per FHIRPath spec
            }
        }

        // String conversion with proper integer parsing
        FhirPathValue::String(s) => {
            match s.trim().parse::<i64>() {
                Ok(i) => Ok(FhirPathValue::Integer(i)),
                Err(_) => Ok(FhirPathValue::Empty), // Return empty collection for invalid conversions per FHIRPath spec
            }
        }

        // Boolean conversion (true = 1, false = 0)
        FhirPathValue::Boolean(b) => {
            let i = if *b { 1 } else { 0 };
            Ok(FhirPathValue::Integer(i))
        }

        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_integer(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }

        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: "Unsupported type".to_string(),
            to: "Integer".to_string(),
        }),
    }
}
