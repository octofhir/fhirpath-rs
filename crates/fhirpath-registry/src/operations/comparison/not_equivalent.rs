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

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use crate::operations::comparison::equivalent::EquivalentOperation;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Not equivalent operator (!~)
/// Returns true if the collections are not equivalent
#[derive(Debug, Clone)]
pub struct NotEquivalentOperation;

impl Default for NotEquivalentOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl NotEquivalentOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "!~",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
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
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(NotEquivalentOperation::create_metadata);
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

        let result = Self::are_not_equivalent(&args[0], &args[1])?;
        Ok(FhirPathValue::Boolean(result))
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
