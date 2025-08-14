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

//! Unified convertsToBoolean() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// Unified convertsToBoolean() function implementation
///
/// Checks if a value can be converted to a boolean
pub struct UnifiedConvertsToBooleanFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedConvertsToBooleanFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("convertsToBoolean", crate::function::FunctionCategory::TypeConversion)
            .display_name("Converts To Boolean")
            .description("Returns true if the value can be converted to a boolean")
            .example("'true'.convertsToBoolean()")
            .example("1.convertsToBoolean()")
            .example("'invalid'.convertsToBoolean()")
            .output_type(TypePattern::Exact(TypeInfo::Boolean))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("convertsToBoolean()")
            .keywords(vec!["convertsToBoolean", "boolean", "convert", "check", "validation"])
            .usage_pattern(
                "Check if convertible to boolean",
                "value.convertsToBoolean()",
                "Type validation before conversion"
            )
            .build();

        Self { metadata }
    }

    /// Check if a string can be converted to boolean according to FHIRPath spec
    fn can_string_convert_to_boolean(s: &str) -> bool {
        let lower = s.trim().to_lowercase();
        matches!(lower.as_str(),
            "true" | "t" | "yes" | "y" | "1" | "1.0" |
            "false" | "f" | "no" | "n" | "0" | "0.0"
        )
    }

    /// Check if a decimal can be converted to boolean (only 0.0 and 1.0)
    fn can_decimal_convert_to_boolean(d: &Decimal) -> bool {
        *d == Decimal::ZERO || *d == Decimal::ONE
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedConvertsToBooleanFunction {
    fn name(&self) -> &str {
        "convertsToBoolean"
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
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::Integer(i) => *i == 0 || *i == 1,
            FhirPathValue::Decimal(d) => Self::can_decimal_convert_to_boolean(d),
            FhirPathValue::String(s) => Self::can_string_convert_to_boolean(s),
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
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_unified_convertsToBoolean_function() {
        let converts_func = UnifiedConvertsToBooleanFunction::new();

        // Test boolean literal
        let context = EvaluationContext::new(FhirPathValue::Boolean(true));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test valid integer (1)
        let context = EvaluationContext::new(FhirPathValue::Integer(1));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test invalid integer (2)
        let context = EvaluationContext::new(FhirPathValue::Integer(2));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }

        // Test valid decimal (1.0)
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::ONE));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test valid string ("true")
        let context = EvaluationContext::new(FhirPathValue::String("true".into()));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test case insensitive string ("False")
        let context = EvaluationContext::new(FhirPathValue::String("False".into()));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(true)));
        } else {
            panic!("Expected collection result");
        }

        // Test invalid string
        let context = EvaluationContext::new(FhirPathValue::String("invalid".into()));
        let result = converts_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Boolean(false)));
        } else {
            panic!("Expected collection result");
        }

        // Test empty value
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = converts_func.evaluate_sync(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test metadata
        assert_eq!(converts_func.name(), "convertsToBoolean");
        assert_eq!(converts_func.execution_mode(), ExecutionMode::Sync);
    }
}
