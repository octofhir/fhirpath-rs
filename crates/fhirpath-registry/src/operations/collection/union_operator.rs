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

//! Union operator (|) implementation for FHIRPath

use crate::metadata::{MetadataBuilder, OperationType};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use rustc_hash::FxHashSet;

/// Union operator (|): returns the union of two collections
pub struct UnionOperator;

impl UnionOperator {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> crate::metadata::OperationMetadata {
        MetadataBuilder::new("|", OperationType::BinaryOperator { 
            associativity: crate::metadata::Associativity::Left,
            precedence: 5  // FHIRPath union precedence
        })
        .description("Returns the union of the left and right collections, removing duplicates")
        .example("Patient.name.given | Patient.name.family")
        .example("Bundle.entry | Bundle.contained")
        .build()
    }
}

#[async_trait]
impl FhirPathOperation for UnionOperator {
    fn identifier(&self) -> &str {
        "|"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator { 
            associativity: crate::metadata::Associativity::Left,
            precedence: 5
        }
    }

    fn metadata(&self) -> &crate::metadata::OperationMetadata {
        static METADATA: once_cell::sync::Lazy<crate::metadata::OperationMetadata> =
            once_cell::sync::Lazy::new(|| UnionOperator::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Delegate to sync implementation
        self.evaluate_union(args, context)
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArguments {
                message: "| operator requires exactly two operands".to_string(),
            });
        }
        Ok(())
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_union(args, context))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl UnionOperator {
    fn evaluate_union(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArguments {
                message: "| operator requires exactly two operands".to_string(),
            });
        }

        let left = &args[0];
        let right = &args[1];

        // Convert both operands to collections
        let left_items = match left {
            FhirPathValue::Collection(items) => items.clone(),
            other => Collection::from(vec![other.clone()]),
        };

        let right_items = match right {
            FhirPathValue::Collection(items) => items.clone(),
            other => Collection::from(vec![other.clone()]),
        };

        // Combine and deduplicate
        let mut seen = FxHashSet::default();
        let mut result = Vec::new();

        // Add left items
        for item in left_items.iter() {
            let key = format!("{:?}", item); // Simple hash key for deduplication
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        // Add right items
        for item in right_items.iter() {
            let key = format!("{:?}", item);
            if seen.insert(key) {
                result.push(item.clone());
            }
        }

        Ok(FhirPathValue::Collection(Collection::from(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_union_empty_collections() {
        let union_op = UnionOperator::new();
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let context = EvaluationContext::new(
            FhirPathValue::Collection(vec![]),
            registry,
            model_provider,
        );

        let empty_collection = FhirPathValue::Collection(vec![]);
        let result = union_op.evaluate(&[empty_collection.clone(), empty_collection], &context).await.unwrap();
        
        match result {
            FhirPathValue::Collection(items) => assert!(items.is_empty()),
            _ => panic!("Expected Collection"),
        }
    }

    #[tokio::test]
    async fn test_union_disjoint_collections() {
        let union_op = UnionOperator::new();
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let context = EvaluationContext::new(
            FhirPathValue::Collection(vec![]),
            registry,
            model_provider,
        );

        let left_collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);
        let right_collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]);
        
        let result = union_op.evaluate(&[left_collection, right_collection], &context).await.unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 4);
                // Should contain 1, 2, 3, 4 in that order
            }
            _ => panic!("Expected Collection"),
        }
    }

    #[tokio::test]
    async fn test_union_overlapping_collections() {
        let union_op = UnionOperator::new();
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let context = EvaluationContext::new(
            FhirPathValue::Collection(vec![]),
            registry,
            model_provider,
        );

        let left_collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        let right_collection = FhirPathValue::Collection(vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(4),
        ]);
        
        let result = union_op.evaluate(&[left_collection, right_collection], &context).await.unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 4); // Should be deduplicated: 1, 2, 3, 4
            }
            _ => panic!("Expected Collection"),
        }
    }

    #[tokio::test]
    async fn test_union_single_items() {
        let union_op = UnionOperator::new();
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let context = EvaluationContext::new(
            FhirPathValue::Collection(vec![]),
            registry,
            model_provider,
        );

        let item1 = FhirPathValue::String("hello".into());
        let item2 = FhirPathValue::String("world".into());
        
        let result = union_op.evaluate(&[item1, item2], &context).await.unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
            }
            _ => panic!("Expected Collection"),
        }
    }

    #[tokio::test]
    async fn test_union_sync_evaluation() {
        let union_op = UnionOperator::new();
        let model_provider = Arc::new(MockModelProvider::new());
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let context = EvaluationContext::new(
            FhirPathValue::Collection(vec![]),
            registry,
            model_provider,
        );

        let left_collection = FhirPathValue::Collection(vec![FhirPathValue::Integer(1)]);
        let right_collection = FhirPathValue::Collection(vec![FhirPathValue::Integer(2)]);
        
        let sync_result = union_op.try_evaluate_sync(&[left_collection, right_collection], &context).unwrap().unwrap();
        
        assert!(union_op.supports_sync());
        
        match sync_result {
            FhirPathValue::Collection(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected Collection"),
        }
    }

    #[tokio::test]
    async fn test_union_metadata() {
        let union_op = UnionOperator::new();
        let metadata = union_op.metadata();
        
        assert_eq!(metadata.basic.name, "|");
        assert!(matches!(metadata.basic.operation_type, OperationType::BinaryOperator { .. }));
    }
}