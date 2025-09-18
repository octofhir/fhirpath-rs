//! Subtraction (-) operator implementation
//!
//! Implements FHIRPath subtraction for numeric types and temporal arithmetic.
//! Uses octofhir_ucum for quantity arithmetic and handles temporal arithmetic.

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::sync::Arc;

use crate::core::temporal::CalendarDuration;
use crate::core::{Collection, FhirPathError, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Subtraction operator evaluator
pub struct SubtractOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl SubtractOperatorEvaluator {
    /// Create a new subtraction operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_subtract_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform subtraction on two FhirPathValues
    fn subtract_values(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<FhirPathValue>> {
        match (left, right) {
            // Integer subtraction
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                Ok(Some(FhirPathValue::integer(l - r)))
            }

            // Decimal subtraction
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Ok(Some(FhirPathValue::decimal(*l - *r)))
            }

            // Integer - Decimal = Decimal
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Ok(Some(FhirPathValue::decimal(left_decimal - *r)))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Ok(Some(FhirPathValue::decimal(*l - right_decimal)))
            }

            // Quantity subtraction - requires same units or compatible units via UCUM
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    ..
                },
            ) => {
                if lu == ru {
                    // Same units - simple subtraction
                    Ok(Some(FhirPathValue::quantity(*lv - *rv, lu.clone())))
                } else {
                    // Different units - would need UCUM conversion
                    // TODO: Integrate with octofhir_ucum library for unit conversion
                    Ok(None)
                }
            }

            // Date - Quantity (time-valued) = Date
            (
                FhirPathValue::Date(date, _, _),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    ucum_unit,
                    calendar_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    self.subtract_temporal_quantity_with_type(
                        &FhirPathValue::date(date.clone()),
                        *value,
                        unit_str,
                        ucum_unit.is_some(),
                        *calendar_unit,
                    )
                } else {
                    Ok(None)
                }
            }

            // DateTime - Quantity (time-valued) = DateTime
            (
                FhirPathValue::DateTime(datetime, _, _),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    ucum_unit,
                    calendar_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    self.subtract_temporal_quantity_with_type(
                        &FhirPathValue::datetime(datetime.clone()),
                        *value,
                        unit_str,
                        ucum_unit.is_some(),
                        *calendar_unit,
                    )
                } else {
                    Ok(None)
                }
            }

            // Time - Quantity (time-valued) = Time
            (
                FhirPathValue::Time(time, _, _),
                FhirPathValue::Quantity {
                    value,
                    unit,
                    ucum_unit,
                    calendar_unit,
                    ..
                },
            ) => {
                if let Some(unit_str) = unit {
                    self.subtract_temporal_quantity_with_type(
                        &FhirPathValue::time(time.clone()),
                        *value,
                        unit_str,
                        ucum_unit.is_some(),
                        *calendar_unit,
                    )
                } else {
                    Ok(None)
                }
            }

            // Date - Date = Quantity (in days)
            (FhirPathValue::Date(_l, _, _), FhirPathValue::Date(_r, _, _)) => {
                // TODO: Implement date difference calculation
                // Should return quantity in days
                Ok(None)
            }

            // DateTime - DateTime = Quantity (in milliseconds)
            (FhirPathValue::DateTime(_l, _, _), FhirPathValue::DateTime(_r, _, _)) => {
                // TODO: Implement datetime difference calculation
                // Should return quantity in milliseconds
                Ok(None)
            }

            // Time - Time = Quantity (in milliseconds)
            (FhirPathValue::Time(_l, _, _), FhirPathValue::Time(_r, _, _)) => {
                // TODO: Implement time difference calculation
                // Should return quantity in milliseconds
                Ok(None)
            }

            // String subtraction should generate an error
            (FhirPathValue::String(_, _, _), FhirPathValue::String(_, _, _)) => {
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0081,
                    "Cannot subtract strings".to_string(),
                ))
            }

            // Invalid combinations
            _ => Ok(None),
        }
    }

    /// Subtract a time-valued quantity from a temporal value with type information
    fn subtract_temporal_quantity_with_type(
        &self,
        temporal: &FhirPathValue,
        quantity_value: Decimal,
        unit: &str,
        is_ucum_unit: bool,
        calendar_unit: Option<crate::core::CalendarUnit>,
    ) -> Result<Option<FhirPathValue>> {
        use crate::core::CalendarUnit;

        // Check for UCUM units that should trigger execution errors
        // Per FHIRPath spec, only calendar unit names are valid for temporal arithmetic
        if is_ucum_unit {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0081,
                format!(
                    "Cannot subtract UCUM unit '{}' from a temporal value. Use word units instead",
                    unit
                ),
            ));
        }

        // Use the already parsed calendar unit
        let calendar_unit = calendar_unit.ok_or_else(|| {
            FhirPathError::evaluation_error(
                crate::core::error_code::FP0081,
                format!("Unknown calendar unit: {}", unit),
            )
        })?;

        // For subtraction, negate the quantity value
        let subtraction_value = -quantity_value;

        // Per FHIRPath spec: "For precisions above seconds, the decimal portion is ignored"
        // For seconds and milliseconds, we can still handle fractional conversion
        let value = match calendar_unit {
            CalendarUnit::Second | CalendarUnit::Millisecond => {
                if subtraction_value.fract() == Decimal::ZERO {
                    subtraction_value.to_i64().ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0081,
                            "Quantity value out of range".to_string(),
                        )
                    })?
                } else {
                    // For fractional values with fixed-duration units, convert to smaller unit
                    match calendar_unit {
                        CalendarUnit::Second => {
                            // Convert fractional seconds to milliseconds
                            let ms = subtraction_value * Decimal::from(1000);
                            if ms.fract() == Decimal::ZERO {
                                let duration = CalendarDuration::new(
                                    ms.to_i64().ok_or_else(|| {
                                        FhirPathError::evaluation_error(
                                            crate::core::error_code::FP0081,
                                            "Quantity value out of range".to_string(),
                                        )
                                    })?,
                                    CalendarUnit::Millisecond,
                                );
                                return Ok(self.apply_calendar_duration(temporal, duration));
                            } else {
                                return Err(FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0081,
                                    "Cannot handle sub-millisecond precision".to_string(),
                                ));
                            }
                        }
                        _ => {
                            return Err(FhirPathError::evaluation_error(
                                crate::core::error_code::FP0081,
                                "Cannot handle fractional milliseconds".to_string(),
                            ));
                        }
                    }
                }
            }
            _ => {
                // For all other calendar units, truncate fractional values per FHIRPath spec
                subtraction_value.trunc().to_i64().ok_or_else(|| {
                    FhirPathError::evaluation_error(
                        crate::core::error_code::FP0081,
                        "Quantity value out of range".to_string(),
                    )
                })?
            }
        };

        let duration = CalendarDuration::new(value, calendar_unit);
        Ok(self.apply_calendar_duration(temporal, duration))
    }

    /// Apply a calendar duration to a temporal value
    fn apply_calendar_duration(
        &self,
        temporal: &FhirPathValue,
        duration: CalendarDuration,
    ) -> Option<FhirPathValue> {
        match temporal {
            FhirPathValue::Date(precision_date, _, _) => duration
                .add_to_date(precision_date)
                .ok()
                .map(FhirPathValue::date),
            FhirPathValue::DateTime(precision_datetime, _, _) => duration
                .add_to_datetime(precision_datetime)
                .ok()
                .map(FhirPathValue::datetime),
            FhirPathValue::Time(precision_time, _, _) => {
                // For time arithmetic, we need to handle time-only duration addition
                self.add_duration_to_time(precision_time, &duration)
            }
            _ => None,
        }
    }

    /// Add a calendar duration to a time value (handles wrapping around 24 hours)
    fn add_duration_to_time(
        &self,
        time: &crate::core::temporal::PrecisionTime,
        duration: &CalendarDuration,
    ) -> Option<FhirPathValue> {
        use crate::core::temporal::PrecisionTime;
        use chrono::Timelike;

        // Only time units make sense for time addition
        let total_ms = duration.to_milliseconds()?;

        // Convert time to total milliseconds since midnight
        let time_ms = time.time.hour() as i64 * 3_600_000
            + time.time.minute() as i64 * 60_000
            + time.time.second() as i64 * 1_000
            + time.time.nanosecond() as i64 / 1_000_000;

        // Add duration and handle wrap-around
        let new_time_ms = (time_ms + total_ms) % (24 * 3_600_000);
        let positive_time_ms = if new_time_ms < 0 {
            new_time_ms + (24 * 3_600_000)
        } else {
            new_time_ms
        };

        // Convert back to time components
        let hours = (positive_time_ms / 3_600_000) as u32;
        let minutes = ((positive_time_ms % 3_600_000) / 60_000) as u32;
        let seconds = ((positive_time_ms % 60_000) / 1_000) as u32;
        let milliseconds = (positive_time_ms % 1_000) as u32;

        // Create new time
        let nanoseconds = milliseconds * 1_000_000;
        let new_time = chrono::NaiveTime::from_hms_nano_opt(hours, minutes, seconds, nanoseconds)?;
        let precision_time = PrecisionTime::new(new_time, time.precision);

        Some(FhirPathValue::time(precision_time))
    }
}

