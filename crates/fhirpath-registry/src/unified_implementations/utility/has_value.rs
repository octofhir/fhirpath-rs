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

//! Unified hasValue() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use octofhir_fhirpath_model::types::TypeInfo;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified hasValue() function implementation
/// 
/// Returns true if the input collection contains any non-empty values.
/// Syntax: hasValue()
pub struct UnifiedHasValueFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedHasValueFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("hasValue", FunctionCategory::Utilities)
            .display_name("Has Value")
            .description("Returns true if the input collection contains any non-empty values")
            .example("Patient.name.hasValue()")
            .example("telecom.where(system = 'phone').hasValue()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("hasValue()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["exists", "value", "non-empty", "present"])
            .usage_pattern(
                "Check for non-empty values",
                "Patient.name.hasValue()",
                "Validating presence of meaningful values"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedHasValueFunction {
    fn name(&self) -> &str {
        "hasValue"
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
        
        let has_value = match &context.input {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => {
                items.iter().any(|item| !matches!(item, FhirPathValue::Empty))
            }
            _ => true, // Single non-empty value
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(has_value)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_has_value_function() {
        let func = UnifiedHasValueFunction::new();
        
        // Test with non-empty value
        let input = FhirPathValue::String("test".into());
        let context = create_test_context(input);
        let result = func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test with empty value
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        
        // Test with collection containing empty values
        let input = FhirPathValue::collection(vec![
            FhirPathValue::Empty,
            FhirPathValue::String("test".into())
        ]);
        let context = create_test_context(input);
        let result = func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
    }
}