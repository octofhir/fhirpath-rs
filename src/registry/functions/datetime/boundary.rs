//! Boundary functions - lowBoundary() and highBoundary() for precision-based bounds

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Timelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// lowBoundary() function - returns the lower bound of a value based on precision
pub struct LowBoundaryFunction;

impl FhirPathFunction for LowBoundaryFunction {
    fn name(&self) -> &str {
        "lowBoundary"
    }

    fn human_friendly_name(&self) -> &str {
        "Low Boundary"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "lowBoundary",
                vec![ParameterInfo::optional("precision", TypeInfo::Integer)],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

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
                        Ok(FhirPathValue::Quantity(crate::model::quantity::Quantity::new(
                            new_value,
                            q.unit.clone(),
                        )))
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
}

/// highBoundary() function - returns the upper bound of a value based on precision
pub struct HighBoundaryFunction;

impl FhirPathFunction for HighBoundaryFunction {
    fn name(&self) -> &str {
        "highBoundary"
    }

    fn human_friendly_name(&self) -> &str {
        "High Boundary"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "highBoundary",
                vec![ParameterInfo::optional("precision", TypeInfo::Integer)],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

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
                        Ok(FhirPathValue::Quantity(crate::model::quantity::Quantity::new(
                            new_value,
                            q.unit.clone(),
                        )))
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
