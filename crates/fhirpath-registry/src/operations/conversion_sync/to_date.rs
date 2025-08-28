//! toDate() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use chrono::NaiveDate;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{
    FhirPathValue,
    temporal::{PrecisionDate, TemporalPrecision},
};

/// toDate(): Converts input to Date where possible
pub struct ToDateFunction;

impl SyncOperation for ToDateFunction {
    fn name(&self) -> &'static str {
        "toDate"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toDate",
            parameters: vec![],
            return_type: ValueType::Date,
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
        convert_to_date(&context.input)
    }
}

fn convert_to_date(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a date
        FhirPathValue::Date(d) => Ok(FhirPathValue::Date(d.clone())),

        // DateTime can be converted to Date (extract date part)
        FhirPathValue::DateTime(dt) => {
            let date_part = dt.datetime.date_naive();
            let precision_date = PrecisionDate::new(date_part, TemporalPrecision::Day);
            Ok(FhirPathValue::Date(precision_date))
        }

        // String conversion with ISO format validation
        FhirPathValue::String(s) => {
            match parse_iso_date_string(s) {
                Some(date) => Ok(FhirPathValue::Date(date)),
                None => Ok(FhirPathValue::Collection(vec![].into())), // Return empty for invalid strings
            }
        }

        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_date(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }

        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: "Unsupported type".to_string(),
            to: "Date".to_string(),
        }),
    }
}

pub fn parse_iso_date_string(s: &str) -> Option<PrecisionDate> {
    // Support partial dates: YYYY, YYYY-MM, YYYY-MM-DD
    let s = s.trim();

    let parts: Vec<&str> = s.split('-').collect();
    if parts.is_empty() || parts[0].is_empty() {
        return None;
    }

    // Parse year (4 digits)
    let year = parts[0].parse::<i32>().ok()?;
    if !(1..=9999).contains(&year) {
        return None;
    }

    match parts.len() {
        1 => {
            // Year only: YYYY
            let date = NaiveDate::from_ymd_opt(year, 1, 1)?;
            Some(PrecisionDate::new(date, TemporalPrecision::Year))
        }
        2 => {
            // Year-Month: YYYY-MM
            let month = parts[1].parse::<u32>().ok()?;
            if !(1..=12).contains(&month) {
                return None;
            }
            let date = NaiveDate::from_ymd_opt(year, month, 1)?;
            Some(PrecisionDate::new(date, TemporalPrecision::Month))
        }
        3 => {
            // Year-Month-Day: YYYY-MM-DD
            let month = parts[1].parse::<u32>().ok()?;
            if !(1..=12).contains(&month) {
                return None;
            }
            let day = parts[2].parse::<u32>().ok()?;
            if !(1..=31).contains(&day) {
                return None;
            }
            let date = NaiveDate::from_ymd_opt(year, month, day)?;
            Some(PrecisionDate::new(date, TemporalPrecision::Day))
        }
        _ => None,
    }
}
