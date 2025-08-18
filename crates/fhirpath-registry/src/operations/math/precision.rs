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

//! Precision function implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Precision function - returns the number of digits of precision in a decimal value
#[derive(Debug, Clone)]
pub struct PrecisionFunction;

impl Default for PrecisionFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl PrecisionFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("precision", OperationType::Function)
            .description("Returns the number of digits of precision in a decimal value")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .example("(3.14).precision()")
            .example("(100.0).precision()")
            .build()
    }

    fn calculate_decimal_precision(&self, decimal: &rust_decimal::Decimal) -> i64 {
        // For FHIRPath precision, we need to count ALL significant digits including trailing zeros
        // This is different from mathematical significance - it's about the precision of the representation
        let decimal_str = decimal.to_string();

        // Count all digits excluding the decimal point and negative sign
        // FHIRPath precision includes trailing zeros as they indicate precision
        let digit_count = decimal_str.chars().filter(|c| c.is_ascii_digit()).count();

        digit_count as i64
    }

    fn calculate_date_precision_from_string(&self, date_str: &str) -> i64 {
        // Remove @ prefix if present
        let clean_str = date_str.strip_prefix('@').unwrap_or(date_str);

        // Date precision based on components present
        // YYYY = 4, YYYY-MM = 6, YYYY-MM-DD = 8
        let components: Vec<&str> = clean_str.split('-').collect();
        match components.len() {
            1 => 4, // Year only
            2 => 6, // Year-Month
            3 => 8, // Year-Month-Day
            _ => 4, // Default to year precision
        }
    }

    fn calculate_datetime_precision_from_string(&self, datetime_str: &str) -> i64 {
        // Remove @ prefix if present
        let clean_str = datetime_str.strip_prefix('@').unwrap_or(datetime_str);

        // DateTime precision includes date + time components
        // YYYY-MM-DDTHH:MM:SS.sss = 17 (with milliseconds)
        // YYYY-MM-DDTHH:MM:SS = 14 (seconds)
        // YYYY-MM-DDTHH:MM = 12 (minutes)
        // YYYY-MM-DDTHH = 10 (hours)

        if clean_str.contains('.') {
            // Has milliseconds - count digits after decimal
            17
        } else if clean_str.matches(':').count() == 2 {
            // Has seconds
            14
        } else if clean_str.matches(':').count() == 1 {
            // Has minutes
            12
        } else if clean_str.contains('T') {
            // Has hours
            10
        } else {
            // Date only
            self.calculate_date_precision_from_string(clean_str)
        }
    }

    fn calculate_time_precision_from_string(&self, time_str: &str) -> i64 {
        // Remove @T prefix if present
        let clean_str = time_str
            .strip_prefix("@T")
            .unwrap_or(time_str.strip_prefix("T").unwrap_or(time_str));

        // Time precision based on components
        // THH:MM:SS.sss = 9 (with milliseconds)
        // THH:MM:SS = 6 (seconds)
        // THH:MM = 4 (minutes)
        // THH = 2 (hours)

        if clean_str.contains('.') {
            // Has milliseconds
            9
        } else if clean_str.matches(':').count() == 2 {
            // Has seconds
            6
        } else if clean_str.matches(':').count() == 1 {
            // Has minutes
            4
        } else {
            // Hours only
            2
        }
    }
}

#[async_trait]
impl FhirPathOperation for PrecisionFunction {
    fn identifier(&self) -> &str {
        "precision"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(PrecisionFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(_) => {
                // Integers have 0 decimal places of precision
                Ok(FhirPathValue::Integer(0))
            }
            FhirPathValue::Decimal(d) => {
                let precision = self.calculate_decimal_precision(d);
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Date(date) => {
                // Convert date back to string representation for precision calculation
                let date_str = format!("{}", date.date.format("%Y-%m-%d"));
                let precision = self.calculate_date_precision_from_string(&date_str);
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::DateTime(datetime) => {
                // Convert datetime back to string representation for precision calculation
                let datetime_str =
                    format!("{}", datetime.datetime.format("%Y-%m-%dT%H:%M:%S%.3f%z"));
                let precision = self.calculate_datetime_precision_from_string(&datetime_str);
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Time(time) => {
                // Convert time back to string representation for precision calculation
                let time_str = if time.time.nanosecond() > 0 {
                    format!("{}", time.time.format("%H:%M:%S%.3f"))
                } else {
                    format!("{}", time.time.format("%H:%M:%S"))
                };
                let precision = self.calculate_time_precision_from_string(&time_str);
                Ok(FhirPathValue::Integer(precision))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(
                        c.first().unwrap().clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );
                    self.evaluate(args, &item_context).await
                } else {
                    Err(FhirPathError::TypeError {
                        message: "precision() can only be applied to single values".to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "precision() can only be applied to numeric, date, time, or datetime values, got {}",
                    context.input.type_name()
                ),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            }));
        }

        match &context.input {
            FhirPathValue::Integer(_) => {
                // Integers have 0 decimal places of precision
                Some(Ok(FhirPathValue::Integer(0)))
            }
            FhirPathValue::Decimal(d) => {
                let precision = self.calculate_decimal_precision(d);
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::Date(date) => {
                // Convert date back to string representation for precision calculation
                let date_str = format!("{}", date.date.format("%Y-%m-%d"));
                let precision = self.calculate_date_precision_from_string(&date_str);
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::DateTime(datetime) => {
                // Convert datetime back to string representation for precision calculation
                let datetime_str =
                    format!("{}", datetime.datetime.format("%Y-%m-%dT%H:%M:%S%.3f%z"));
                let precision = self.calculate_datetime_precision_from_string(&datetime_str);
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::Time(time) => {
                // Convert time back to string representation for precision calculation
                let time_str = if time.time.nanosecond() > 0 {
                    format!("{}", time.time.format("%H:%M:%S%.3f"))
                } else {
                    format!("{}", time.time.format("%H:%M:%S"))
                };
                let precision = self.calculate_time_precision_from_string(&time_str);
                Some(Ok(FhirPathValue::Integer(precision)))
            }
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(
                        c.first().unwrap().clone(),
                        context.registry.clone(),
                        context.model_provider.clone(),
                    );
                    self.try_evaluate_sync(args, &item_context)
                } else {
                    Some(Err(FhirPathError::TypeError {
                        message: "precision() can only be applied to single values".to_string(),
                    }))
                }
            }
            _ => Some(Err(FhirPathError::TypeError {
                message: format!(
                    "precision() can only be applied to numeric, date, time, or datetime values, got {}",
                    context.input.type_name()
                ),
            })),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
