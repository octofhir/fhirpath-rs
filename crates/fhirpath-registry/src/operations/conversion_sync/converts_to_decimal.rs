//! convertsToDecimal() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_core::FhirPathValue;

/// convertsToDecimal(): Returns true if the input can be converted to Decimal
pub struct ConvertsToDecimalFunction;

impl SyncOperation for ConvertsToDecimalFunction {
    fn name(&self) -> &'static str {
        "convertsToDecimal"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToDecimal",
            parameters: vec![],
            return_type: ValueType::Boolean,
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
        // Handle collections by applying convertsToDecimal to each element
        match &context.input {
            FhirPathValue::Collection(col) => {
                // Apply convertsToDecimal to each element in the collection
                let results: Result<Vec<FhirPathValue>> = col
                    .iter()
                    .map(|item| {
                        let can_convert = can_convert_to_decimal(item)?;
                        Ok(FhirPathValue::Boolean(can_convert))
                    })
                    .collect();

                Ok(FhirPathValue::collection(results?))
            }
            _ => {
                // Single element - original behavior
                let can_convert = can_convert_to_decimal(&context.input)?;
                Ok(FhirPathValue::Boolean(can_convert))
            }
        }
    }
}

fn can_convert_to_decimal(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a decimal
        FhirPathValue::Decimal(_) => Ok(true),

        // Integer can be converted to decimal
        FhirPathValue::Integer(_) => Ok(true),

        // String values that can be parsed as decimal
        FhirPathValue::String(s) => {
            use rust_decimal::Decimal;
            use std::str::FromStr;
            Ok(Decimal::from_str(s.trim()).is_ok())
        }

        // Boolean can be converted (true = 1.0, false = 0.0)
        FhirPathValue::Boolean(_) => Ok(true),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_decimal(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to decimal
        _ => Ok(false),
    }
}
