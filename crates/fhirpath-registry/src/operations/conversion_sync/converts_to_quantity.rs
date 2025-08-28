//! convertsToQuantity() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToQuantity(): Returns true if the input can be converted to Quantity
pub struct ConvertsToQuantityFunction;

impl SyncOperation for ConvertsToQuantityFunction {
    fn name(&self) -> &'static str {
        "convertsToQuantity"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToQuantity",
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
        let can_convert = can_convert_to_quantity(&context.input)?;
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

fn can_convert_to_quantity(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a quantity
        FhirPathValue::Quantity(_) => Ok(true),

        // Integer can be converted (becomes quantity with unit "1")
        FhirPathValue::Integer(_) => Ok(true),

        // Decimal can be converted (becomes quantity with unit "1")
        FhirPathValue::Decimal(_) => Ok(true),

        // String values that can be parsed as quantity format
        FhirPathValue::String(s) => Ok(parse_quantity_string(s).is_some()),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_quantity(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to quantity
        _ => Ok(false),
    }
}

fn parse_quantity_string(s: &str) -> Option<(f64, Option<String>)> {
    let s = s.trim();

    // Try to parse just a number (no unit)
    if let Ok(value) = s.parse::<f64>() {
        return Some((value, None));
    }

    // Try to parse "value unit" format
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 2 {
        if let Ok(value) = parts[0].parse::<f64>() {
            let unit = parts[1].to_string();
            // Remove quotes from unit if present
            let unit = if unit.starts_with('\'') && unit.ends_with('\'') {
                unit[1..unit.len() - 1].to_string()
            } else {
                unit
            };
            return Some((value, Some(unit)));
        }
    }

    // Try to parse formats like "5mg", "10.5kg", etc.
    let mut split_pos = None;
    for (i, c) in s.char_indices() {
        if c.is_alphabetic() || c == '\'' {
            split_pos = Some(i);
            break;
        }
    }

    if let Some(pos) = split_pos {
        let (value_part, unit_part) = s.split_at(pos);
        if let Ok(value) = value_part.parse::<f64>() {
            let unit = unit_part.trim();
            let unit = if unit.starts_with('\'') && unit.ends_with('\'') {
                unit[1..unit.len() - 1].to_string()
            } else {
                unit.to_string()
            };
            return Some((value, Some(unit)));
        }
    }

    None
}
