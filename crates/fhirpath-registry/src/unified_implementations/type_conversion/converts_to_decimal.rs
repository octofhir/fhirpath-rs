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

//! Unified convertsToDecimal() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::Decimal;

/// Unified convertsToDecimal() function implementation
///
/// Checks if a value can be converted to a decimal
pub struct UnifiedConvertsToDecimalFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedConvertsToDecimalFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("convertsToDecimal", crate::function::FunctionCategory::TypeConversion)
            .display_name("Converts To Decimal")
            .description("Returns true if the value can be converted to a decimal")
            .example("'3.14'.convertsToDecimal()")
            .example("'not-a-number'.convertsToDecimal()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("convertsToDecimal()")
            .keywords(vec!["convertsToDecimal", "decimal", "convert", "check", "validation"])
            .usage_pattern(
                "Check if convertible to decimal",
                "value.convertsToDecimal()",
                "Type validation before conversion"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedConvertsToDecimalFunction {
    fn name(&self) -> &str {
        "convertsToDecimal"
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

        let can_convert = match &context.input {
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::String(s) => s.trim().parse::<Decimal>().is_ok(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => false,
        };

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
    async fn test_unified_convertsToDecimal_function() {
        let converts_func = UnifiedConvertsToDecimalFunction::new();

        // Test valid decimal string
        let context = EvaluationContext::new(FhirPathValue::String("3.14".into()));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test valid integer string
        let context = EvaluationContext::new(FhirPathValue::String("42".into()));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test integer value
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test boolean value
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test invalid decimal string
        let context = EvaluationContext::new(FhirPathValue::String("not-a-number".into()));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }

        // Test metadata
        assert_eq!(converts_func.name(), "convertsToDecimal");
        assert_eq!(converts_func.execution_mode(), ExecutionMode::Sync);
    }
}
