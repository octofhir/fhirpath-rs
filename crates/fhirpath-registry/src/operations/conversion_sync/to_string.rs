//! toString() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// toString(): Converts input to String where possible
pub struct ToStringFunction;

impl SyncOperation for ToStringFunction {
    fn name(&self) -> &'static str {
        "toString"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toString",
            parameters: vec![],
            return_type: ValueType::String,
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
        convert_to_string(&context.input)
    }
}

fn convert_to_string(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a string
        FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),

        // Integer conversion
        FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string().into())),

        // Decimal conversion with proper formatting
        FhirPathValue::Decimal(d) => {
            // Format decimal according to FHIRPath specification
            let formatted = format_decimal(*d);
            Ok(FhirPathValue::String(formatted.into()))
        }

        // Boolean conversion
        FhirPathValue::Boolean(b) => {
            let s = if *b { "true" } else { "false" };
            Ok(FhirPathValue::String(s.into()))
        }

        // Date conversion
        FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.to_string().into())),

        // DateTime conversion
        FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.to_string().into())),

        // Time conversion
        FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.to_string().into())),

        // Quantity conversion
        FhirPathValue::Quantity(q) => {
            let formatted_value = format_decimal(q.value);
            let result = if let Some(unit) = &q.unit {
                // Only quote UCUM units, leave standard units unquoted
                match unit.as_str() {
                    "wk" | "mo" | "a" | "d" => format!("{formatted_value} '{unit}'"),
                    _ => format!("{formatted_value} {unit}"),
                }
            } else {
                formatted_value
            };
            Ok(FhirPathValue::String(result.into()))
        }

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_string(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }

        // Empty input
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Unsupported types
        _ => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection for unsupported types
    }
}

fn format_decimal(decimal: rust_decimal::Decimal) -> String {
    // Format decimal according to FHIRPath specification
    let s = decimal.to_string();

    // Remove trailing zeros after decimal point
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        s
    }
}
