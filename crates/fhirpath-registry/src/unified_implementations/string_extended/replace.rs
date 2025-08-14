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

//! Unified replace() function implementation

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

/// Unified replace() function implementation
/// 
/// Replaces all occurrences of a pattern with a replacement string.
/// Syntax: replace(pattern, replacement)
pub struct UnifiedReplaceFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedReplaceFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 2 required string parameters
        let signature = FunctionSignature::new(
            "replace",
            vec![
                ParameterInfo::required("pattern", TypeInfo::String),
                ParameterInfo::required("replacement", TypeInfo::String),
            ],
            TypeInfo::String,
        );

        let metadata = MetadataBuilder::new("replace", FunctionCategory::StringOperations)
            .display_name("Replace")
            .description("Replaces all occurrences of a pattern with a replacement string")
            .example("'123456'.replace('234', 'X')")
            .example("'hello world'.replace('world', 'universe')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::StringLike)
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("replace('${1:pattern}', '${2:replacement}')")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["replace", "substitute", "regex", "pattern"])
            .usage_pattern(
                "String replacement",
                "string.replace(pattern, replacement)",
                "Replacing text using regular expressions"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedReplaceFunction {
    fn name(&self) -> &str {
        "replace"
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
        // Validate arguments - exactly 2 required (pattern and replacement)
        if args.len() != 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 2,
                max: Some(2),
                actual: args.len(),
            });
        }
        
        let pattern = match &args[0] {
            FhirPathValue::String(p) => p.to_string(),
            FhirPathValue::Empty => {
                // Empty pattern - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::String(p)) => p.to_string(),
                        _ => return Ok(FhirPathValue::Empty), // Invalid collection item
                    }
                } else {
                    // Multi-item collection or empty collection is invalid
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                // Invalid pattern argument type - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };
        
        let replacement = match &args[1] {
            FhirPathValue::String(r) => r.to_string(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::String(r)) => r.to_string(),
                        _ => return Ok(FhirPathValue::Empty), // Invalid collection item
                    }
                } else {
                    // Multi-item collection or empty collection is invalid
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                // Invalid replacement argument type - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Treat single item as a collection of one
                return self.replace_single_value(single_item, &pattern, &replacement);
            }
        };
        
        // Compile the regex once
        let regex = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(e) => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Invalid regular expression: {}", e),
            }),
        };
        
        let mut results = Vec::new();
        
        for item in input_collection.iter() {
            match self.replace_item_with_regex(item, &regex, &replacement) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }
        
        if results.is_empty() {
            Ok(FhirPathValue::Empty)
        } else if results.len() == 1 {
            Ok(results.into_iter().next().unwrap())
        } else {
            Ok(FhirPathValue::collection(results))
        }
    }
}

impl UnifiedReplaceFunction {
    /// Handle replace operation on a single value
    fn replace_single_value(&self, value: &FhirPathValue, pattern: &str, replacement: &str) -> FunctionResult<FhirPathValue> {
        let regex = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Invalid regular expression: {}", e),
            }),
        };
        
        self.replace_item_with_regex(value, &regex, replacement)
    }
    
    /// Apply string replacement to a single item
    fn replace_item_with_regex(&self, item: &FhirPathValue, regex: &Regex, replacement: &str) -> FunctionResult<FhirPathValue> {
        match item {
            FhirPathValue::String(s) => {
                // Check if pattern is empty - special case
                if regex.as_str().is_empty() {
                    // Empty pattern means insert replacement between every character
                    let mut result = String::new();
                    result.push_str(replacement); // Start with replacement
                    for ch in s.chars() {
                        result.push(ch);
                        result.push_str(replacement);
                    }
                    Ok(FhirPathValue::String(result.into()))
                } else {
                    // Normal replacement using regex
                    let result = regex.replace_all(s, replacement).to_string();
                    Ok(FhirPathValue::String(result.into()))
                }
            }
            _ => {
                // Non-string input returns empty
                Ok(FhirPathValue::Empty)
            }
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
    async fn test_replace_basic() {
        let func = UnifiedReplaceFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        let args = vec![
            FhirPathValue::String("world".into()),
            FhirPathValue::String("universe".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("hello universe".into()));
    }
    
    #[tokio::test]
    async fn test_replace_regex() {
        let func = UnifiedReplaceFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello123world456".into()));
        let args = vec![
            FhirPathValue::String("\\d+".into()), // All digits
            FhirPathValue::String("_".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("hello_world_".into()));
    }
    
    #[tokio::test]
    async fn test_replace_collection() {
        let func = UnifiedReplaceFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("hello world".into()),
            FhirPathValue::String("goodbye world".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![
            FhirPathValue::String("world".into()),
            FhirPathValue::String("universe".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("hello universe".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("goodbye universe".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_replace_no_match() {
        let func = UnifiedReplaceFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![
            FhirPathValue::String("world".into()),
            FhirPathValue::String("universe".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("hello".into())); // Unchanged
    }
    
    #[tokio::test]
    async fn test_replace_invalid_regex() {
        let func = UnifiedReplaceFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![
            FhirPathValue::String("[invalid".into()), // Invalid regex
            FhirPathValue::String("replacement".into()),
        ];
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedReplaceFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "replace");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}