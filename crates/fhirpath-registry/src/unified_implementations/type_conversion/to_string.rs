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

//! Unified toString() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified toString() function implementation
///
/// Converts values to their string representation
pub struct UnifiedToStringFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToStringFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toString", crate::function::FunctionCategory::TypeConversion)
            .display_name("To String")
            .description("Converts a value to its string representation")
            .example("42.toString()")
            .example("true.toString()")
            .output_type(TypePattern::Exact(TypeInfo::String))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("toString()")
            .keywords(vec!["toString", "string", "convert", "cast"])
            .usage_pattern(
                "Convert value to string",
                "value.toString()",
                "Type conversion and string manipulation"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToStringFunction {
    fn name(&self) -> &str {
        "toString"
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

        // Handle different input types
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if items.len() == 1 {
                    // Single item collection - convert the item
                    if let Some(item) = items.get(0) {
                        if let Some(s) = item.to_string_value() {
                            Ok(FhirPathValue::collection(vec![FhirPathValue::String(s.into())]))
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else {
                    // Multiple items - per FHIRPath spec, functions on collections with multiple items return empty
                    Ok(FhirPathValue::Empty)
                }
            },
            single_value => {
                // Try to convert the single value
                if let Some(s) = single_value.to_string_value() {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::String(s.into())]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
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
    async fn test_unified_toString_function() {
        let to_string_func = UnifiedToStringFunction::new();

        // Test integer to string
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = to_string_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("42".into())));
        } else {
            panic!("Expected collection result");
        }

        // Test boolean to string
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = to_string_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("true".into())));
        } else {
            panic!("Expected collection result");
        }

        // Test metadata
        assert_eq!(to_string_func.name(), "toString");
        assert_eq!(to_string_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(to_string_func.metadata().basic.display_name, "To String");
    }
}
