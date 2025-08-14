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

//! Unified truncate() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::ToPrimitive;

/// Unified truncate() function implementation
/// 
/// Returns the integer part of a decimal (truncates towards zero)
pub struct UnifiedTruncateFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedTruncateFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::math_function("truncate")
            .display_name("Truncate")
            .description("Returns the integer part of a decimal by truncating towards zero")
            .example("(3.8).truncate()")
            .example("(-1.9).truncate()")
            .output_type(TypePattern::Exact(TypeInfo::Integer))
            .lsp_snippet("truncate()")
            .keywords(vec!["truncate", "trunc", "integer", "math"])
            .usage_pattern(
                "Truncate decimal to integer",
                "value.truncate()",
                "Mathematical truncation operations"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedTruncateFunction {
    fn name(&self) -> &str {
        "truncate"
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
            FhirPathValue::Decimal(d) => d.trunc().to_i64().unwrap_or(0),
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
    async fn test_unified_truncate_function() {
        let truncate_func = UnifiedTruncateFunction::new();
        
        // Test positive decimal truncation
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(3.8).unwrap()));
        let result = truncate_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test negative decimal truncation
        let context = EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_f64(-1.9).unwrap()));
        let result = truncate_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::Integer(-1)));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(truncate_func.name(), "truncate");
        assert_eq!(truncate_func.execution_mode(), ExecutionMode::Sync);
    }
}