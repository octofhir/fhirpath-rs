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

//! Not equals operator (!=) implementation

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::comparison::equals::EqualsOperation;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Not equals operator (!=)
#[derive(Debug, Clone)]
pub struct NotEqualsOperation;

impl NotEqualsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "!=",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
        .description("Not equals comparison operator")
        .example("1 != 2")
        .example("'hello' != 'world'")
        .returns(TypeConstraint::Specific(FhirPathType::Boolean))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn compare_not_equal(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        let equals_result = EqualsOperation::compare_equal(left, right)?;
        Ok(!equals_result)
    }
}

#[async_trait]
impl FhirPathOperation for NotEqualsOperation {
    fn identifier(&self) -> &str {
        "!="
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| NotEqualsOperation::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_binary_operator(&args[0], &args[1], Self::compare_not_equal)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_binary_operator(
            &args[0],
            &args[1],
            Self::compare_not_equal,
        ))
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
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        use std::sync::Arc;

        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_not_equals_operation() {
        let op = NotEqualsOperation::new();
        let ctx = create_test_context();

        // Test integer inequality
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(3)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test equality (should return false)
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(5)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
}
