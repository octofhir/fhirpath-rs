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

//! In operator implementation

use crate::operation::FhirPathOperation;
use crate::operations::comparison::equals::EqualsOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// In operator - collection membership
/// Returns true if the left operand (single item) is in the right collection using equality semantics
#[derive(Debug, Clone)]
pub struct InOperation;

impl InOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("in", OperationType::BinaryOperator {
            precedence: 10,
            associativity: Associativity::Left,
        })
            .description("Collection membership operator - returns true if left operand is in right collection")
            .example("'John' in Patient.name.given")
            .example("2 in {1, 2, 3}")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn evaluate_in(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        // Left operand must be single item
        let left_collection = left.as_collection();
        if left_collection.len() != 1 {
            return Err(FhirPathError::InvalidArguments { message:
                "Left operand of 'in' must be a single item".to_string()
            });
        }

        let search_item = &left_collection[0];
        let search_collection = right.as_collection();

        // If right-hand side is empty, result is false
        if search_collection.is_empty() {
            return Ok(false);
        }

        // Search for the item using equality semantics
        for item in &search_collection {
            if EqualsOperation::compare_equal(search_item, item)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[async_trait]
impl FhirPathOperation for InOperation {
    fn identifier(&self) -> &str {
        "in"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 10,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            InOperation::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len()
            });
        }

        let result = Self::evaluate_in(&args[0], &args[1])?;
        Ok(FhirPathValue::Boolean(result))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len()
            }});
        }

        match Self::evaluate_in(&args[0], &args[1]) {
            Ok(result) => Some(Ok(FhirPathValue::Boolean(result))),
            Err(e) => Some(Err(e)),
        }
    }

    fn supports_sync() -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;

        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_in_with_collection() {
        let op = InOperation::new();
        let ctx = create_test_context();

        // Test 2 in {1, 2, 3} (true)
        let args = vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3)
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test 4 in {1, 2, 3} (false)
        let args = vec![
            FhirPathValue::Integer(4),
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3)
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_in_with_strings() {
        let op = InOperation::new();
        let ctx = create_test_context();

        // Test "John" in {"John", "Jane"} (true)
        let args = vec![
            FhirPathValue::String("John".into()),
            FhirPathValue::Collection(vec![
                FhirPathValue::String("John".into()),
                FhirPathValue::String("Jane".into())
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test "Bob" in {"John", "Jane"} (false)
        let args = vec![
            FhirPathValue::String("Bob".into()),
            FhirPathValue::Collection(vec![
                FhirPathValue::String("John".into()),
                FhirPathValue::String("Jane".into())
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_in_empty_collection() {
        let op = InOperation::new();
        let ctx = create_test_context();

        // Test 1 in {} (false)
        let args = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Empty
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_in_single_item_as_collection() {
        let op = InOperation::new();
        let ctx = create_test_context();

        // Test single item treated as collection
        let args = vec![
            FhirPathValue::Integer(5),
            FhirPathValue::Integer(5)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_in_invalid_left_operand() {
        let op = InOperation::new();
        let ctx = create_test_context();

        // Test with multi-item left operand (should error)
        let args = vec![
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2)
            ]),
            FhirPathValue::Collection(vec![FhirPathValue::Integer(1)])
        ];
        let result = op.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sync_evaluation() {
        let op = InOperation::new();
        let ctx = create_test_context();

        let args = vec![
            FhirPathValue::Integer(2),
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2)
            ])
        ];
        let result = op.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
