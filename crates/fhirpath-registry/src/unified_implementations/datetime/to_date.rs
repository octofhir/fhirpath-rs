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

//! Unified toDate() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use chrono::NaiveDate;

/// Unified toDate() function implementation
///
/// Converts the input to a Date if possible, returns empty otherwise
pub struct UnifiedToDateFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToDateFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toDate", FunctionCategory::DateTime)
            .display_name("To Date")
            .description("Converts the input to a Date if possible")
            .example("'2023-05-15'.toDate()")
            .example("Patient.birthDate.toDate()")
            .output_type(TypePattern::Exact(TypeInfo::Date))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("toDate()")
            .keywords(vec!["toDate", "convert", "date", "cast", "type"])
            .usage_pattern(
                "Convert value to date",
                "value.toDate()",
                "Date conversion and type casting"
            )
            .build();

        Self { metadata }
    }

    /// Helper function to parse a string as a date
    fn parse_date_string(s: &str) -> Option<NaiveDate> {
        // FHIR date format can be YYYY, YYYY-MM, or YYYY-MM-DD
        if s.len() == 4 {
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
        }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToDateFunction {
    fn name(&self) -> &str {
        "toDate"
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
                        message: "toDate() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                let item = items.first().unwrap();
                match item {
                    FhirPathValue::Date(date) => Ok(FhirPathValue::Date(date.clone())),
                    FhirPathValue::DateTime(datetime) => {
                        // Extract date part from datetime
                        Ok(FhirPathValue::Date(datetime.date_naive()))
                    },
                    FhirPathValue::String(s) => {
                        if let Some(date) = Self::parse_date_string(s.as_ref()) {
                            Ok(FhirPathValue::Date(date))
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
                    FhirPathValue::Date(date) => Ok(FhirPathValue::Date(date.clone())),
                    FhirPathValue::DateTime(datetime) => {
                        // Extract date part from datetime
                        Ok(FhirPathValue::Date(datetime.date_naive()))
                    },
                    FhirPathValue::String(s) => {
                        if let Some(date) = Self::parse_date_string(s.as_ref()) {
                            Ok(FhirPathValue::Date(date))
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
    async fn test_unified_to_date_function() {
        let to_date_func = UnifiedToDateFunction::new();

        // Test valid date string
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15".to_string()));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Date(date) => {
                assert_eq!(date.year(), 2023);
                assert_eq!(date.month(), 5);
                assert_eq!(date.day(), 15);
            },
            _ => panic!("Expected Date result"),
        }

        // Test valid year string (should expand to YYYY-01-01)
        let context = EvaluationContext::new(FhirPathValue::String("2023".to_string()));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Date(date) => {
                assert_eq!(date.year(), 2023);
                assert_eq!(date.month(), 1);
                assert_eq!(date.day(), 1);
            },
            _ => panic!("Expected Date result"),
        }

        // Test valid year-month string (should expand to YYYY-MM-01)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05".to_string()));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Date(date) => {
                assert_eq!(date.year(), 2023);
                assert_eq!(date.month(), 5);
                assert_eq!(date.day(), 1);
            },
            _ => panic!("Expected Date result"),
        }

        // Test invalid date string
        let context = EvaluationContext::new(FhirPathValue::String("invalid-date".to_string()));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test date value (should return as-is)
        let original_date = chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        let context = EvaluationContext::new(FhirPathValue::Date(original_date));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Date(date) => {
                assert_eq!(date, original_date);
            },
            _ => panic!("Expected Date result"),
        }

        // Test datetime value (should extract date part)
        let datetime = chrono::DateTime::parse_from_rfc3339("2023-05-15T10:30:00Z").unwrap();
        let context = EvaluationContext::new(FhirPathValue::DateTime(datetime));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        match result {
            FhirPathValue::Date(date) => {
                assert_eq!(date.year(), 2023);
                assert_eq!(date.month(), 5);
                assert_eq!(date.day(), 15);
            },
            _ => panic!("Expected Date result"),
        }

        // Test integer (should return empty)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15".to_string()));
        let result = to_date_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(to_date_func.name(), "toDate");
        assert_eq!(to_date_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(to_date_func.metadata().basic.display_name, "To Date");
        assert!(to_date_func.metadata().basic.is_pure);
    }
}
