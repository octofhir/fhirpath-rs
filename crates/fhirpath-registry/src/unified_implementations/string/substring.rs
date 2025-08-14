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

//! Unified substring() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified substring() function implementation
/// 
/// Extracts a substring from a string value
pub struct UnifiedSubstringFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSubstringFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "substring",
            vec![
                ParameterInfo::required("start", TypeInfo::Integer),
                ParameterInfo::optional("length", TypeInfo::Integer),
            ],
            TypeInfo::String,
        );
        
        let metadata = MetadataBuilder::string_function("substring")
            .display_name("Substring")
            .description("Returns a substring starting at the specified index")
            .example("Patient.name.family.substring(0, 3)")
            .example("'Hello World'.substring(6)")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::String))
            .lsp_snippet("substring(${1:start})")
            .keywords(vec!["substring", "string", "extract", "slice"])
            .usage_pattern(
                "Extract substring",
                "name.substring(0, 5)",
                "String processing and validation"
            )
            .build();
        
        Self { metadata }
    }
    
    /// Extract substring from a string with the given arguments
    fn extract_substring(&self, input_string: &str, args: &[FhirPathValue]) -> FunctionResult<Option<FhirPathValue>> {
        // Get start index - return None for invalid arguments per FHIRPath spec
        let start = match &args[0] {
            FhirPathValue::Integer(i) => {
                if *i < 0 {
                    // Negative start index is invalid - return empty per FHIRPath spec
                    return Ok(None);
                }
                *i as usize
            }
            FhirPathValue::Empty => {
                // Empty argument means invalid - return empty per FHIRPath spec
                return Ok(None);
            }
            FhirPathValue::Collection(items) => {
                // Collection argument - check if it's a single integer
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Integer(i)) => {
                            if *i < 0 {
                                return Ok(None);
                            }
                            *i as usize
                        }
                        _ => return Ok(None), // Invalid collection item
                    }
                } else {
                    // Multi-item collection or empty collection is invalid
                    return Ok(None);
                }
            }
            _ => {
                // Invalid argument type - return empty per FHIRPath spec
                return Ok(None);
            }
        };
        
        // Check bounds
        if start >= input_string.len() {
            return Ok(Some(FhirPathValue::String("".into())));
        }
        
        let result = if args.len() == 2 {
            // With length parameter
            let length = match &args[1] {
                FhirPathValue::Integer(i) => {
                    if *i < 0 {
                        // Negative length is invalid - return empty
                        return Ok(None);
                    }
                    *i as usize
                }
                FhirPathValue::Empty => {
                    // Empty length argument is invalid - return empty
                    return Ok(None);
                }
                FhirPathValue::Collection(items) => {
                    // Collection argument - check if it's a single integer
                    if items.len() == 1 {
                        match items.get(0) {
                            Some(FhirPathValue::Integer(i)) => {
                                if *i < 0 {
                                    return Ok(None);
                                }
                                *i as usize
                            }
                            _ => return Ok(None), // Invalid collection item
                        }
                    } else {
                        // Multi-item collection or empty collection is invalid
                        return Ok(None);
                    }
                }
                _ => {
                    // Invalid length argument type - return empty
                    return Ok(None);
                }
            };
            
            let end = std::cmp::min(start + length, input_string.len());
            input_string.chars().skip(start).take(end - start).collect::<String>()
        } else {
            // Without length parameter (to end of string)
            input_string.chars().skip(start).collect::<String>()
        };
        
        Ok(Some(FhirPathValue::String(result.into())))
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSubstringFunction {
    fn name(&self) -> &str {
        "substring"
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
        // Validate arguments
        if args.len() < 1 || args.len() > 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(2),
                actual: args.len(),
            });
        }
        
        // Handle input based on its type
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut result = Vec::new();
                for item in items.iter() {
                    // Convert item to string if possible
                    match item {
                        FhirPathValue::String(s) => {
                            if let Some(substring) = self.extract_substring(s.as_ref(), &args)? {
                                result.push(substring);
                            }
                        }
                        FhirPathValue::Integer(i) => {
                            let string_repr = i.to_string();
                            if let Some(substring) = self.extract_substring(&string_repr, &args)? {
                                result.push(substring);
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            let string_repr = d.to_string();
                            if let Some(substring) = self.extract_substring(&string_repr, &args)? {
                                result.push(substring);
                            }
                        }
                        FhirPathValue::Boolean(b) => {
                            let string_repr = if *b { "true" } else { "false" };
                            if let Some(substring) = self.extract_substring(string_repr, &args)? {
                                result.push(substring);
                            }
                        }
                        FhirPathValue::Empty => {
                            // Skip empty values in collections
                            continue;
                        }
                        _ => return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: format!("Cannot convert {} to string for substring operation", item.type_name()),
                        }),
                    };
                    // If extract_substring returns None, skip this item (invalid arguments)
                }
                return Ok(FhirPathValue::collection(result));
            }
            FhirPathValue::String(s) => {
                if let Some(substring) = self.extract_substring(s.as_ref(), &args)? {
                    return Ok(FhirPathValue::collection(vec![substring]));
                } else {
                    // Invalid arguments - return empty per FHIRPath spec
                    return Ok(FhirPathValue::Empty);
                }
            }
            FhirPathValue::Integer(i) => {
                let string_value = i.to_string();
                if let Some(substring) = self.extract_substring(&string_value, &args)? {
                    return Ok(FhirPathValue::collection(vec![substring]));
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            FhirPathValue::Decimal(d) => {
                let string_value = d.to_string();
                if let Some(substring) = self.extract_substring(&string_value, &args)? {
                    return Ok(FhirPathValue::collection(vec![substring]));
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            FhirPathValue::Boolean(b) => {
                let string_value = if *b { "true" } else { "false" };
                if let Some(substring) = self.extract_substring(string_value, &args)? {
                    return Ok(FhirPathValue::collection(vec![substring]));
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Cannot convert {} to string for substring operation", context.input.type_name()),
            }),
        }
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_substring_function() {
        let substring_func = UnifiedSubstringFunction::new();
        
        // Test substring with start only
        let context = EvaluationContext::new(FhirPathValue::String("Hello World".into()));
        let args = vec![FhirPathValue::Integer(6)];
        let result = substring_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("World".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test substring with start and length
        let args = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(5)];
        let result = substring_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("Hello".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(substring_func.name(), "substring");
        assert_eq!(substring_func.execution_mode(), ExecutionMode::Sync);
    }
    
    #[tokio::test]
    async fn test_substring_with_empty_arguments() {
        let substring_func = UnifiedSubstringFunction::new();
        
        // Test 'string'.substring({}).empty() = true
        let context = EvaluationContext::new(FhirPathValue::String("string".into()));
        let args = vec![FhirPathValue::Empty]; // {} evaluates to Empty
        let result = substring_func.evaluate_sync(&args, &context).unwrap();
        
        // Should return Empty (which makes .empty() = true)
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with negative start index
        let args = vec![FhirPathValue::Integer(-1)];
        let result = substring_func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with empty collection argument
        let args = vec![FhirPathValue::collection(vec![])];
        let result = substring_func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
        
        // Test with negative length
        let args = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(-1)];
        let result = substring_func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }
}