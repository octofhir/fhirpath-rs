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

//! Unified convertsToString() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified convertsToString() function implementation
///
/// Checks if a value can be converted to a string
pub struct UnifiedConvertsToStringFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedConvertsToStringFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("convertsToString", crate::function::FunctionCategory::TypeConversion)
            .display_name("Converts To String")
            .description("Returns true if the value can be converted to a string")
            .example("42.convertsToString()")
            .example("Patient.convertsToString()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("convertsToString()")
            .keywords(vec!["convertsToString", "string", "convert", "check", "validation"])
            .usage_pattern(
                "Check if convertible to string",
                "value.convertsToString()",
                "Type validation before conversion"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedConvertsToStringFunction {
    fn name(&self) -> &str {
        "convertsToString"
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

        if matches!(&context.input, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }

        let can_convert = context.input.to_string_value().is_some();
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
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
    async fn test_unified_convertsToString_function() {
        let converts_func = UnifiedConvertsToStringFunction::new();

        // Test convertible value
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test metadata
        assert_eq!(converts_func.name(), "convertsToString");
        assert_eq!(converts_func.execution_mode(), ExecutionMode::Sync);
    }
}
