//! toDecimal() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use rust_decimal::Decimal;
use std::str::FromStr;

/// toDecimal(): Converts input to Decimal where possible
pub struct ToDecimalFunction;

impl SyncOperation for ToDecimalFunction {
    fn name(&self) -> &'static str {
        "toDecimal"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toDecimal",
            parameters: vec![],
            return_type: ValueType::Decimal,
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
        convert_to_decimal(&context.input)
    }
}

fn convert_to_decimal(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a decimal
        FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),

        // Integer can be converted to decimal
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(Decimal::new(*i, 0))),

        // String conversion with proper decimal parsing
        FhirPathValue::String(s) => {
            match Decimal::from_str(s.trim()) {
                Ok(decimal) => Ok(FhirPathValue::Decimal(decimal)),
                Err(_) => Ok(FhirPathValue::Empty), // Return empty collection for invalid conversions per FHIRPath spec
            }
        }

        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![]))
            } else if c.len() == 1 {
                convert_to_decimal(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![]))
            }
        }

        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: "Unsupported type".to_string(),
            to: "Decimal".to_string(),
        }),
    }
}
