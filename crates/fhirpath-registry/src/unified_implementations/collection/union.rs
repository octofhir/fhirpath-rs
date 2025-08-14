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

//! Unified union() function implementation

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

/// Unified union() function implementation
/// 
/// Merges two collections into one, eliminating duplicates.
/// This is distinct from combine() which preserves duplicates.
/// Syntax: union(other)
pub struct UnifiedUnionFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedUnionFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("union", FunctionCategory::Collections)
            .display_name("Union")
            .description("Merges two collections into one, eliminating duplicates (distinct from combine)")
            .example("(1 | 2).union(2 | 3)")
            .example("Patient.name.union(Patient.alias)")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Quadratic) // O(n^2) for deduplication
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("union(${1:other})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["union", "merge", "combine", "distinct"])
            .usage_pattern(
                "Set operations",
                "collection.union(other)",
                "Merging collections without duplicates"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedUnionFunction {
    fn name(&self) -> &str {
        "union"
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
            FhirPathValue::Collection(items) => items.iter().cloned().collect::<Vec<_>>(),
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        };
        
        self.union_collections(input_collection, other_collection)
    }
}

impl UnifiedUnionFunction {
    /// Normalize a value to a collection for union operation
    fn normalize_to_collection(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.iter().cloned().collect(),
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        }
    }
    
    /// Combine two collections, eliminating duplicates
    fn union_collections(&self, mut input_collection: Vec<FhirPathValue>, other_collection: Vec<FhirPathValue>) -> FunctionResult<FhirPathValue> {
        // Add items from other collection, but only if they don't already exist
        for other_item in other_collection {
            let already_exists = input_collection.iter().any(|existing_item| {
                self.values_are_equal(existing_item, &other_item)
            });
            
            if !already_exists {
                input_collection.push(other_item);
            }
        }
        
        // Return distinct result
        if input_collection.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(input_collection))
        }
    }
    
    /// Check if two FhirPathValues are equal for union purposes
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
    async fn test_union_basic() {
        let func = UnifiedUnionFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::Integer(3)));
            assert_eq!(items.iter().nth(3), Some(&FhirPathValue::Integer(4)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_union_with_duplicates() {
        let func = UnifiedUnionFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(2), // Duplicate - should be eliminated
            FhirPathValue::Integer(3),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should eliminate duplicates (unlike combine)
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3); // Not 4 because duplicate 2 is eliminated
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_union_strings() {
        let func = UnifiedUnionFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("apple".into()),
            FhirPathValue::String("banana".into()),
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("banana".into()), // Duplicate
            FhirPathValue::String("cherry".into()),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 3); // Duplicate "banana" eliminated
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("apple".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("banana".into())));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::String("cherry".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_union_empty_input() {
        let func = UnifiedUnionFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should return the other collection
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_union_both_empty() {
        let func = UnifiedUnionFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let args = vec![FhirPathValue::Empty];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedUnionFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "union");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}