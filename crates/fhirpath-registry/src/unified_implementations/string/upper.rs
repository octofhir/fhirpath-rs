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

//! Unified upper() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified upper() function implementation
/// 
/// Converts string to uppercase
pub struct UnifiedUpperFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedUpperFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::string_function("upper")
            .display_name("Upper Case")
            .description("Converts the string to uppercase")
            .example("Patient.name.family.upper()")
            .example("'hello world'.upper()")
            .output_type(TypePattern::Exact(TypeInfo::String))
            .lsp_snippet("upper()")
            .keywords(vec!["upper", "uppercase", "string", "case"])
            .usage_pattern(
                "Convert to uppercase",
                "name.upper()",
                "String normalization and comparison"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedUpperFunction {
    fn name(&self) -> &str {
        "upper"
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
        // Validate no arguments
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }
        
        let input_string = match &context.input {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String, got {}", context.input.type_name()),
            }),
        };
        
        let upper_string = input_string.to_uppercase();
        Ok(FhirPathValue::collection(vec![FhirPathValue::String(upper_string.into())]))
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
    async fn test_unified_upper_function() {
        let upper_func = UnifiedUpperFunction::new();
        
        // Test upper conversion
        let context = EvaluationContext::new(FhirPathValue::String("Hello World".into()));
        let result = upper_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("HELLO WORLD".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(upper_func.name(), "upper");
        assert_eq!(upper_func.execution_mode(), ExecutionMode::Sync);
    }
}