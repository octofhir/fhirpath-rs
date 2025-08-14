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

//! Unified max() function implementation

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

/// Unified max() function implementation
/// 
/// Returns the maximum value from a collection of numeric values.
/// Syntax: max()
pub struct UnifiedMaxFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedMaxFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("max", FunctionCategory::MathNumbers)
            .display_name("Maximum")
            .description("Returns the maximum value from all numeric values in the input collection")
            .example("Bundle.entry.resource.ofType(Observation).valueQuantity.value.max()")
            .example("Patient.extension.valueDecimal.max()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Numeric))])
            .output_type(TypePattern::Numeric)
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("max()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["max", "maximum", "largest", "aggregate"])
            .usage_pattern(
                "Numeric aggregation",
                "values.max()",
                "Finding maximum numeric value in collections"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedMaxFunction {
    fn name(&self) -> &str {
        "max"
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
                return self.max_single_value(single_item);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let mut max_value: Option<FhirPathValue> = None;
        
        for item in input_collection.iter() {
            let comparable_value = match item {
                FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => item.clone(),
                FhirPathValue::Quantity(q) => {
                    // For quantities, compare using the numeric value (ignoring units for now)
                    FhirPathValue::Decimal(q.value)
                }
                _ => {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "max() can only be applied to numeric values".to_string(),
                    });
                }
            };
            
            match &max_value {
                None => {
                    max_value = Some(comparable_value);
                }
                Some(current_max) => {
                    if self.is_greater_than(&comparable_value, current_max)? {
                        max_value = Some(comparable_value);
                    }
                }
            }
        }
        
        Ok(max_value.unwrap_or(FhirPathValue::Empty))
    }
}

impl UnifiedMaxFunction {
    /// Handle max of single value
    fn max_single_value(&self, value: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        match value {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) | FhirPathValue::Quantity(_) => {
                Ok(value.clone())
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "max() can only be applied to numeric values".to_string(),
            })
        }
    }
    
    /// Compare two numeric values
    fn is_greater_than(&self, left: &FhirPathValue, right: &FhirPathValue) -> FunctionResult<bool> {
        match (left, right) {
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => Ok(l > r),
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => Ok(l > r),
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                Ok(Decimal::from(*l) > *r)
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                Ok(*l > Decimal::from(*r))
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Cannot compare non-numeric values".to_string(),
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
    async fn test_max_integers() {
        let func = UnifiedMaxFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(3));
    }
    
    #[tokio::test]
    async fn test_max_decimals() {
        let func = UnifiedMaxFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Decimal(Decimal::new(15, 1)), // 1.5
            FhirPathValue::Decimal(Decimal::new(35, 1)), // 3.5
            FhirPathValue::Decimal(Decimal::new(25, 1)), // 2.5
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(35, 1))); // 3.5
    }
    
    #[tokio::test]
    async fn test_max_mixed_types() {
        let func = UnifiedMaxFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Decimal(Decimal::new(35, 1)), // 3.5
            FhirPathValue::Integer(1),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Decimal(Decimal::new(35, 1))); // 3.5
    }
    
    #[tokio::test]
    async fn test_max_empty_collection() {
        let func = UnifiedMaxFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedMaxFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "max");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::MathNumbers);
    }
}