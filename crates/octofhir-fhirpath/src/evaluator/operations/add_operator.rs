//! Addition (+) operator implementation
//!
//! Implements FHIRPath addition for numeric types and string concatenation.
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

/// Addition operator evaluator
pub struct AddOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl AddOperatorEvaluator {
    /// Create a new addition operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_add_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform addition on two FhirPathValues
    fn add_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        match (left, right) {
            // Integer addition
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                Some(FhirPathValue::integer(l + r))
            }

            // Decimal addition
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some(FhirPathValue::decimal(*l + *r))
            }

            // Integer + Decimal = Decimal
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(FhirPathValue::decimal(left_decimal + *r))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(FhirPathValue::decimal(*l + right_decimal))
            }

            // Quantity addition - requires same units or compatible units via UCUM
            (FhirPathValue::Quantity { value: lv, unit: lu, .. }, FhirPathValue::Quantity { value: rv, unit: ru, .. }) => {
                if lu == ru {
                    // Same units - simple addition
                    Some(FhirPathValue::quantity(*lv + *rv, lu.clone()))
                } else {
                    // Different units - would need UCUM conversion
                    // TODO: Integrate with octofhir_ucum library for unit conversion
                    // For now, return None to indicate incompatible units
                    None
                }
            }

            // Date + Quantity (time-valued) = Date
            (FhirPathValue::Date(date, _, _), FhirPathValue::Quantity { value, unit, .. }) => {
                if let Some(unit_str) = unit {
                    self.add_temporal_quantity(
                        &FhirPathValue::date(date.clone()),
                        *value,
                        unit_str
                    )
                } else {
                    None
                }
            }

            // DateTime + Quantity (time-valued) = DateTime
            (FhirPathValue::DateTime(datetime, _, _), FhirPathValue::Quantity { value, unit, .. }) => {
                if let Some(unit_str) = unit {
                    self.add_temporal_quantity(
                        &FhirPathValue::datetime(datetime.clone()),
                        *value,
                        unit_str
                    )
                } else {
                    None
                }
            }

            // Time + Quantity (time-valued) = Time
            (FhirPathValue::Time(time, _, _), FhirPathValue::Quantity { value, unit, .. }) => {
                if let Some(unit_str) = unit {
                    self.add_temporal_quantity(
                        &FhirPathValue::time(time.clone()),
                        *value,
                        unit_str
                    )
                } else {
                    None
                }
            }

            // String concatenation (+ operator acts as concatenation for strings)
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => {
                Some(FhirPathValue::string(format!("{}{}", l, r)))
            }

            // Invalid combinations
            _ => None,
        }
    }

    /// Add a time-valued quantity to a temporal value
    fn add_temporal_quantity(
        &self,
        temporal: &FhirPathValue,
        quantity_value: Decimal,
        unit: &str,
    ) -> Option<FhirPathValue> {
        // TODO: Implement proper temporal arithmetic using calendar units
        // This requires reading the FHIRPath specification for calendar units
        // and potentially integrating with a temporal library

        match unit {
            "year" | "years" => {
                // Add years to the temporal value
                // TODO: Implement calendar year addition
                None
            }
            "month" | "months" => {
                // Add months to the temporal value
                // TODO: Implement calendar month addition
                None
            }
            "day" | "days" => {
                // Add days to the temporal value
                // TODO: Implement day addition
                None
            }
            "hour" | "hours" => {
                // Add hours to the temporal value
                // TODO: Implement hour addition
                None
            }
            "minute" | "minutes" => {
                // Add minutes to the temporal value
                // TODO: Implement minute addition
                None
            }
            "second" | "seconds" => {
                // Add seconds to the temporal value
                // TODO: Implement second addition
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
impl OperationEvaluator for AddOperatorEvaluator {
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

        match self.add_values(left_value, right_value) {
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

/// Create metadata for the addition operator
fn create_add_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Any, // Return type depends on operands
    );

    OperatorMetadata {
        name: "+".to_string(),
        description: "Addition for numeric types, temporal arithmetic, and string concatenation".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                // Numeric addition
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Integer], FhirPathType::Integer),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Integer, FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Decimal, FhirPathType::Integer], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Quantity, FhirPathType::Quantity], FhirPathType::Quantity),

                // Temporal arithmetic
                TypeSignature::new(vec![FhirPathType::Date, FhirPathType::Quantity], FhirPathType::Date),
                TypeSignature::new(vec![FhirPathType::DateTime, FhirPathType::Quantity], FhirPathType::DateTime),
                TypeSignature::new(vec![FhirPathType::Time, FhirPathType::Quantity], FhirPathType::Time),

                // String concatenation
                TypeSignature::new(vec![FhirPathType::String, FhirPathType::String], FhirPathType::String),
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
    async fn test_add_integers() {
        let evaluator = AddOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![FhirPathValue::integer(3)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(8));
    }

    #[tokio::test]
    async fn test_add_decimals() {
        let evaluator = AddOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::decimal(5.5)];
        let right = vec![FhirPathValue::decimal(3.2)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(Decimal::from_f64_retain(8.7).unwrap()));
    }

    #[tokio::test]
    async fn test_add_integer_decimal() {
        let evaluator = AddOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![FhirPathValue::decimal(3.5)];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_decimal(), Some(Decimal::from_f64_retain(8.5).unwrap()));
    }

    #[tokio::test]
    async fn test_add_strings() {
        let evaluator = AddOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![FhirPathValue::string(" World".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_string(), Some("Hello World".to_string()));
    }

    #[tokio::test]
    async fn test_add_quantities_same_unit() {
        let evaluator = AddOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::quantity(5.0, "kg".to_string())];
        let right = vec![FhirPathValue::quantity(3.0, "kg".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        if let FhirPathValue::Quantity { value, unit, .. } = result.value.first().unwrap() {
            assert_eq!(*value, Decimal::from_f64_retain(8.0).unwrap());
            assert_eq!(*unit, "kg");
        } else {
            panic!("Expected quantity result");
        }
    }

    #[tokio::test]
    async fn test_add_empty_propagation() {
        let evaluator = AddOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(5)];
        let right = vec![]; // Empty collection

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert!(result.value.is_empty());
    }
}