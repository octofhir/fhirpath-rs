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

//! Unified timeOfDay() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use async_trait::async_trait;
use chrono::{Local, Timelike};
use octofhir_fhirpath_model::FhirPathValue;

/// Unified timeOfDay() function implementation
/// 
/// Returns the current time (without date) in the local timezone.
/// Companion to now() and today() functions.
/// Syntax: timeOfDay()
pub struct UnifiedTimeOfDayFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTimeOfDayFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("timeOfDay", FunctionCategory::DateTime)
            .display_name("Time of Day")
            .description("Returns the current time (without date) in the local timezone")
            .example("timeOfDay()")
            .example("timeOfDay() > @T14:30:00")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![])
            .output_type(TypePattern::DateTime)
            .supports_collections(false)
            .requires_collection(false)
            .pure(false) // Not pure - returns current time
            .complexity(PerformanceComplexity::Constant)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("timeOfDay()")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["timeOfDay", "time", "current", "now"])
            .usage_pattern(
                "Current time",
                "timeOfDay()",
                "Getting current time for temporal calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTimeOfDayFunction {
    fn name(&self) -> &str {
        "timeOfDay"
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
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - no arguments required
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }
        
        // Get current local time
        let local_time = Local::now().time();
        
        // Create NaiveTime with microsecond precision
        let naive_time = local_time
            .with_nanosecond(0) // Round to seconds for consistency
            .ok_or_else(|| FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Failed to create time value".to_string(),
            })?;
        
        Ok(FhirPathValue::Time(naive_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_time_of_day() {
        let func = UnifiedTimeOfDayFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let args = vec![];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Verify we get a Time value
        match result {
            FhirPathValue::Time(time) => {
                // Verify it's a valid time (hours should be 0-23)
                assert!(time.hour() < 24);
                assert!(time.minute() < 60);
                assert!(time.second() < 60);
            }
            _ => panic!("Expected Time result from timeOfDay function"),
        }
    }
    
    #[tokio::test]
    async fn test_time_of_day_no_args() {
        let func = UnifiedTimeOfDayFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let args = vec![];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should return current time
        assert!(matches!(result, FhirPathValue::Time(_)));
    }
    
    #[tokio::test]
    async fn test_time_of_day_with_args_fails() {
        let func = UnifiedTimeOfDayFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        // timeOfDay() should not accept arguments
        let args = vec![FhirPathValue::String("invalid".into())];
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
        if let Err(FunctionError::InvalidArity { actual, min, max, .. }) = result {
            assert_eq!(actual, 1);
            assert_eq!(min, 0);
            assert_eq!(max, Some(0));
        } else {
            panic!("Expected InvalidArity error");
        }
    }
    
    #[tokio::test]
    async fn test_time_consistency() {
        let func = UnifiedTimeOfDayFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        // Call timeOfDay() twice in quick succession
        let args = vec![];
        let result1 = func.evaluate_sync(&args, &context).unwrap();
        let result2 = func.evaluate_sync(&args, &context).unwrap();
        
        // Both should be Time values (exact equality depends on system clock precision)
        assert!(matches!(result1, FhirPathValue::Time(_)));
        assert!(matches!(result2, FhirPathValue::Time(_)));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedTimeOfDayFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "timeOfDay");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(!metadata.performance.is_pure); // Not pure - returns current time
        assert_eq!(metadata.basic.category, FunctionCategory::DateTime);
    }
}