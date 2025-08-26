//! convertsToDate() sync implementation

use crate::signature::{FunctionSignature, ValueType};
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
    fn test_converts_to_date() {
        let op = ConvertsToDateFunction;

        // Test date input
        use chrono::NaiveDate;
        use octofhir_fhirpath_model::TemporalPrecision;
        let date = PrecisionDate::new(
            NaiveDate::from_ymd_opt(2023, 12, 25).unwrap(),
            TemporalPrecision::Day,
        );
        let context = create_context(FhirPathValue::Date(date));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid ISO date string
        let context = create_context(FhirPathValue::String("2023-12-25".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid date string
        let context = create_context(FhirPathValue::String("invalid-date".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test non-date input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
