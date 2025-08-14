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

//! Unified convertsToTime() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use chrono::NaiveTime;

/// Unified convertsToTime() function implementation
///
/// Returns true if the input can be converted to a Time, false otherwise
pub struct UnifiedConvertsToTimeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedConvertsToTimeFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("convertsToTime", FunctionCategory::DateTime)
            .display_name("Converts To Time")
            .description("Returns true if the input can be converted to a Time")
            .example("'10:30:00'.convertsToTime()")
            .example("'invalid'.convertsToTime()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true) // Pure function - same input always produces same output
            .lsp_snippet("convertsToTime()")
            .keywords(vec!["convertsToTime", "convert", "time", "validation", "type", "check"])
            .usage_pattern(
                "Check if value can be converted to time",
                "value.convertsToTime()",
                "Type validation and conditional conversion"
            )
            .build();

        Self { metadata }
    }

    /// Helper function to check if a string can be parsed as a time
    fn can_parse_time(s: &str) -> bool {
        // Try various time formats per FHIRPath spec: hh:mm:ss.fff(+/-)hh:mm
        let formats = [
            "%H:%M:%S%.f", // hh:mm:ss.fff
            "%H:%M:%S",    // hh:mm:ss
            "%H:%M",       // hh:mm
            "%H",          // hh
        ];

        for format in &formats {
            if NaiveTime::parse_from_str(s, format).is_ok() {
                return true;
            }
        }

        false
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedConvertsToTimeFunction {
    fn name(&self) -> &str {
        "convertsToTime"
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
                        message: "convertsToTime() can only be applied to single items".to_string(),
                    });
                }

                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }

                let item = items.first().unwrap();
                let can_convert = match item {
                    FhirPathValue::Time(_) => true,
                    FhirPathValue::String(s) => Self::can_parse_time(s.as_ref()),
                    _ => false,
                };

                Ok(FhirPathValue::Boolean(can_convert))
            }
            _ => {
                // Single item case
                let can_convert = match input {
                    FhirPathValue::Time(_) => true,
                    FhirPathValue::String(s) => Self::can_parse_time(s.as_ref()),
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
    async fn test_unified_converts_to_time_function() {
        let converts_to_time_func = UnifiedConvertsToTimeFunction::new();

        // Test valid time string
        let context = EvaluationContext::new(FhirPathValue::String("10:30:00".to_string()));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid partial time string
        let context = EvaluationContext::new(FhirPathValue::String("14:30".to_string()));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid hour-only time string
        let context = EvaluationContext::new(FhirPathValue::String("14".to_string()));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid time with milliseconds
        let context = EvaluationContext::new(FhirPathValue::String("14:30:00.123".to_string()));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid time string
        let context = EvaluationContext::new(FhirPathValue::String("invalid-time".to_string()));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test time value
        let time = chrono::NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let context = EvaluationContext::new(FhirPathValue::Time(time));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test integer (should be false)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = converts_to_time_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with arguments (should fail)
        let context = EvaluationContext::new(FhirPathValue::String("10:30:00".to_string()));
        let result = converts_to_time_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());

        // Test metadata
        assert_eq!(converts_to_time_func.name(), "convertsToTime");
        assert_eq!(converts_to_time_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(converts_to_time_func.metadata().basic.display_name, "Converts To Time");
        assert!(converts_to_time_func.metadata().basic.is_pure);
    }
}