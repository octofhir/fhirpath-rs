//! toTime() sync implementation
use octofhir_fhirpath_core::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use chrono::NaiveTime;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_core::FhirPathValue;

/// toTime(): Converts input to Time where possible
pub struct ToTimeFunction;

impl SyncOperation for ToTimeFunction {
    fn name(&self) -> &'static str {
        "toTime"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toTime",
            parameters: vec![],
            return_type: ValueType::Time,
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
        convert_to_time(&context.input)
    }
}

fn convert_to_time(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a time
        FhirPathValue::Time(t) => Ok(FhirPathValue::Time(t.clone())),

        // DateTime can be converted to Time (extract time part)
        FhirPathValue::DateTime(dt) => {
            let time_part = dt.datetime.time();
            let precision_time = PrecisionTime::new(time_part, TemporalPrecision::Second);
            Ok(FhirPathValue::Time(precision_time))
        }

        // String conversion with ISO format validation
        FhirPathValue::String(s) => match parse_iso_time_string(s) {
            Some(time) => Ok(FhirPathValue::Time(time)),
            None => Err(FhirPathError::ConversionError {
                from: format!("String('{s}')"),
                to: "Time".to_string(),
            }),
        },

        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![]))
            } else if c.len() == 1 {
                convert_to_time(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![]))
            }
        }

        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: "Unsupported type".to_string(),
            to: "Time".to_string(),
        }),
    }
}

fn parse_iso_time_string(s: &str) -> Option<PrecisionTime> {
    let s = s.trim();

    // Remove timezone and fractional seconds for basic parsing
    let time_part = if let Some(tz_pos) = s.find('+') {
        &s[..tz_pos]
    } else if let Some(tz_pos) = s.rfind('-') {
        // Only consider as timezone if it's after time (position > 8 for HH:MM:SS)
        if tz_pos > 8 { &s[..tz_pos] } else { s }
    } else {
        s
    };

    let base_time = if let Some(dot_pos) = time_part.find('.') {
        &time_part[..dot_pos]
    } else {
        time_part
    };

    // Should be exactly HH:MM:SS (8 characters)
    if base_time.len() != 8 {
        return None;
    }

    let parts: Vec<&str> = base_time.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    // Parse hour (00-23)
    let hour = parts[0].parse::<u32>().ok()?;
    if hour > 23 {
        return None;
    }

    // Parse minute (00-59)
    let minute = parts[1].parse::<u32>().ok()?;
    if minute > 59 {
        return None;
    }

    // Parse second (00-59)
    let second = parts[2].parse::<u32>().ok()?;
    if second > 59 {
        return None;
    }

    // Create the time
    NaiveTime::from_hms_opt(hour, minute, second)
        .map(|naive_time| PrecisionTime::new(naive_time, TemporalPrecision::Second))
}
