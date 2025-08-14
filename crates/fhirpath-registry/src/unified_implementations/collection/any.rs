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

//! Unified any() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified any() function implementation
pub struct UnifiedAnyFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAnyFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("any", vec![], TypeInfo::Boolean);
        
        let metadata = MetadataBuilder::collection_function("any")
            .display_name("Any")
            .description("Returns true if any element in the collection is true")
            .signature(signature)
            .example("Patient.active.any()")
            .example("Observation.component.value.ofType(Boolean).any()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .output_is_collection(true) // Returns collection with single boolean
            .lsp_snippet("any()")
            .keywords(vec!["any", "some", "boolean", "collection", "logic"])
            .usage_pattern_with_frequency(
                "Check any element is true",
                "Patient.active.any()",
                "Validation that at least one boolean value is true",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Existence check",
                "Bundle.entry.resource.ofType(Patient).any()",
                "Checking if any resources of a type exist",
                UsageFrequency::Common
            )
            .related_function("all")
            .related_function("exists")
            .related_function("empty")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAnyFunction {
    fn name(&self) -> &str {
        "any"
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
            FhirPathValue::Empty => false, // Empty collection: any() returns false
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    false // Empty collection: any() returns false
                } else {
                    // Check if any item is true (when converted to boolean)
                    items.iter().any(|item| match item {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::Empty => false,
                        _ => true, // Non-empty, non-boolean values are considered truthy
                    })
                }
            }
            value => {
                // Single value: return its boolean conversion
                match value {
                    FhirPathValue::Boolean(b) => *b,
                    FhirPathValue::Empty => false,
                    _ => true,
                }
            }
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_any_function() {
        let any_func = UnifiedAnyFunction::new();
        
        // Test metadata
        assert_eq!(any_func.name(), "any");
        assert_eq!(any_func.execution_mode(), ExecutionMode::Sync);
        assert!(any_func.is_pure());
        
        // Test empty collection returns false
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = any_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        
        // Test all false values
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
        ]));
        let result = any_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
        
        // Test mixed values (one true)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
        ]));
        let result = any_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
        
        // Test single false value
        let context = EvaluationContext::new(FhirPathValue::Boolean(false));
        let result = any_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
    }
}