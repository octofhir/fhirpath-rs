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

//! Unified subsetOf() function implementation

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

/// Unified subsetOf() function implementation
/// 
/// Tests whether the input collection is a subset of another collection.
/// Returns true if all items in the input are members of the other collection.
/// Syntax: subsetOf(other)
pub struct UnifiedSubsetOfFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSubsetOfFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("subsetOf", FunctionCategory::Collections)
            .display_name("Subset Of")
            .description("Tests whether the input collection is a subset of another collection")
            .example("(1 | 2).subsetOf(1 | 2 | 3)")
            .example("Patient.name.subsetOf(AllNames)")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Quadratic) // O(n*m) comparison
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("subsetOf(${1:other})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["subsetOf", "subset", "contains", "member"])
            .usage_pattern(
                "Set operations",
                "collection.subsetOf(other)",
                "Testing subset relationships between collections"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSubsetOfFunction {
    fn name(&self) -> &str {
        "subsetOf"
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
        // Validate arguments - exactly 1 required (other collection)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let other_collection = self.normalize_to_collection(&args[0]);
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => {
                items.iter().cloned().collect::<Vec<_>>()
            }
            FhirPathValue::Empty => {
                // Empty set is subset of any set
                return Ok(FhirPathValue::Boolean(true));
            }
            single_item => vec![single_item.clone()],
        };
        
        // Check if all items in input collection exist in other collection
        let is_subset = input_collection.iter().all(|input_item| {
            other_collection.iter().any(|other_item| {
                self.values_are_equal(input_item, other_item)
            })
        });
        
        Ok(FhirPathValue::Boolean(is_subset))
    }
}

impl UnifiedSubsetOfFunction {
    /// Normalize a value to a collection for subset checking
    fn normalize_to_collection(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.iter().cloned().collect(),
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        }
    }
    
    /// Check if two FhirPathValues are equal for subset purposes
    fn values_are_equal(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        match (left, right) {
            (FhirPathValue::String(l), FhirPathValue::String(r)) => l == r,
            (FhirPathValue::Integer(l), FhirPathValue::Integer(r)) => l == r,
            (FhirPathValue::Decimal(l), FhirPathValue::Decimal(r)) => l == r,
            (FhirPathValue::Boolean(l), FhirPathValue::Boolean(r)) => l == r,
            (FhirPathValue::Date(l), FhirPathValue::Date(r)) => l == r,
            (FhirPathValue::DateTime(l), FhirPathValue::DateTime(r)) => l == r,
            (FhirPathValue::Time(l), FhirPathValue::Time(r)) => l == r,
            (FhirPathValue::Empty, FhirPathValue::Empty) => true,
            // Cross-type numeric comparisons
            (FhirPathValue::Integer(l), FhirPathValue::Decimal(r)) => {
                use rust_decimal::Decimal;
                Decimal::from(*l) == *r
            }
            (FhirPathValue::Decimal(l), FhirPathValue::Integer(r)) => {
                use rust_decimal::Decimal;
                *l == Decimal::from(*r)
            }
            // For more complex types, use direct comparison
            _ => left == right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_subset_basic() {
        let func = UnifiedSubsetOfFunction::new();
        
        let subset = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(subset);
        
        let superset = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let args = vec![superset];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_not_subset() {
        let func = UnifiedSubsetOfFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(4), // Not in other collection
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_equal_sets() {
        let func = UnifiedSubsetOfFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection.clone());
        
        let args = vec![collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_empty_subset() {
        let func = UnifiedSubsetOfFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Empty set is subset of any set
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedSubsetOfFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "subsetOf");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}