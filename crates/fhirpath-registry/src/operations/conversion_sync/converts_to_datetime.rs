//! convertsToDateTime() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToDateTime(): Returns true if the input can be converted to DateTime
pub struct ConvertsToDateTimeFunction;

impl SyncOperation for ConvertsToDateTimeFunction {
    fn name(&self) -> &'static str {
        "convertsToDateTime"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToDateTime",
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
        let can_convert = can_convert_to_datetime(&context.input)?;
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

fn can_convert_to_datetime(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a datetime
        FhirPathValue::DateTime(_) => Ok(true),

        // Date can be converted to DateTime (add time 00:00:00)
        FhirPathValue::Date(_) => Ok(true),

        // String values that can be parsed as ISO datetime format
        FhirPathValue::String(s) => Ok(parse_iso_datetime_string(s).is_some()),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_datetime(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to datetime
        _ => Ok(false),
    }
}

fn parse_iso_datetime_string(s: &str) -> Option<()> {
    // ISO datetime formats:
    // YYYY-MM-DD
    // YYYY-MM-DDTHH:MM:SS
    // YYYY-MM-DDTHH:MM:SS.sss
    // YYYY-MM-DDTHH:MM:SS+ZZ:ZZ
    // YYYY-MM-DDTHH:MM:SS.sss+ZZ:ZZ

    if s.len() >= 10 {
        // Check if it starts with a valid date part
        let date_part = &s[..10];
        if parse_iso_date_part(date_part).is_some() {
            // If it's just a date, it's valid
            if s.len() == 10 {
                return Some(());
            }

            // If there's more, check for 'T' separator
            if s.len() > 10 && s.chars().nth(10) == Some('T') {
                // Basic time validation - could be more comprehensive
                return Some(());
            }
        }
    }
    None
}

fn parse_iso_date_part(s: &str) -> Option<()> {
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
