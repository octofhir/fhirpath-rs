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

//! Unified replaceMatches() function implementation

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

/// Unified replaceMatches() function implementation
/// 
/// Replace all regex pattern matches with substitution text.
/// Similar to replace() but specifically for regex patterns with capture groups.
/// Syntax: replaceMatches(regex, substitution)
pub struct UnifiedReplaceMatchesFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedReplaceMatchesFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("replaceMatches", FunctionCategory::StringOperations)
            .display_name("Replace Matches")
            .description("Replace all regex pattern matches with substitution text, supporting capture groups")
            .example("'hello world'.replaceMatches('(\\\\w+)', '[$1]')")
            .example("Patient.name.replaceMatches('([A-Z])', '_$1')")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::StringLike)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("replaceMatches(${1:'pattern'}, ${2:'substitution'})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["replaceMatches", "regex", "substitute", "pattern", "capture"])
            .usage_pattern(
                "Regex substitution",
                "string.replaceMatches(pattern, substitution)",
                "Advanced text replacement with capture groups"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedReplaceMatchesFunction {
    fn name(&self) -> &str {
        "replaceMatches"
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
        // Validate arguments - exactly 2 required (pattern, substitution)
        if args.len() != 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 2,
                max: Some(2),
                actual: args.len(),
            });
        }
        
        let pattern_str = match &args[0] {
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Empty => {
                // Empty pattern - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::String(s)) => s.to_string(),
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
        
        let substitution_str = match &args[1] {
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Empty => {
                // Empty substitution - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::String(s)) => s.to_string(),
                        _ => return Ok(FhirPathValue::Empty), // Invalid collection item
                    }
                } else {
                    // Multi-item collection or empty collection is invalid
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => {
                // Invalid substitution argument type - return empty per FHIRPath spec
                return Ok(FhirPathValue::Empty);
            }
        };
        
        // Compile the regex pattern
        let regex = Regex::new(&pattern_str).map_err(|e| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Invalid regex pattern '{}': {}", pattern_str, e),
            }
        })?;
        
        // Handle collections and single values
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::String(s) => {
                            let result = regex.replace_all(s, substitution_str.as_str()).to_string();
                            results.push(FhirPathValue::String(result.into()));
                        }
                        _ => {
                            // Non-string items are converted to string first
                            let s = self.value_to_string(item);
                            let result = regex.replace_all(&s, substitution_str.as_str()).to_string();
                            results.push(FhirPathValue::String(result.into()));
                        }
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::String(s) => {
                let result = regex.replace_all(s, substitution_str.as_str()).to_string();
                Ok(FhirPathValue::String(result.into()))
            }
            single_value => {
                // Convert to string and apply replacement
                let s = self.value_to_string(single_value);
                let result = regex.replace_all(&s, substitution_str.as_str()).to_string();
                Ok(FhirPathValue::String(result.into()))
            }
        }
    }
}

impl UnifiedReplaceMatchesFunction {
    /// Convert FhirPathValue to string for regex operations
    fn value_to_string(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Date(d) => d.to_string(),
            FhirPathValue::DateTime(dt) => dt.to_string(),
            FhirPathValue::Time(t) => t.to_string(),
            FhirPathValue::Empty => String::new(),
            _ => format!("{:?}", value), // Fallback for complex types
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
    async fn test_replace_matches_basic() {
        let func = UnifiedReplaceMatchesFunction::new();
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        
        // Replace word characters with brackets
        let args = vec![
            FhirPathValue::String("(\\w+)".into()),
            FhirPathValue::String("[$1]".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("[hello] [world]".into()));
    }
    
    #[tokio::test]
    async fn test_replace_matches_capture_groups() {
        let func = UnifiedReplaceMatchesFunction::new();
        let context = create_test_context(FhirPathValue::String("John Doe".into()));
        
        // Swap first and last name
        let args = vec![
            FhirPathValue::String("(\\w+)\\s+(\\w+)".into()),
            FhirPathValue::String("$2, $1".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("Doe, John".into()));
    }
    
    #[tokio::test]
    async fn test_replace_matches_digits() {
        let func = UnifiedReplaceMatchesFunction::new();
        let context = create_test_context(FhirPathValue::String("Phone: 123-456-7890".into()));
        
        // Replace digits with X
        let args = vec![
            FhirPathValue::String("\\d".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("Phone: XXX-XXX-XXXX".into()));
    }
    
    #[tokio::test]
    async fn test_replace_matches_no_match() {
        let func = UnifiedReplaceMatchesFunction::new();
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        
        // Pattern that doesn't match
        let args = vec![
            FhirPathValue::String("\\d+".into()),
            FhirPathValue::String("NUMBER".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should return original string unchanged
        assert_eq!(result, FhirPathValue::String("hello world".into()));
    }
    
    #[tokio::test]
    async fn test_replace_matches_collection() {
        let func = UnifiedReplaceMatchesFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test123".into()),
            FhirPathValue::String("abc456".into()),
        ]);
        let context = create_test_context(collection);
        
        // Replace digits with X
        let args = vec![
            FhirPathValue::String("\\d".into()),
            FhirPathValue::String("X".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("testXXX".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("abcXXX".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_replace_matches_invalid_regex() {
        let func = UnifiedReplaceMatchesFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        
        // Invalid regex pattern
        let args = vec![
            FhirPathValue::String("(unclosed group".into()),
            FhirPathValue::String("replacement".into()),
        ];
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
        if let Err(FunctionError::EvaluationError { message, .. }) = result {
            assert!(message.contains("Invalid regex pattern"));
        } else {
            panic!("Expected EvaluationError");
        }
    }
    
    #[tokio::test]
    async fn test_replace_matches_empty_arguments() {
        let func = UnifiedReplaceMatchesFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        
        // Test with Empty pattern argument
        let args = vec![
            FhirPathValue::Empty,
            FhirPathValue::String("replacement".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with Empty substitution argument
        let args = vec![
            FhirPathValue::String("\\w".into()),
            FhirPathValue::Empty,
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with empty collection as pattern
        let args = vec![
            FhirPathValue::collection(vec![]),
            FhirPathValue::String("replacement".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with multi-item collection as substitution
        let args = vec![
            FhirPathValue::String("\\w".into()),
            FhirPathValue::collection(vec![
                FhirPathValue::String("a".into()),
                FhirPathValue::String("b".into()),
            ]),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedReplaceMatchesFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "replaceMatches");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}