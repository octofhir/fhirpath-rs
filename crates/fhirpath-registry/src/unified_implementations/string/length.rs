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

//! Unified length() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified length() function implementation
/// 
/// Returns the length of string values or collections
pub struct UnifiedLengthFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedLengthFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::string_function("length")
            .display_name("Length")
            .description("Returns the length of a string or the number of items in a collection")
            .example("Patient.name.family.length()")
            .example("'Hello World'.length()")
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .lsp_snippet("length()")
            .keywords(vec!["length", "size", "count", "string"])
            .usage_pattern(
                "Get string length",
                "name.length()",
                "String validation and processing"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedLengthFunction {
    fn name(&self) -> &str {
        "length"
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
        // Validate no arguments
        if !args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(0),
                actual: args.len(),
            });
        }
        
        let length = match &context.input {
            FhirPathValue::String(s) => s.len() as i64,
            FhirPathValue::Collection(items) => items.len() as i64,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String or Collection, got {}", context.input.type_name()),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(length)]))
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
    async fn test_unified_length_function() {
        let length_func = UnifiedLengthFunction::new();
        
        // Test string length
        let context = EvaluationContext::new(FhirPathValue::String("Hello".into()));
        let result = length_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(5)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test collection length
        let test_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let context = EvaluationContext::new(test_collection);
        let result = length_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(length_func.name(), "length");
        assert_eq!(length_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(length_func.metadata().basic.display_name, "Length");
    }
}