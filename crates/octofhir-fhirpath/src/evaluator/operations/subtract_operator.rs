//! Subtraction (-) operator implementation
//!
//! Implements FHIRPath subtraction for numeric types and temporal arithmetic.
//! Uses octofhir_ucum for quantity arithmetic and handles temporal arithmetic.

use std::sync::Arc;
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};

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
    fn subtract_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        match (left, right) {
            // Integer subtraction
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                Some(FhirPathValue::integer(l - r))
            }

            // Decimal subtraction
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some(FhirPathValue::decimal(*l - *r))
            }

            // Integer - Decimal = Decimal
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(FhirPathValue::decimal(left_decimal - *r))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(FhirPathValue::decimal(*l - right_decimal))
            }

            // Quantity subtraction - requires same units or compatible units via UCUM
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                if lu == ru {
                    // Same units - simple subtraction
                    Some(FhirPathValue::quantity(*lv - *rv, lu.clone()))
                } else {
                    // Different units - would need UCUM conversion
                    // TODO: Integrate with octofhir_ucum library for unit conversion
                    None
                }
            }

            // Date - Quantity (time-valued) = Date
            (FhirPathValue::Date(date, _, _), FhirPathValue::Quantity { value, unit, .. }) => {
                if let Some(unit_str) = unit {
                    self.subtract_temporal_quantity(
                        &FhirPathValue::date(date.clone()),
                        *value,
                        unit_str
                    )
                } else {
                    None
                }
            }

            // DateTime - Quantity (time-valued) = DateTime
            (FhirPathValue::DateTime(datetime, _, _), FhirPathValue::Quantity { value, unit, .. }) => {
                if let Some(unit_str) = unit {
                    self.subtract_temporal_quantity(
                        &FhirPathValue::datetime(datetime.clone()),
                        *value,
                        unit_str
                    )
                } else {
                    None
                }
            }

            // Time - Quantity (time-valued) = Time
            (FhirPathValue::Time(time, _, _), FhirPathValue::Quantity { value, unit, .. }) => {
                if let Some(unit_str) = unit {
                    self.subtract_temporal_quantity(
                        &FhirPathValue::time(time.clone()),
                        *value,
                        unit_str
                    )
                } else {
                    None
                }
            }

            // Date - Date = Quantity (in days)
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => {
                // TODO: Implement date difference calculation
                // Should return quantity in days
                None
            }

            // DateTime - DateTime = Quantity (in milliseconds)
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                // TODO: Implement datetime difference calculation
                // Should return quantity in milliseconds
                None
            }

            // Time - Time = Quantity (in milliseconds)
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => {
                // TODO: Implement time difference calculation
                // Should return quantity in milliseconds
                None
            }

            // Invalid combinations
            _ => None,
        }
    }

    /// Subtract a time-valued quantity from a temporal value
    fn subtract_temporal_quantity(
        &self,
        temporal: &FhirPathValue,
        quantity_value: Decimal,
        unit: &str,
    ) -> Option<FhirPathValue> {
        // TODO: Implement proper temporal arithmetic using calendar units
        // This requires reading the FHIRPath specification for calendar units

        match unit {
            "year" | "years" => {
                // Subtract years from the temporal value
                // TODO: Implement calendar year subtraction
                None
            }
            "month" | "months" => {
                // Subtract months from the temporal value
                // TODO: Implement calendar month subtraction
                None
            }
            "day" | "days" => {
                // Subtract days from the temporal value
                // TODO: Implement day subtraction
                None
            }
            "hour" | "hours" => {
                // Subtract hours from the temporal value
                // TODO: Implement hour subtraction
                None
            }
            "minute" | "minutes" => {
                // Subtract minutes from the temporal value
                // TODO: Implement minute subtraction
                None
            }
            "second" | "seconds" => {
                // Subtract seconds from the temporal value
                // TODO: Implement second subtraction
                None
            }
            _ => {
                // Unknown time unit
                None
            }
        }
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

        match self.subtract_values(left_value, right_value) {
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
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Integer], FhirPathType::Integer),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Integer], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Quantity, FhirPathType::Quantity], FhirPathType::Quantity),

                // Temporal arithmetic
                TypeSignature::new(vec![FhirPathType::Date, FhirPathType::Quantity], FhirPathType::Date),
                TypeSignature::new(vec![FhirPathType::DateTime, FhirPathType::Quantity], FhirPathType::DateTime),
                TypeSignature::new(vec![FhirPathType::Time, FhirPathType::Quantity], FhirPathType::Time),

                // Temporal differences
                TypeSignature::new(vec![FhirPathType::Date, FhirPathType::Date], FhirPathType::Quantity),
                TypeSignature::new(vec![FhirPathType::DateTime, FhirPathType::DateTime], FhirPathType::Quantity),
                TypeSignature::new(vec![FhirPathType::Time, FhirPathType::Time], FhirPathType::Quantity),
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
        ).await;

        let left = vec![FhirPathValue::integer(8)];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

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
        ).await;

        let left = vec![FhirPathValue::decimal(8.7)];
        let right = vec![FhirPathValue::decimal(3.2)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(Decimal::from_f64_retain(5.5).unwrap()));
    }

    #[tokio::test]
    async fn test_subtract_quantities_same_unit() {
        let evaluator = SubtractOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::quantity(8.0, "kg".to_string())];
        let right = vec![FhirPathValue::quantity(3.0, "kg".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from_f64_retain(5.0).unwrap());
            assert_eq!(*unit, "kg");
        } else {
            panic!("Expected quantity result");
        }
    }
}