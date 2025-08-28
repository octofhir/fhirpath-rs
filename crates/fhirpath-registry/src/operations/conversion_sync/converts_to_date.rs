//! convertsToDate() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToDate(): Returns true if the input can be converted to Date
pub struct ConvertsToDateFunction;

impl SyncOperation for ConvertsToDateFunction {
    fn name(&self) -> &'static str {
        "convertsToDate"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToDate",
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
        let can_convert = can_convert_to_date(&context.input)?;
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

fn can_convert_to_date(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a date
        FhirPathValue::Date(_) => Ok(true),

        // DateTime can be converted to Date (truncate time part)
        FhirPathValue::DateTime(_) => Ok(true),

        // String values that can be parsed as ISO date format
        FhirPathValue::String(s) => Ok(parse_iso_date_string(s).is_some()),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_date(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to date
        _ => Ok(false),
    }
}

fn parse_iso_date_string(s: &str) -> Option<()> {
    // Basic ISO date format validation: YYYY-MM-DD
    if s.len() == 10 {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 3 {
            // Check year (4 digits)
            if parts[0].len() == 4 && parts[0].chars().all(|c| c.is_ascii_digit()) {
                // Check month (2 digits, 01-12)
                if parts[1].len() == 2 && parts[1].chars().all(|c| c.is_ascii_digit()) {
                    if let Ok(month) = parts[1].parse::<u32>() {
                        if (1..=12).contains(&month) {
                            // Check day (2 digits, 01-31)
                            if parts[2].len() == 2 && parts[2].chars().all(|c| c.is_ascii_digit()) {
                                if let Ok(day) = parts[2].parse::<u32>() {
                                    if (1..=31).contains(&day) {
                                        return Some(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
