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

//! Logical AND operator implementation

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;

/// Logical AND operator
#[derive(Debug, Clone)]
pub struct AndOperation;

impl AndOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("and", OperationType::BinaryOperator {
            precedence: 2,
            associativity: Associativity::Left,
        })
            .description("Logical AND operator with three-valued logic")
            .example("true and true")
            .example("false and true")
            .example("empty() and true")
            .parameter("left", TypeConstraint::Specific(FhirPathType::Boolean), false)
            .parameter("right", TypeConstraint::Specific(FhirPathType::Boolean), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }


    fn to_boolean(value: &FhirPathValue) -> Result<Option<bool>> {
        match value {
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            // Non-boolean values in logical operations result in empty (not error)
            // This follows FHIRPath specification for type conversion in boolean context
            _ => Ok(None),
        }
    }

    pub fn and_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<Option<bool>> {
        let left_bool = Self::to_boolean(left)?;
        let right_bool = Self::to_boolean(right)?;

        // Three-valued logic for AND
        // false AND anything = false
        // true AND true = true
        // true AND empty = empty
        // empty AND true = empty
        // empty AND empty = empty
        let result = match (left_bool, right_bool) {
            (Some(false), _) | (_, Some(false)) => Some(false),
            (Some(true), Some(true)) => Some(true),
            _ => None, // If either is empty/null and the other is not false
        };

        Ok(result)
    }

}

#[async_trait]
impl FhirPathOperation for AndOperation {
    fn identifier(&self) -> &str {
        "and"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 2,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            AndOperation::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "and".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_logical_operator(&args[0], &args[1], Self::and_values)
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "and".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_logical_operator(&args[0], &args[1], Self::and_values))
    }

    fn supports_sync(&self) -> bool {
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
        use octofhir_fhirpath_model::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_and_operation() {
        let op = AndOperation::new();
        let ctx = create_test_context();

        // Test true and true
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(true)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test true and false
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));

        // Test false and true
        let args = vec![FhirPathValue::Boolean(false), FhirPathValue::Boolean(true)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));

        // Test false and false
        let args = vec![FhirPathValue::Boolean(false), FhirPathValue::Boolean(false)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));

        // Test empty and true (should return empty collection)
        let args = vec![FhirPathValue::Empty, FhirPathValue::Boolean(true)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test false and empty (should return false)
        let args = vec![FhirPathValue::Boolean(false), FhirPathValue::Empty];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));
    }

    #[tokio::test]
    async fn test_and_operation_with_collections() {
        let op = AndOperation::new();
        let ctx = create_test_context();

        // Test single-element collection inputs
        let args = vec![
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])),
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)]))
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));

        // Test empty collection with boolean
        let args = vec![
            FhirPathValue::Collection(Collection::from(vec![])),
            FhirPathValue::Boolean(true)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test multi-element collection (should return empty)
        let args = vec![
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)])),
            FhirPathValue::Boolean(true)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let op = AndOperation::new();
        let ctx = create_test_context();

        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)];
        let sync_result = op.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));
        assert!(op.supports_sync());
    }
}
