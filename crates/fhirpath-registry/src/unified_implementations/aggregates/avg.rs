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

//! Unified avg() function implementation

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

/// Unified avg() function implementation
/// 
/// Calculates the average of numeric values in a collection.
/// Syntax: avg()
pub struct UnifiedAvgFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAvgFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("avg", FunctionCategory::MathNumbers)
            .display_name("Average")
            .description("Returns the average of all numeric values in the input collection")
            .example("Bundle.entry.resource.ofType(Observation).valueQuantity.value.avg()")
            .example("Patient.extension.valueDecimal.avg()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Numeric))])
            .output_type(TypePattern::Numeric)
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("avg()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["avg", "average", "mean", "aggregate"])
            .usage_pattern(
                "Numeric aggregation",
                "values.avg()",
                "Calculating average of numeric values in collections"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAvgFunction {
    fn name(&self) -> &str {
        "avg"
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
                return self.avg_single_value(single_item);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let mut sum = Decimal::new(0, 0);
        let mut count = 0;
        
        for item in input_collection.iter() {
            match item {
                FhirPathValue::Integer(i) => {
                    sum += Decimal::from(*i);
                    count += 1;
                }
                FhirPathValue::Decimal(d) => {
                    sum += d;
                    count += 1;
                }
                FhirPathValue::Quantity(q) => {
                    // For quantities, use the numeric value (ignoring units for now)
                    sum += q.value;
                    count += 1;
                }
                _ => {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "avg() can only be applied to numeric values".to_string(),
                    });
                }
            }
        }
        
        if count == 0 {
            Ok(FhirPathValue::Empty)
        } else {
            let avg = sum / Decimal::from(count);
            Ok(FhirPathValue::Decimal(avg))
        }
    }
}

impl UnifiedAvgFunction {
    /// Handle average of single value
    fn avg_single_value(&self, value: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        match value {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) | FhirPathValue::Quantity(_) => {
                Ok(value.clone())
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "avg() can only be applied to numeric values".to_string(),
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
    async fn test_avg_integers() {
        let func = UnifiedAvgFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(2, 0))); // 2.0
    }
    
    #[tokio::test]
    async fn test_avg_decimals() {
        let func = UnifiedAvgFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Decimal(Decimal::new(10, 1)), // 1.0
            FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(20, 1))); // 2.0
    }
    
    #[tokio::test]
    async fn test_avg_mixed_types() {
        let func = UnifiedAvgFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Decimal(Decimal::new(30, 1)), // 3.0
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(20, 1))); // 2.0
    }
    
    #[tokio::test]
    async fn test_avg_empty_collection() {
        let func = UnifiedAvgFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedAvgFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "avg");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::MathNumbers);
    }
}