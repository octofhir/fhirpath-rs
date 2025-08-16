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

//! Multiplication operation (*) implementation for FHIRPath

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;

/// Multiplication operation (*) for FHIRPath
pub struct MultiplicationOperation;

impl Default for MultiplicationOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiplicationOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "*",
            OperationType::BinaryOperator {
                precedence: 7,
                associativity: Associativity::Left,
            },
        )
        .description("Binary multiplication operation")
        .example("3 * 4")
        .example("2.5 * 1.2")
        .returns(TypeConstraint::OneOf(vec![
            FhirPathType::Integer,
            FhirPathType::Decimal,
        ]))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn multiply_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a
                .checked_mul(*b)
                .map(FhirPathValue::Integer)
                .ok_or_else(|| FhirPathError::ArithmeticError {
                    message: "Integer overflow in multiplication".to_string(),
                }),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a * b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal * b)),
                Err(_) => Err(FhirPathError::ArithmeticError {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Ok(FhirPathValue::Decimal(a * b_decimal)),
                Err(_) => Err(FhirPathError::ArithmeticError {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            // Quantity * Quantity = Quantity with combined units
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                Ok(FhirPathValue::Quantity(std::sync::Arc::new(a.multiply(b))))
            }
            // Scalar * Quantity = Quantity (scalar multiplication)
            (FhirPathValue::Integer(scalar), FhirPathValue::Quantity(q)) => {
                match Decimal::try_from(*scalar) {
                    Ok(scalar_decimal) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                        q.multiply_scalar(scalar_decimal),
                    ))),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal for quantity multiplication"
                            .to_string(),
                    }),
                }
            }
            (FhirPathValue::Decimal(scalar), FhirPathValue::Quantity(q)) => Ok(
                FhirPathValue::Quantity(std::sync::Arc::new(q.multiply_scalar(*scalar))),
            ),
            // Quantity * Scalar = Quantity (scalar multiplication)
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(scalar)) => {
                match Decimal::try_from(*scalar) {
                    Ok(scalar_decimal) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                        q.multiply_scalar(scalar_decimal),
                    ))),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal for quantity multiplication"
                            .to_string(),
                    }),
                }
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(scalar)) => Ok(
                FhirPathValue::Quantity(std::sync::Arc::new(q.multiply_scalar(*scalar))),
            ),
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot multiply {} and {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for MultiplicationOperation {
    fn identifier(&self) -> &str {
        "*"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(MultiplicationOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_arithmetic_operator(
            &args[0],
            &args[1],
            Self::multiply_values,
        )
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_arithmetic_operator(
            &args[0],
            &args[1],
            Self::multiply_values,
        ))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "*".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
