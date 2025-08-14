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

//! Unified take() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::{FunctionSignature, ParameterInfo};
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified take() function implementation
pub struct UnifiedTakeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTakeFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "take", 
            vec![ParameterInfo::required("count", TypeInfo::Integer)], 
            TypeInfo::Collection(Box::new(TypeInfo::Any))
        );
        
        let metadata = MetadataBuilder::collection_function("take")
            .display_name("Take")
            .description("Returns the first N elements from the collection")
            .signature(signature)
            .example("Patient.name.take(1)")
            .example("Bundle.entry.take(5)")
            .output_type(TypePattern::Any)
            .output_is_collection(true)
            .lsp_snippet("take(${1:count})")
            .keywords(vec!["take", "first", "limit", "collection", "subset"])
            .usage_pattern_with_frequency(
                "Limit collection size",
                "Patient.name.take(1)",
                "Taking only the first few elements",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Pagination",
                "Bundle.entry.take(10)",
                "Implementing simple pagination",
                UsageFrequency::Common
            )
            .related_function("first")
            .related_function("skip")
            .related_function("last")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTakeFunction {
    fn name(&self) -> &str {
        "take"
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
                name: "take".to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        // Get the count argument
        let count = match &args[0] {
            FhirPathValue::Integer(n) => {
                if *n < 0 {
                    return Err(FunctionError::EvaluationError {
                        name: "take".to_string(),
                        message: "Count must be non-negative".to_string(),
                    });
                }
                *n as usize
            }
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: "take".to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };
        
        let result = match &context.input {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Collection(items) => {
                if items.is_empty() || count == 0 {
                    FhirPathValue::collection(vec![])
                } else {
                    let take_count = std::cmp::min(count, items.len());
                    let taken_items: Vec<FhirPathValue> = items.iter()
                        .take(take_count)
                        .cloned()
                        .collect();
                    FhirPathValue::collection(taken_items)
                }
            }
            value => {
                // Single value: take 1 returns the value, take 0 returns empty
                if count > 0 {
                    value.clone()
                } else {
                    FhirPathValue::Empty
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
    async fn test_unified_take_function() {
        let take_func = UnifiedTakeFunction::new();
        
        // Test metadata
        assert_eq!(take_func.name(), "take");
        assert_eq!(take_func.execution_mode(), ExecutionMode::Sync);
        assert!(take_func.is_pure());
        
        // Test take from collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
            FhirPathValue::Integer(5),
        ]));
        
        // Take 3 elements
        let args = vec![FhirPathValue::Integer(3)];
        let result = take_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.get(1), Some(&FhirPathValue::Integer(2)));
            assert_eq!(items.get(2), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
        
        // Take 0 elements
        let args = vec![FhirPathValue::Integer(0)];
        let result = take_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected empty collection result");
        }
        
        // Take more than available
        let args = vec![FhirPathValue::Integer(10)];
        let result = take_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 5); // Should return all 5 items
        } else {
            panic!("Expected collection result");
        }
        
        // Test error cases
        let args = vec![FhirPathValue::Integer(-1)];
        let result = take_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
        
        let args = vec![FhirPathValue::String("invalid".into())];
        let result = take_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
        
        // Test no arguments (should fail)
        let args = vec![];
        let result = take_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
    }
}