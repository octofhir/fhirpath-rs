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

//! Unified sum() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;

/// Unified sum() function implementation
/// 
/// Calculates the sum of numeric values in a collection.
/// Syntax: sum()
pub struct UnifiedSumFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSumFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("sum", FunctionCategory::MathNumbers)
            .display_name("Sum")
            .description("Returns the sum of all numeric values in the input collection")
            .example("Bundle.entry.resource.ofType(Observation).value.sum()")
            .example("Patient.extension.value.sum()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Numeric))])
            .output_type(TypePattern::Numeric)
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("sum()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["sum", "aggregate", "total", "add"])
            .usage_pattern(
                "Numeric aggregation",
                "values.sum()",
                "Summing numeric values in collections"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSumFunction {
    fn name(&self) -> &str {
        "sum"
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
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Treat single item as a collection of one
                return self.sum_single_value(single_item);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let mut has_integer = false;
        let mut has_decimal = false;
        let mut integer_sum = 0i64;
        let mut decimal_sum = Decimal::new(0, 0);
        
        for item in input_collection.iter() {
            match item {
                FhirPathValue::Integer(i) => {
                    has_integer = true;
                    integer_sum += i;
                    decimal_sum += Decimal::from(*i);
                }
                FhirPathValue::Decimal(d) => {
                    has_decimal = true;
                    decimal_sum += d;
                }
                FhirPathValue::Quantity(q) => {
                    // For quantities, sum the numeric value (ignoring units for now)
                    has_decimal = true;
                    decimal_sum += q.value;
                }
                _ => {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "sum() can only be applied to numeric values".to_string(),
                    });
                }
            }
        }
        
        // Return the appropriate type based on what we encountered
        if has_decimal {
            Ok(FhirPathValue::Decimal(decimal_sum))
        } else if has_integer {
            Ok(FhirPathValue::Integer(integer_sum))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

impl UnifiedSumFunction {
    /// Handle sum of single value
    fn sum_single_value(&self, value: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        match value {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) | FhirPathValue::Quantity(_) => {
                Ok(value.clone())
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "sum() can only be applied to numeric values".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_sum_integers() {
        let func = UnifiedSumFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(6));
    }
    
    #[tokio::test]
    async fn test_sum_decimals() {
        let func = UnifiedSumFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Decimal(Decimal::new(15, 1)), // 1.5
            FhirPathValue::Decimal(Decimal::new(25, 1)), // 2.5
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(40, 1))); // 4.0
    }
    
    #[tokio::test]
    async fn test_sum_mixed_types() {
        let func = UnifiedSumFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Decimal(Decimal::new(25, 1)), // 2.5
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(35, 1))); // 3.5
    }
    
    #[tokio::test]
    async fn test_sum_empty_collection() {
        let func = UnifiedSumFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_sum_non_numeric() {
        let func = UnifiedSumFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context);
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedSumFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "sum");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::MathNumbers);
    }
}