//! convertsToTime() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToTime(): Returns true if the input can be converted to Time
pub struct ConvertsToTimeFunction;

impl SyncOperation for ConvertsToTimeFunction {
    fn name(&self) -> &'static str {
        "convertsToTime"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToTime",
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
        let can_convert = can_convert_to_time(&context.input)?;
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

fn can_convert_to_time(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a time
        FhirPathValue::Time(_) => Ok(true),

        // DateTime can be converted to Time (extract time part)
        FhirPathValue::DateTime(_) => Ok(true),

        // String values that can be parsed as ISO time format
        FhirPathValue::String(s) => Ok(parse_iso_time_string(s).is_some()),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_time(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to time
        _ => Ok(false),
    }
}

fn parse_iso_time_string(s: &str) -> Option<()> {
    // ISO time formats:
    // HH:MM:SS
    // HH:MM:SS.sss
    // HH:MM:SS+ZZ:ZZ
    // HH:MM:SS.sss+ZZ:ZZ

    let s = s.trim();

    // Must be at least HH:MM:SS (8 characters)
    if s.len() < 8 {
        return None;
    }

    // Basic time validation: HH:MM:SS format
    let time_part = if let Some(tz_pos) = s.find('+') {
        &s[..tz_pos]
    } else if let Some(tz_pos) = s.find('-') {
        // Only consider as timezone if it's after at least HH:MM:SS
        if tz_pos >= 8 { &s[..tz_pos] } else { s }
    } else {
        s
    };

    // Remove fractional seconds for basic validation
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

    // Validate hour (00-23)
    if parts[0].len() != 2 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if let Ok(hour) = parts[0].parse::<u32>() {
        if hour > 23 {
            return None;
        }
    } else {
        return None;
    }

    // Validate minute (00-59)
    if parts[1].len() != 2 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if let Ok(minute) = parts[1].parse::<u32>() {
        if minute > 59 {
            return None;
        }
    } else {
        return None;
    }

    // Validate second (00-59)
    if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if let Ok(second) = parts[2].parse::<u32>() {
        if second > 59 {
            return None;
        }
    } else {
        return None;
    }

    Some(())
}

#[cfg(not(test))]
mod tests {
    use super::*;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input.clone(), std::sync::Arc::new(input), model_provider)
    }

    #[test]
    fn test_converts_to_time() {
        let op = ConvertsToTimeFunction;

        // Test time input
        let time = FhirTime::from_hms(10, 30, 45).unwrap();
        let context = create_context(FhirPathValue::Time(time));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test datetime input (can extract time)
        let datetime = FhirDateTime::from_ymd_hms(2023, 12, 25, 10, 30, 45).unwrap();
        let context = create_context(FhirPathValue::DateTime(datetime));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid ISO time string
        let context = create_context(FhirPathValue::String("10:30:45".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid ISO time string with fractional seconds
        let context = create_context(FhirPathValue::String("10:30:45.123".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid ISO time string with timezone
        let context = create_context(FhirPathValue::String("10:30:45+01:00".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid time string
        let context = create_context(FhirPathValue::String("25:30:45".into())); // Invalid hour
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        let context = create_context(FhirPathValue::String("invalid-time".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test non-time input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_parse_iso_time_string() {
        // Valid time formats
        assert_eq!(parse_iso_time_string("10:30:45"), Some(()));
        assert_eq!(parse_iso_time_string("00:00:00"), Some(()));
        assert_eq!(parse_iso_time_string("23:59:59"), Some(()));
        assert_eq!(parse_iso_time_string("10:30:45.123"), Some(()));
        assert_eq!(parse_iso_time_string("10:30:45+01:00"), Some(()));

        // Invalid time formats
        assert_eq!(parse_iso_time_string("25:30:45"), None); // Invalid hour
        assert_eq!(parse_iso_time_string("10:60:45"), None); // Invalid minute
        assert_eq!(parse_iso_time_string("10:30:60"), None); // Invalid second
        assert_eq!(parse_iso_time_string("10:30"), None); // Missing seconds
        assert_eq!(parse_iso_time_string("invalid"), None); // Invalid format
    }
}
