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

//! Unified skip() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::{FunctionSignature, ParameterInfo};
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified skip() function implementation
pub struct UnifiedSkipFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSkipFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "skip", 
            vec![ParameterInfo::required("count", TypeInfo::Integer)], 
            TypeInfo::Collection(Box::new(TypeInfo::Any))
        );
        
        let metadata = MetadataBuilder::collection_function("skip")
            .display_name("Skip")
            .description("Returns all elements except the first N elements")
            .signature(signature)
            .example("Bundle.entry.skip(2)")
            .example("Patient.name.skip(1)")
            .output_type(TypePattern::Any)
            .output_is_collection(true)
            .lsp_snippet("skip(${1:count})")
            .keywords(vec!["skip", "drop", "offset", "collection", "subset"])
            .usage_pattern_with_frequency(
                "Skip elements",
                "Bundle.entry.skip(5)",
                "Implementing pagination or skipping headers",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Remove first item",
                "Patient.name.skip(1)",
                "Getting all but the first element",
                UsageFrequency::Common
            )
            .related_function("take")
            .related_function("first")
            .related_function("tail")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSkipFunction {
    fn name(&self) -> &str {
        "skip"
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
                name: "skip".to_string(),
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
                        name: "skip".to_string(),
                        message: "Count must be non-negative".to_string(),
                    });
                }
                *n as usize
            }
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: "skip".to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };
        
        let result = match &context.input {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Collection(items) => {
                if items.is_empty() || count >= items.len() {
                    FhirPathValue::collection(vec![])
                } else {
                    let skipped_items: Vec<FhirPathValue> = items.iter()
                        .skip(count)
                        .cloned()
                        .collect();
                    FhirPathValue::collection(skipped_items)
                }
            }
            value => {
                // Single value: skip 0 returns the value, skip >= 1 returns empty
                if count == 0 {
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
    async fn test_unified_skip_function() {
        let skip_func = UnifiedSkipFunction::new();
        
        // Test metadata
        assert_eq!(skip_func.name(), "skip");
        assert_eq!(skip_func.execution_mode(), ExecutionMode::Sync);
        assert!(skip_func.is_pure());
        
        // Test skip from collection
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
            FhirPathValue::Integer(5),
        ]));
        
        // Skip 2 elements
        let args = vec![FhirPathValue::Integer(2)];
        let result = skip_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
            assert_eq!(items.get(1), Some(&FhirPathValue::Integer(4)));
            assert_eq!(items.get(2), Some(&FhirPathValue::Integer(5)));
        } else {
            panic!("Expected collection result");
        }
        
        // Skip all elements
        let args = vec![FhirPathValue::Integer(5)];
        let result = skip_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected empty collection result");
        }
        
        // Test error cases
        let args = vec![FhirPathValue::Integer(-1)];
        let result = skip_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
    }
}