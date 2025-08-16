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

//! Integer division operation (div) implementation for FHIRPath

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// Integer division operation (div) - returns integer result
pub struct DivOperation;

impl Default for DivOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl DivOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "div",
            OperationType::BinaryOperator {
                precedence: 7,
                associativity: Associativity::Left,
            },
        )
        .description(
            "Integer division operation - returns integer result by truncating towards zero",
        )
        .example("10 div 3")
        .example("7 div 2")
        .example("-10 div 3")
        .returns(TypeConstraint::Specific(FhirPathType::Integer))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn div_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        // Special handling for division by zero - return empty instead of error for div operation
        let check_zero = |val: &FhirPathValue| -> bool {
            match val {
                FhirPathValue::Integer(i) => *i == 0,
                FhirPathValue::Decimal(d) => *d == Decimal::ZERO,
                _ => false,
            }
        };

        if check_zero(right) {
            // Return empty for division by zero
            return Err(FhirPathError::ArithmeticError {
                message: "Division by zero".to_string(),
            });
        }

        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                // Integer division - truncate towards zero
                Ok(FhirPathValue::Integer(a / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                let result = a / b;
                // Truncate towards zero
                let truncated = result.trunc();

                match truncated.to_i64() {
                    Some(int_result) => Ok(FhirPathValue::Integer(int_result)),
                    None => Err(FhirPathError::ArithmeticError {
                        message: "Integer division result too large".to_string(),
                    }),
                }
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => {
                    let result = a_decimal / b;
                    let truncated = result.trunc();
                    match truncated.to_i64() {
                        Some(int_result) => Ok(FhirPathValue::Integer(int_result)),
                        None => Err(FhirPathError::ArithmeticError {
                            message: "Integer division result too large".to_string(),
                        }),
                    }
                }
                Err(_) => Err(FhirPathError::ArithmeticError {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => {
                    let result = a / b_decimal;
                    let truncated = result.trunc();
                    match truncated.to_i64() {
                        Some(int_result) => Ok(FhirPathValue::Integer(int_result)),
                        None => Err(FhirPathError::ArithmeticError {
                            message: "Integer division result too large".to_string(),
                        }),
                    }
                }
                Err(_) => Err(FhirPathError::ArithmeticError {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot perform integer division of {} by {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for DivOperation {
    fn identifier(&self) -> &str {
        "div"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(DivOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        // Special handling for div - division by zero returns empty collection instead of error
        match binary_operator_utils::evaluate_arithmetic_operator(
            &args[0],
            &args[1],
            Self::div_values,
        ) {
            Ok(result) => Ok(result),
            Err(FhirPathError::ArithmeticError { message }) if message == "Division by zero" => {
                use octofhir_fhirpath_model::Collection;
                Ok(FhirPathValue::Collection(Collection::from(vec![])))
            }
            Err(e) => Err(e),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "div".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        // Special handling for div - division by zero returns empty collection instead of error
        match binary_operator_utils::evaluate_arithmetic_operator(
            &args[0],
            &args[1],
            Self::div_values,
        ) {
            Ok(result) => Some(Ok(result)),
            Err(FhirPathError::ArithmeticError { message }) if message == "Division by zero" => {
                use octofhir_fhirpath_model::Collection;
                Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))))
            }
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
