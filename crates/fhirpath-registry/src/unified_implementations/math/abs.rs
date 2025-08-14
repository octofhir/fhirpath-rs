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

//! Unified abs() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified abs() function implementation
/// 
/// Returns the absolute value of numeric values
pub struct UnifiedAbsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAbsFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::math_function("abs")
            .display_name("Absolute Value")
            .description("Returns the absolute value of a number or quantity")
            .example("(-5).abs()")
            .example("(-5.5 'mg').abs()")
            .example("Patient.age.abs()")
            .output_type(TypePattern::Numeric)
            .lsp_snippet("abs()")
            .keywords(vec!["abs", "absolute", "math", "number"])
            .usage_pattern(
                "Get absolute value",
                "value.abs()",
                "Mathematical calculations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAbsFunction {
    fn name(&self) -> &str {
        "abs"
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
        
        let result = match &context.input {
            FhirPathValue::Integer(i) => FhirPathValue::Integer(i.abs()),
            FhirPathValue::Decimal(d) => FhirPathValue::Decimal(d.abs()),
            FhirPathValue::Quantity(q) => {
                // Create a new quantity with absolute value but same unit
                let abs_value = q.value.abs();
                let abs_quantity = octofhir_fhirpath_model::Quantity::new(abs_value, q.unit.clone());
                FhirPathValue::Quantity(std::sync::Arc::new(abs_quantity))
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected numeric value or quantity, got {}", context.input.type_name()),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![result]))
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
    use rust_decimal::prelude::FromPrimitive;

    #[tokio::test]
    async fn test_unified_abs_function() {
        let abs_func = UnifiedAbsFunction::new();
        
        // Test positive integer
        let context = EvaluationContext::new(FhirPathValue::Integer(5));
        let result = abs_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(5)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test negative integer
        let context = EvaluationContext::new(FhirPathValue::Integer(-5));
        let result = abs_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(5)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test negative decimal
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(-3.14).unwrap()));
        let result = abs_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Decimal(Decimal::from_f64(3.14).unwrap())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(abs_func.name(), "abs");
        assert_eq!(abs_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(abs_func.metadata().basic.display_name, "Absolute Value");
    }
}