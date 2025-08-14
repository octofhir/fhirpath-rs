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

//! Unified exclude() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified exclude() function implementation
/// 
/// Returns a collection with all items from the first collection except those in the second collection.
/// Syntax: exclude(other)
pub struct UnifiedExcludeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedExcludeFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "exclude",
            vec![ParameterInfo::required("items", TypeInfo::Collection(Box::new(TypeInfo::Any)))],
            TypeInfo::Collection(Box::new(TypeInfo::Any)),
        );
        
        let metadata = MetadataBuilder::new("exclude", FunctionCategory::Collections)
            .display_name("Exclude")
            .description("Returns all items from the first collection that are not in the second collection")
            .example("(1 | 2 | 3).exclude(2 | 4)")
            .example("Patient.name.exclude(Patient.name.where(use = 'official'))")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Quadratic) // O(n*m) comparison
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("exclude(${1:other})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["exclude", "except", "difference", "subtract"])
            .usage_pattern(
                "Set difference",
                "collection.exclude(other)",
                "Removing specific items from collections"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedExcludeFunction {
    fn name(&self) -> &str {
        "exclude"
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
        
        let exclude_collection = self.normalize_to_collection(&args[0]);
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Treat single item as a collection of one
                let temp_collection = vec![single_item.clone()];
                return self.exclude_from_collection(&temp_collection, &exclude_collection);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        let input_slice: Vec<FhirPathValue> = input_collection.iter().cloned().collect();
        self.exclude_from_collection(&input_slice, &exclude_collection)
    }
}

impl UnifiedExcludeFunction {
    /// Normalize a value to a collection for exclusion checking
    fn normalize_to_collection(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.iter().cloned().collect(),
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        }
    }
    
    /// Exclude items from a collection
    fn exclude_from_collection(&self, input_collection: &[FhirPathValue], exclude_collection: &[FhirPathValue]) -> FunctionResult<FhirPathValue> {
        let mut result = Vec::new();
        
        'outer: for item in input_collection {
            // Check if this item should be excluded
            for exclude_item in exclude_collection {
                if self.values_are_equal(item, exclude_item) {
                    continue 'outer; // Skip this item (exclude it)
                }
            }
            // Item not found in exclude collection, include it
            result.push(item.clone());
        }
        
        if result.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(result))
        }
    }
    
    /// Check if two FhirPathValues are equal for exclusion purposes
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
    use rust_decimal::Decimal;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_exclude_basic() {
        let func = UnifiedExcludeFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]);
        let context = create_test_context(collection);
        
        let exclude_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(4),
        ]);
        let args = vec![exclude_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_exclude_strings() {
        let func = UnifiedExcludeFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("apple".into()),
            FhirPathValue::String("banana".into()),
            FhirPathValue::String("cherry".into()),
        ]);
        let context = create_test_context(collection);
        
        let exclude_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("banana".into()),
        ]);
        let args = vec![exclude_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("apple".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("cherry".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_exclude_no_matches() {
        let func = UnifiedExcludeFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection.clone());
        
        let exclude_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(5),
            FhirPathValue::Integer(6),
        ]);
        let args = vec![exclude_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should return original collection since nothing matches
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_exclude_all_items() {
        let func = UnifiedExcludeFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        
        let exclude_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let args = vec![exclude_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_exclude_empty_input() {
        let func = UnifiedExcludeFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let exclude_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
        ]);
        let args = vec![exclude_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedExcludeFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "exclude");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}