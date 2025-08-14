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

//! Unified split() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use regex::Regex;

/// Unified split() function implementation
/// 
/// Splits a string into a collection using a separator pattern.
/// Syntax: split(separator)
pub struct UnifiedSplitFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSplitFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 required string parameter  
        let signature = FunctionSignature::new(
            "split",
            vec![ParameterInfo::required("separator", TypeInfo::String)],
            TypeInfo::Collection(Box::new(TypeInfo::String)),
        );

        let metadata = MetadataBuilder::new("split", FunctionCategory::StringOperations)
            .display_name("Split")
            .description("Splits a string into a collection using a separator pattern")
            .example("'one,two,three'.split(',')")
            .example("Patient.name.text.split('\\\\s+')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::StringLike)))
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("split('${1:separator}')")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["split", "separate", "divide", "tokenize"])
            .usage_pattern(
                "String splitting",
                "string.split(separator)",
                "Breaking strings into parts using delimiters"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSplitFunction {
    fn name(&self) -> &str {
        "split"
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
        // Validate arguments - exactly 1 required (separator pattern)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let separator = match &args[0] {
            FhirPathValue::String(s) => s.to_string(),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "split() requires a string separator argument".to_string(),
            }),
        };
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Treat single item as a collection of one
                return self.split_single_value(single_item, &separator);
            }
        };
        
        // Compile the regex once
        let regex = match Regex::new(&separator) {
            Ok(r) => r,
            Err(e) => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Invalid regular expression: {}", e),
            }),
        };
        
        let mut all_results = Vec::new();
        
        for item in input_collection.iter() {
            match self.split_item_with_regex(item, &regex) {
                Ok(result) => {
                    // Flatten results from each string into the final collection
                    match result {
                        FhirPathValue::Collection(parts) => {
                            for part in parts.iter() {
                                all_results.push(part.clone());
                            }
                        }
                        single_part => all_results.push(single_part),
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        if all_results.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(all_results))
        }
    }
}

impl UnifiedSplitFunction {
    /// Handle split operation on a single value
    fn split_single_value(&self, value: &FhirPathValue, separator: &str) -> FunctionResult<FhirPathValue> {
        let regex = match Regex::new(separator) {
            Ok(r) => r,
            Err(e) => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Invalid regular expression: {}", e),
            }),
        };
        
        self.split_item_with_regex(value, &regex)
    }
    
    /// Apply regex splitting to a single item
    fn split_item_with_regex(&self, item: &FhirPathValue, regex: &Regex) -> FunctionResult<FhirPathValue> {
        match item {
            FhirPathValue::String(s) => {
                let parts: Vec<FhirPathValue> = regex
                    .split(s)
                    .map(|part| FhirPathValue::String(part.into()))
                    .collect();
                
                if parts.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(parts))
                }
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "split() can only be applied to string values".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_split_basic() {
        let func = UnifiedSplitFunction::new();
        
        let context = create_test_context(FhirPathValue::String("one,two,three".into()));
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("one".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("two".into())));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::String("three".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_split_regex() {
        let func = UnifiedSplitFunction::new();
        
        let context = create_test_context(FhirPathValue::String("one  two\tthree".into()));
        let args = vec![FhirPathValue::String("\\s+".into())]; // Split on whitespace
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("one".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("two".into())));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::String("three".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_split_no_separator() {
        let func = UnifiedSplitFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should return single item collection
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("hello".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_split_empty_string() {
        let func = UnifiedSplitFunction::new();
        
        let context = create_test_context(FhirPathValue::String("".into()));
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Empty string split should return collection with one empty string
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_split_collection() {
        let func = UnifiedSplitFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a,b".into()),
            FhirPathValue::String("c,d".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should flatten all split results
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("a".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("b".into())));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::String("c".into())));
            assert_eq!(items.iter().nth(3), Some(&FhirPathValue::String("d".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_split_invalid_regex() {
        let func = UnifiedSplitFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::String("[invalid".into())]; // Invalid regex
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedSplitFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "split");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}