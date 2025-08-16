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

//! Comparable function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Quantity};

/// Comparable function - returns true if the input quantity is comparable with the argument quantity
#[derive(Debug, Clone)]
pub struct ComparableFunction;

impl Default for ComparableFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ComparableFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("comparable", OperationType::Function)
            .description(
                "Returns true if the input quantity is comparable with the argument quantity",
            )
            .example("1 'cm'.comparable(1 '[in_i]')")
            .example("1 'cm'.comparable(1 's')")
            .parameter(
                "quantity",
                TypeConstraint::Specific(FhirPathType::Quantity),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn is_comparable(&self, left: &Quantity, right: &Quantity) -> bool {
        left.has_compatible_dimensions(right)
    }
}

#[async_trait]
impl FhirPathOperation for ComparableFunction {
    fn identifier(&self) -> &str {
        "comparable"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ComparableFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let input = &context.input;
        let arg = &args[0];

        // Both input and argument must be quantities
        let left_quantity = match input {
            FhirPathValue::Quantity(q) => q,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Quantity(q)) => q,
                        _ => return Ok(FhirPathValue::Boolean(false)),
                    }
                } else {
                    return Ok(FhirPathValue::Boolean(false));
                }
            }
            _ => return Ok(FhirPathValue::Boolean(false)),
        };

        let right_quantity = match arg {
            FhirPathValue::Quantity(q) => q,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Quantity(q)) => q,
                        _ => return Ok(FhirPathValue::Boolean(false)),
                    }
                } else {
                    return Ok(FhirPathValue::Boolean(false));
                }
            }
            _ => return Ok(FhirPathValue::Boolean(false)),
        };

        let is_comparable = self.is_comparable(left_quantity, right_quantity);
        Ok(FhirPathValue::Boolean(is_comparable))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        let input = &context.input;
        let arg = &args[0];

        // Both input and argument must be quantities
        let left_quantity = match input {
            FhirPathValue::Quantity(q) => q,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Quantity(q)) => q,
                        _ => return Some(Ok(FhirPathValue::Boolean(false))),
                    }
                } else {
                    return Some(Ok(FhirPathValue::Boolean(false)));
                }
            }
            _ => return Some(Ok(FhirPathValue::Boolean(false))),
        };

        let right_quantity = match arg {
            FhirPathValue::Quantity(q) => q,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Quantity(q)) => q,
                        _ => return Some(Ok(FhirPathValue::Boolean(false))),
                    }
                } else {
                    return Some(Ok(FhirPathValue::Boolean(false)));
                }
            }
            _ => return Some(Ok(FhirPathValue::Boolean(false))),
        };

        let is_comparable = self.is_comparable(left_quantity, right_quantity);
        Some(Ok(FhirPathValue::Boolean(is_comparable)))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
