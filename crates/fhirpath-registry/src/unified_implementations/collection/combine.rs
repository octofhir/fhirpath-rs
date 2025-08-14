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

//! Unified combine() function implementation

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
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;

/// Unified combine() function implementation
/// 
/// Merges the input collection with another collection, returning the union of both collections.
/// Unlike union operator (|), this function preserves duplicates.
/// Syntax: combine(other)
pub struct UnifiedCombineFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedCombineFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "combine",
            vec![ParameterInfo::required("other", TypeInfo::Collection(Box::new(TypeInfo::Any)))],
            TypeInfo::Collection(Box::new(TypeInfo::Any)),
        );
        
        let metadata = MetadataBuilder::new("combine", FunctionCategory::Collections)
            .display_name("Combine")
            .description("Combines two collections into one, preserving all items including duplicates")
            .example("(1 | 2).combine(2 | 3)")
            .example("Patient.name.combine(Patient.alias)")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("combine(${1:other})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["combine", "merge", "union", "concat"])
            .usage_pattern(
                "Collection concatenation",
                "collection.combine(other)",
                "Merging multiple collections with duplicates preserved"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedCombineFunction {
    fn name(&self) -> &str {
        "combine"
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
        
        self.combine_collections(input_collection, other_collection)
    }
}

impl UnifiedCombineFunction {
    /// Normalize a value to a collection for combination
    fn normalize_to_collection(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.iter().cloned().collect(),
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        }
    }
    
    /// Combine two collections, preserving all items including duplicates
    fn combine_collections(&self, mut input_collection: Vec<FhirPathValue>, other_collection: Vec<FhirPathValue>) -> FunctionResult<FhirPathValue> {
        // Simply append all items from the other collection
        input_collection.extend(other_collection);
        
        if input_collection.is_empty() {
            Ok(FhirPathValue::Empty)
        } else {
            Ok(FhirPathValue::collection(input_collection))
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
    async fn test_combine_basic() {
        let func = UnifiedCombineFunction::new();
        
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
    async fn test_combine_with_duplicates() {
        let func = UnifiedCombineFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should preserve duplicates (2 appears twice)
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::Integer(2))); // Duplicate preserved
            assert_eq!(items.iter().nth(3), Some(&FhirPathValue::Integer(3)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_combine_strings() {
        let func = UnifiedCombineFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("apple".into()),
            FhirPathValue::String("banana".into()),
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("cherry".into()),
            FhirPathValue::String("date".into()),
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 4);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("apple".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("banana".into())));
            assert_eq!(items.iter().nth(2), Some(&FhirPathValue::String("cherry".into())));
            assert_eq!(items.iter().nth(3), Some(&FhirPathValue::String("date".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_combine_empty_input() {
        let func = UnifiedCombineFunction::new();
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
    async fn test_combine_empty_other() {
        let func = UnifiedCombineFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection.clone());
        
        let args = vec![FhirPathValue::Empty];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Should return the original collection
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_combine_both_empty() {
        let func = UnifiedCombineFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        let args = vec![FhirPathValue::Empty];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_combine_single_values() {
        let func = UnifiedCombineFunction::new();
        let context = create_test_context(FhirPathValue::Integer(1));
        
        let args = vec![FhirPathValue::Integer(2)];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::Integer(1)));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::Integer(2)));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedCombineFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "combine");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}