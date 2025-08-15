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

//! AllFalse function implementation for FHIRPath

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;

/// AllFalse function: returns true if all boolean items in the collection are false
pub struct AllFalseFunction;

impl AllFalseFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("allFalse", OperationType::Function)
            .description("Returns true if all boolean items in the collection are false. Returns true for an empty collection. Non-boolean items are ignored.")
            .example("Patient.active.allFalse()")
            .example("Bundle.entry.resource.active.allFalse()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for AllFalseFunction {
    fn identifier(&self) -> &str {
        "allFalse"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            AllFalseFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync path first for performance
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        // Fallback to async evaluation (though allFalse is always sync)
        self.evaluate_all_false(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_all_false(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AllFalseFunction {
    fn evaluate_all_false(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments { 
                message: "allFalse() takes no arguments".to_string()
            });
        }

        match &context.input {
            FhirPathValue::Collection(items) => {
                // Filter to only boolean items
                let boolean_items: Vec<bool> = items
                    .iter()
                    .filter_map(|item| match item {
                        FhirPathValue::Boolean(b) => Some(*b),
                        _ => None,
                    })
                    .collect();

                // If no boolean items, return true (empty collection logic)
                if boolean_items.is_empty() {
                    Ok(FhirPathValue::Boolean(true))
                } else {
                    // All must be false
                    Ok(FhirPathValue::Boolean(boolean_items.iter().all(|&b| !b)))
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => {
                // Non-boolean single item - return true (no boolean values to check)
                Ok(FhirPathValue::Boolean(true))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let registry = Arc::new(crate::FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_all_false_empty_collection() {
        let all_false_fn = AllFalseFunction::new();
        let empty_collection = FhirPathValue::collection(vec![]);
        let context = create_test_context(empty_collection);
        
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_all_false_single_boolean() {
        let all_false_fn = AllFalseFunction::new();
        
        // Single true boolean
        let single_true = FhirPathValue::Boolean(true);
        let context = create_test_context(single_true);
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
        
        // Single false boolean
        let single_false = FhirPathValue::Boolean(false);
        let context = create_test_context(single_false);
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_all_false_multiple_booleans() {
        let all_false_fn = AllFalseFunction::new();
        
        // All false booleans
        let all_false_collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
        ]);
        let context = create_test_context(all_false_collection);
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Mixed booleans (some true)
        let mixed_collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(true),
            FhirPathValue::Boolean(false),
        ]);
        let context = create_test_context(mixed_collection);
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_all_false_mixed_types() {
        let all_false_fn = AllFalseFunction::new();
        
        // Collection with boolean and non-boolean items
        let mixed_type_collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::String("text".into()),
            FhirPathValue::Boolean(false),
            FhirPathValue::Integer(42),
        ]);
        let context = create_test_context(mixed_type_collection);
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Only boolean values are considered
    }

    #[tokio::test]
    async fn test_all_false_no_booleans() {
        let all_false_fn = AllFalseFunction::new();
        
        // Collection with no boolean items
        let no_boolean_collection = FhirPathValue::collection(vec![
            FhirPathValue::String("text".into()),
            FhirPathValue::Integer(42),
        ]);
        let context = create_test_context(no_boolean_collection);
        let result = all_false_fn.evaluate(&[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // No boolean values to evaluate
    }

    #[tokio::test]
    async fn test_all_false_with_arguments_error() {
        let all_false_fn = AllFalseFunction::new();
        let collection = FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]);
        let context = create_test_context(collection);
        
        let result = all_false_fn.evaluate(&[FhirPathValue::Boolean(true)], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let all_false_fn = AllFalseFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Boolean(false),
        ]);
        let context = create_test_context(collection);

        let sync_result = all_false_fn.try_evaluate_sync(&[], &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Boolean(true));
        assert!(all_false_fn.supports_sync());
    }

    #[test]
    fn test_metadata() {
        let all_false_fn = AllFalseFunction::new();
        let metadata = all_false_fn.metadata();

        assert_eq!(metadata.basic.name, "allFalse");
        assert_eq!(metadata.basic.operation_type, OperationType::Function);
        assert!(!metadata.basic.description.is_empty());
        assert!(!metadata.basic.examples.is_empty());
    }
}