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

//! Unified now() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use chrono::Utc;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified now() function implementation
/// 
/// Returns the current date and time, including timezone information
pub struct UnifiedNowFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedNowFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("now", FunctionCategory::DateTime)
            .display_name("Now")
            .description("Returns the current date and time, including timezone information")
            .example("now()")
            .example("now() > Patient.birthDate")
            .output_type(TypePattern::Exact(TypeInfo::DateTime))
            .execution_mode(ExecutionMode::Sync)
            .pure(false) // Not pure because it returns different values each time
            .lsp_snippet("now()")
            .keywords(vec!["now", "current", "datetime", "time", "timestamp"])
            .usage_pattern(
                "Get current date and time",
                "now()",
                "Temporal comparisons and calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedNowFunction {
    fn name(&self) -> &str {
        "now"
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
        
        let now = Utc::now();
        Ok(FhirPathValue::DateTime(
            now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        ))
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
    async fn test_unified_now_function() {
        let now_func = UnifiedNowFunction::new();
        
        // Test now() function
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = now_func.evaluate_sync(&[], &context).unwrap();
        
        // Verify it returns a DateTime
        match result {
            FhirPathValue::DateTime(_) => {
                // Success - we got a datetime
            },
            _ => panic!("Expected DateTime result from now() function"),
        }
        
        // Test that calling now() twice gives different results (or at least doesn't fail)
        let result2 = now_func.evaluate_sync(&[], &context).unwrap();
        match result2 {
            FhirPathValue::DateTime(_) => {
                // Success - we got a datetime
            },
            _ => panic!("Expected DateTime result from second now() call"),
        }
        
        // Test with arguments (should fail)
        let result = now_func.evaluate_sync(&[FhirPathValue::Integer(1)], &context);
        assert!(result.is_err());
        
        // Test metadata
        assert_eq!(now_func.name(), "now");
        assert_eq!(now_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(now_func.metadata().basic.display_name, "Now");
        assert!(!now_func.metadata().basic.is_pure); // Should not be pure
    }
}