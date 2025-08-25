//! LowBoundary function implementation - sync version

use crate::traits::{SyncOperation, EvaluationContext, validation};
use crate::signature::{FunctionSignature, ValueType};
use chrono::{DateTime, NaiveDate, TimeZone, Datelike, Timelike};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, temporal::{PrecisionDateTime, TemporalPrecision}};
use rust_decimal::{Decimal, prelude::ToPrimitive};

/// LowBoundary function - gets low precision boundary of date/time values
#[derive(Debug, Clone)]
pub struct LowBoundaryFunction;

impl LowBoundaryFunction {
    pub fn new() -> Self {
        Self
    }

    fn get_low_boundary(date: &NaiveDate) -> DateTime<chrono::FixedOffset> {
        // Low boundary of date is start of day (00:00:00.000)
        let start_of_day = date.and_hms_milli_opt(0, 0, 0, 0).unwrap();
        chrono::FixedOffset::east_opt(0).unwrap().from_local_datetime(&start_of_day).unwrap()
    }

    fn get_datetime_low_boundary(datetime: &PrecisionDateTime) -> PrecisionDateTime {
        // Low boundary depends on precision
        match datetime.precision {
            TemporalPrecision::Year => {
                // Start of year: January 1, 00:00:00.000
                let year = datetime.datetime.year();
                let start_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap()
                    .and_hms_milli_opt(0, 0, 0, 0).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&start_of_year).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Month => {
                // Start of month
                let year = datetime.datetime.year();
                let month = datetime.datetime.month();
                let start_of_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap()
                    .and_hms_milli_opt(0, 0, 0, 0).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&start_of_month).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Day => {
                // Start of day
                let date = datetime.datetime.date_naive();
                let start_of_day = date.and_hms_milli_opt(0, 0, 0, 0).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&start_of_day).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Hour => {
                // Start of hour
                let dt = datetime.datetime;
                let start_of_hour = dt.date_naive()
                    .and_hms_milli_opt(dt.hour(), 0, 0, 0).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&start_of_hour).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Minute => {
                // Start of minute
                let dt = datetime.datetime;
                let start_of_minute = dt.date_naive()
                    .and_hms_milli_opt(dt.hour(), dt.minute(), 0, 0).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&start_of_minute).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Second => {
                // Start of second
                let dt = datetime.datetime;
                let start_of_second = dt.date_naive()
                    .and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), 0).unwrap();
                let fixed_dt = datetime.datetime.timezone().from_local_datetime(&start_of_second).unwrap();
                PrecisionDateTime::new(fixed_dt, TemporalPrecision::Millisecond)
            }
            TemporalPrecision::Millisecond => {
                // Already at highest precision
                datetime.clone()
            }
        }
    }

    fn get_numeric_low_boundary(value: f64, precision: usize) -> Result<FhirPathValue> {
        // For FHIRPath boundary functions:
        // The input value represents a range based on its implicit precision
        // For 1.587 (3 decimal places), it represents the range [1.5865, 1.5875)
        // lowBoundary(precision) returns the low boundary of that range at the specified precision
        
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
            // For integer precision, low boundary is value - 0.5
            let low_boundary = (value - 0.5).ceil() as i64;
            Ok(FhirPathValue::Integer(low_boundary))
        } else {
            // Calculate the uncertainty based on the implicit precision
            let implicit_scale = 10_f64.powi(implicit_precision as i32);
            let implicit_half_unit = 0.5 / implicit_scale;
            
            // The low boundary is the input value minus the implicit uncertainty
            let low_boundary = value - implicit_half_unit;
            
            // Format to the requested precision
            let target_scale = 10_f64.powi(precision as i32);
            let rounded_boundary = (low_boundary * target_scale).round() / target_scale;
            
            Ok(FhirPathValue::Decimal(Decimal::try_from(rounded_boundary).map_err(|_| {
                FhirPathError::EvaluationError {
                    message: "Unable to convert low boundary to decimal".into(),
                    expression: None,
                    location: None,
                }
            })?))
        }
    }
}

impl SyncOperation for LowBoundaryFunction {
    fn name(&self) -> &'static str {
        "lowBoundary"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature {
                name: "lowBoundary",
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
                function_name: "lowBoundary".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }
        
        let precision = if args.is_empty() {
            None
        } else {
            Some(validation::extract_integer_arg(args, 0, "lowBoundary", "precision")?)
        };

        let boundary = match &context.input {
            FhirPathValue::Integer(n) => {
                if let Some(prec) = precision {
                    if prec < 0 {
                        return Err(FhirPathError::EvaluationError {
                            message: "lowBoundary() precision must be >= 0".into(),
                            expression: None,
                            location: None,
                        });
                    }
                    Self::get_numeric_low_boundary(*n as f64, prec as usize)?
                } else {
                    // For integers without precision, return the integer itself
                    FhirPathValue::Integer(*n)
                }
            }
            FhirPathValue::Decimal(d) => {
                if let Some(prec) = precision {
                    if prec < 0 {
                        return Err(FhirPathError::EvaluationError {
                            message: "lowBoundary() precision must be >= 0".into(),
                            expression: None,
                            location: None,
                        });
                    }
                    Self::get_numeric_low_boundary(d.to_f64().unwrap_or(0.0), prec as usize)?
                } else {
                    // For decimals without precision, determine current precision and truncate to that
                    let decimal_str = d.to_string();
                    let current_precision = if let Some(dot_pos) = decimal_str.find('.') {
                        decimal_str.len() - dot_pos - 1
                    } else {
                        0
                    };
                    let target_precision = current_precision + 1;
                    Self::get_numeric_low_boundary(d.to_f64().unwrap_or(0.0), target_precision)?
                }
            }
            FhirPathValue::Date(date) => {
                if precision.is_some() {
                    return Err(FhirPathError::EvaluationError {
                        message: "lowBoundary() with precision parameter is not supported for Date values".into(),
                        expression: None,
                        location: None,
                    });
                }
                let low_boundary = Self::get_low_boundary(&date.date);
                FhirPathValue::DateTime(PrecisionDateTime::new(low_boundary, TemporalPrecision::Millisecond))
            }
            FhirPathValue::DateTime(datetime) => {
                if precision.is_some() {
                    return Err(FhirPathError::EvaluationError {
                        message: "lowBoundary() with precision parameter is not supported for DateTime values".into(),
                        expression: None,
                        location: None,
                    });
                }
                let low_boundary = Self::get_datetime_low_boundary(datetime);
                FhirPathValue::DateTime(low_boundary)
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() != 1 {
                    return Err(FhirPathError::EvaluationError {
                        message: "lowBoundary() can only be called on single-item collections".into(),
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
                message: "lowBoundary() can only be called on Date, DateTime, or numeric values".to_string()
            }),
        };

        Ok(boundary)
    }
}

impl Default for LowBoundaryFunction {
    fn default() -> Self {
        Self::new()
    }
}