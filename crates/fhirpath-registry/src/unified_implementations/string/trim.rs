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

//! Unified trim() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified trim() function implementation
///
/// Removes leading and trailing whitespace from string values
pub struct UnifiedTrimFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTrimFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::string_function("trim")
            .display_name("Trim")
            .description("Removes leading and trailing whitespace from a string")
            .example("'  hello world  '.trim()")
            .example("name.family.trim()")
            .output_type(TypePattern::Exact(TypeInfo::String))
            .lsp_snippet("trim()")
            .keywords(vec!["trim", "whitespace", "clean", "string"])
            .usage_pattern(
                "Remove whitespace",
                "name.trim()",
                "String cleaning and normalization"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTrimFunction {
    fn name(&self) -> &str {
        "trim"
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

        match &context.input {
            FhirPathValue::String(s) => {
                let trimmed = s.trim();
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(trimmed.into())]))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected String, got {}", context.input.type_name()),
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
    async fn test_unified_trim_function() {
        let trim_func = UnifiedTrimFunction::new();

        // Test basic trimming
        let context = EvaluationContext::new(FhirPathValue::String("  hello world  ".into()));
        let result = trim_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("hello world".into())));
        } else {
            panic!("Expected collection result");
        }

        // Test no trimming needed
        let context = EvaluationContext::new(FhirPathValue::String("hello world".into()));
        let result = trim_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("hello world".into())));
        } else {
            panic!("Expected collection result");
        }

        // Test whitespace-only string becomes empty
        let context = EvaluationContext::new(FhirPathValue::String("   ".into()));
        let result = trim_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("".into())));
        } else {
            panic!("Expected collection result");
        }

        // Test empty input
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = trim_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test metadata
        assert_eq!(trim_func.name(), "trim");
        assert_eq!(trim_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(trim_func.metadata().basic.display_name, "Trim");
    }
}
