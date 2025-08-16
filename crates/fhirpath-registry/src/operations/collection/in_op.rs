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

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use crate::operations::comparison::equals::EqualsOperation;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// In operator - collection membership
/// Returns true if the left operand (single item) is in the right collection using equality semantics
#[derive(Debug, Clone)]
pub struct InOperation;

impl Default for InOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl InOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "in",
            OperationType::BinaryOperator {
                precedence: 10,
                associativity: Associativity::Left,
            },
        )
        .description(
            "Collection membership operator - returns true if left operand is in right collection",
        )
        .example("'John' in Patient.name.given")
        .example("2 in {1, 2, 3}")
        .returns(TypeConstraint::Specific(FhirPathType::Boolean))
        .performance(PerformanceComplexity::Linear, true)
        .build()
    }

    fn evaluate_in(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        // Convert left operand to collection to handle both single items and collections
        let left_collection = left.clone().to_collection();

        // If left is empty, result is false (nothing can be "in" anything)
        if left_collection.is_empty() {
            return Ok(false);
        }

        let search_collection = right.clone().to_collection();

        // If right-hand side is empty, result is false
        if search_collection.is_empty() {
            return Ok(false);
        }

        // For each item on the left, check if it's in the right collection
        for left_item in left_collection.iter() {
            for right_item in search_collection.iter() {
                if EqualsOperation::compare_equal(left_item, right_item)? {
                    return Ok(true);
                }
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
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(InOperation::create_metadata);
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

        let result = Self::evaluate_in(&args[0], &args[1])?;
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

        match Self::evaluate_in(&args[0], &args[1]) {
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
