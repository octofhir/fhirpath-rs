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

//! Unified last() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified last() function implementation
pub struct UnifiedLastFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedLastFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("last", vec![], TypeInfo::Any);
        
        let metadata = MetadataBuilder::collection_function("last")
            .display_name("Last")
            .description("Returns the last element in a collection")
            .signature(signature)
            .example("Patient.name.last()")
            .example("Bundle.entry.last()")
            .example("Observation.component.last()")
            .output_type(TypePattern::Any)
            .output_is_collection(false) // Returns single element, not collection
            .lsp_snippet("last()")
            .keywords(vec!["last", "tail", "end", "collection"])
            .usage_pattern_with_frequency(
                "Get last element",
                "Patient.name.last()",
                "Accessing the final element in a collection",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Latest value",
                "Observation.value.last()",
                "Getting the most recent value",
                UsageFrequency::Common
            )
            .related_function("first")
            .related_function("single")
            .related_function("skip")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedLastFunction {
    fn name(&self) -> &str {
        "last"
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
        
        let result = match &context.input {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    items.get(items.len() - 1).unwrap().clone()
                }
            }
            value => value.clone(), // Single value returns itself
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_last_function() {
        let last_func = UnifiedLastFunction::new();
        
        // Test metadata
        assert_eq!(last_func.name(), "last");
        assert_eq!(last_func.execution_mode(), ExecutionMode::Sync);
        assert!(last_func.is_pure());
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = last_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test single item
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = last_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
        
        // Test collection with multiple items
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        let result = last_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));
        
        // Test empty collection 
        let context = EvaluationContext::new(FhirPathValue::collection(vec![]));
        let result = last_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }
}