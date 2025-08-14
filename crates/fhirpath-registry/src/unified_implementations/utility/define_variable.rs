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

//! Unified defineVariable() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;

/// Unified defineVariable() function implementation
/// 
/// Defines a variable in the evaluation context.
/// Syntax: defineVariable(name, value)
pub struct UnifiedDefineVariableFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedDefineVariableFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "defineVariable",
            vec![
                ParameterInfo::required("name", TypeInfo::String),
                ParameterInfo::required("value", TypeInfo::Any),
            ],
            TypeInfo::Any,
        );
        
        let metadata = MetadataBuilder::new("defineVariable", FunctionCategory::Utilities)
            .display_name("Define Variable")
            .description("Defines a variable with a given name and value in the evaluation context")
            .example("defineVariable('patientName', Patient.name.first())")
            .example("defineVariable('isActive', Patient.active)")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike, TypePattern::Any])
            .output_type(TypePattern::Any)
            .supports_collections(true)
            .pure(false) // Has side effects (modifies context)
            .complexity(PerformanceComplexity::Constant)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("defineVariable(${1:'variableName'}, ${2:value})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["variable", "define", "context", "assign"])
            .usage_pattern(
                "Variable definition",
                "defineVariable('patientName', Patient.name.first())",
                "Creating reusable values in expressions"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedDefineVariableFunction {
    fn name(&self) -> &str {
        "defineVariable"
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
        // Validate arguments - exactly 2 required
        if args.len() != 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 2,
                max: Some(2),
                actual: args.len(),
            });
        }
        
        let variable_name = match &args[0] {
            FhirPathValue::String(name) => name.to_string(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.get(0) {
                    Some(FhirPathValue::String(name)) => name.to_string(),
                    _ => return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "First argument must be a string (variable name)".to_string(),
                    }),
                }
            }
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "First argument must be a string (variable name)".to_string(),
            }),
        };
        
        let value = args[1].clone();
        
        // Validate that we're not trying to override system variables
        if self.is_system_variable(&variable_name) {
            return Ok(FhirPathValue::Empty);
        }

        // Since we can't modify the evaluation context directly, and the engine needs
        // to handle variable storage, we'll implement this as a pseudo-operation
        // that signals variable definition by returning a special collection structure
        
        // For now, we'll store the variable definition as metadata in a special format
        // that the evaluation engine should recognize and handle properly
        
        // The FHIRPath specification requires that defineVariable() returns the input
        // unchanged, but makes the variable available to subsequent operations.
        // Since this needs engine-level support, we return the input for now.
        
        // TODO: This needs proper engine integration to:
        // 1. Intercept defineVariable calls at the expression evaluation level
        // 2. Store variables in a mutable context that persists across chained operations
        // 3. Make variables accessible via %variableName syntax
        
        // For immediate compatibility, we return the input context
        Ok(context.input.clone())
    }
}

impl UnifiedDefineVariableFunction {
    /// Check if a variable name is a reserved system variable
    fn is_system_variable(&self, name: &str) -> bool {
        matches!(name, 
            "context" | "resource" | "rootResource" | 
            "sct" | "loinc" | "ucum" | 
            "this" | "index" | "total"
        )
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
    async fn test_define_variable_function() {
        let func = UnifiedDefineVariableFunction::new();
        let context = create_test_context(FhirPathValue::String("context".into()));
        
        // Test valid variable definition
        let args = vec![
            FhirPathValue::String("testVar".into()),
            FhirPathValue::String("testValue".into()),
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        // Should return the input context unchanged for now
        assert_eq!(result, FhirPathValue::String("context".into()));
    }
}