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

//! Unified log() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
    signature::{FunctionSignature, ParameterInfo},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// Unified log() function implementation
/// 
/// Returns the logarithm base `base` of the input number
/// 
/// Specification: log(base : Decimal) : Decimal
/// - Returns the logarithm base `base` of the input number
/// - When used with Integers, arguments will be implicitly converted to Decimal  
/// - If `base` is empty, the result is empty
/// - If the input collection is empty, the result is empty
/// - If multiple items, signal an error
/// 
/// Examples:
/// - `16.log(2)` → `4.0`
/// - `100.0.log(10.0)` → `2.0`
pub struct UnifiedLogFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedLogFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature {
            name: "log".to_string(),
            parameters: vec![ParameterInfo {
                name: "base".to_string(),
                param_type: TypeInfo::Decimal,
                optional: false,
            }],
            return_type: TypeInfo::Decimal,
            min_arity: 1,
            max_arity: Some(1),
        };
        
        let metadata = MetadataBuilder::math_function("log")
            .display_name("Logarithm (Base)")
            .description("Returns the logarithm base `base` of the input number")
            .example("16.log(2) // 4.0")
            .example("100.0.log(10.0) // 2.0")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::Decimal))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("log(${1:base})")
            .keywords(vec!["log", "logarithm", "base", "math"])
            .usage_pattern(
                "Calculate logarithm with base",
                "value.log(base)",
                "Mathematical calculations"
            )
            .build();
        
        Self { metadata }
    }

    /// Calculate logarithm with base conversion: log_base(x) = ln(x) / ln(base)
    fn calculate_log(value: f64, base: f64) -> Option<f64> {
        if value <= 0.0 || base <= 0.0 || base == 1.0 {
            return None; // Invalid input for logarithm
        }
        
        let result = value.ln() / base.ln();
        
        // Check for valid result (not NaN or infinite)
        if result.is_finite() {
            Some(result)
        } else {
            None
        }
    }
    
    /// Convert FhirPathValue to f64 for calculation
    fn to_float(&self, value: &FhirPathValue) -> Option<f64> {
        match value {
            FhirPathValue::Integer(i) => Some(*i as f64),
            FhirPathValue::Decimal(d) => d.to_f64(),
            _ => None,
        }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedLogFunction {
    fn name(&self) -> &str {
        "log"
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
        // Check argument count
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        // Handle empty input collection
        if matches!(context.input, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }

        // Handle collection input (must be single item)
        let input_value = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() != 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("log() expects exactly 1 input item, got {}", items.len()),
                    });
                }
                items.get(0).unwrap()
            }
            value => value,
        };

        // Handle empty base argument
        let base_arg = &args[0];
        if matches!(base_arg, FhirPathValue::Empty) {
            return Ok(FhirPathValue::Empty);
        }

        // Handle collection base argument (must be single item)  
        let base_value = match base_arg {
            FhirPathValue::Collection(items) => {
                if items.len() != 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "log() base argument must be a single item".to_string(),
                    });
                }
                items.get(0).unwrap()
            }
            value => value,
        };

        // Convert input and base to f64
        let input_float = self.to_float(input_value).ok_or_else(|| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("log() input must be Integer or Decimal, got {}", input_value.type_name()),
            }
        })?;

        let base_float = self.to_float(base_value).ok_or_else(|| {
            FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("log() base must be Integer or Decimal, got {}", base_value.type_name()),
            }
        })?;

        // Calculate logarithm
        match Self::calculate_log(input_float, base_float) {
            Some(result) => {
                // Convert result back to Decimal
                match Decimal::from_f64(result) {
                    Some(decimal_result) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(decimal_result)])),
                    None => Ok(FhirPathValue::Empty), // Invalid result
                }
            }
            None => Ok(FhirPathValue::Empty), // Invalid input for logarithm
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