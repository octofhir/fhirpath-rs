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

//! Unified empty() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified empty() function implementation
pub struct UnifiedEmptyFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedEmptyFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("empty", vec![], TypeInfo::Boolean);
        
        let metadata = MetadataBuilder::collection_function("empty")
            .display_name("Empty")
            .description("Returns true if the collection is empty")
            .signature(signature)
            .example("Patient.name.empty()")
            .example("Bundle.entry.empty()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .output_is_collection(true)
            .lsp_snippet("empty()")
            .keywords(vec!["empty", "null", "exists", "collection"])
            .usage_pattern_with_frequency(
                "Check if collection is empty",
                "Patient.name.empty()",
                "Conditional logic based on presence",
                UsageFrequency::VeryCommon
            )
            .related_function("exists")
            .related_function("count")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedEmptyFunction {
    fn name(&self) -> &str {
        "empty"
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
        self.validate_args(args)?;
        
        let is_empty = match &context.input {
            FhirPathValue::Empty => true,
            FhirPathValue::Collection(items) => items.is_empty(),
            _ => false,
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_empty)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_empty_function() {
        let empty_func = UnifiedEmptyFunction::new();
        
        // Test metadata
        assert_eq!(empty_func.name(), "empty");
        assert_eq!(empty_func.execution_mode(), ExecutionMode::Sync);
        assert!(empty_func.is_pure());
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = empty_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test non-empty single item
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = empty_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        
        // Test non-empty collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
        ]));
        let result = empty_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
    }
}