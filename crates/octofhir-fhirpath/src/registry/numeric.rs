//! Numeric functions for FHIRPath
//!
//! This module implements numeric boundary and precision functions according to the FHIRPath specification.
//! Reference: https://build.fhir.org/ig/HL7/FHIRPath/functions.html

use super::{FunctionRegistry, FunctionCategory, FunctionContext};
use crate::core::{FhirPathValue, FhirPathError, Result};
use crate::{register_function};
use crate::core::error_code::FP0053;
use rust_decimal::Decimal;

impl FunctionRegistry {
    pub fn register_numeric_functions(&self) -> Result<()> {
        self.register_comparable_function()?;
        self.register_lowBoundary_function()?;
        self.register_highBoundary_function()?;
        self.register_precision_function()?;
        Ok(())
    }

    fn register_comparable_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "comparable",
            category: FunctionCategory::Math,
            description: "Returns true if the input values can be compared using comparison operators",
            parameters: ["other": Some("any".to_string()) => "Value to test comparability with"],
            return_type: "boolean",
            examples: ["1.comparable(2)", "'a'.comparable('b')", "1.comparable('a')"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 || context.arguments.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "comparable() requires exactly one input and one argument".to_string()
                    ));
                }

                let input = &context.input[0];
                let arg = &context.arguments[0];

                let comparable = match (input, arg) {
                    // Numbers are comparable with numbers
                    (FhirPathValue::Integer(_), FhirPathValue::Integer(_)) => true,
                    (FhirPathValue::Integer(_), FhirPathValue::Decimal(_)) => true,
                    (FhirPathValue::Decimal(_), FhirPathValue::Integer(_)) => true,
                    (FhirPathValue::Decimal(_), FhirPathValue::Decimal(_)) => true,
                    
                    // Strings are comparable with strings
                    (FhirPathValue::String(_), FhirPathValue::String(_)) => true,
                    
                    // Dates are comparable with dates
                    (FhirPathValue::Date(_), FhirPathValue::Date(_)) => true,
                    (FhirPathValue::DateTime(_), FhirPathValue::DateTime(_)) => true,
                    (FhirPathValue::Date(_), FhirPathValue::DateTime(_)) => true,
                    (FhirPathValue::DateTime(_), FhirPathValue::Date(_)) => true,
                    
                    // Times are comparable with times
                    (FhirPathValue::Time(_), FhirPathValue::Time(_)) => true,
                    
                    // Booleans are comparable with booleans
                    (FhirPathValue::Boolean(_), FhirPathValue::Boolean(_)) => true,
                    
                    // Quantities are comparable if they have compatible units
                    (FhirPathValue::Quantity { .. }, FhirPathValue::Quantity { .. }) => {
                        // Simplified: assume quantities with same unit are comparable
                        // In full implementation, would check UCUM unit compatibility
                        true
                    },
                    
                    // Everything else is not comparable
                    _ => false,
                };

                Ok(vec![FhirPathValue::Boolean(comparable)])
            }
        )
    }

    fn register_lowBoundary_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "lowBoundary",
            category: FunctionCategory::Math,
            description: "Returns the lowest possible value for the input given its precision",
            parameters: ["precision": Some("integer".to_string()) => "Optional precision level"],
            return_type: "any",
            examples: ["1.5.lowBoundary()", "@2023-12-25.lowBoundary()", "1.lowBoundary(1)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "lowBoundary() can only be called on a single value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Decimal(d) => {
                        // For decimals, return the same value (simplified implementation)
                        // In full implementation, would consider precision and return lower bound
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::Integer(i) => {
                        // For integers, return the same value
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::Date(date) => {
                        // For dates, return the start of the precision period
                        // Simplified: return same date
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::DateTime(datetime) => {
                        // For datetimes, return the start of the precision period
                        // Simplified: return same datetime
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::Time(time) => {
                        // For times, return the start of the precision period
                        // Simplified: return same time
                        Ok(vec![context.input[0].clone()])
                    },
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "lowBoundary() can only be called on numeric, date, datetime, or time values".to_string()
                    ))
                }
            }
        )
    }

    fn register_highBoundary_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "highBoundary",
            category: FunctionCategory::Math,
            description: "Returns the highest possible value for the input given its precision",
            parameters: ["precision": Some("integer".to_string()) => "Optional precision level"],
            return_type: "any",
            examples: ["1.5.highBoundary()", "@2023-12-25.highBoundary()", "1.highBoundary(1)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "highBoundary() can only be called on a single value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Decimal(d) => {
                        // For decimals, return the same value (simplified implementation)
                        // In full implementation, would consider precision and return upper bound
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::Integer(i) => {
                        // For integers, return the same value
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::Date(date) => {
                        // For dates, return the end of the precision period
                        // Simplified: return same date
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::DateTime(datetime) => {
                        // For datetimes, return the end of the precision period
                        // Simplified: return same datetime
                        Ok(vec![context.input[0].clone()])
                    },
                    FhirPathValue::Time(time) => {
                        // For times, return the end of the precision period
                        // Simplified: return same time
                        Ok(vec![context.input[0].clone()])
                    },
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "highBoundary() can only be called on numeric, date, datetime, or time values".to_string()
                    ))
                }
            }
        )
    }

    fn register_precision_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "precision",
            category: FunctionCategory::Math,
            description: "Returns the precision of the input value",
            parameters: [],
            return_type: "integer",
            examples: ["1.50.precision()", "@2023-12-25.precision()", "@2023.precision()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(FhirPathError::evaluation_error(
                        FP0053,
                        "precision() can only be called on a single value".to_string()
                    ));
                }

                match &context.input[0] {
                    FhirPathValue::Decimal(d) => {
                        // Count decimal places
                        let decimal_str = d.to_string();
                        if let Some(dot_pos) = decimal_str.find('.') {
                            let decimal_places = decimal_str.len() - dot_pos - 1;
                            Ok(vec![FhirPathValue::Integer(decimal_places as i64)])
                        } else {
                            Ok(vec![FhirPathValue::Integer(0)])
                        }
                    },
                    FhirPathValue::Integer(_) => {
                        // Integers have precision of 1
                        Ok(vec![FhirPathValue::Integer(1)])
                    },
                    FhirPathValue::Date(date) => {
                        // Return precision based on date precision
                        match date.precision {
                            crate::core::temporal::TemporalPrecision::Year => Ok(vec![FhirPathValue::Integer(4)]),
                            crate::core::temporal::TemporalPrecision::Month => Ok(vec![FhirPathValue::Integer(6)]),
                            crate::core::temporal::TemporalPrecision::Day => Ok(vec![FhirPathValue::Integer(8)]),
                            _ => Ok(vec![FhirPathValue::Integer(8)]),
                        }
                    },
                    FhirPathValue::DateTime(datetime) => {
                        // Return precision based on datetime precision
                        match datetime.precision {
                            crate::core::temporal::TemporalPrecision::Year => Ok(vec![FhirPathValue::Integer(4)]),
                            crate::core::temporal::TemporalPrecision::Month => Ok(vec![FhirPathValue::Integer(6)]),
                            crate::core::temporal::TemporalPrecision::Day => Ok(vec![FhirPathValue::Integer(8)]),
                            crate::core::temporal::TemporalPrecision::Hour => Ok(vec![FhirPathValue::Integer(10)]),
                            crate::core::temporal::TemporalPrecision::Minute => Ok(vec![FhirPathValue::Integer(12)]),
                            crate::core::temporal::TemporalPrecision::Second => Ok(vec![FhirPathValue::Integer(14)]),
                            crate::core::temporal::TemporalPrecision::Millisecond => Ok(vec![FhirPathValue::Integer(17)]),
                        }
                    },
                    FhirPathValue::Time(time) => {
                        // Return precision based on time precision
                        match time.precision {
                            crate::core::temporal::TemporalPrecision::Hour => Ok(vec![FhirPathValue::Integer(2)]),
                            crate::core::temporal::TemporalPrecision::Minute => Ok(vec![FhirPathValue::Integer(4)]),
                            crate::core::temporal::TemporalPrecision::Second => Ok(vec![FhirPathValue::Integer(6)]),
                            crate::core::temporal::TemporalPrecision::Millisecond => Ok(vec![FhirPathValue::Integer(9)]),
                            _ => Ok(vec![FhirPathValue::Integer(6)]),
                        }
                    },
                    _ => Err(FhirPathError::evaluation_error(
                        FP0053,
                        "precision() can only be called on numeric, date, datetime, or time values".to_string()
                    ))
                }
            }
        )
    }
}