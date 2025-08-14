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

//! Unified flatten() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified flatten() function implementation
pub struct UnifiedFlattenFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedFlattenFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("flatten", vec![], TypeInfo::Any);
        
        let metadata = MetadataBuilder::collection_function("flatten")
            .display_name("Flatten")
            .description("Flattens nested collections into a single collection")
            .signature(signature)
            .example("Bundle.entry.resource.contained.flatten()")
            .example("Patient.name.extension.flatten()")
            .output_type(TypePattern::Any)
            .output_is_collection(true)
            .lsp_snippet("flatten()")
            .keywords(vec!["flatten", "nested", "collection", "merge"])
            .usage_pattern_with_frequency(
                "Flatten nested collections",
                "Bundle.entry.resource.contained.flatten()",
                "Converting nested structures to flat collections",
                UsageFrequency::Moderate
            )
            .usage_pattern_with_frequency(
                "Merge collections",
                "Patient.name.extension.flatten()",
                "Combining multiple nested collections",
                UsageFrequency::Moderate
            )
            .related_function("distinct")
            .related_function("union")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedFlattenFunction {
    fn name(&self) -> &str {
        "flatten"
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
                    FhirPathValue::collection(vec![])
                } else {
                    let mut flattened = Vec::new();
                    
                    for item in items.iter() {
                        match item {
                            FhirPathValue::Collection(nested_items) => {
                                // Flatten nested collection
                                flattened.extend(nested_items.iter().cloned());
                            }
                            FhirPathValue::Empty => {
                                // Skip empty values
                            }
                            _ => {
                                // Add non-collection items directly
                                flattened.push(item.clone());
                            }
                        }
                    }
                    
                    FhirPathValue::collection(flattened)
                }
            }
            value => {
                // Single value: flatten has no effect
                value.clone()
            }
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_flatten_function() {
        let flatten_func = UnifiedFlattenFunction::new();
        
        // Test metadata
        assert_eq!(flatten_func.name(), "flatten");
        assert_eq!(flatten_func.execution_mode(), ExecutionMode::Sync);
        assert!(flatten_func.is_pure());
        
        // Test flatten nested collections
        let nested_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::collection(vec![
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
            ]),
            FhirPathValue::Integer(4),
            FhirPathValue::collection(vec![
                FhirPathValue::Integer(5),
            ]),
        ]);
        
        let context = EvaluationContext::new(nested_collection);
        let result = flatten_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 5);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.get(1), Some(&FhirPathValue::Integer(2)));
            assert_eq!(items.get(2), Some(&FhirPathValue::Integer(3)));
            assert_eq!(items.get(3), Some(&FhirPathValue::Integer(4)));
            assert_eq!(items.get(4), Some(&FhirPathValue::Integer(5)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test flatten with no nested collections
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]));
        let result = flatten_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.get(1), Some(&FhirPathValue::Integer(2)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test flatten single value
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = flatten_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }
}