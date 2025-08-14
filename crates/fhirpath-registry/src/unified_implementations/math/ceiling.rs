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

//! Unified ceiling() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::ToPrimitive;

/// Unified ceiling() function implementation
/// 
/// Returns the ceiling (smallest integer greater than or equal to) of a decimal
pub struct UnifiedCeilingFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedCeilingFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::math_function("ceiling")
            .display_name("Ceiling")
            .description("Returns the smallest integer greater than or equal to the input")
            .example("(3.2).ceiling()")
            .example("(-1.5).ceiling()")
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .lsp_snippet("ceiling()")
            .keywords(vec!["ceiling", "ceil", "round", "math"])
            .usage_pattern(
                "Round up to nearest integer",
                "value.ceiling()",
                "Mathematical rounding operations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedCeilingFunction {
    fn name(&self) -> &str {
        "ceiling"
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
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Decimal(d) => d.ceil().to_i64().unwrap_or(0),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Expected numeric value, got {}", context.input.type_name()),
            }),
        };
        
        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(result)]))
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
    async fn test_unified_ceiling_function() {
        let ceiling_func = UnifiedCeilingFunction::new();
        
        // Test positive decimal
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(3.2).unwrap()));
        let result = ceiling_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(4)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test negative decimal
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(-1.5).unwrap()));
        let result = ceiling_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(-1)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(ceiling_func.name(), "ceiling");
        assert_eq!(ceiling_func.execution_mode(), ExecutionMode::Sync);
    }
}