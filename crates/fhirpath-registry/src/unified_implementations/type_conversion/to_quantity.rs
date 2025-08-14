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

//! Unified toQuantity() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified toQuantity() function implementation
///
/// Converts values to their quantity representation
pub struct UnifiedToQuantityFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedToQuantityFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("toQuantity", crate::function::FunctionCategory::TypeConversion)
            .display_name("To Quantity")
            .description("Converts a value to its quantity representation")
            .example("42.toQuantity()")
            .example("'5 kg'.toQuantity()")
            .example("'1 day'.toQuantity()")
            .output_type(TypePattern::Exact(TypeInfo::Quantity))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("toQuantity()")
            .keywords(vec!["toQuantity", "quantity", "convert", "cast", "unit"])
            .usage_pattern(
                "Convert value to quantity",
                "value.toQuantity()",
                "Type conversion for numeric values with units"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedToQuantityFunction {
    fn name(&self) -> &str {
        "toQuantity"
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

        // Follow FHIRPath conversion rules using the model's helper
        if matches!(&context.input, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }

        if let Some(q) = context.input.to_quantity_value() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(q)]));
        }

        Ok(FhirPathValue::Empty) // Per FHIRPath spec: return empty for unsupported types
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
    async fn test_unified_to_quantity_function() {
        let to_quantity_func = UnifiedToQuantityFunction::new();

        // Test integer to quantity
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = to_quantity_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Quantity(q)) = items.get(0) {
                assert_eq!(q.value, Decimal::from(42));
                assert_eq!(q.unit, None);
            } else {
                panic!("Expected quantity result");
            }
        } else {
            panic!("Expected collection result");
        }

        // Test decimal to quantity
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str_exact("3.14").unwrap()));
        let result = to_quantity_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Quantity(q)) = items.get(0) {
                assert_eq!(q.value, Decimal::from_str_exact("3.14").unwrap());
                assert_eq!(q.unit, None);
            } else {
                panic!("Expected quantity result");
            }
        } else {
            panic!("Expected collection result");
        }

        // Test string with unit to quantity
        let context = EvaluationContext::new(FhirPathValue::String("5 kg".into()));
        let result = to_quantity_func.evaluate_sync(&[], &context).unwrap();

        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            if let Some(FhirPathValue::Quantity(q)) = items.get(0) {
                assert_eq!(q.value, Decimal::from(5));
                assert_eq!(q.unit, Some("kg".to_string()));
            } else {
                panic!("Expected quantity result");
            }
        } else {
            panic!("Expected collection result");
        }

        // Test metadata
        assert_eq!(to_quantity_func.name(), "toQuantity");
        assert_eq!(to_quantity_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(to_quantity_func.metadata().basic.display_name, "To Quantity");
    }

    #[tokio::test]
    async fn test_ucum_time_units() {
        let to_quantity_func = UnifiedToQuantityFunction::new();

        // Test quoted UCUM time units
        let test_cases = vec![
            ("1 'wk'", "week"),
            ("1 'mo'", "month"),
            ("1 'a'", "year"),
            ("1 'd'", "day"),
            ("1 day", "day"),
        ];

        for (input, expected_unit) in test_cases {
            let context = EvaluationContext::new(FhirPathValue::String(input.into()));
            let result = to_quantity_func.evaluate_sync(&[], &context).unwrap();

            if let FhirPathValue::Collection(items) = result {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Quantity(q)) = items.get(0) {
                    assert_eq!(q.value, Decimal::from(1));
                    assert_eq!(q.unit, Some(expected_unit.to_string()));
                } else {
                    panic!("Expected quantity result for {}", input);
                }
            } else {
                panic!("Expected collection result for {}", input);
            }
        }
    }
}