#[async_trait]
impl OperationEvaluator for SubtractOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if either operand is empty, result is empty
        if left.is_empty() || right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // For arithmetic, we use the first elements (singleton evaluation)
        let left_value = left.first().unwrap();
        let right_value = right.first().unwrap();

        match self.subtract_values(left_value, right_value)? {
            Some(result) => Ok(EvaluationResult {
                value: Collection::single(result),
            }),
            None => Ok(EvaluationResult {
                value: Collection::empty(),
            }),
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the subtraction operator
fn create_subtract_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Any, // Return type depends on operands
    );

    OperatorMetadata {
        name: "-".to_string(),
        description: "Subtraction for numeric types and temporal arithmetic".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                // Numeric subtraction
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Integer],
                    FhirPathType::Integer,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Decimal],
                    FhirPathType::Decimal,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::Decimal],
                    FhirPathType::Decimal,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::Integer],
                    FhirPathType::Decimal,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Quantity, FhirPathType::Quantity],
                    FhirPathType::Quantity,
                ),
                // Temporal arithmetic
                TypeSignature::new(
                    vec![FhirPathType::Date, FhirPathType::Quantity],
                    FhirPathType::Date,
                ),
                TypeSignature::new(
                    vec![FhirPathType::DateTime, FhirPathType::Quantity],
                    FhirPathType::DateTime,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Time, FhirPathType::Quantity],
                    FhirPathType::Time,
                ),
                // Temporal differences
                TypeSignature::new(
                    vec![FhirPathType::Date, FhirPathType::Date],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::DateTime, FhirPathType::DateTime],
                    FhirPathType::Quantity,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Time, FhirPathType::Time],
                    FhirPathType::Quantity,
                ),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 7, // FHIRPath arithmetic precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_subtract_integers() {
        let evaluator = SubtractOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(8)];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(5));
    }

    #[tokio::test]
    async fn test_subtract_decimals() {
        let evaluator = SubtractOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::decimal(8.7)];
        let right = vec![FhirPathValue::decimal(3.2)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_decimal(),
            Some(Decimal::from_f64_retain(5.5).unwrap())
        );
    }

    #[tokio::test]
    async fn test_subtract_quantities_same_unit() {
        let evaluator = SubtractOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::quantity(8.0, "kg".to_string())];
        let right = vec![FhirPathValue::quantity(3.0, "kg".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from_f64_retain(5.0).unwrap());
            assert_eq!(*unit, "kg");
        } else {
            panic!("Expected quantity result");
        }
    }
}
