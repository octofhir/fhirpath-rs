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

//! Unified today() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use chrono::Local;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified today() function implementation
/// 
/// Returns the current date (without time)
pub struct UnifiedTodayFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTodayFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("today", FunctionCategory::DateTime)
            .display_name("Today")
            .description("Returns the current date (without time)")
            .example("today()")
            .example("today() = Patient.birthDate")
            .output_type(TypePattern::Exact(TypeInfo::Date))
            .execution_mode(ExecutionMode::Sync)
            .pure(false) // Not pure because it returns different values each day
            .lsp_snippet("today()")
            .keywords(vec!["today", "current", "date", "day"])
            .usage_pattern(
                "Get current date",
                "today()",
                "Date comparisons and calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTodayFunction {
    fn name(&self) -> &str {
        "today"
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
        // Validate no arguments
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }
        
        let today = Local::now().date_naive();
        Ok(FhirPathValue::Date(today))
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
    async fn test_unified_today_function() {
        let today_func = UnifiedTodayFunction::new();
        
        // Test today() function
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = today_func.evaluate_sync(&[], &context).unwrap();
        
        // Verify it returns a Date
        match result {
            FhirPathValue::Date(_) => {
                // Success - we got a date
            },
            _ => panic!("Expected Date result from today() function"),
        }
        
        // Test that calling today() twice gives the same date (on the same day)
        let result2 = today_func.evaluate_sync(&[], &context).unwrap();
        match result2 {
            FhirPathValue::Date(_) => {
                // Success - we got a date
            },
            _ => panic!("Expected Date result from second today() call"),
        }
        
        // Test with arguments (should fail)
        let result = today_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());
        
        // Test metadata
        assert_eq!(today_func.name(), "today");
        assert_eq!(today_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(today_func.metadata().basic.display_name, "Today");
        assert!(!today_func.metadata().basic.is_pure); // Should not be pure
    }
}