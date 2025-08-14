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

//! Unified tail() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified tail() function implementation  
pub struct UnifiedTailFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTailFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("tail", vec![], TypeInfo::Any);
        
        let metadata = MetadataBuilder::collection_function("tail")
            .display_name("Tail")
            .description("Returns all elements except the first element")
            .signature(signature)
            .example("Patient.name.tail()")
            .example("Bundle.entry.tail()")
            .output_type(TypePattern::Any)
            .output_is_collection(true)
            .lsp_snippet("tail()")
            .keywords(vec!["tail", "rest", "remaining", "collection"])
            .usage_pattern_with_frequency(
                "Get all but first",
                "Patient.name.tail()",
                "Processing remaining elements after first",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Skip header",
                "Bundle.entry.tail()",
                "Skip first entry and process rest",
                UsageFrequency::Moderate
            )
            .related_function("first")
            .related_function("skip")
            .related_function("last")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTailFunction {
    fn name(&self) -> &str {
        "tail"
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
                if items.len() <= 1 {
                    FhirPathValue::collection(vec![])
                } else {
                    let tail_items: Vec<FhirPathValue> = items.iter()
                        .skip(1)
                        .cloned()
                        .collect();
                    FhirPathValue::collection(tail_items)
                }
            }
            _value => {
                // Single value has no tail
                FhirPathValue::Empty
            }
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_tail_function() {
        let tail_func = UnifiedTailFunction::new();
        
        // Test metadata
        assert_eq!(tail_func.name(), "tail");
        assert_eq!(tail_func.execution_mode(), ExecutionMode::Sync);
        assert!(tail_func.is_pure());
        
        // Test tail of collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        
        let result = tail_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(2)));
            assert_eq!(items.get(1), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test tail of single item
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
        ]));
        let result = tail_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected empty collection result");
        }
        
        // Test tail of single value (not collection)
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = tail_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }
}