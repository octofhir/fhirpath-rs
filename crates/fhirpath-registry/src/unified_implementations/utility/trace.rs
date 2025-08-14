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

//! Unified trace() function implementation

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

/// Unified trace() function implementation
/// 
/// Debug function that prints the input value and returns it unchanged.
/// Per FHIRPath spec: trace(name : String [, projection: Expression]) : collection
/// Syntax: trace(name, projection?)
pub struct UnifiedTraceFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTraceFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 required string parameter and 1 optional expression parameter
        let signature = FunctionSignature::new(
            "trace",
            vec![
                ParameterInfo::required("name", TypeInfo::String),
                ParameterInfo::optional("projection", TypeInfo::Any),
            ],
            TypeInfo::Any,
        );

        let metadata = MetadataBuilder::new("trace", FunctionCategory::Utilities)
            .display_name("Trace")
            .description("Outputs the input value for debugging and returns it unchanged. Optional projection parameter controls what is logged")
            .example("Patient.name.trace('patient name')")
            .example("name.trace('test', given)")
            .example("collection.trace('debug').count()")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::Any)
            .supports_collections(true)
            .pure(false) // Has side effects (prints output)
            .complexity(PerformanceComplexity::Constant)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("trace(${1:'label'})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["debug", "print", "log", "trace"])
            .usage_pattern(
                "Debug output",
                "Patient.name.trace('patient name')",
                "Debugging expression evaluation"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTraceFunction {
    fn name(&self) -> &str {
        "trace"
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
        // Validate arguments - 1 or 2 arguments allowed per FHIRPath spec
        // trace(name : String [, projection: Expression])
        if args.is_empty() || args.len() > 2 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(2),
                actual: args.len(),
            });
        }
        
        // First argument is always the name/label (required)
        let label = match &args[0] {
            FhirPathValue::String(s) => s.to_string(),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "First argument to trace() must be a string label".to_string(),
            }),
        };
        
        // If there's a second argument, it's a projection expression result
        // Note: In a full implementation, we would need to evaluate the expression
        // For now, we'll use the second argument directly if provided
        let trace_value = if args.len() == 2 {
            &args[1]
        } else {
            &context.input
        };
        
        // Print the trace output
        eprintln!("TRACE [{}]: {:?}", label, trace_value);
        
        // Return the input value unchanged
        Ok(context.input.clone())
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
    async fn test_trace_function() {
        let func = UnifiedTraceFunction::new();
        let input = FhirPathValue::String("test_value".into());
        let context = create_test_context(input.clone());
        
        // Test trace with label only (minimum required)
        let args = vec![FhirPathValue::String("test_label".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, input);
        
        // Test trace with label and projection
        let projection_value = FhirPathValue::String("projection_result".into());
        let args = vec![
            FhirPathValue::String("test_label".into()), 
            projection_value.clone()
        ];
        let result = func.evaluate_sync(&args, &context).unwrap();
        assert_eq!(result, input); // Should always return the input unchanged
    }
}