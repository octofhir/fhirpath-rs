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

//! Unified allFalse() function implementation

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

/// Unified allFalse() function implementation
/// 
/// Returns true if all values in the collection are false or can be converted to false.
/// Syntax: allFalse()
pub struct UnifiedAllFalseFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAllFalseFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("allFalse", FunctionCategory::BooleanLogic)
            .display_name("All False")
            .description("Returns true if all values in the collection are false or can be converted to false")
            .example("Bundle.entry.resource.ofType(Patient).active.allFalse()")
            .example("Patient.name.given.empty().allFalse()")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Boolean))])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Minimal)
            .lsp_snippet("allFalse()")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["allFalse", "all", "false", "boolean", "logical"])
            .usage_pattern(
                "Boolean logic",
                "collection.allFalse()",
                "Checking if all values in collection are false"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAllFalseFunction {
    fn name(&self) -> &str {
        "allFalse"
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
                return Ok(FhirPathValue::Boolean(true)); // Empty collection vacuously has all values false
            }
            single_item => {
                // Treat single item as a collection of one
                return self.all_false_single_value(single_item);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Boolean(true)); // Empty collection vacuously has all values false
        }
        
        // Check if all items are false or can be converted to false
        for item in input_collection.iter() {
            match item {
                FhirPathValue::Boolean(true) => {
                    return Ok(FhirPathValue::Boolean(false)); // Found a true value
                }
                FhirPathValue::Boolean(false) => {
                    // Continue checking other items
                }
                FhirPathValue::Integer(0) => {
                    // 0 converts to false, continue
                }
                FhirPathValue::Integer(_) => {
                    return Ok(FhirPathValue::Boolean(false)); // Non-zero integer converts to true
                }
                FhirPathValue::Decimal(d) => {
                    if !d.is_zero() {
                        return Ok(FhirPathValue::Boolean(false)); // Non-zero decimal converts to true
                    }
                    // 0.0 converts to false, continue
                }
                FhirPathValue::String(s) => {
                    if !s.is_empty() {
                        return Ok(FhirPathValue::Boolean(false)); // Non-empty string converts to true
                    }
                    // Empty string converts to false, continue
                }
                FhirPathValue::Empty => {
                    // Empty converts to false, continue
                }
                FhirPathValue::Collection(c) => {
                    if !c.is_empty() {
                        return Ok(FhirPathValue::Boolean(false)); // Non-empty collection converts to true
                    }
                    // Empty collection converts to false, continue
                }
                _ => {
                    // Other types generally convert to true
                    return Ok(FhirPathValue::Boolean(false));
                }
            }
        }
        
        // All values are false or convert to false
        Ok(FhirPathValue::Boolean(true))
    }
}

impl UnifiedAllFalseFunction {
    /// Handle allFalse operation on a single value
    fn all_false_single_value(&self, value: &FhirPathValue) -> FunctionResult<FhirPathValue> {
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
    async fn test_all_false_with_all_false_values() {
        let func = UnifiedAllFalseFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
            FhirPathValue::Integer(0),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_all_false_with_some_true() {
        let func = UnifiedAllFalseFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(true), // This one is true
            FhirPathValue::Boolean(false),
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_all_false_empty_collection() {
        let func = UnifiedAllFalseFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true)); // Vacuously true
    }
    
    #[tokio::test]
    async fn test_all_false_with_convertible_values() {
        let func = UnifiedAllFalseFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Integer(0), // Converts to false
            FhirPathValue::String("".into()), // Empty string converts to false
        ]);
        let context = create_test_context(collection);
        let result = func.evaluate_sync(&[], &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedAllFalseFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "allFalse");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::BooleanLogic);
    }
}