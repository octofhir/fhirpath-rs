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

//! Unified first() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified first() function implementation
pub struct UnifiedFirstFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedFirstFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("first", vec![], TypeInfo::Any);
        
        let metadata = MetadataBuilder::collection_function("first")
            .display_name("First")
            .description("Returns the first element in a collection")
            .signature(signature)
            .example("Patient.name.first()")
            .example("Bundle.entry.first()")
            .example("Observation.component.first()")
            .output_type(TypePattern::Any)
            .output_is_collection(false) // Returns single element, not collection
            .lsp_snippet("first()")
            .keywords(vec!["first", "head", "begin", "collection"])
            .usage_pattern_with_frequency(
                "Get first element",
                "Patient.name.first()",
                "Accessing the primary element in a collection",
                UsageFrequency::VeryCommon
            )
            .usage_pattern_with_frequency(
                "Safe navigation",
                "Patient.name.first().family",
                "Avoiding errors when collection might be empty",
                UsageFrequency::Common
            )
            .related_function("last")
            .related_function("single")
            .related_function("take")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedFirstFunction {
    fn name(&self) -> &str {
        "first"
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
                    items.get(0).unwrap().clone()
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
    async fn test_unified_first_function() {
        let first_func = UnifiedFirstFunction::new();
        
        // Test metadata
        assert_eq!(first_func.name(), "first");
        assert_eq!(first_func.execution_mode(), ExecutionMode::Sync);
        assert!(first_func.is_pure());
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = first_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test single item
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = first_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
        
        // Test collection with multiple items
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        let result = first_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
        
        // Test empty collection 
        let context = EvaluationContext::new(FhirPathValue::collection(vec![]));
        let result = first_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }
}