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

//! Unified startsWith() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified startsWith() function implementation
/// 
/// Checks if a string starts with a prefix
pub struct UnifiedStartsWithFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedStartsWithFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "startsWith",
            vec![ParameterInfo::required("prefix", TypeInfo::String)],
            TypeInfo::Boolean,
        );
        
        let metadata = MetadataBuilder::string_function("startsWith")
            .display_name("Starts With")
            .description("Returns true if the string starts with the specified prefix")
            .example("Patient.name.family.startsWith('Dr')")
            .example("'Hello World'.startsWith('Hello')")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .lsp_snippet("startsWith(${1:prefix})")
            .keywords(vec!["startsWith", "string", "prefix", "begins"])
            .usage_pattern(
                "Check string starts with prefix",
                "name.startsWith('Dr')",
                "String prefix validation"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedStartsWithFunction {
    fn name(&self) -> &str {
        "startsWith"
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
        
        let input_string = match &context.input {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String, got {}", context.input.type_name()),
            }),
        };
        
        let prefix = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String, got {}", args[0].type_name()),
            }),
        };
        
        let starts_with = input_string.starts_with(prefix);
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(starts_with)]))
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
    async fn test_unified_startsWith_function() {
        let starts_with_func = UnifiedStartsWithFunction::new();
        
        // Test startsWith true
        let context = EvaluationContext::new(FhirPathValue::String("Hello World".into()));
        let args = vec![FhirPathValue::String("Hello".into())];
        let result = starts_with_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test startsWith false
        let args = vec![FhirPathValue::String("World".into())];
        let result = starts_with_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(starts_with_func.name(), "startsWith");
        assert_eq!(starts_with_func.execution_mode(), ExecutionMode::Sync);
    }
}