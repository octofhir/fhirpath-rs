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

//! Division operation (/) implementation for FHIRPath

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

/// Division operation (/) - returns decimal result
pub struct DivisionOperation;

impl Default for DivisionOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl DivisionOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "/",
            OperationType::BinaryOperator {
                precedence: 7,
                associativity: Associativity::Left,
            },
        )
        .description("Division operation - returns decimal result")
        .example("10 / 3")
        .example("6.0 / 2.0")
        .returns(TypeConstraint::Specific(FhirPathType::Decimal))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn divide_values(left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    match (Decimal::try_from(*a), Decimal::try_from(*b)) {
                        (Ok(a_decimal), Ok(b_decimal)) => {
                            let result = a_decimal / b_decimal;
                            // Return integer if the result is a whole number
                            if result.fract() == Decimal::ZERO {
                                match i64::try_from(result) {
                                    Ok(int_result) => Ok(FhirPathValue::Integer(int_result)),
                                    Err(_) => Ok(FhirPathValue::Decimal(result)), // Fallback to decimal if too large
                                }
                            } else {
                                Ok(FhirPathValue::Decimal(result))
                            }
                        }
                        _ => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integers to decimal for division".to_string(),
                        }),
                    }
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if *b == Decimal::ZERO {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Decimal(a / b))
                }
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if *b == Decimal::ZERO {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    match Decimal::try_from(*a) {
                        Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal / b)),
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal".to_string(),
                        }),
                    }
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    // Division by zero returns empty according to FHIRPath specification
                    Ok(FhirPathValue::Empty)
                } else {
                    match Decimal::try_from(*b) {
                        Ok(b_decimal) => Ok(FhirPathValue::Decimal(a / b_decimal)),
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal".to_string(),
                        }),
                    }
                }
            }
            // Quantity / Quantity = Quantity with combined units
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                match a.divide(b) {
                    Some(result) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(result))),
                    None => Ok(FhirPathValue::Empty), // Division by zero returns empty
                }
            }
            // Quantity / Scalar = Quantity (scalar division)
            (FhirPathValue::Quantity(q), FhirPathValue::Integer(scalar)) => {
                if *scalar == 0 {
                    Ok(FhirPathValue::Empty) // Division by zero returns empty
                } else {
                    match Decimal::try_from(*scalar) {
                        Ok(scalar_decimal) => match q.divide_scalar(scalar_decimal) {
                            Some(result) => {
                                Ok(FhirPathValue::Quantity(std::sync::Arc::new(result)))
                            }
                            None => Ok(FhirPathValue::Empty), // Division by zero returns empty
                        },
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal for quantity division"
                                .to_string(),
                        }),
                    }
                }
            }
            (FhirPathValue::Quantity(q), FhirPathValue::Decimal(scalar)) => {
                match q.divide_scalar(*scalar) {
                    Some(result) => Ok(FhirPathValue::Quantity(std::sync::Arc::new(result))),
                    None => Ok(FhirPathValue::Empty), // Division by zero returns empty
                }
            }
            // Scalar / Quantity - this creates a quantity with reciprocal unit
            (FhirPathValue::Integer(scalar), FhirPathValue::Quantity(q)) => {
                if q.value.is_zero() {
                    Ok(FhirPathValue::Empty) // Division by zero returns empty
                } else {
                    match Decimal::try_from(*scalar) {
                        Ok(scalar_decimal) => {
                            let result_value = scalar_decimal / q.value;
                            let result_unit = if let Some(unit) = &q.unit {
                                if unit == "1" || unit.is_empty() {
                                    Some("1".to_string())
                                } else {
                                    Some(format!("1/{unit}"))
                                }
                            } else {
                                Some("1".to_string())
                            };
                            Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                                octofhir_fhirpath_model::Quantity::new(result_value, result_unit),
                            )))
                        }
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal for quantity division"
                                .to_string(),
                        }),
                    }
                }
            }
            (FhirPathValue::Decimal(scalar), FhirPathValue::Quantity(q)) => {
                if q.value.is_zero() {
                    Ok(FhirPathValue::Empty) // Division by zero returns empty
                } else {
                    let result_value = scalar / q.value;
                    let result_unit = if let Some(unit) = &q.unit {
                        if unit == "1" || unit.is_empty() {
                            Some("1".to_string())
                        } else {
                            Some(format!("1/{unit}"))
                        }
                    } else {
                        Some("1".to_string())
                    };
                    Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                        octofhir_fhirpath_model::Quantity::new(result_value, result_unit),
                    )))
                }
            }
            _ => Err(FhirPathError::invalid_operation_with_types(
                "division",
                Some(left.type_name().to_string()),
                Some(right.type_name().to_string()),
                format!("Cannot divide {} by {}", left.type_name(), right.type_name())
            )),
        }
    }
}

#[async_trait]
impl FhirPathOperation for DivisionOperation {
    fn identifier(&self) -> &str {
        "/"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(DivisionOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        binary_operator_utils::evaluate_arithmetic_operator(&args[0], &args[1], Self::divide_values)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "/".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        Some(binary_operator_utils::evaluate_arithmetic_operator(
            &args[0],
            &args[1],
            Self::divide_values,
        ))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
