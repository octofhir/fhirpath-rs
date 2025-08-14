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

//! Unified single() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified single() function implementation
pub struct UnifiedSingleFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSingleFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("single", vec![], TypeInfo::Any);
        
        let metadata = MetadataBuilder::collection_function("single")
            .display_name("Single")
            .description("Returns the single element in a collection, or throws an error if there is not exactly one element")
            .signature(signature)
            .example("Patient.name.single()")
            .example("Patient.identifier.where(use='official').single()")
            .output_type(TypePattern::Any)
            .output_is_collection(false) // Returns single element, not collection
            .lsp_snippet("single()")
            .keywords(vec!["single", "only", "unique", "collection", "validation"])
            .usage_pattern_with_frequency(
                "Enforce single element",
                "Patient.name.single()",
                "Validation that exactly one element exists",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Extract required element",
                "Patient.identifier.where(use='official').single()",
                "Getting a required unique element with validation",
                UsageFrequency::Common
            )
            .related_function("first")
            .related_function("last")
            .related_function("count")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSingleFunction {
    fn name(&self) -> &str {
        "single"
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
            FhirPathValue::Empty => {
                return Err(FunctionError::EvaluationError {
                    name: "single".to_string(),
                    message: "Collection is empty, expected exactly one element".to_string(),
                });
            }
            FhirPathValue::Collection(items) => {
                match items.len() {
                    0 => {
                        return Err(FunctionError::EvaluationError {
                            name: "single".to_string(),
                            message: "Collection is empty, expected exactly one element".to_string(),
                        });
                    }
                    1 => items.get(0).unwrap().clone(),
                    n => {
                        return Err(FunctionError::EvaluationError {
                            name: "single".to_string(),
                            message: format!("Collection has {} elements, expected exactly one element", n),
                        });
                    }
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
    async fn test_unified_single_function() {
        let single_func = UnifiedSingleFunction::new();
        
        // Test metadata
        assert_eq!(single_func.name(), "single");
        assert_eq!(single_func.execution_mode(), ExecutionMode::Sync);
        assert!(single_func.is_pure());
        
        // Test single item succeeds
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = single_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
        
        // Test collection with single item succeeds
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
        ]));
        let result = single_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
        
        // Test empty collection fails
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = single_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Collection is empty"));
        
        // Test empty collection (explicit) fails
        let context = EvaluationContext::new(FhirPathValue::collection(vec![]));
        let result = single_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Collection is empty"));
        
        // Test multiple items fails
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        let result = single_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("3 elements"));
    }
}