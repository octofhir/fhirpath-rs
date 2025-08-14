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

//! Unified endsWith() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified endsWith() function implementation
/// 
/// Checks if a string ends with a suffix
pub struct UnifiedEndsWithFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedEndsWithFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "endsWith",
            vec![ParameterInfo::required("suffix", TypeInfo::String)],
            TypeInfo::Boolean,
        );
        
        let metadata = MetadataBuilder::string_function("endsWith")
            .display_name("Ends With")
            .description("Returns true if the string ends with the specified suffix")
            .example("Patient.name.family.endsWith('son')")
            .example("'Hello World'.endsWith('World')")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .lsp_snippet("endsWith(${1:suffix})")
            .keywords(vec!["endsWith", "string", "suffix", "ends"])
            .usage_pattern(
                "Check string ends with suffix",
                "name.endsWith('son')",
                "String suffix validation"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedEndsWithFunction {
    fn name(&self) -> &str {
        "endsWith"
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
        let suffix = match &args[0] {
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
        
        let ends_with = input_string.ends_with(suffix);
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(ends_with)]))
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
    async fn test_unified_endsWith_function() {
        let ends_with_func = UnifiedEndsWithFunction::new();
        
        // Test endsWith true
        let context = EvaluationContext::new(FhirPathValue::String("Hello World".into()));
        let args = vec![FhirPathValue::String("World".into())];
        let result = ends_with_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test endsWith false
        let args = vec![FhirPathValue::String("Hello".into())];
        let result = ends_with_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(ends_with_func.name(), "endsWith");
        assert_eq!(ends_with_func.execution_mode(), ExecutionMode::Sync);
    }
}