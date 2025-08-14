// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unified toDateTime() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use chrono::{NaiveDate, DateTime, FixedOffset, TimeZone};

/// Unified toDateTime() function implementation
///
/// Converts the input to a DateTime if possible, returns empty otherwise
pub struct UnifiedToDateTimeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToDateTimeFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toDateTime", FunctionCategory::DateTime)
            .display_name("To DateTime")
            .description("Converts the input to a DateTime if possible")
            .example("'2023-05-15T10:30:00Z'.toDateTime()")
            .example("Patient.birthDate.toDateTime()")
            .output_type(TypePattern::Exact(TypeInfo::DateTime))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("toDateTime()")
            .keywords(vec!["toDateTime", "convert", "datetime", "cast", "type"])
            .usage_pattern(
                "Convert value to datetime",
                "value.toDateTime()",
                "DateTime conversion and type casting"
            )
            .build();

        Self { metadata }
    }

    /// Helper function to parse a string as a date and convert to datetime
    fn parse_date_string_to_datetime(s: &str) -> Option<DateTime<FixedOffset>> {
        // FHIR date format can be YYYY, YYYY-MM, or YYYY-MM-DD
        let date = if s.len() == 4 {
            // Year only: YYYY -> YYYY-01-01
            if let Ok(year) = s.parse::<i32>() {
                NaiveDate::from_ymd_opt(year, 1, 1)
            } else {
                None
            }
        } else if s.len() == 7 {
            // Year-Month: YYYY-MM -> YYYY-MM-01
            if let Ok(date) = NaiveDate::parse_from_str(&format!("{}-01", s), "%Y-%m-%d") {
                Some(date)
            } else {
                None
            }
        } else if s.len() == 10 {
            // Full date: YYYY-MM-DD
            NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
        } else {
            None
        };

        // Convert date to datetime with empty time components
        date.and_then(|d| {
            let datetime = d.and_hms_opt(0, 0, 0)?;
            FixedOffset::east_opt(0).and_then(|tz| tz.from_local_datetime(&datetime).single())
        })
    }

    /// Helper function to parse a string as a datetime
    fn parse_datetime_string(s: &str) -> Option<DateTime<FixedOffset>> {
        // First try if it's a simple date (which can be converted to datetime)
        if let Some(dt) = Self::parse_date_string_to_datetime(s) {
            return Some(dt);
        }

        // Try parsing as various datetime formats
        let formats = [
            // Full RFC3339/ISO8601 formats
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S%.f%z",
            "%Y-%m-%dT%H:%M:%S%z",
        ];

        for format in &formats {
            if let Ok(dt) = DateTime::<FixedOffset>::parse_from_str(s, format) {
                return Some(dt);
            }
        }

        // Try parsing without timezone as naive datetime, then add UTC
        let naive_formats = [
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%dT%H",
        ];

        for format in &naive_formats {
            if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(s, format) {
                if let Some(tz) = FixedOffset::east_opt(0) {
                    if let Some(dt) = tz.from_local_datetime(&ndt).single() {
                        return Some(dt);
                    }
                }
            }
        }

        None
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToDateTimeFunction {
    fn name(&self) -> &str {
        "toDateTime"
    }

    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }

    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }

    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate no arguments - this is a member function
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }

        // Get the input collection from context
        let input = &context.input;

        match input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "toDateTime() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                let item = items.first().unwrap();
                match item {
                    FhirPathValue::DateTime(datetime) => Ok(FhirPathValue::DateTime(datetime.clone())),
                    FhirPathValue::Date(date) => {
                        // Convert date to datetime with empty time components
                        let datetime = date.and_hms_opt(0, 0, 0)
                            .and_then(|dt| FixedOffset::east_opt(0)?.from_local_datetime(&dt).single());

                        if let Some(dt) = datetime {
                            Ok(FhirPathValue::DateTime(dt))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    FhirPathValue::String(s) => {
                        if let Some(datetime) = Self::parse_datetime_string(s.as_ref()) {
                            Ok(FhirPathValue::DateTime(datetime))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            _ => {
                // Single item case
                match input {
                    FhirPathValue::DateTime(datetime) => Ok(FhirPathValue::DateTime(datetime.clone())),
                    FhirPathValue::Date(date) => {
                        // Convert date to datetime with empty time components
                        let datetime = date.and_hms_opt(0, 0, 0)
                            .and_then(|dt| FixedOffset::east_opt(0)?.from_local_datetime(&dt).single());

                        if let Some(dt) = datetime {
                            Ok(FhirPathValue::DateTime(dt))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    FhirPathValue::String(s) => {
                        if let Some(datetime) = Self::parse_datetime_string(s.as_ref()) {
                            Ok(FhirPathValue::DateTime(datetime))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    },
                    _ => Ok(FhirPathValue::Empty),
                }
            }
        }
    }

    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_to_datetime_function() {
        let to_datetime_func = UnifiedToDateTimeFunction::new();

        // Test valid datetime string
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15T10:30:00Z".to_string()));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::DateTime(datetime) => {
                assert_eq!(datetime.year(), 2023);
                assert_eq!(datetime.month(), 5);
                assert_eq!(datetime.day(), 15);
                assert_eq!(datetime.hour(), 10);
                assert_eq!(datetime.minute(), 30);
                assert_eq!(datetime.second(), 0);
            },
            _ => panic!("Expected DateTime result"),
        }

        // Test valid date string (should convert to datetime with empty time)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15".to_string()));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::DateTime(datetime) => {
                assert_eq!(datetime.year(), 2023);
                assert_eq!(datetime.month(), 5);
                assert_eq!(datetime.day(), 15);
                assert_eq!(datetime.hour(), 0);
                assert_eq!(datetime.minute(), 0);
                assert_eq!(datetime.second(), 0);
            },
            _ => panic!("Expected DateTime result"),
        }

        // Test valid year string (should expand to YYYY-01-01T00:00:00)
        let context = EvaluationContext::new(FhirPathValue::String("2023".to_string()));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::DateTime(datetime) => {
                assert_eq!(datetime.year(), 2023);
                assert_eq!(datetime.month(), 1);
                assert_eq!(datetime.day(), 1);
                assert_eq!(datetime.hour(), 0);
                assert_eq!(datetime.minute(), 0);
                assert_eq!(datetime.second(), 0);
            },
            _ => panic!("Expected DateTime result"),
        }

        // Test partial datetime strings
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15T14:30".to_string()));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::DateTime(datetime) => {
                assert_eq!(datetime.year(), 2023);
                assert_eq!(datetime.month(), 5);
                assert_eq!(datetime.day(), 15);
                assert_eq!(datetime.hour(), 14);
                assert_eq!(datetime.minute(), 30);
            },
            _ => panic!("Expected DateTime result"),
        }

        // Test invalid datetime string
        let context = EvaluationContext::new(FhirPathValue::String("invalid-datetime".to_string()));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test datetime value (should return as-is)
        let original_datetime = chrono::DateTime::parse_from_rfc3339("2023-05-15T10:30:00Z").unwrap();
        let context = EvaluationContext::new(FhirPathValue::DateTime(original_datetime));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::DateTime(datetime) => {
                assert_eq!(datetime, original_datetime);
            },
            _ => panic!("Expected DateTime result"),
        }

        // Test date value (should convert to datetime with empty time)
        let date = chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        let context = EvaluationContext::new(FhirPathValue::Date(date));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::DateTime(datetime) => {
                assert_eq!(datetime.year(), 2023);
                assert_eq!(datetime.month(), 5);
                assert_eq!(datetime.day(), 15);
                assert_eq!(datetime.hour(), 0);
                assert_eq!(datetime.minute(), 0);
                assert_eq!(datetime.second(), 0);
            },
            _ => panic!("Expected DateTime result"),
        }

        // Test integer (should return empty)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15T10:30:00Z".to_string()));
        let result = to_datetime_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(to_datetime_func.name(), "toDateTime");
        assert_eq!(to_datetime_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(to_datetime_func.metadata().basic.display_name, "To DateTime");
        assert!(to_datetime_func.metadata().basic.is_pure);
    }
}
