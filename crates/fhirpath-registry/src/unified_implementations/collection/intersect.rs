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

//! Unified intersect() function implementation

use crate::enhanced_metadata::{EnhancedFunctionMetadata, TypePattern, UsageFrequency};
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::signature::FunctionSignature;
use crate::unified_function::{ExecutionMode, UnifiedFhirPathFunction};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Unified intersect() function implementation
pub struct UnifiedIntersectFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedIntersectFunction {
    pub fn new() -> Self {
        use crate::signature::ParameterInfo;
        
        let signature = FunctionSignature::new(
            "intersect", 
            vec![ParameterInfo::required("other", TypeInfo::Any)], 
            TypeInfo::Collection(Box::new(TypeInfo::Any))
        );
        
        let metadata = MetadataBuilder::collection_function("intersect")
            .display_name("Intersect")
            .description("Returns the intersection of two collections")
            .signature(signature)
            .example("Patient.name.intersect($otherNames)")
            .example("Bundle.entry.intersect($filteredEntries)")
            .output_type(TypePattern::Any)
            .output_is_collection(true)
            .lsp_snippet("intersect(${1:other})")
            .keywords(vec!["intersect", "intersection", "common", "collection", "set"])
            .usage_pattern_with_frequency(
                "Find common elements",
                "Patient.name.intersect($otherNames)",
                "Finding shared elements between collections",
                UsageFrequency::Moderate
            )
            .usage_pattern_with_frequency(
                "Set operations",
                "Bundle.entry.intersect($subset)",
                "Implementing set intersection logic",
                UsageFrequency::Rare
            )
            .related_function("distinct")
            .related_function("union")
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedIntersectFunction {
    fn name(&self) -> &str {
        "intersect"
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
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: "intersect".to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let other_collection = &args[0];
        
        let result = match (&context.input, other_collection) {
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                FhirPathValue::collection(vec![])
            }
            (FhirPathValue::Collection(items1), FhirPathValue::Collection(items2)) => {
                let mut intersection = Vec::new();
                let mut seen = std::collections::HashSet::new();
                
                // Create a set of items from the second collection for fast lookup
                let items2_set: std::collections::HashSet<String> = items2.iter()
                    .map(|item| format!("{:?}", item))
                    .collect();
                
                // Find intersection
                for item in items1.iter() {
                    let item_key = format!("{:?}", item);
                    if items2_set.contains(&item_key) && seen.insert(item_key) {
                        intersection.push(item.clone());
                    }
                }
                
                FhirPathValue::collection(intersection)
            }
            (FhirPathValue::Collection(items), value) | (value, FhirPathValue::Collection(items)) => {
                // One is a collection, one is a single value
                let value_key = format!("{:?}", value);
                for item in items.iter() {
                    if format!("{:?}", item) == value_key {
                        return Ok(FhirPathValue::collection(vec![item.clone()]));
                    }
                }
                FhirPathValue::collection(vec![])
            }
            (value1, value2) => {
                // Both are single values
                if format!("{:?}", value1) == format!("{:?}", value2) {
                    FhirPathValue::collection(vec![value1.clone()])
                } else {
                    FhirPathValue::collection(vec![])
                }
            }
        };
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unified_intersect_function() {
        let intersect_func = UnifiedIntersectFunction::new();
        
        // Test metadata
        assert_eq!(intersect_func.name(), "intersect");
        assert_eq!(intersect_func.execution_mode(), ExecutionMode::Sync);
        assert!(intersect_func.is_pure());
        
        // Test intersect of two collections
        let context = EvaluationContext::new(FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]));
        
        let args = vec![FhirPathValue::collection(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ])];
        
        let result = intersect_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            // Should contain 2 and 3 (intersection)
            assert!(items.iter().any(|item| matches!(item, FhirPathValue::Integer(2))));
            assert!(items.iter().any(|item| matches!(item, FhirPathValue::Integer(3))));
        } else {
            panic!("Expected collection result");
        }
        
        // Test with empty collection
        let args = vec![FhirPathValue::collection(vec![])];
        let result = intersect_func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 0);
        } else {
            panic!("Expected empty collection result");
        }
    }
}