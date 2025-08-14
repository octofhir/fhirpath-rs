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

//! Unified supersetOf() function implementation

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

/// Unified supersetOf() function implementation
/// 
/// Tests whether the input collection is a superset of another collection.
/// Returns true if all items in the other collection are members of the input.
/// Syntax: supersetOf(other)
pub struct UnifiedSupersetOfFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedSupersetOfFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("supersetOf", FunctionCategory::Collections)
            .display_name("Superset Of")
            .description("Tests whether the input collection is a superset of another collection")
            .example("(1 | 2 | 3).supersetOf(1 | 2)")
            .example("AllNames.supersetOf(Patient.name)")
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::Any))])
            .output_type(TypePattern::Boolean)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Quadratic) // O(n*m) comparison
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("supersetOf(${1:other})")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["supersetOf", "superset", "includes", "contains"])
            .usage_pattern(
                "Set operations",
                "collection.supersetOf(other)",
                "Testing superset relationships between collections"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedSupersetOfFunction {
    fn name(&self) -> &str {
        "supersetOf"
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
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        };
        
        // Empty other collection means any set is a superset
        if other_collection.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }
        
        // Check if all items in other collection exist in input collection
        let is_superset = other_collection.iter().all(|other_item| {
            input_collection.iter().any(|input_item| {
                self.values_are_equal(input_item, other_item)
            })
        });
        
        Ok(FhirPathValue::Boolean(is_superset))
    }
}

impl UnifiedSupersetOfFunction {
    /// Normalize a value to a collection for superset checking
    fn normalize_to_collection(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::Collection(items) => items.iter().cloned().collect(),
            FhirPathValue::Empty => vec![],
            single_item => vec![single_item.clone()],
        }
    }
    
    /// Check if two FhirPathValues are equal for superset purposes
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
    async fn test_superset_basic() {
        let func = UnifiedSupersetOfFunction::new();
        
        let superset = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let context = create_test_context(superset);
        
        let subset = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let args = vec![subset];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_not_superset() {
        let func = UnifiedSupersetOfFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        
        let other_collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(4), // Not in input collection
        ]);
        let args = vec![other_collection];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
    
    #[tokio::test]
    async fn test_equal_sets() {
        let func = UnifiedSupersetOfFunction::new();
        
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
    async fn test_superset_of_empty() {
        let func = UnifiedSupersetOfFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let context = create_test_context(collection);
        
        let args = vec![FhirPathValue::Empty];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        // Any set is superset of empty set
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedSupersetOfFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "supersetOf");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
    }
}