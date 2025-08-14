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

//! Unified count() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified count() function implementation
/// 
/// Migrated from AsyncFhirPathFunction to UnifiedFhirPathFunction
/// This function is actually synchronous despite being in the async registry
pub struct UnifiedCountFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedCountFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("count", vec![], TypeInfo::Integer);
        
        let metadata = MetadataBuilder::collection_function("count")
            .display_name("Count")
            .description("Returns the number of items in the collection")
            .signature(signature)
            .example("Patient.name.count()")
            .example("Bundle.entry.count()")
            .example("Observation.component.count()")
            .lsp_snippet("count()")
            .keywords(vec!["count", "size", "length", "collection"])
            .usage_pattern_with_frequency(
                "Count collection elements",
                "Patient.name.count()",
                "Checking collection size for validation",
                UsageFrequency::VeryCommon
            )
            .usage_pattern_with_frequency(
                "Verify single element",
                "Patient.name.count() = 1",
                "Ensuring exactly one element exists",
                UsageFrequency::Common
            )
            .related_function("empty")
            .related_function("exists")
            .related_function("length")
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .output_is_collection(true) // Always returns collection with single integer
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedCountFunction {
    fn name(&self) -> &str {
        "count"
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
        
        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(count as i64)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_count_function() {
        let count_func = UnifiedCountFunction::new();
        
        // Test metadata
        assert_eq!(count_func.name(), "count");
        assert_eq!(count_func.execution_mode(), ExecutionMode::Sync);
        assert!(count_func.is_pure());
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = count_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Integer(0)]));
        
        // Test single item
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = count_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Integer(1)]));
        
        // Test collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        let result = count_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Integer(3)]));
        
        // Test async evaluation (should work via default implementation)
        let result = count_func.evaluate_async(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Integer(3)]));
    }
}