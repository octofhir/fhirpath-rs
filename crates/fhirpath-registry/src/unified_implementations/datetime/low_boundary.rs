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

//! Unified lowBoundary() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Timelike};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Unified lowBoundary() function implementation
///
/// Returns the lower bound of a value based on precision
pub struct UnifiedLowBoundaryFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedLowBoundaryFunction {
    pub fn new() -> Self {
        let metadata = MetadataBuilder::new("lowBoundary", FunctionCategory::DateTime)
            .display_name("Low Boundary")
            .description("Returns the lower bound of a value based on precision")
            .example("(2.58).lowBoundary()")
            .example("(2.58).lowBoundary(1)")
            .output_type(TypePattern::Any) // Can return different types based on input
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("lowBoundary($0)")
            .keywords(vec!["lowBoundary", "boundary", "precision", "lower", "bound"])
            .usage_pattern(
                "Calculate lower boundary",
                "value.lowBoundary(precision)",
                "Precision-based calculations and ranges"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedLowBoundaryFunction {
    fn name(&self) -> &str {
        "lowBoundary"
    }

    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }

    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }

    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments (0 or 1 argument)
        if args.len() > 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(1),
                actual: args.len(),
            });
        }

        let precision = if args.is_empty() {
            None
        } else {
            match &args[0] {
                FhirPathValue::Integer(p) => {
                    if *p < 0 {
                        // Invalid precision, return empty
                        return Ok(FhirPathValue::Empty);
                    }
                    Some(*p as u32)
                }
                FhirPathValue::Empty => None,
                _ => {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Precision parameter must be an integer".to_string(),
                    });
                }
            }
        };

        match &context.input {
            FhirPathValue::Decimal(d) => match calculate_low_boundary(d, precision) {
                Ok(low_bound) => {
                    // If precision is 0, return Integer; otherwise return Decimal
                    if precision == Some(0) {
                        Ok(FhirPathValue::Integer(low_bound.to_i64().unwrap_or(0)))
                    } else {
                        Ok(FhirPathValue::Decimal(low_bound))
                    }
                }
                Err(FunctionError::EvaluationError { message, .. })
                    if message.contains("Precision exceeds maximum") =>
                {
                    Ok(FhirPathValue::Empty)
                }
                Err(e) => Err(e),
            },
            FhirPathValue::Integer(i) => {
                let decimal = Decimal::from(*i);
                match calculate_low_boundary(&decimal, precision) {
                    Ok(low_bound) => {
                        // If precision is 0, return Integer; otherwise return Decimal
                        if precision == Some(0) {
                            Ok(FhirPathValue::Integer(low_bound.to_i64().unwrap_or(*i)))
                        } else {
                            Ok(FhirPathValue::Decimal(low_bound))
                        }
                    }
                    Err(FunctionError::EvaluationError { message, .. })
                        if message.contains("Precision exceeds maximum") =>
                    {
                        Ok(FhirPathValue::Empty)
                    }
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Date(d) => {
                let low_bound = calculate_date_low_boundary(d)?;
                Ok(FhirPathValue::DateTime(low_bound))
            }
            FhirPathValue::Quantity(q) => {
                match calculate_low_boundary(&q.value, precision) {
                    Ok(low_bound) => {
                        // If precision is 0, return Integer Quantity; otherwise return Decimal Quantity
                        let new_value = if precision == Some(0) {
                            rust_decimal::Decimal::from(low_bound.to_i64().unwrap_or(0))
                        } else {
                            low_bound
                        };
                        Ok(FhirPathValue::Quantity(
                            octofhir_fhirpath_model::Quantity::new(new_value, q.unit.clone())
                                .into(),
                        ))
                    }
                    Err(FunctionError::EvaluationError { message, .. })
                        if message.contains("Precision exceeds maximum") =>
                    {
                        Ok(FhirPathValue::Empty)
                    }
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::DateTime(dt) => {
                let low_bound = calculate_datetime_low_boundary(dt)?;
                Ok(FhirPathValue::DateTime(low_bound))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

fn calculate_low_boundary(value: &Decimal, precision: Option<u32>) -> FunctionResult<Decimal> {
    let scale = precision.unwrap_or_else(|| {
        // Default precision: use at least 8 for Decimal as per FHIRPath spec
        std::cmp::max(8, value.scale())
    });

    // Check for maximum precision limit (28 is Decimal's limit)
    if scale > 28 {
        return Err(FunctionError::EvaluationError {
            name: "boundary".to_string(),
            message: "Precision exceeds maximum allowed value".to_string(),
        });
    }

    if scale == 0 {
        // For integer precision, the low boundary represents the range that could truncate to this integer
        // For positive numbers: truncate towards zero, so 1 comes from [1, 2), low boundary is 1
        // For negative numbers: truncate towards zero, so -1 comes from [-2, -1), low boundary is -2
        let truncated = value.trunc();
        if value.is_sign_negative() && *value != truncated {
            // For negative non-integers, the range is [trunc-1, trunc), so low boundary is trunc-1
            Ok(truncated - Decimal::ONE)
        } else {
            // For integers or positive numbers, low boundary is the truncated value
            Ok(truncated)
        }
    } else {
        // Calculate the low boundary
        let input_scale = value.scale();
        if scale <= input_scale {
            // Reducing precision
            let truncated = value.trunc_with_scale(scale);
            if value.is_sign_negative() {
                // For negative numbers, low boundary is truncated minus one ULP (further from zero)
                let ulp = Decimal::new(1, scale);
                Ok(truncated - ulp)
            } else {
                // For positive numbers, low boundary is just truncated
                Ok(truncated)
            }
        } else {
            // Adding precision - the low boundary is the value extended with zeros,
            // then subtract half ULP at the next digit position
            let extended = value.trunc_with_scale(scale);
            let half_ulp = Decimal::new(5, input_scale + 1); // 0.5 at the next position after original precision
            Ok(extended - half_ulp)
        }
    }
}

/// Calculate low boundary for Date values
/// For dates like "2001-05-06", the low boundary is "2001-05-06T00:00:00.000"
fn calculate_date_low_boundary(date: &NaiveDate) -> FunctionResult<DateTime<FixedOffset>> {
    let datetime = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| FunctionError::EvaluationError {
            name: "lowBoundary".to_string(),
            message: "Invalid date for boundary calculation".to_string(),
        })?;

    // Convert to UTC for consistent boundary calculation
    Ok(datetime.and_utc().fixed_offset())
}

/// Calculate low boundary for DateTime values
/// For partial datetimes, fills missing precision with minimum values (0)
fn calculate_datetime_low_boundary(
    datetime: &DateTime<FixedOffset>,
) -> FunctionResult<DateTime<FixedOffset>> {
    // For datetime values, the low boundary is the datetime with seconds/milliseconds set to 0
    // if they weren't specified originally (indicated by being 0)
    let naive = datetime.naive_local();

    // Create low boundary: if seconds are 0, assume they weren't specified, same for nanoseconds
    let low_boundary = if naive.second() == 0 && naive.nanosecond() == 0 {
        // Seconds and nanoseconds not specified, keep them at 0
        naive
    } else {
        // They were specified, keep the exact value
        naive
    };

    datetime
        .timezone()
        .from_local_datetime(&low_boundary)
        .single()
        .ok_or_else(|| FunctionError::EvaluationError {
            name: "lowBoundary".to_string(),
            message: "Invalid datetime for boundary calculation".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_low_boundary_function() {
        let low_boundary_func = UnifiedLowBoundaryFunction::new();

        // Test decimal low boundary
        let context = EvaluationContext::new(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_f64(2.58).unwrap()
        ));
        let result = low_boundary_func.evaluate_sync(&[], &context).unwrap();

        // Should return a decimal
        match result {
            FhirPathValue::Decimal(_) => {
                // Success
            },
            _ => panic!("Expected Decimal result from lowBoundary function"),
        }

        // Test with precision argument
        let args = vec![FhirPathValue::Integer(1)];
        let result = low_boundary_func.evaluate_sync(&args, &context).unwrap();

        match result {
            FhirPathValue::Decimal(_) => {
                // Success
            },
            _ => panic!("Expected Decimal result from lowBoundary function with precision"),
        }

        // Test with precision 0 (should return integer)
        let args = vec![FhirPathValue::Integer(0)];
        let result = low_boundary_func.evaluate_sync(&args, &context).unwrap();

        match result {
            FhirPathValue::Integer(_) => {
                // Success
            },
            _ => panic!("Expected Integer result from lowBoundary function with precision 0"),
        }

        // Test with integer input
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = low_boundary_func.evaluate_sync(&[], &context).unwrap();

        match result {
            FhirPathValue::Decimal(_) => {
                // Success
            },
            _ => panic!("Expected Decimal result from lowBoundary function with integer input"),
        }

        // Test metadata
        assert_eq!(low_boundary_func.name(), "lowBoundary");
        assert_eq!(low_boundary_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(low_boundary_func.metadata().basic.display_name, "Low Boundary");
        assert!(low_boundary_func.metadata().basic.is_pure);
    }
}
