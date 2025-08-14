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

//! Unified contains() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified contains() function implementation
/// 
/// Checks if a string contains a substring
pub struct UnifiedContainsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedContainsFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "contains",
            vec![ParameterInfo::required("substring", TypeInfo::String)],
            TypeInfo::Boolean,
        );
        
        let metadata = MetadataBuilder::string_function("contains")
            .display_name("Contains")
            .description("Returns true if the string contains the specified substring")
            .example("Patient.name.family.contains('Smith')")
            .example("'Hello World'.contains('World')")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .lsp_snippet("contains(${1:substring})")
            .keywords(vec!["contains", "string", "search", "find"])
            .usage_pattern(
                "Check string contains substring",
                "name.contains('test')",
                "String searching and filtering"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedContainsFunction {
    fn name(&self) -> &str {
        "contains"
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
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        // Extract string from input (handling single-item collections)
        let input_string = match &context.input {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s.as_ref()
                    } else {
                        return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: format!("Expected String, got {}", items.get(0).unwrap().type_name()),
                        });
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Multiple items return empty per FHIRPath spec
                }
            },
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String, got {}", context.input.type_name()),
            }),
        };
        
        // Extract string from argument (handling single-item collections)
        let search_string = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s.as_ref()
                    } else {
                        return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: format!("Expected String, got {}", items.get(0).unwrap().type_name()),
                        });
                    }
                } else {
                    return Ok(FhirPathValue::Empty); // Multiple items return empty per FHIRPath spec
                }
            },
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String, got {}", args[0].type_name()),
            }),
        };
        
        let contains = input_string.contains(search_string);
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(contains)]))
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
    async fn test_unified_contains_function() {
        let contains_func = UnifiedContainsFunction::new();
        
        // Test contains true
        let context = EvaluationContext::new(FhirPathValue::String("Hello World".into()));
        let args = vec![FhirPathValue::String("World".into())];
        let result = contains_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test contains false
        let args = vec![FhirPathValue::String("xyz".into())];
        let result = contains_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(contains_func.name(), "contains");
        assert_eq!(contains_func.execution_mode(), ExecutionMode::Sync);
    }
}