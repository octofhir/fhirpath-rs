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

//! Unified distinct() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified distinct() function implementation
pub struct UnifiedDistinctFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedDistinctFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new("distinct", vec![], TypeInfo::Any);
        
        let metadata = MetadataBuilder::collection_function("distinct")
            .display_name("Distinct")
            .description("Returns a collection with duplicate elements removed")
            .signature(signature)
            .example("Patient.name.family.distinct()")
            .example("Bundle.entry.resource.resourceType.distinct()")
            .output_type(TypePattern::Any)
            .output_is_collection(true)
            .lsp_snippet("distinct()")
            .keywords(vec!["distinct", "unique", "duplicate", "collection", "set"])
            .usage_pattern_with_frequency(
                "Remove duplicates",
                "Patient.name.family.distinct()",
                "Getting unique values from a collection",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Unique types",
                "Bundle.entry.resource.resourceType.distinct()",
                "Finding unique resource types in a bundle",
                UsageFrequency::Common
            )
            .related_function("count")
            .related_function("intersect")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedDistinctFunction {
    fn name(&self) -> &str {
        "distinct"
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
                    let mut unique_items = Vec::new();
                    let mut seen = std::collections::HashSet::new();
                    
                    for item in items.iter() {
                        // Create a simple hash key based on the value
                        let key = format!("{:?}", item);
                        if seen.insert(key) {
                            unique_items.push(item.clone());
                        }
                    }
                    
                    FhirPathValue::collection(unique_items)
                }
            }
            value => value.clone(), // Single value is already distinct
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_distinct_function() {
        let distinct_func = UnifiedDistinctFunction::new();
        
        // Test metadata
        assert_eq!(distinct_func.name(), "distinct");
        assert_eq!(distinct_func.execution_mode(), ExecutionMode::Sync);
        assert!(distinct_func.is_pure());
        
        // Test empty collection
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test collection with duplicates
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2),
        ]));
        let result = distinct_func.evaluate_sync(&[], &context).unwrap();
        
        // Check that result is a collection with unique values
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3); // Should have 3 unique values
            // Note: Order may not be preserved, so we just check the count
        } else {
            panic!("Expected collection result");
        }
        
        // Test single value
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = distinct_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }
}