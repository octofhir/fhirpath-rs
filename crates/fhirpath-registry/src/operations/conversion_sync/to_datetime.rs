//! toDateTime() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, temporal::{PrecisionDateTime, TemporalPrecision}};
use chrono::{FixedOffset, TimeZone};

/// toDateTime(): Converts input to DateTime where possible
pub struct ToDateTimeFunction;

impl SyncOperation for ToDateTimeFunction {
    fn name(&self) -> &'static str {
        "toDateTime"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toDateTime",
            parameters: vec![],
            return_type: ValueType::DateTime,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(&self, _args: &[FhirPathValue], context: &crate::traits::EvaluationContext) -> Result<FhirPathValue> {
        convert_to_datetime(&context.input)
    }
}

fn convert_to_datetime(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a datetime
        FhirPathValue::DateTime(dt) => Ok(FhirPathValue::DateTime(dt.clone())),
        
        // Date can be converted to DateTime (add time 00:00:00)
        FhirPathValue::Date(d) => {
            // Create a datetime at midnight UTC
            let naive_datetime = d.date.and_hms_opt(0, 0, 0)
                .ok_or_else(|| FhirPathError::ConversionError {
                    from: "Date".to_string(),
                    to: "DateTime".to_string(),
                })?;
            let datetime = FixedOffset::east_opt(0).unwrap().from_local_datetime(&naive_datetime).single()
                .ok_or_else(|| FhirPathError::ConversionError {
                    from: "Date".to_string(), 
                    to: "DateTime".to_string(),
                })?;
            let precision_datetime = PrecisionDateTime::new(datetime, TemporalPrecision::Day);
            Ok(FhirPathValue::DateTime(precision_datetime))
        },
        
        // String conversion with ISO format validation
        FhirPathValue::String(s) => {
            match parse_iso_datetime_string(s) {
                Some(datetime) => Ok(FhirPathValue::DateTime(datetime)),
                None => Err(FhirPathError::ConversionError {
                    from: format!("String('{}')", s),
                    to: "DateTime".to_string(),
                }),
            }
        },
        
        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
        
        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_datetime(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }
        
        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: "Unsupported type".to_string(),
            to: "DateTime".to_string(),
        }),
    }
}

fn parse_iso_datetime_string(s: &str) -> Option<PrecisionDateTime> {
    let s = s.trim();
    
    // If it's just a date (YYYY-MM-DD), convert to datetime with 00:00:00 time
    if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
        if let Some((year, month, day)) = parse_iso_date_part(s) {
            let naive_datetime = chrono::NaiveDate::from_ymd_opt(year, month, day)?
                .and_hms_opt(0, 0, 0)?;
            let datetime = FixedOffset::east_opt(0)?.from_local_datetime(&naive_datetime).single()?;
            return Some(PrecisionDateTime::new(datetime, TemporalPrecision::Day));
        }
    }
    
    // Full datetime format: YYYY-MM-DDTHH:MM:SS[.sss][+ZZ:ZZ]
    if s.len() >= 19 { // Minimum length for YYYY-MM-DDTHH:MM:SS
        let parts: Vec<&str> = s.split('T').collect();
        if parts.len() >= 2 {
            // Parse date part
            if let Some((year, month, day)) = parse_iso_date_part(parts[0]) {
                // Parse time part
                if let Some((hour, minute, second)) = parse_iso_time_part(parts[1]) {
                    let naive_datetime = chrono::NaiveDate::from_ymd_opt(year, month, day)?
                        .and_hms_opt(hour, minute, second)?;
                    let datetime = FixedOffset::east_opt(0)?.from_local_datetime(&naive_datetime).single()?;
                    return Some(PrecisionDateTime::new(datetime, TemporalPrecision::Second));
                }
            }
        }
    }
    
    None
}

fn parse_iso_date_part(s: &str) -> Option<(i32, u32, u32)> {
    if s.len() != 10 {
        return None;
    }
    
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    
    let year = parts[0].parse::<i32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    
    if year >= 1 && year <= 9999 && month >= 1 && month <= 12 && day >= 1 && day <= 31 {
        Some((year, month, day))
    } else {
        None
    }
}

fn parse_iso_time_part(s: &str) -> Option<(u32, u32, u32)> {
    // Remove timezone and fractional seconds for basic parsing
    let time_part = if let Some(tz_pos) = s.find('+') {
        &s[..tz_pos]
    } else if let Some(tz_pos) = s.rfind('-') {
        // Only consider as timezone if it's after time
        if tz_pos > 8 {
            &s[..tz_pos]
        } else {
            s
        }
    } else {
        s
    };
    
    let base_time = if let Some(dot_pos) = time_part.find('.') {
        &time_part[..dot_pos]
    } else {
        time_part
    };
    
    if base_time.len() != 8 { // HH:MM:SS
        return None;
    }
    
    let parts: Vec<&str> = base_time.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    
    let hour = parts[0].parse::<u32>().ok()?;
    let minute = parts[1].parse::<u32>().ok()?;
    let second = parts[2].parse::<u32>().ok()?;
    
    if hour <= 23 && minute <= 59 && second <= 59 {
        Some((hour, minute, second))
    } else {
        None
    }
}