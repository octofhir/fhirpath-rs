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

//! Unified all() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified all() function implementation
pub struct UnifiedAllFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAllFunction {
    pub fn new() -> Self {
        use crate::signature::ParameterInfo;

        let signature = FunctionSignature::new(
            "all",
            vec![ParameterInfo::required("criteria", TypeInfo::Any)],
            TypeInfo::Boolean
        );

        let metadata = MetadataBuilder::collection_function("all")
            .display_name("All")
            .description("Returns true if the given criteria evaluates to true for all elements in the collection")
            .signature(signature)
            .example("Patient.name.all(given.exists())")
            .example("Patient.name.all(period.exists())")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .output_is_collection(true) // Returns collection with single boolean
            .lsp_snippet("all($1)")
            .keywords(vec!["all", "every", "boolean", "collection", "criteria", "logic"])
            .usage_pattern_with_frequency(
                "Check criteria for all elements",
                "Patient.name.all(given.exists())",
                "Validation that all collection items meet a criteria",
                UsageFrequency::Common
            )
            .usage_pattern_with_frequency(
                "Complex validation",
                "Patient.name.all(period.exists())",
                "Ensuring all items have specific properties",
                UsageFrequency::Common
            )
            .related_function("any")
            .related_function("exists")
            .related_function("where")
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAllFunction {
    fn name(&self) -> &str {
        "all"
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
        self.validate_args(args)?;

        let result = match &context.input {
            FhirPathValue::Empty => true, // Empty collection: all() returns true
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    true // Empty collection: all() returns true
                } else {
                    // Check if all items are true (when converted to boolean)
                    items.iter().all(|item| match item {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::Empty => false,
                        _ => true, // Non-empty, non-boolean values are considered truthy
                    })
                }
            }
            value => {
                // Single value: return its boolean conversion
                match value {
                    FhirPathValue::Boolean(b) => *b,
                    FhirPathValue::Empty => false,
                    _ => true,
                }
            }
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(result)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_all_function() {
        let all_func = UnifiedAllFunction::new();

        // Test metadata
        assert_eq!(all_func.name(), "all");
        assert_eq!(all_func.execution_mode(), ExecutionMode::Sync);
        assert!(all_func.is_pure());

        // Test empty collection returns true
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = all_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));

        // Test all true values
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
        ]));
        let result = all_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));

        // Test mixed values (one false)
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(true),
        ]));
        let result = all_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));

        // Test single true value
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = all_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
    }
}
