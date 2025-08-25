//! toDate() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, temporal::{PrecisionDate, TemporalPrecision}};
use chrono::NaiveDate;

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
        };
        &SIGNATURE
    }

    fn execute(&self, _args: &[FhirPathValue], context: &crate::traits::EvaluationContext) -> Result<FhirPathValue> {
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
        },
        
        // String conversion with ISO format validation
        FhirPathValue::String(s) => {
            match parse_iso_date_string(s) {
                Some(date) => Ok(FhirPathValue::Date(date)),
                None => Err(FhirPathError::ConversionError {
                    from: format!("String('{}')", s),
                    to: "Date".to_string(),
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

fn parse_iso_date_string(s: &str) -> Option<PrecisionDate> {
    // ISO date format: YYYY-MM-DD
    let s = s.trim();
    if s.len() != 10 {
        return None;
    }
    
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    
    // Parse year (4 digits)
    let year = parts[0].parse::<i32>().ok()?;
    if year < 1 || year > 9999 {
        return None;
    }
    
    // Parse month (2 digits, 01-12)
    let month = parts[1].parse::<u32>().ok()?;
    if month < 1 || month > 12 {
        return None;
    }
    
    // Parse day (2 digits, 01-31)
    let day = parts[2].parse::<u32>().ok()?;
    if day < 1 || day > 31 {
        return None;
    }
    
    // Create the date (this will validate the actual date)
    NaiveDate::from_ymd_opt(year, month, day)
        .map(|naive_date| PrecisionDate::new(naive_date, TemporalPrecision::Day))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, model_provider)
    }

    #[test]
    fn test_to_date() {
        let op = ToDateFunction;

        // Test valid date string
        let context = create_context(FhirPathValue::String("2023-12-25".into()));
        let result = op.execute(&[], &context).unwrap();
        if let FhirPathValue::Date(d) = result {
            assert_eq!(d.date.year(), 2023);
            assert_eq!(d.date.month(), 12);
            assert_eq!(d.date.day(), 25);
        } else {
            panic!("Expected Date value");
        }

        // Test invalid date string
        let context = create_context(FhirPathValue::String("invalid-date".into()));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test non-date input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));
    }

    #[test]
    fn test_parse_iso_date_string() {
        // Valid dates
        assert!(parse_iso_date_string("2023-12-25").is_some());
        assert!(parse_iso_date_string("2000-01-01").is_some());
        assert!(parse_iso_date_string("9999-12-31").is_some());
        
        // Invalid formats
        assert!(parse_iso_date_string("2023-12").is_none());     // Too short
        assert!(parse_iso_date_string("2023-12-255").is_none()); // Too long
        assert!(parse_iso_date_string("invalid").is_none());     // Invalid format
        
        // Invalid dates
        assert!(parse_iso_date_string("2023-13-01").is_none()); // Invalid month
        assert!(parse_iso_date_string("2023-12-32").is_none()); // Invalid day
        assert!(parse_iso_date_string("0000-01-01").is_none()); // Invalid year
    }
}