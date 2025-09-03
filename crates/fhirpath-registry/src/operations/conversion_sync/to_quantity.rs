//! toQuantity { value: , .. } sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;
use rust_decimal::Decimal;
use std::sync::Arc;

/// toQuantity { value: , .. }: Converts input to Quantity where possible
pub struct ToQuantityFunction;

impl SyncOperation for ToQuantityFunction {
    fn name(&self) -> &'static str {
        "toQuantity"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toQuantity",
            parameters: vec![],
            return_type: ValueType::Quantity,
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
        convert_to_quantity(&context.input)
    }
}

fn convert_to_quantity(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a quantity
        FhirPathValue::Quantity { value, unit, .. } => Ok(FhirPathValue::Quantity { 
            value: value.clone(), 
            unit: unit.clone(), 
            ucum_expr: None 
        }),

        // Integer can be converted (becomes quantity with unit "1")
        FhirPathValue::Integer(i) => {
            Ok(FhirPathValue::Quantity { 
                value: Decimal::new(*i, 0), 
                unit: Some("1".to_string()), 
                ucum_expr: None 
            })
        }

        // Decimal can be converted (becomes quantity with unit "1")
        FhirPathValue::Decimal(d) => {
            Ok(FhirPathValue::Quantity { 
                value: *d, 
                unit: Some("1".to_string()), 
                ucum_expr: None 
            })
        }

        // String conversion with quantity parsing
        FhirPathValue::String(s) => match parse_quantity_string(s) {
            Some((value, unit)) => {
                Ok(FhirPathValue::Quantity { 
                    value: Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO), 
                    unit, 
                    ucum_expr: None 
                })
            }
            None => Err(FhirPathError::ConversionError {
                from: format!("Cannot convert string '{s}' to Quantity"),
                to: "Quantity".to_string(),
            }),
        },

        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![]))
            } else if c.len() == 1 {
                convert_to_quantity(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![]))
            }
        }

        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: format!("Cannot convert {} to Quantity", value.type_name()),
            to: "Quantity".to_string(),
        }),
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
