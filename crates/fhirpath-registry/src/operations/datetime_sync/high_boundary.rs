//! HighBoundary function implementation - sync version

use crate::traits::{SyncOperation, EvaluationContext, validation};
use crate::signature::{FunctionSignature, ValueType, ParameterType};
use chrono::{DateTime, NaiveDate, TimeZone, Datelike, Timelike};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, temporal::{PrecisionDateTime, TemporalPrecision}};
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// HighBoundary function - gets high precision boundary of date/time values
#[derive(Debug, Clone)]
pub struct HighBoundaryFunction;

impl HighBoundaryFunction {
    pub fn new() -> Self {
        Self
    }

    fn get_high_boundary(date: &NaiveDate) -> DateTime<chrono::FixedOffset> {
        // High boundary of date is end of day (23:59:59.999)
        let end_of_day = date.and_hms_milli_opt(23, 59, 59, 999).unwrap();
        chrono::FixedOffset::east_opt(0).unwrap().from_local_datetime(&end_of_day).unwrap()
    }

    fn get_datetime_high_boundary(datetime: &PrecisionDateTime) -> PrecisionDateTime {
        // High boundary depends on precision
        match datetime.precision {
            TemporalPrecision::Year => {
                // End of year: December 31, 23:59:59.999
                let year = datetime.datetime.year();
                let end_of_year = NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
                    .and_hms_milli_opt(23, 59, 59, 999).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&end_of_year).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Month => {
                // End of month
                let year = datetime.datetime.year();
                let month = datetime.datetime.month();
                let last_day = if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap().pred_opt().unwrap()
                } else {
                    NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap().pred_opt().unwrap()
                };
                let end_of_month = last_day.and_hms_milli_opt(23, 59, 59, 999).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&end_of_month).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Day => {
                // End of day
                let date = datetime.datetime.date_naive();
                let end_of_day = date.and_hms_milli_opt(23, 59, 59, 999).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&end_of_day).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Hour => {
                // End of hour
                let dt = datetime.datetime;
                let end_of_hour = dt.date_naive()
                    .and_hms_milli_opt(dt.hour(), 59, 59, 999).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&end_of_hour).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Minute => {
                // End of minute
                let dt = datetime.datetime;
                let end_of_minute = dt.date_naive()
                    .and_hms_milli_opt(dt.hour(), dt.minute(), 59, 999).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&end_of_minute).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Second => {
                // End of second
                let dt = datetime.datetime;
                let end_of_second = dt.date_naive()
                    .and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), 999).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&end_of_second).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Millisecond => {
                // Already at highest precision
                datetime.clone()
            }
        }
    }

    fn get_numeric_high_boundary(value: f64, precision: usize) -> Result<FhirPathValue> {
        // For FHIRPath boundary functions:
        // The input value represents a range based on its implicit precision
        // For 1.587 (3 decimal places), it represents the range [1.5865, 1.5875)
        // highBoundary(precision) returns the high boundary of that range at the specified precision
        
        if precision > 28 {
            // Return empty for very high precision (per test expectations)
            return Ok(FhirPathValue::Empty);
        }
        
        // Determine the implicit precision of the input value
        let value_str = format!("{}", value);
        let implicit_precision = if let Some(dot_pos) = value_str.find('.') {
            value_str.len() - dot_pos - 1
        } else {
            0
        };
        
        if precision == 0 {
            // For integer precision, high boundary is value + 0.5, but we return the highest integer
            let high_boundary = (value + 0.5).floor() as i64;
            Ok(FhirPathValue::Integer(high_boundary))
        } else {
            // Calculate the uncertainty based on the implicit precision
            let implicit_scale = 10_f64.powi(implicit_precision as i32);
            let implicit_half_unit = 0.5 / implicit_scale;
            
            // The high boundary is the input value plus the implicit uncertainty (exclusive)
            // But since FHIRPath wants the inclusive high boundary, we subtract a tiny amount
            let high_boundary = value + implicit_half_unit - (implicit_half_unit * 0.0001);
            
            // Format to the requested precision
            let target_scale = 10_f64.powi(precision as i32);
            let rounded_boundary = (high_boundary * target_scale).round() / target_scale;
            
            Ok(FhirPathValue::Decimal(Decimal::try_from(rounded_boundary).map_err(|_| {
                FhirPathError::EvaluationError {
                    message: "Unable to convert high boundary to decimal".into(),
                    expression: None,
                    location: None,
                }
            })?))
        }
    }
}

