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

//! Unified indexOf() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified indexOf() function implementation
pub struct UnifiedIndexOfFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedIndexOfFunction {
    pub fn new() -> Self {
        use crate::signature::ParameterInfo;
        
        let signature = FunctionSignature::new(
            "indexOf", 
            vec![ParameterInfo::required("element", TypeInfo::Any)], 
            TypeInfo::Integer
        );
        
        let metadata = MetadataBuilder::collection_function("indexOf")
            .display_name("IndexOf")
            .description("Returns the 0-based index of the given element in a collection or substring in a string. Returns -1 if not found.")
            .signature(signature)
            .example("'LogicalModel-Person'.indexOf('-')")
            .example("Bundle.entry.indexOf($entry)")
            .example("Patient.name.indexOf($specificName)")
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .output_is_collection(false)
            .lsp_snippet("indexOf(${1:element})")
            .keywords(vec!["indexOf", "index", "position", "collection", "search", "string"])
            .usage_pattern_with_frequency(
                "Find substring position",
                "'LogicalModel-Person'.indexOf('-')",
                "String manipulation and parsing",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Find element position",
                "Bundle.entry.indexOf($entry)",
                "Locating specific elements in collections",
                UsageFrequency::Moderate
            )
            .usage_pattern_with_frequency(
                "Check element presence",
                "Patient.identifier.indexOf($id) >= 0",
                "Testing if element exists and getting position",
                UsageFrequency::Moderate
            )
            .related_function("count")
            .related_function("exists")
            .related_function("contains")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedIndexOfFunction {
    fn name(&self) -> &str {
        "indexOf"
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
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: "indexOf".to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let search_element = &args[0];
        
        let result = match &context.input {
            FhirPathValue::Empty => {
                // Empty collection: indexOf returns empty
                FhirPathValue::Empty
            }
            FhirPathValue::String(haystack) => {
                // String indexOf: find substring position
                match search_element {
                    FhirPathValue::String(needle) => {
                        if needle.is_empty() {
                            // Empty string found at position 0
                            FhirPathValue::Integer(0)
                        } else {
                            // Find the index of the substring
                            match haystack.find(needle.as_ref()) {
                                Some(index) => FhirPathValue::Integer(index as i64),
                                None => FhirPathValue::Integer(-1), // Not found
                            }
                        }
                    }
                    _ => {
                        // Can't search for non-string in string
                        FhirPathValue::Empty
                    }
                }
            }
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    // Find the index of the element
                    for (index, item) in items.iter().enumerate() {
                        // Simple equality check (could be enhanced for deeper comparison)
                        if format!("{:?}", item) == format!("{:?}", search_element) {
                            return Ok(FhirPathValue::Integer(index as i64));
                        }
                    }
                    // Element not found
                    FhirPathValue::Integer(-1) // Not found
                }
            }
            value => {
                // Single value: check if it matches the search element
                if format!("{:?}", value) == format!("{:?}", search_element) {
                    FhirPathValue::Integer(0)
                } else {
                    FhirPathValue::Integer(-1) // Not found
                }
            }
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_indexOf_function() {
        let indexOf_func = UnifiedIndexOfFunction::new();
        
        // Test metadata
        assert_eq!(indexOf_func.name(), "indexOf");
        assert_eq!(indexOf_func.execution_mode(), ExecutionMode::Sync);
        assert!(indexOf_func.is_pure());
        
        // Test indexOf in collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]));
        
        // Find element at index 1
        let args = vec![FhirPathValue::String("b".into())];
        let result = indexOf_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
        } else {
            panic!("Expected collection result");
        }
        
        // Element not found
        let args = vec![FhirPathValue::String("z".into())];
        let result = indexOf_func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with single value
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));
        let args = vec![FhirPathValue::String("test".into())];
        let result = indexOf_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(0)));
        } else {
            panic!("Expected collection result");
        }
    }
}