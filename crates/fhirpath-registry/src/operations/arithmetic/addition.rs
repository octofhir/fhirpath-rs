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

//! Addition operation (+) implementation for FHIRPath

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use num_traits::ToPrimitive;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use rust_decimal::Decimal;

/// Addition operation (+) - supports both binary and unary operations
pub struct AdditionOperation;

impl Default for AdditionOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl AdditionOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "+",
            OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            },
        )
        .description("Binary addition operation and unary plus")
        .example("1 + 2")
        .example("+'5'")
        .returns(TypeConstraint::OneOf(vec![
            FhirPathType::Integer,
            FhirPathType::Decimal,
            FhirPathType::String,
        ]))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    fn evaluate_binary_sync(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<Result<FhirPathValue>> {
        // Handle empty collections per FHIRPath spec
        match (left, right) {
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.is_empty() || r.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                if l.len() > 1 || r.len() > 1 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                // Single element collections - unwrap and proceed
                let left_val = l.first().unwrap();
                let right_val = r.first().unwrap();
                return self.evaluate_binary_sync(left_val, right_val);
            }
            (FhirPathValue::Collection(l), other) => {
                if l.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                if l.len() > 1 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                let left_val = l.first().unwrap();
                return self.evaluate_binary_sync(left_val, other);
            }
            (other, FhirPathValue::Collection(r)) => {
                if r.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                if r.len() > 1 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                let right_val = r.first().unwrap();
                return self.evaluate_binary_sync(other, right_val);
            }
            _ => {}
        }

        // Actual arithmetic operations on scalar values
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a
                .checked_add(*b)
                .map(FhirPathValue::Integer)
                .ok_or_else(|| FhirPathError::ArithmeticError {
                    message: "Integer overflow in addition".to_string(),
                }),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a + b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => match Decimal::try_from(*a) {
                Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal + b)),
                Err(_) => Err(FhirPathError::ArithmeticError {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => match Decimal::try_from(*b) {
                Ok(b_decimal) => Ok(FhirPathValue::Decimal(a + b_decimal)),
                Err(_) => Err(FhirPathError::ArithmeticError {
                    message: "Cannot convert integer to decimal".to_string(),
                }),
            },
            // Quantity addition - requires compatible units
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // For addition, quantities must have compatible dimensions
                if a.has_compatible_dimensions(b) {
                    match b.convert_to_compatible_unit(a.unit.as_ref().unwrap_or(&"1".to_string()))
                    {
                        Ok(converted_b) => {
                            let result_value = a.value + converted_b.value;
                            Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                                octofhir_fhirpath_model::Quantity::new(
                                    result_value,
                                    a.unit.clone(),
                                ),
                            )))
                        }
                        Err(_) => return None, // Conversion failed, fallback to async
                    }
                } else {
                    return None; // Incompatible units, fallback to async for error handling
                }
            }
            // Date + Quantity operations
            (FhirPathValue::Date(date), FhirPathValue::Quantity(qty)) => {
                match self.add_quantity_to_date(date, qty) {
                    Ok(result) => Ok(result),
                    Err(_) => return None, // Fallback to async for error handling
                }
            }
            // DateTime + Quantity operations
            (FhirPathValue::DateTime(datetime), FhirPathValue::Quantity(qty)) => {
                match self.add_quantity_to_datetime(datetime, qty) {
                    Ok(result) => Ok(result),
                    Err(_) => return None, // Fallback to async for error handling
                }
            }
            // Time + Quantity operations
            (FhirPathValue::Time(time), FhirPathValue::Quantity(qty)) => {
                match self.add_quantity_to_time(time, qty) {
                    Ok(result) => Ok(result),
                    Err(_) => return None, // Fallback to async for error handling
                }
            }
            _ => return None, // Fallback to async for complex cases
        };

        // Wrap result in collection as per FHIRPath spec
        Some(result.map(|val| FhirPathValue::Collection(Collection::from(vec![val]))))
    }

    async fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Unwrap single-item collections
        let left_unwrapped = self.unwrap_single_collection(left);
        let right_unwrapped = self.unwrap_single_collection(right);

        // Try sync path first
        if let Some(result) = self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped) {
            return result;
        }

        // Handle string concatenation and other complex cases
        let result = match (&left_unwrapped, &right_unwrapped) {
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{a}{b}").into()))
            }
            (FhirPathValue::Empty, _) => {
                return Ok(FhirPathValue::Collection(Collection::from(vec![])));
            }
            (_, FhirPathValue::Empty) => {
                return Ok(FhirPathValue::Collection(Collection::from(vec![])));
            }
            _ => Err(FhirPathError::type_mismatch_with_context(
                "numeric types",
                format!("{} and {}", left_unwrapped.type_name(), right_unwrapped.type_name()),
                "addition operation"
            )),
        };

        // Wrap result in collection as per FHIRPath spec
        result.map(|val| FhirPathValue::Collection(Collection::from(vec![val])))
    }

    async fn evaluate_unary(
        &self,
        value: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle collections first
        let unwrapped = self.unwrap_single_collection(value);

        // Unary plus - return the value unchanged for numbers
        let result = match &unwrapped {
            FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => Ok(unwrapped),
            FhirPathValue::String(s) => {
                // Try to convert string to number
                if let Ok(int_val) = s.parse::<i64>() {
                    Ok(FhirPathValue::Integer(int_val))
                } else if let Ok(decimal_val) = s.parse::<Decimal>() {
                    Ok(FhirPathValue::Decimal(decimal_val))
                } else {
                    Err(FhirPathError::type_mismatch_with_context(
                        "numeric string",
                        format!("non-numeric string '{s}'"),
                        "unary plus operation"
                    ))
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            FhirPathValue::Collection(items) if items.is_empty() => {
                return Ok(FhirPathValue::Collection(Collection::from(vec![])));
            }
            FhirPathValue::Collection(items) if items.len() > 1 => {
                return Ok(FhirPathValue::Collection(Collection::from(vec![])));
            }
            _ => Err(FhirPathError::type_mismatch_with_context(
                "numeric types",
                unwrapped.type_name(),
                "unary plus operation"
            )),
        };

        // Wrap result in collection as per FHIRPath spec
        result.map(|val| FhirPathValue::Collection(Collection::from(vec![val])))
    }
}

#[async_trait]
impl FhirPathOperation for AdditionOperation {
    fn identifier(&self) -> &str {
        "+"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AdditionOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match args.len() {
            1 => self.evaluate_unary(&args[0], context).await,
            2 => self.evaluate_binary(&args[0], &args[1], context).await,
            _ => Err(FhirPathError::InvalidArgumentCount {
                function_name: "+".to_string(),
                expected: 1, // Can be 1 or 2
                actual: args.len(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        match args.len() {
            1 => {
                // Unary plus sync evaluation
                let unwrapped = self.unwrap_single_collection(&args[0]);
                match &unwrapped {
                    FhirPathValue::Integer(_) | FhirPathValue::Decimal(_) => {
                        Some(Ok(FhirPathValue::Collection(Collection::from(vec![
                            unwrapped,
                        ]))))
                    }
                    FhirPathValue::Empty => {
                        Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))))
                    }
                    _ => None, // Fallback to async for string conversion
                }
            }
            2 => {
                let left_unwrapped = self.unwrap_single_collection(&args[0]);
                let right_unwrapped = self.unwrap_single_collection(&args[1]);
                self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped)
            }
            _ => Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "+".to_string(),
                expected: 1,
                actual: args.len(),
            })),
        }
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl AdditionOperation {
    /// Unwrap single-item collections to their contained value
    fn unwrap_single_collection(&self, value: &FhirPathValue) -> FhirPathValue {
        match value {
            FhirPathValue::Collection(items) if items.len() == 1 => items.first().unwrap().clone(),
            _ => value.clone(),
        }
    }

    /// Add a quantity to a date value
    fn add_quantity_to_date(
        &self,
        date: &chrono::NaiveDate,
        qty: &octofhir_fhirpath_model::Quantity,
    ) -> Result<FhirPathValue> {
        let default_unit = "1".to_string();
        let unit = qty.unit.as_ref().unwrap_or(&default_unit).as_str();
        let value = qty.value.to_i64().ok_or(FhirPathError::TypeError {
            message: "Quantity value must be an integer for date arithmetic".to_string(),
        })?;

        let result_date = match unit {
            "year" | "years" | "a" => date
                .checked_add_months(chrono::Months::new((value * 12) as u32))
                .ok_or(FhirPathError::ArithmeticError {
                    message: "Date overflow in year addition".to_string(),
                })?,
            "month" | "months" | "mo" => date
                .checked_add_months(chrono::Months::new(value as u32))
                .ok_or(FhirPathError::ArithmeticError {
                    message: "Date overflow in month addition".to_string(),
                })?,
            "week" | "weeks" | "wk" => date
                .checked_add_days(chrono::Days::new((value * 7) as u64))
                .ok_or(FhirPathError::ArithmeticError {
                    message: "Date overflow in week addition".to_string(),
                })?,
            "day" | "days" | "d" => date
                .checked_add_days(chrono::Days::new(value as u64))
                .ok_or(FhirPathError::ArithmeticError {
                    message: "Date overflow in day addition".to_string(),
                })?,
            _ => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "Invalid unit '{unit}' for date addition. Must be years, months, weeks, or days"
                    ),
                });
            }
        };

        Ok(FhirPathValue::Date(result_date))
    }

    /// Add a quantity to a datetime value
    fn add_quantity_to_datetime(
        &self,
        datetime: &chrono::DateTime<chrono::FixedOffset>,
        qty: &octofhir_fhirpath_model::Quantity,
    ) -> Result<FhirPathValue> {
        let default_unit = "1".to_string();
        let unit = qty.unit.as_ref().unwrap_or(&default_unit).as_str();
        let value = qty.value;

        let result_datetime = match unit {
            "year" | "years" | "a" => {
                let months = (value * rust_decimal::Decimal::from(12)).to_i64().ok_or(
                    FhirPathError::TypeError {
                        message: "Invalid year value".to_string(),
                    },
                )?;
                datetime
                    .date_naive()
                    .checked_add_months(chrono::Months::new(months as u32))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in year addition".to_string(),
                    })?
                    .and_time(datetime.time())
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or(FhirPathError::TypeError {
                        message: "Timezone conversion error".to_string(),
                    })?
            }
            "month" | "months" | "mo" => {
                let months = value.to_i64().ok_or(FhirPathError::TypeError {
                    message: "Invalid month value".to_string(),
                })?;
                datetime
                    .date_naive()
                    .checked_add_months(chrono::Months::new(months as u32))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in month addition".to_string(),
                    })?
                    .and_time(datetime.time())
                    .and_local_timezone(datetime.timezone())
                    .single()
                    .ok_or(FhirPathError::TypeError {
                        message: "Timezone conversion error".to_string(),
                    })?
            }
            "week" | "weeks" | "wk" => {
                let days = (value * rust_decimal::Decimal::from(7)).to_i64().ok_or(
                    FhirPathError::TypeError {
                        message: "Invalid week value".to_string(),
                    },
                )?;
                datetime
                    .checked_add_days(chrono::Days::new(days as u64))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in week addition".to_string(),
                    })?
            }
            "day" | "days" | "d" => {
                let days = value.to_i64().ok_or(FhirPathError::TypeError {
                    message: "Invalid day value".to_string(),
                })?;
                datetime
                    .checked_add_days(chrono::Days::new(days as u64))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in day addition".to_string(),
                    })?
            }
            "hour" | "hours" | "h" => {
                let hours = (value * rust_decimal::Decimal::from(3600)).to_i64().ok_or(
                    FhirPathError::TypeError {
                        message: "Invalid hour value".to_string(),
                    },
                )?;
                datetime
                    .checked_add_signed(chrono::Duration::seconds(hours))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in hour addition".to_string(),
                    })?
            }
            "minute" | "minutes" | "min" => {
                let minutes = (value * rust_decimal::Decimal::from(60)).to_i64().ok_or(
                    FhirPathError::TypeError {
                        message: "Invalid minute value".to_string(),
                    },
                )?;
                datetime
                    .checked_add_signed(chrono::Duration::seconds(minutes))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in minute addition".to_string(),
                    })?
            }
            "second" | "seconds" | "s" => {
                let seconds = value.to_i64().ok_or(FhirPathError::TypeError {
                    message: "Invalid second value".to_string(),
                })?;
                datetime
                    .checked_add_signed(chrono::Duration::seconds(seconds))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in second addition".to_string(),
                    })?
            }
            "millisecond" | "milliseconds" | "ms" => {
                let millis = value.to_i64().ok_or(FhirPathError::TypeError {
                    message: "Invalid millisecond value".to_string(),
                })?;
                datetime
                    .checked_add_signed(chrono::Duration::milliseconds(millis))
                    .ok_or(FhirPathError::ArithmeticError {
                        message: "DateTime overflow in millisecond addition".to_string(),
                    })?
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: format!("Invalid unit '{unit}' for datetime addition"),
                });
            }
        };

        Ok(FhirPathValue::DateTime(result_datetime))
    }

    /// Add a quantity to a time value
    fn add_quantity_to_time(
        &self,
        time: &chrono::NaiveTime,
        qty: &octofhir_fhirpath_model::Quantity,
    ) -> Result<FhirPathValue> {
        let default_unit = "1".to_string();
        let unit = qty.unit.as_ref().unwrap_or(&default_unit).as_str();
        let value = qty.value;

        let result_time = match unit {
            "hour" | "hours" | "h" => {
                let hours = (value * rust_decimal::Decimal::from(3600)).to_i64().ok_or(
                    FhirPathError::TypeError {
                        message: "Invalid hour value".to_string(),
                    },
                )?;
                time.overflowing_add_signed(chrono::Duration::seconds(hours))
                    .0
            }
            "minute" | "minutes" | "min" => {
                let minutes = (value * rust_decimal::Decimal::from(60)).to_i64().ok_or(
                    FhirPathError::TypeError {
                        message: "Invalid minute value".to_string(),
                    },
                )?;
                time.overflowing_add_signed(chrono::Duration::seconds(minutes))
                    .0
            }
            "second" | "seconds" | "s" => {
                let seconds = value.to_i64().ok_or(FhirPathError::TypeError {
                    message: "Invalid second value".to_string(),
                })?;
                time.overflowing_add_signed(chrono::Duration::seconds(seconds))
                    .0
            }
            "millisecond" | "milliseconds" | "ms" => {
                let millis = value.to_i64().ok_or(FhirPathError::TypeError {
                    message: "Invalid millisecond value".to_string(),
                })?;
                time.overflowing_add_signed(chrono::Duration::milliseconds(millis))
                    .0
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: format!(
                        "Invalid unit '{unit}' for time addition. Must be hours, minutes, seconds, or milliseconds"
                    ),
                });
            }
        };

        Ok(FhirPathValue::Time(result_time))
    }
}
