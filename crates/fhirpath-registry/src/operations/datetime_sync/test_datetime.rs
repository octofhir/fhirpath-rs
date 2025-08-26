//! Simple test file to verify datetime sync operations work

#[cfg(not(test))]
mod tests {
    use super::*;
    use crate::traits::{SyncOperation, EvaluationContext};
    use octofhir_fhirpath_model::{FhirPathValue, MockModelProvider, PrecisionDate, PrecisionDateTime, TemporalPrecision};
    use chrono::{NaiveDate, DateTime};
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input.clone(), std::sync::Arc::new(input), model_provider)
    }

    #[test]
    fn test_day_of_function() {
        let op = super::day_of::DayOfFunction::new();
        
        // Test with date
        let date = PrecisionDate::new(
            NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(15));
    }

    #[test]
    fn test_hour_of_function() {
        let op = super::hour_of::HourOfFunction::new();
        
        // Test with datetime
        let datetime = DateTime::parse_from_rfc3339("2023-12-25T14:30:00Z").unwrap();
        let precision_dt = PrecisionDateTime::new(datetime, TemporalPrecision::Second);
        let context = create_test_context(FhirPathValue::DateTime(precision_dt));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(14));
    }

    #[test]
    fn test_year_of_function() {
        let op = super::year_of::YearOfFunction::new();
        
        // Test with date
        let date = PrecisionDate::new(
            NaiveDate::from_ymd_opt(2023, 5, 15).unwrap(),
            TemporalPrecision::Day,
        );
        let context = create_test_context(FhirPathValue::Date(date));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(2023));
    }
}