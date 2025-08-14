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

//! Unified convertsToDate() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use chrono::NaiveDate;

/// Unified convertsToDate() function implementation
///
/// Returns true if the input can be converted to a Date, false otherwise
pub struct UnifiedConvertsToDateFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedConvertsToDateFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("convertsToDate", FunctionCategory::DateTime)
            .display_name("Converts To Date")
            .description("Returns true if the input can be converted to a Date")
            .example("'2023-05-15'.convertsToDate()")
            .example("Patient.birthDate.convertsToDate()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("convertsToDate()")
            .keywords(vec!["convertsToDate", "convert", "date", "validation", "type", "check"])
            .usage_pattern(
                "Check if value can be converted to date",
                "value.convertsToDate()",
                "Type validation and conditional conversion"
            )
            .build();

        Self { metadata }
    }

    /// Helper function to check if a string can be parsed as a date
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
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedConvertsToDateFunction {
    fn name(&self) -> &str {
        "convertsToDate"
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

        // Handle empty input
        if matches!(&context.input, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }

        // Check if input can be converted to date
        let can_convert = match &context.input {
            FhirPathValue::Date(_) => true,
            FhirPathValue::DateTime(_) => true,
            FhirPathValue::String(s) => Self::can_parse_date(&s),
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "convertsToDate() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                if let Some(item) = items.get(0) {
                    match item {
                        FhirPathValue::Date(_) => true,
                        FhirPathValue::DateTime(_) => true,
                        FhirPathValue::String(s) => Self::can_parse_date(&s),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        Ok(FhirPathValue::Boolean(can_convert))
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
    async fn test_unified_converts_to_date_function() {
        let converts_to_date_func = UnifiedConvertsToDateFunction::new();

        // Test valid date string
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15".to_string().into()));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid year string
        let context = EvaluationContext::new(FhirPathValue::String("2023".to_string().into()));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid year-month string
        let context = EvaluationContext::new(FhirPathValue::String("2023-05".to_string().into()));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid date string
        let context = EvaluationContext::new(FhirPathValue::String("invalid-date".to_string().into()));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test date value
        let date = chrono::NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
        let context = EvaluationContext::new(FhirPathValue::Date(date));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test datetime value
        let datetime = chrono::DateTime::parse_from_rfc3339("2023-05-15T10:30:00Z").unwrap();
        let context = EvaluationContext::new(FhirPathValue::DateTime(datetime));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test integer (should be false)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = converts_to_date_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("2023-05-15".to_string()));
        let result = converts_to_date_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(converts_to_date_func.name(), "convertsToDate");
        assert_eq!(converts_to_date_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(converts_to_date_func.metadata().basic.display_name, "Converts To Date");
        assert!(converts_to_date_func.metadata().basic.is_pure);
    }
}
