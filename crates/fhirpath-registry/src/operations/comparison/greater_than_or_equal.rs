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

//! Greater than or equal operator (>=) implementation

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

/// Greater than or equal operator (>=)
#[derive(Debug, Clone)]
pub struct GreaterThanOrEqualOperation;

impl Default for GreaterThanOrEqualOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl GreaterThanOrEqualOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            ">=",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
        .description("Greater than or equal comparison operator")
        .example("5 >= 3")
        .example("'beta' >= 'alpha'")
        .returns(TypeConstraint::Specific(FhirPathType::Boolean))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn compare_greater_or_equal(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<bool>> {
        match (left, right) {
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(None),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(Some(a >= b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(Some(a >= b)),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(Some(Decimal::from(*a) >= *b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(Some(*a >= Decimal::from(*b)))
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Ok(Some(a >= b)),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Ok(Some(a >= b)),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Ok(Some(a >= b)),
            // Date vs DateTime comparisons - use low boundary for Date in >= comparison
            (FhirPathValue::Date(date), FhirPathValue::DateTime(datetime)) => {
                // For Date >= DateTime, use low boundary of Date (start of day)
                let date_low_boundary = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "Invalid date for conversion to datetime".to_string(),
                    })?
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or_else(|| FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "Timezone conversion error".to_string(),
                    })?;
                Ok(Some(date_low_boundary >= *datetime))
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Date(date)) => {
                // For DateTime >= Date, use low boundary of Date (start of day)
                let date_low_boundary = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "Invalid date for conversion to datetime".to_string(),
                    })?
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or_else(|| FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "Timezone conversion error".to_string(),
                    })?;
                Ok(Some(*datetime >= date_low_boundary))
            }
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Ok(Some(a >= b)),
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Compare quantities with unit conversion
                match (a.has_compatible_dimensions(b), &a.unit, &b.unit) {
                    (true, Some(unit_a), Some(_)) => {
                        // Try to convert b to a's unit and compare
                        match b.convert_to_compatible_unit(unit_a) {
                            Ok(converted_b) => Ok(Some(a.value >= converted_b.value)),
                            Err(_) => Ok(Some(false)), // If conversion fails, comparison is false
                        }
                    }
                    (true, None, None) => {
                        // Both unitless quantities
                        Ok(Some(a.value >= b.value))
                    }
                    _ => Ok(Some(false)), // Incompatible units
                }
            }
            // Collections - compare if both single items
            (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) => {
                if a.len() == 1 && b.len() == 1 {
                    Self::compare_greater_or_equal(a.get(0).unwrap(), b.get(0).unwrap())
                } else {
                    Err(FhirPathError::InvalidArguments {
                        message: "Collections with multiple items cannot be compared".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot compare {} >= {}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for GreaterThanOrEqualOperation {
    fn identifier(&self) -> &str {
        ">="
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(GreaterThanOrEqualOperation::create_metadata);
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

        binary_operator_utils::evaluate_binary_operator_optional(
            &args[0],
            &args[1],
            Self::compare_greater_or_equal,
        )
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

        Some(binary_operator_utils::evaluate_binary_operator_optional(
            &args[0],
            &args[1],
            Self::compare_greater_or_equal,
        ))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
