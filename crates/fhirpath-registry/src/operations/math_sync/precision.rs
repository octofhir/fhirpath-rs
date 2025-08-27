//! Simplified precision function implementation for FHIRPath

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, temporal::TemporalPrecision};
use rust_decimal::prelude::ToPrimitive;

/// Simplified precision function: returns the number of significant decimal places
pub struct SimplePrecisionFunction;

impl SimplePrecisionFunction {
    pub fn new() -> Self {
        Self
    }

    fn count_decimal_places(&self, value: f64) -> i64 {
        let value_str = format!("{value:.15}");

        if !value_str.contains('.') {
            return 0;
        }

        let parts: Vec<&str> = value_str.split('.').collect();
        if parts.len() != 2 {
            return 0;
        }

        let decimal_part = parts[1].trim_end_matches('0');
        decimal_part.len() as i64
    }

    fn temporal_precision_value(&self, precision: TemporalPrecision) -> i64 {
        match precision {
            TemporalPrecision::Year => 4,         // YYYY
            TemporalPrecision::Month => 6,        // YYYY-MM
            TemporalPrecision::Day => 8,          // YYYY-MM-DD
            TemporalPrecision::Hour => 11,        // YYYY-MM-DDTHH
            TemporalPrecision::Minute => 14,      // YYYY-MM-DDTHH:MM
            TemporalPrecision::Second => 17,      // YYYY-MM-DDTHH:MM:SS
            TemporalPrecision::Millisecond => 21, // YYYY-MM-DDTHH:MM:SS.SSS
        }
    }
}

impl Default for SimplePrecisionFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncOperation for SimplePrecisionFunction {
    fn name(&self) -> &'static str {
        "precision"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "precision",
                parameters: vec![],
                return_type: ValueType::Integer,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "precision".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(_) => Ok(FhirPathValue::Integer(0)),
            FhirPathValue::Decimal(n) => {
                let precision = self.count_decimal_places(n.to_f64().unwrap_or(0.0));
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Quantity(q) => {
                let precision = self.count_decimal_places(q.value.to_f64().unwrap_or(0.0));
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Date(date) => {
                let precision_value = self.temporal_precision_value(date.precision);
                Ok(FhirPathValue::Integer(precision_value))
            }
            FhirPathValue::DateTime(datetime) => {
                let precision_value = self.temporal_precision_value(datetime.precision);
                Ok(FhirPathValue::Integer(precision_value))
            }
            FhirPathValue::Time(time) => {
                let precision_value = self.temporal_precision_value(time.precision);
                Ok(FhirPathValue::Integer(precision_value))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item = c.first().unwrap();
                    let item_context = context.with_input(item.clone());
                    self.execute(args, &item_context)
                } else {
                    Err(FhirPathError::TypeError {
                        message: "precision() can only be applied to single values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "precision() can only be applied to numeric or temporal values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }
}
