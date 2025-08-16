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

use crate::operations::{EvaluationContext, binary_operator_utils};
use crate::{
    FhirPathOperation,
    metadata::{
        Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
        PerformanceComplexity, TypeConstraint,
    },
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Logical AND operator
#[derive(Debug, Clone)]
pub struct AndOperation;

impl Default for AndOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl AndOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "and",
            OperationType::BinaryOperator {
                precedence: 2,
                associativity: Associativity::Left,
            },
        )
        .description("Logical AND operator with three-valued logic")
        .example("true and true")
        .example("false and true")
        .example("empty() and true")
        .parameter(
            "left",
            TypeConstraint::Specific(FhirPathType::Boolean),
            false,
        )
        .parameter(
            "right",
            TypeConstraint::Specific(FhirPathType::Boolean),
            false,
        )
        .returns(TypeConstraint::Specific(FhirPathType::Boolean))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    fn to_boolean(value: &FhirPathValue) -> Result<Option<bool>> {
        match value {
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(None)
                } else if items.len() == 1 {
                    Self::to_boolean(items.first().unwrap())
                } else {
                    // Multi-element collections in logical operations result in empty (not error)
                    Ok(None)
                }
            }
            // Per FHIRPath spec: "IF the collection contains a single node AND the expected input type is Boolean THEN The collection evaluates to true"
            // Non-boolean single values in boolean context evaluate to true
            _ => Ok(Some(true)),
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
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AndOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "and".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_logical_operator(&args[0], &args[1], Self::and_values)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "and".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_logical_operator(
            &args[0],
            &args[1],
            Self::and_values,
        ))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
