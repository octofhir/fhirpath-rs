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

//! Greater than operator (>) implementation

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

/// Greater than operator (>)
#[derive(Debug, Clone)]
pub struct GreaterThanOperation;

impl Default for GreaterThanOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl GreaterThanOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            ">",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
        .description("Greater than comparison operator")
        .example("2 > 1")
        .example("@2023-12-31 > @2023-01-01")
        .returns(TypeConstraint::Specific(FhirPathType::Boolean))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    pub fn compare_greater_than(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<bool>> {
        match (left, right) {
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(Some(false)),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(Some(a > b)),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(Some(a > b)),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(Some(Decimal::from(*a) > *b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(Some(*a > Decimal::from(*b)))
            }
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Ok(Some(a > b)),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Ok(Some(a > b)),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Ok(Some(a > b)),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Ok(Some(a > b)),
            // Date vs DateTime comparison - per FHIRPath spec, check for precision ambiguity
            (FhirPathValue::Date(date), FhirPathValue::DateTime(datetime)) => {
                // Check if DateTime clearly falls outside the Date range
                let date_start = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Invalid date for comparison".to_string(),
                    })?
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Timezone conversion error".to_string(),
                    })?;
                let date_end = date
                    .and_hms_milli_opt(23, 59, 59, 999)
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Invalid date for comparison".to_string(),
                    })?
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Timezone conversion error".to_string(),
                    })?;

                // If DateTime is clearly before or after the entire Date range, we can compare
                if *datetime < date_start {
                    Ok(Some(false)) // Date is definitely after DateTime
                } else if *datetime > date_end {
                    Ok(Some(false)) // Date is definitely before DateTime
                } else {
                    // DateTime falls within the Date range - ambiguous, return empty
                    Ok(None)
                }
            }
            (FhirPathValue::DateTime(datetime), FhirPathValue::Date(date)) => {
                // Check if DateTime clearly falls outside the Date range
                let date_start = date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Invalid date for comparison".to_string(),
                    })?
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Timezone conversion error".to_string(),
                    })?;
                let date_end = date
                    .and_hms_milli_opt(23, 59, 59, 999)
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Invalid date for comparison".to_string(),
                    })?
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or_else(|| FhirPathError::TypeError {
                        message: "Timezone conversion error".to_string(),
                    })?;

                // If DateTime is clearly before or after the entire Date range, we can compare
                if *datetime < date_start {
                    Ok(Some(false)) // DateTime is before Date range
                } else if *datetime > date_end {
                    Ok(Some(true)) // DateTime is after Date range
                } else {
                    // DateTime falls within the Date range - ambiguous, return empty
                    Ok(None)
                }
            }
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Compare quantities with unit conversion
                match (a.has_compatible_dimensions(b), &a.unit, &b.unit) {
                    (true, Some(unit_a), Some(_)) => {
                        // Try to convert b to a's unit and compare
                        match b.convert_to_compatible_unit(unit_a) {
                            Ok(converted_b) => Ok(Some(a.value > converted_b.value)),
                            Err(_) => Ok(Some(false)), // If conversion fails, comparison is false
                        }
                    }
                    (true, None, None) => {
                        // Both unitless quantities
                        Ok(Some(a.value > b.value))
                    }
                    _ => Ok(Some(false)), // Incompatible units
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot compare {:?} > {:?}",
                    left.type_name(),
                    right.type_name()
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for GreaterThanOperation {
    fn identifier(&self) -> &str {
        ">"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(GreaterThanOperation::create_metadata);
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
            Self::compare_greater_than,
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
            Self::compare_greater_than,
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
