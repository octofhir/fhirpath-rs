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

//! Not equivalent operator (!~) implementation

use crate::operation::FhirPathOperation;
use crate::operations::comparison::equivalent::EquivalentOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Not equivalent operator (!~)
/// Returns true if the collections are not equivalent
#[derive(Debug, Clone)]
pub struct NotEquivalentOperation;

impl NotEquivalentOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("!~", OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        })
            .description("Not equivalent comparison operator - negation of equivalent (~)")
            .example("'Hello' !~ 'world'")
            .example("'Hello' !~ 'WORLD'")
            .example("{1, 2} !~ {3, 4}")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    pub fn are_not_equivalent(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        // Simply negate the equivalent operation
        let equivalent = EquivalentOperation::are_equivalent(left, right)?;
        Ok(!equivalent)
    }
}

#[async_trait]
impl FhirPathOperation for NotEquivalentOperation {
    fn identifier(&self) -> &str {
        "!~"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            NotEquivalentOperation::create_metadata()
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

        let result = Self::are_not_equivalent(&args[0], &args[1])?;
        Ok(FhirPathValue::Boolean(result))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 2, 
                actual: args.len() 
            }));
        }

        match Self::are_not_equivalent(&args[0], &args[1]) {
            Ok(result) => Some(Ok(FhirPathValue::Boolean(result))),
            Err(e) => Some(Err(e)),
        }
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
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_not_equivalent_strings() {
        let op = NotEquivalentOperation::new();
        let ctx = create_test_context();

        // Test "Hello" !~ "world" (true)
        let args = vec![
            FhirPathValue::String("Hello".into()),
            FhirPathValue::String("world".into())
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test "Hello" !~ "hello" (false - they are equivalent)
        let args = vec![
            FhirPathValue::String("Hello".into()),
            FhirPathValue::String("hello".into())
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_not_equivalent_integers() {
        let op = NotEquivalentOperation::new();
        let ctx = create_test_context();

        // Test 5 !~ 3 (true)
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(3)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test 5 !~ 5 (false)
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(5)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_not_equivalent_collections() {
        let op = NotEquivalentOperation::new();
        let ctx = create_test_context();

        // Test {1, 2} !~ {3, 4} (true)
        let args = vec![
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2)
            ]),
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(4)
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test {1, 2} !~ {2, 1} (false - they are equivalent)
        let args = vec![
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2)
            ]),
            FhirPathValue::Collection(vec![
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(1)
            ])
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[tokio::test]
    async fn test_not_equivalent_different_types() {
        let op = NotEquivalentOperation::new();
        let ctx = create_test_context();

        // Test "5" !~ 5 (true - different types)
        let args = vec![
            FhirPathValue::String("5".into()),
            FhirPathValue::Integer(5)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }

    #[tokio::test]
    async fn test_sync_evaluation() {
        let op = NotEquivalentOperation::new();
        let ctx = create_test_context();

        let args = vec![
            FhirPathValue::String("Hello".into()),
            FhirPathValue::String("world".into())
        ];
        let result = op.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}