//! Tests for datetime functions following FHIRPath specification

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::{FhirPathValue};
    use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime, TemporalPrecision};
    use chrono::{NaiveDate, NaiveTime, FixedOffset, TimeZone};

    fn create_sample_date() -> FhirPathValue {
        let date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        FhirPathValue::Date(PrecisionDate::new(date, TemporalPrecision::Day))
    }

    fn create_sample_datetime() -> FhirPathValue {
        let datetime = FixedOffset::west_opt(5 * 3600).unwrap()
            .with_ymd_and_hms(2023, 5, 15, 14, 30, 45).unwrap();
        FhirPathValue::DateTime(PrecisionDateTime::new(datetime, TemporalPrecision::Second))
    }

    fn create_sample_time() -> FhirPathValue {
        let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
        FhirPathValue::Time(PrecisionTime::new(time, TemporalPrecision::Second))
    }

    #[test]
    fn test_datetime_functions_registration() {
        let registry = FunctionRegistry::new();
        assert!(registry.register_datetime_functions().is_ok());
        
        // Test all official FHIRPath datetime functions are registered
        let official_functions = vec![
            "now", "today", "timeOfDay", 
            "yearOf", "monthOf", "dayOf", 
            "hourOf", "minuteOf", "secondOf", 
            "millisecondOf", "timezoneOffsetOf"
        ];
        
        for function_name in official_functions {
            assert!(registry.get_sync_function(function_name).is_some(), 
                   "Function '{}' should be registered", function_name);
            
            let (_, metadata) = registry.get_sync_function(function_name).unwrap();
            assert_eq!(metadata.category, FunctionCategory::DateTime);
            assert!(!metadata.is_async);
            assert!(!metadata.description.is_empty());
        }
        
        // Test additional implementation-specific functions
        let additional_functions = vec!["dayOfWeek", "dayOfYear"];
        
        for function_name in additional_functions {
            assert!(registry.get_sync_function(function_name).is_some(), 
                   "Function '{}' should be registered", function_name);
        }
    }

    #[test]
    fn test_current_time_functions_return_correct_types() {
        use crate::mock_provider::MockModelProvider;
        use std::collections::HashMap;
        
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let input = vec![];
        let arguments = vec![];
        
        let context = FunctionContext {
            input: &input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        // Test now() returns DateTime
        let (now_func, _) = registry.get_sync_function("now").unwrap();
        let now_result = now_func(&context).unwrap();
        assert_eq!(now_result.len(), 1);
        assert!(matches!(now_result[0], FhirPathValue::DateTime(_)));
        
        // Test today() returns Date
        let (today_func, _) = registry.get_sync_function("today").unwrap();
        let today_result = today_func(&context).unwrap();
        assert_eq!(today_result.len(), 1);
        assert!(matches!(today_result[0], FhirPathValue::Date(_)));
        
        // Test timeOfDay() returns Time
        let (time_func, _) = registry.get_sync_function("timeOfDay").unwrap();
        let time_result = time_func(&context).unwrap();
        assert_eq!(time_result.len(), 1);
        assert!(matches!(time_result[0], FhirPathValue::Time(_)));
    }

    #[test]
    fn test_component_extraction_functions() {
        use crate::mock_provider::MockModelProvider;
        use std::collections::HashMap;
        
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let arguments = vec![];
        
        // Test yearOf with Date
        let date_input = vec![create_sample_date()];
        let context = FunctionContext {
            input: &date_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        let (year_func, _) = registry.get_sync_function("yearOf").unwrap();
        let result = year_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(year) => assert_eq!(*year, 2023),
            _ => panic!("yearOf() should return an integer"),
        }
        
        // Test monthOf with Date
        let (month_func, _) = registry.get_sync_function("monthOf").unwrap();
        let result = month_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(month) => assert_eq!(*month, 5),
            _ => panic!("monthOf() should return an integer"),
        }
        
        // Test dayOf with Date
        let (day_func, _) = registry.get_sync_function("dayOf").unwrap();
        let result = day_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(day) => assert_eq!(*day, 15),
            _ => panic!("dayOf() should return an integer"),
        }
    }

    #[test]
    fn test_time_component_extraction() {
        use crate::mock_provider::MockModelProvider;
        use std::collections::HashMap;
        
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let arguments = vec![];
        
        // Test with Time value
        let time_input = vec![create_sample_time()];
        let context = FunctionContext {
            input: &time_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        // Test hourOf with Time
        let (hour_func, _) = registry.get_sync_function("hourOf").unwrap();
        let result = hour_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(hour) => assert_eq!(*hour, 14),
            _ => panic!("hourOf() should return an integer"),
        }
        
        // Test minuteOf with Time
        let (minute_func, _) = registry.get_sync_function("minuteOf").unwrap();
        let result = minute_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(minute) => assert_eq!(*minute, 30),
            _ => panic!("minuteOf() should return an integer"),
        }
        
        // Test secondOf with Time
        let (second_func, _) = registry.get_sync_function("secondOf").unwrap();
        let result = second_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(second) => assert_eq!(*second, 45),
            _ => panic!("secondOf() should return an integer"),
        }
    }

    #[test]
    fn test_datetime_timezone_functions() {
        use crate::mock_provider::MockModelProvider;
        use std::collections::HashMap;
        use rust_decimal::prelude::ToPrimitive;
        
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let arguments = vec![];
        
        // Test with DateTime value that has timezone
        let datetime_input = vec![create_sample_datetime()];
        let context = FunctionContext {
            input: &datetime_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        // Test timezoneOffsetOf
        let (tz_func, _) = registry.get_sync_function("timezoneOffsetOf").unwrap();
        let result = tz_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Decimal(offset) => {
                // Our sample datetime is -5 hours (west_opt(5 * 3600))
                assert_eq!(offset.to_f64().unwrap(), -5.0);
            },
            _ => panic!("timezoneOffsetOf() should return a decimal"),
        }
        
        // Test millisecondOf
        let (ms_func, _) = registry.get_sync_function("millisecondOf").unwrap();
        let result = ms_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(ms) => {
                // Our sample datetime has no fractional seconds, so should be 0
                assert_eq!(*ms, 0);
            },
            _ => panic!("millisecondOf() should return an integer"),
        }
    }

    #[test]
    fn test_fhirpath_spec_compliance_empty_results() {
        use crate::mock_provider::MockModelProvider;
        use std::collections::HashMap;
        
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let arguments = vec![];
        
        // Test empty input returns empty result (FHIRPath spec compliance)
        let empty_input = vec![];
        let context = FunctionContext {
            input: &empty_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        let (year_func, _) = registry.get_sync_function("yearOf").unwrap();
        let result = year_func(&context).unwrap();
        assert_eq!(result.len(), 0); // Empty as per spec
        
        // Test multiple items returns empty result (FHIRPath spec compliance)
        let multiple_input = vec![create_sample_date(), create_sample_datetime()];
        let context = FunctionContext {
            input: &multiple_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        let result = year_func(&context).unwrap();
        assert_eq!(result.len(), 0); // Empty if multiple items as per spec
        
        // Test invalid input type returns empty result
        let invalid_input = vec![FhirPathValue::String("not a date".to_string())];
        let context = FunctionContext {
            input: &invalid_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        let result = year_func(&context).unwrap();
        assert_eq!(result.len(), 0); // Empty if not Date or DateTime as per spec
    }

    #[test]
    fn test_additional_datetime_functions() {
        use crate::mock_provider::MockModelProvider;
        use std::collections::HashMap;
        
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let arguments = vec![];
        
        let date_input = vec![create_sample_date()]; // 2023-05-15 (Monday)
        let context = FunctionContext {
            input: &date_input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        };
        
        // Test dayOfWeek (Monday = 1 in FHIRPath)
        let (dow_func, _) = registry.get_sync_function("dayOfWeek").unwrap();
        let result = dow_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(weekday) => assert_eq!(*weekday, 1), // Monday = 1
            _ => panic!("dayOfWeek() should return an integer"),
        }
        
        // Test dayOfYear (May 15th = day 135)
        let (doy_func, _) = registry.get_sync_function("dayOfYear").unwrap();
        let result = doy_func(&context).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            FhirPathValue::Integer(day_of_year) => {
                // May 15th should be day 135 (31+28+31+30+15 = 135) in 2023
                assert_eq!(*day_of_year, 135);
            },
            _ => panic!("dayOfYear() should return an integer"),
        }
    }

    #[test]
    fn test_function_metadata() {
        let registry = FunctionRegistry::new();
        registry.register_datetime_functions().unwrap();
        
        let metadata = registry.get_function_metadata("now").unwrap();
        assert_eq!(metadata.category, FunctionCategory::DateTime);
        assert!(!metadata.is_async);
        assert!(!metadata.description.is_empty());
        assert!(!metadata.examples.is_empty());
        
        // Verify the description matches FHIRPath specification
        assert!(metadata.description.contains("current date and time"));
        assert!(metadata.description.contains("timezone offset"));
    }
}