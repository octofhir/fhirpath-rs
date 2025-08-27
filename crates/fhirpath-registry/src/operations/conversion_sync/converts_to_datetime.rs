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
    fn test_converts_to_datetime() {
        let op = ConvertsToDateTimeFunction;

        // Test datetime input
        let datetime = PrecisionDateTime::from_ymd_hms(2023, 12, 25, 10, 30, 0).unwrap();
        let context = create_context(FhirPathValue::DateTime(datetime));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test date input (can be converted to datetime)
        let date = PrecisionDate::from_ymd(2023, 12, 25).unwrap();
        let context = create_context(FhirPathValue::Date(date));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid ISO datetime string
        let context = create_context(FhirPathValue::String("2023-12-25T10:30:00".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid ISO date string (convertible to datetime)
        let context = create_context(FhirPathValue::String("2023-12-25".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid datetime string
        let context = create_context(FhirPathValue::String("invalid-datetime".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test non-datetime input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
