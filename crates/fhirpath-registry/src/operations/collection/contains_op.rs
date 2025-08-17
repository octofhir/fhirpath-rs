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

//! Contains operator implementation

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

/// Contains operator - collection containership
/// Returns true if the left collection contains the right operand using equality semantics
#[derive(Debug, Clone)]
pub struct ContainsOperation;

impl Default for ContainsOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("contains", OperationType::BinaryOperator {
            precedence: 10,
            associativity: Associativity::Left,
        })
            .description("Collection containership operator - returns true if left collection contains right operand")
            .example("Patient.name.given contains 'John'")
            .example("{1, 2, 3} contains 2")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn evaluate_contains(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        // Right operand must be single item
        let right_collection = right.clone().to_collection();
        if right_collection.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "Right operand of 'contains' must be a single item".to_string(),
            });
        }

        let search_item = right_collection.get(0).unwrap();
        let search_collection = left.clone().to_collection();

        // If left-hand side is empty, result is false
        if search_collection.is_empty() {
            return Ok(false);
        }

        // Search for the item using equality semantics
        for item in search_collection.iter() {
            if EqualsOperation::compare_equal(item, search_item)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[async_trait]
impl FhirPathOperation for ContainsOperation {
    fn identifier(&self) -> &str {
        "contains"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 10,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ContainsOperation::create_metadata);
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

        let result = Self::evaluate_contains(&args[0], &args[1])?;
        Ok(FhirPathValue::Boolean(result))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                message: format!(
                    "contains operator requires exactly 2 arguments, got {}",
                    args.len()
                ),
            }));
        }

        match Self::evaluate_contains(&args[0], &args[1]) {
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
