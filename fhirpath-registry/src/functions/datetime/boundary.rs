//! Boundary functions - lowBoundary() and highBoundary() for precision-based bounds

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, FixedOffset, TimeZone, Timelike};

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
                _ => return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Precision parameter must be an integer".to_string(),
                }),
            }
        };

        match &context.input {
            FhirPathValue::Decimal(d) => {
                match calculate_low_boundary(d, precision) {
                    Ok(low_bound) => Ok(FhirPathValue::Decimal(low_bound)),
                    Err(FunctionError::EvaluationError { message, .. }) if message.contains("Precision exceeds maximum") => {
                        Ok(FhirPathValue::Empty)
                    },
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Integer(i) => {
                let decimal = Decimal::from(*i);
                match calculate_low_boundary(&decimal, precision) {
                    Ok(low_bound) => Ok(FhirPathValue::Decimal(low_bound)),
                    Err(FunctionError::EvaluationError { message, .. }) if message.contains("Precision exceeds maximum") => {
                        Ok(FhirPathValue::Empty)
                    },
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Date(d) => {
                let low_bound = calculate_date_low_boundary(d)?;
                Ok(FhirPathValue::DateTime(low_bound))
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
                _ => return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Precision parameter must be an integer".to_string(),
                }),
            }
        };

        match &context.input {
            FhirPathValue::Decimal(d) => {
                match calculate_high_boundary(d, precision) {
                    Ok(high_bound) => Ok(FhirPathValue::Decimal(high_bound)),
                    Err(FunctionError::EvaluationError { message, .. }) if message.contains("Precision exceeds maximum") => {
                        Ok(FhirPathValue::Empty)
                    },
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Integer(i) => {
                let decimal = Decimal::from(*i);
                match calculate_high_boundary(&decimal, precision) {
                    Ok(high_bound) => Ok(FhirPathValue::Decimal(high_bound)),
                    Err(FunctionError::EvaluationError { message, .. }) if message.contains("Precision exceeds maximum") => {
                        Ok(FhirPathValue::Empty)
                    },
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Date(d) => {
                let high_bound = calculate_date_high_boundary(d)?;
                Ok(FhirPathValue::DateTime(high_bound))
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
        // Default precision: use the scale of the input value + 1
        value.scale() + 1
    });

    // Check for maximum precision limit (28 is Decimal's limit)
    if scale > 28 {
        return Err(FunctionError::EvaluationError {
            name: "boundary".to_string(),
            message: "Precision exceeds maximum allowed value".to_string(),
        });
    }

    if scale == 0 {
        // For integer precision, subtract 1 from the integer part
        let truncated = value.trunc();
        Ok(truncated - Decimal::ONE)
    } else if scale >= value.scale() {
        // We're adding precision or keeping same, subtract half ULP at the new scale
        let truncated = value.trunc_with_scale(scale);
        let half_ulp = Decimal::new(5, scale + 1); // 0.5 at the given scale
        Ok(truncated - half_ulp)
    } else {
        // We're reducing precision - truncate to the given scale
        Ok(value.trunc_with_scale(scale))
    }
}

fn calculate_high_boundary(value: &Decimal, precision: Option<u32>) -> FunctionResult<Decimal> {
    let scale = precision.unwrap_or_else(|| {
        // Default precision: use the scale of the input value + 1
        value.scale() + 1
    });

    // Check for maximum precision limit (28 is Decimal's limit)
    if scale > 28 {
        return Err(FunctionError::EvaluationError {
            name: "boundary".to_string(),
            message: "Precision exceeds maximum allowed value".to_string(),
        });
    }

    if scale == 0 {
        // For integer precision, add 1 to the integer part
        let truncated = value.trunc();
        Ok(truncated + Decimal::ONE)
    } else if scale >= value.scale() {
        // We're adding precision or keeping same, add half ULP at the new scale
        let truncated = value.trunc_with_scale(scale);
        let half_ulp = Decimal::new(5, scale + 1); // 0.5 at the given scale
        Ok(truncated + half_ulp)
    } else {
        // We're reducing precision - truncate and add one ULP
        let truncated = value.trunc_with_scale(scale);
        let ulp = Decimal::new(1, scale);
        Ok(truncated + ulp)
    }
}

/// Calculate low boundary for Date values 
/// For dates like "2001-05-06", the low boundary is "2001-05-06T00:00:00.000"
fn calculate_date_low_boundary(date: &NaiveDate) -> FunctionResult<DateTime<FixedOffset>> {
    let datetime = date.and_hms_opt(0, 0, 0)
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
    let datetime = date.and_hms_milli_opt(23, 59, 59, 999)
        .ok_or_else(|| FunctionError::EvaluationError {
            name: "highBoundary".to_string(),
            message: "Invalid date for boundary calculation".to_string(),
        })?;
    
    // Convert to UTC for consistent boundary calculation
    Ok(datetime.and_utc().fixed_offset())
}

/// Calculate low boundary for DateTime values
/// For partial datetimes, fills missing precision with minimum values (0)
fn calculate_datetime_low_boundary(datetime: &DateTime<FixedOffset>) -> FunctionResult<DateTime<FixedOffset>> {
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
    
    Ok(datetime.timezone().from_local_datetime(&low_boundary).single()
        .ok_or_else(|| FunctionError::EvaluationError {
            name: "lowBoundary".to_string(),
            message: "Invalid datetime for boundary calculation".to_string(),
        })?)
}

/// Calculate high boundary for DateTime values  
/// For partial datetimes, fills missing precision with maximum values (59, 999)
fn calculate_datetime_high_boundary(datetime: &DateTime<FixedOffset>) -> FunctionResult<DateTime<FixedOffset>> {
    // For datetime values, the high boundary fills in missing precision with maximum values
    let naive = datetime.naive_local();
    let original_timezone = datetime.timezone();
    
    // Create high boundary: if seconds are 0, assume they weren't specified and fill with 59
    let high_boundary_naive = if naive.second() == 0 && naive.nanosecond() == 0 {
        // Seconds and nanoseconds weren't specified, fill with maximum values
        naive.with_second(59)
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
    Ok(original_timezone.from_local_datetime(&high_boundary_naive).single()
        .ok_or_else(|| FunctionError::EvaluationError {
            name: "highBoundary".to_string(),
            message: "Invalid datetime for boundary calculation".to_string(),
        })?)
}