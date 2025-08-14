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

//! Unified convertsToDateTime() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use chrono::{NaiveDate, DateTime, FixedOffset};

/// Unified convertsToDateTime() function implementation
///
/// Returns true if the input can be converted to a DateTime, false otherwise
pub struct UnifiedConvertsToDateTimeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedConvertsToDateTimeFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("convertsToDateTime", FunctionCategory::DateTime)
            .display_name("Converts To DateTime")
            .description("Returns true if the input can be converted to a DateTime")
            .example("'2023-05-15T10:30:00Z'.convertsToDateTime()")
            .example("Patient.birthDate.convertsToDateTime()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("convertsToDateTime()")
            .keywords(vec!["convertsToDateTime", "convert", "datetime", "validation", "type", "check"])
            .usage_pattern(
                "Check if value can be converted to datetime",
                "value.convertsToDateTime()",
                "Type validation and conditional conversion"
            )
            .build();

        Self { metadata }
    }

    /// Helper function to check if a string can be parsed as a date (which can be converted to datetime)
    fn can_parse_date(s: &str) -> bool {
        // FHIR date format can be YYYY, YYYY-MM, or YYYY-MM-DD
        if s.len() == 4 {
            // Year only: YYYY
            s.parse::<i32>().is_ok() && s.len() == 4
        } else if s.len() == 7 {
            // Year-Month: YYYY-MM
            NaiveDate::parse_from_str(&format!("{}-01", s), "%Y-%m-%d").is_ok()
        } else if s.len() == 10 {
            // Full date: YYYY-MM-DD
            NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
        } else {
            false
        }
    }

    /// Helper function to check if a string can be parsed as a datetime
    fn can_parse_datetime(s: &str) -> bool {
        // Try various datetime formats

        // First check if it's a simple date (which can be converted to datetime)
        if Self::can_parse_date(s) {
            return true;
        }

        // Try parsing as various datetime formats
        let formats = [
            // Full RFC3339/ISO8601 formats
            "%Y-%m-%dT%H:%M:%S%.fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%dT%H:%M:%S%.f%z",
            "%Y-%m-%dT%H:%M:%S%z",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            // Partial datetime formats
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%dT%H",
        ];

        for format in &formats {
            if DateTime::<FixedOffset>::parse_from_str(s, format).is_ok() {
                return true;
            }
        }

        // Try parsing without timezone as naive datetime
        let naive_formats = [
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%dT%H:%M",
            "%Y-%m-%dT%H",
        ];

        for format in &naive_formats {
            if chrono::NaiveDateTime::parse_from_str(s, format).is_ok() {
                return true;
            }
        }

        false
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedConvertsToDateTimeFunction {
    fn name(&self) -> &str {
        "convertsToDateTime"
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
                        message: "convertsToDateTime() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                let item = items.first().unwrap();
                let can_convert = match item {
                    FhirPathValue::DateTime(_) => true,
                    FhirPathValue::Date(_) => true,
                    FhirPathValue::String(s) => Self::can_parse_datetime(s.as_ref()),
                    _ => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
            _ => {
                // Single item case
                let can_convert = match input {
                    FhirPathValue::DateTime(_) => true,
                    FhirPathValue::Date(_) => true,
                    FhirPathValue::String(s) => Self::can_parse_datetime(s.as_ref()),
                    _ => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
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
    async fn test_unified_converts_to_datetime_function() {
        let converts_to_datetime_func = UnifiedConvertsToDateTimeFunction::new();

        // Test valid datetime string
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15T10:30:00Z".to_string()));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid date string (can be converted to datetime)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15".to_string()));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid year string
        let context = EvaluationContext::new(FhirPathValue::String("2023".to_string()));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid year-month string
        let context = EvaluationContext::new(FhirPathValue::String("2023-05".to_string()));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test partial datetime strings
        let test_cases = [
            "2015-02-04T14",
            "2015-02-04T14:34",
            "2015-02-04T14:34:28",
            "2015-02-04T14:34:28.123",
            "2015-02-04T14:34:28Z",
            "2015-02-04T14:34:28+10:00",
        ];

        for test_case in &test_cases {
            let context = EvaluationContext::new(FhirPathValue::String(test_case.to_string()));
            let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
            assert_eq!(result, FhirPathValue::Boolean(true), "Failed for: {}", test_case);
        }

        // Test invalid datetime string
        let context = EvaluationContext::new(FhirPathValue::String("invalid-datetime".to_string()));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test datetime value
        let datetime = chrono::DateTime::parse_from_rfc3339("2023-05-15T10:30:00Z").unwrap();
        let context = EvaluationContext::new(FhirPathValue::DateTime(datetime));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test date value (can be converted to datetime)
        let date = chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        let context = EvaluationContext::new(FhirPathValue::Date(date));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test integer (should be false)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = converts_to_datetime_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15T10:30:00Z".to_string()));
        let result = converts_to_datetime_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(converts_to_datetime_func.name(), "convertsToDateTime");
        assert_eq!(converts_to_datetime_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(converts_to_datetime_func.metadata().basic.display_name, "Converts To DateTime");
        assert!(converts_to_datetime_func.metadata().basic.is_pure);
    }
}
