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

//! Unified matches() function implementation

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

/// Unified matches() function implementation
/// 
/// Tests if a string matches a regular expression pattern.
/// Syntax: matches(pattern)
pub struct UnifiedMatchesFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedMatchesFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 required string parameter
        let signature = FunctionSignature::new(
            "matches",
            vec![ParameterInfo::required("pattern", TypeInfo::String)],
            TypeInfo::Boolean,
        );

        let metadata = MetadataBuilder::new("matches", FunctionCategory::StringOperations)
            .display_name("Matches")
            .description("Returns true if the input string matches the given regular expression")
            .example("Patient.name.family.matches('^[A-Z][a-z]+$')")
            .example("'hello123'.matches('^\\\\w+$')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("matches('${1:pattern}')")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["matches", "regex", "pattern", "test"])
            .usage_pattern(
                "Pattern matching",
                "string.matches(regex)",
                "Testing strings against regular expressions"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedMatchesFunction {
    fn name(&self) -> &str {
        "matches"
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
        // Validate arguments - exactly 1 required (regex pattern)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let pattern = match &args[0] {
            FhirPathValue::String(p) => p.to_string(),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "matches() requires a string pattern argument".to_string(),
            }),
        };
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Treat single item as a collection of one
                return self.matches_single_value(single_item, &pattern);
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
            match self.matches_item_with_regex(item, &regex) {
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

impl UnifiedMatchesFunction {
    /// Handle matches operation on a single value
    fn matches_single_value(&self, value: &FhirPathValue, pattern: &str) -> FunctionResult<FhirPathValue> {
        let regex = match Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Invalid regular expression: {}", e),
            }),
        };
        
        self.matches_item_with_regex(value, &regex)
    }
    
    /// Apply regex matching to a single item
    fn matches_item_with_regex(&self, item: &FhirPathValue, regex: &Regex) -> FunctionResult<FhirPathValue> {
        match item {
            FhirPathValue::String(s) => {
                let matches = regex.is_match(s);
                Ok(FhirPathValue::Boolean(matches))
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "matches() can only be applied to string values".to_string(),
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
    async fn test_matches_basic() {
        let func = UnifiedMatchesFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::String("^h.*o$".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_matches_no_match() {
        let func = UnifiedMatchesFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::String("^world$".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_matches_collection() {
        let func = UnifiedMatchesFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("test123".into()),
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("123test".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("\\d+".into())]; // Contains digits
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Boolean(true)));  // test123
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Boolean(false))); // hello
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::Boolean(true)));  // 123test
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_matches_invalid_regex() {
        let func = UnifiedMatchesFunction::new();
        
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let args = vec![FhirPathValue::String("[invalid".into())]; // Invalid regex
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedMatchesFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "matches");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}