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

//! Unified anyFalse() function implementation

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

/// Unified anyFalse() function implementation
/// 
/// Returns true if any value in the collection is false or can be converted to false.
/// Syntax: anyFalse()
pub struct UnifiedAnyFalseFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAnyFalseFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("anyFalse", FunctionCategory::BooleanLogic)
            .display_name("Any False")
            .description("Returns true if any value in the collection is false or can be converted to false")
            .example("Bundle.entry.resource.ofType(Patient).active.anyFalse()")
            .example("Patient.name.given.empty().anyFalse()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Boolean))])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("anyFalse()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["anyFalse", "any", "false", "boolean", "logical"])
            .usage_pattern(
                "Boolean logic",
                "collection.anyFalse()",
                "Checking if any value in collection is false"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAnyFalseFunction {
    fn name(&self) -> &str {
        "anyFalse"
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
                return Ok(FhirPathValue::Boolean(false)); // Empty collection has no false values
            }
            single_item => {
                // Treat single item as a collection of one
                return self.any_false_single_value(single_item);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Boolean(false));
        }
        
        // Check if any item is false or can be converted to false
        for item in input_collection.iter() {
            match item {
                FhirPathValue::Boolean(false) => {
                    return Ok(FhirPathValue::Boolean(true));
                }
                FhirPathValue::Boolean(true) => {
                    // Continue checking other items
                }
                FhirPathValue::Integer(0) => {
                    return Ok(FhirPathValue::Boolean(true)); // 0 converts to false
                }
                FhirPathValue::Integer(_) => {
                    // Non-zero integers convert to true, continue
                }
                FhirPathValue::Decimal(d) => {
                    if d.is_zero() {
                        return Ok(FhirPathValue::Boolean(true)); // 0.0 converts to false
                    }
                    // Non-zero decimals convert to true, continue
                }
                FhirPathValue::String(s) => {
                    if s.is_empty() {
                        return Ok(FhirPathValue::Boolean(true)); // Empty string converts to false
                    }
                    // Non-empty strings convert to true, continue
                }
                FhirPathValue::Empty => {
                    return Ok(FhirPathValue::Boolean(true)); // Empty converts to false
                }
                FhirPathValue::Collection(c) => {
                    if c.is_empty() {
                        return Ok(FhirPathValue::Boolean(true)); // Empty collection converts to false
                    }
                    // Non-empty collections convert to true, continue
                }
                _ => {
                    // Other types generally convert to true, continue
                }
            }
        }
        
        // No false values found
        Ok(FhirPathValue::Boolean(false))
    }
}

impl UnifiedAnyFalseFunction {
    /// Handle anyFalse operation on a single value
    fn any_false_single_value(&self, value: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        let is_false = match value {
            FhirPathValue::Boolean(false) => true,
            FhirPathValue::Boolean(true) => false,
            FhirPathValue::Integer(0) => true,
            FhirPathValue::Integer(_) => false,
            FhirPathValue::Decimal(d) => d.is_zero(),
            FhirPathValue::String(s) => s.is_empty(),
            FhirPathValue::Empty => true,
            FhirPathValue::Collection(c) => c.is_empty(),
            _ => false, // Other types generally convert to true
        };
        
        Ok(FhirPathValue::Boolean(is_false))
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
    async fn test_any_false_with_false_values() {
        let func = UnifiedAnyFalseFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_any_false_all_true() {
        let func = UnifiedAnyFalseFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_any_false_with_convertible_false() {
        let func = UnifiedAnyFalseFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Integer(0), // Converts to false
            FhirPathValue::String("hello".into()),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_any_false_empty_collection() {
        let func = UnifiedAnyFalseFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedAnyFalseFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "anyFalse");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::BooleanLogic);
    }
}