impl SyncOperation for HighBoundaryFunction {
    fn name(&self) -> &'static str {
        "highBoundary"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "highBoundary",
                parameters: vec![], // No required parameters, precision is optional
                return_type: ValueType::Any,
                variadic: true, // Allow 0 or 1 arguments
            }
        });
        &SIGNATURE
    }

    fn execute(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Precision parameter is optional
        if args.len() > 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "highBoundary".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        
        let precision = if args.is_empty() {
            None
        } else {
            Some(validation::extract_integer_arg(args, 0, "highBoundary", "precision")?)
        };

        let boundary = match &context.input {
            FhirPathValue::Integer(n) => {
                if let Some(prec) = precision {
                    if prec < 0 {
                        return Err(FhirPathError::EvaluationError {
                            message: "highBoundary() precision must be >= 0".into(),
                            expression: None,
                            location: None,
                        });
                    }
                    Self::get_numeric_high_boundary(*n as f64, prec as usize)?
                } else {
                    // For integers without precision, return the integer itself
                    FhirPathValue::Integer(*n)
                }
            }
            FhirPathValue::Decimal(d) => {
                if let Some(prec) = precision {
                    if prec < 0 {
                        return Err(FhirPathError::EvaluationError {
                            message: "highBoundary() precision must be >= 0".into(),
                            expression: None,
                            location: None,
                        });
                    }
                    Self::get_numeric_high_boundary(d.to_f64().unwrap_or(0.0), prec as usize)?
                } else {
                    // For decimals without precision, determine current precision and add one digit
                    let decimal_str = d.to_string();
                    let current_precision = if let Some(dot_pos) = decimal_str.find('.') {
                        decimal_str.len() - dot_pos - 1
                    } else {
                        0
                    };
                    let target_precision = current_precision + 1;
                    Self::get_numeric_high_boundary(d.to_f64().unwrap_or(0.0), target_precision)?
                }
            }
            FhirPathValue::Date(date) => {
                if precision.is_some() {
                    return Err(FhirPathError::EvaluationError {
                        message: "highBoundary() with precision parameter is not supported for Date values".into(),
                        expression: None,
                        location: None,
                    });
                }
                let high_boundary = Self::get_high_boundary(&date.date);
                FhirPathValue::DateTime(PrecisionDateTime::new(high_boundary, TemporalPrecision::Millisecond))
            }
            FhirPathValue::DateTime(datetime) => {
                if precision.is_some() {
                    return Err(FhirPathError::EvaluationError {
                        message: "highBoundary() with precision parameter is not supported for DateTime values".into(),
                        expression: None,
                        location: None,
                    });
                }
                let high_boundary = Self::get_datetime_high_boundary(datetime);
                FhirPathValue::DateTime(high_boundary)
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() != 1 {
                    return Err(FhirPathError::EvaluationError {
                        message: "highBoundary() can only be called on single-item collections".into(),
                        expression: None,
                        location: None,
                    });
                }
                let item = items.first().unwrap();
                let context_with_item = EvaluationContext {
                    input: item.clone(),
                    root: context.root.clone(),
                    model_provider: context.model_provider.clone(),
                    variables: context.variables.clone(),
                };
                return self.execute(args, &context_with_item);
            }
            _ => return Err(FhirPathError::TypeError {
                message: "highBoundary() can only be called on Date, DateTime, or numeric values".to_string()
            }),
        };

        Ok(boundary)
    }
}

impl Default for HighBoundaryFunction {
    fn default() -> Self {
        Self::new()
    }
}