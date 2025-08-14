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

//! Unified highBoundary() function implementation

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

/// Unified highBoundary() function implementation
///
/// Returns the upper bound of a value based on precision
pub struct UnifiedHighBoundaryFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedHighBoundaryFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 optional integer parameter
        let signature = FunctionSignature::new(
            "highBoundary",
            vec![ParameterInfo::optional("precision", TypeInfo::Integer)],
            TypeInfo::Any, // Can return different types based on input
        );

        let metadata = MetadataBuilder::new("highBoundary", FunctionCategory::DateTime)
            .display_name("High Boundary")
            .description("Returns the upper bound of a value based on precision")
            .example("(2.58).highBoundary()")
            .example("(2.58).highBoundary(1)")
            .signature(signature)
            .output_type(TypePattern::Any) // Can return different types based on input
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("highBoundary($0)")
            .keywords(vec!["highBoundary", "boundary", "precision", "upper", "bound"])
            .usage_pattern(
                "Calculate upper boundary",
                "value.highBoundary(precision)",
                "Precision-based calculations and ranges"
            )
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedHighBoundaryFunction {
    fn name(&self) -> &str {
        "highBoundary"
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
            FhirPathValue::Decimal(d) => match calculate_high_boundary(d, precision) {
                Ok(high_bound) => {
                    // If precision is 0, return Integer; otherwise return Decimal
                    if precision == Some(0) {
                        Ok(FhirPathValue::Integer(high_bound.to_i64().unwrap_or(0)))
                    } else {
                        Ok(FhirPathValue::Decimal(high_bound))
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
                match calculate_high_boundary(&decimal, precision) {
                    Ok(high_bound) => {
                        // If precision is 0, return Integer; otherwise return Decimal
                        if precision == Some(0) {
                            Ok(FhirPathValue::Integer(high_bound.to_i64().unwrap_or(*i)))
                        } else {
                            Ok(FhirPathValue::Decimal(high_bound))
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
                let high_bound = calculate_date_high_boundary(d)?;
                Ok(FhirPathValue::DateTime(high_bound))
            }
            FhirPathValue::Quantity(q) => {
                match calculate_high_boundary(&q.value, precision) {
                    Ok(high_bound) => {
                        // If precision is 0, return Integer Quantity; otherwise return Decimal Quantity
                        let new_value = if precision == Some(0) {
                            rust_decimal::Decimal::from(high_bound.to_i64().unwrap_or(0))
                        } else {
                            high_bound
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
                let high_bound = calculate_datetime_high_boundary(dt)?;
                Ok(FhirPathValue::DateTime(high_bound))
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

fn calculate_high_boundary(value: &Decimal, precision: Option<u32>) -> FunctionResult<Decimal> {
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
        // For integer precision, the high boundary represents the range that could truncate to this integer
        // For positive numbers: truncate towards zero, so 1 comes from [1, 2), high boundary is 2
        // For negative numbers: truncate towards zero, so -1 comes from [-2, -1), high boundary is -1
        let truncated = value.trunc();
        if value.is_sign_negative() && *value != truncated {
            // For negative non-integers, the range is [trunc-1, trunc), so high boundary is trunc
            Ok(truncated)
        } else {
            // For integers or positive numbers, high boundary is trunc + 1
            Ok(truncated + Decimal::ONE)
        }
    } else {
        // Calculate the high boundary
        let input_scale = value.scale();
        if scale <= input_scale {
            // Reducing precision
            let truncated = value.trunc_with_scale(scale);
            if value.is_sign_negative() {
                // For negative numbers, high boundary is just truncated (closer to zero)
                Ok(truncated)
            } else {
                // For positive numbers, high boundary is truncated plus one ULP
                let ulp = Decimal::new(1, scale);
                Ok(truncated + ulp)
            }
        } else {
            // Adding precision - the high boundary is the value extended with zeros,
            // then add half ULP at the next digit position
            let extended = value.trunc_with_scale(scale);
            let half_ulp = Decimal::new(5, input_scale + 1); // 0.5 at the next position after original precision
            Ok(extended + half_ulp)
        }
    }
}

/// Calculate high boundary for Date values
/// For dates like "2001-05-06", the high boundary is "2001-05-06T23:59:59.999"
fn calculate_date_high_boundary(date: &NaiveDate) -> FunctionResult<DateTime<FixedOffset>> {
    let datetime =
        date.and_hms_milli_opt(23, 59, 59, 999)
            .ok_or_else(|| FunctionError::EvaluationError {
                name: "highBoundary".to_string(),
                message: "Invalid date for boundary calculation".to_string(),
            })?;

    // Convert to UTC for consistent boundary calculation
    Ok(datetime.and_utc().fixed_offset())
}

/// Calculate high boundary for DateTime values
/// For partial datetimes, fills missing precision with maximum values (59, 999)
fn calculate_datetime_high_boundary(
    datetime: &DateTime<FixedOffset>,
) -> FunctionResult<DateTime<FixedOffset>> {
    // For datetime values, the high boundary fills in missing precision with maximum values
    let naive = datetime.naive_local();
    let original_timezone = datetime.timezone();

    // Create high boundary: if seconds are 0, assume they weren't specified and fill with 59
    let high_boundary_naive = if naive.second() == 0 && naive.nanosecond() == 0 {
        // Seconds and nanoseconds weren't specified, fill with maximum values
        naive
            .with_second(59)
            .and_then(|dt| dt.with_nanosecond(999_000_000)) // 999 milliseconds
            .ok_or_else(|| FunctionError::EvaluationError {
                name: "highBoundary".to_string(),
                message: "Invalid datetime for boundary calculation".to_string(),
            })?
    } else {
        // They were specified, keep the exact value
        naive
    };

    // Reconstruct datetime with original timezone
    original_timezone
        .from_local_datetime(&high_boundary_naive)
        .single()
        .ok_or_else(|| FunctionError::EvaluationError {
            name: "highBoundary".to_string(),
            message: "Invalid datetime for boundary calculation".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_high_boundary_function() {
        let high_boundary_func = UnifiedHighBoundaryFunction::new();

        // Test decimal high boundary
        let context = EvaluationContext::new(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_f64(2.58).unwrap()
        ));
        let result = high_boundary_func.evaluate_sync(&[], &context).unwrap();

        // Should return a decimal
        match result {
            FhirPathValue::Decimal(_) => {
                // Success
            },
            _ => panic!("Expected Decimal result from highBoundary function"),
        }

        // Test with precision argument
        let args = vec![FhirPathValue::Integer(1)];
        let result = high_boundary_func.evaluate_sync(&args, &context).unwrap();

        match result {
            FhirPathValue::Decimal(_) => {
                // Success
            },
            _ => panic!("Expected Decimal result from highBoundary function with precision"),
        }

        // Test with precision 0 (should return integer)
        let args = vec![FhirPathValue::Integer(0)];
        let result = high_boundary_func.evaluate_sync(&args, &context).unwrap();

        match result {
            FhirPathValue::Integer(_) => {
                // Success
            },
            _ => panic!("Expected Integer result from highBoundary function with precision 0"),
        }

        // Test with integer input
        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = high_boundary_func.evaluate_sync(&[], &context).unwrap();

        match result {
            FhirPathValue::Decimal(_) => {
                // Success
            },
            _ => panic!("Expected Decimal result from highBoundary function with integer input"),
        }

        // Test metadata
        assert_eq!(high_boundary_func.name(), "highBoundary");
        assert_eq!(high_boundary_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(high_boundary_func.metadata().basic.display_name, "High Boundary");
        assert!(high_boundary_func.metadata().basic.is_pure);
    }
}
