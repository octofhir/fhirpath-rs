//! Sum function implementation
//!
//! Returns the sum of all numeric items in the input collection.
//! Supports Integer, Decimal, and Quantity types with UCUM unit conversion.
//!
//! ## Specification
//! - **Signature:** `sum() : Integer | Decimal | Quantity`
//! - **Returns:** Sum of all numeric values, or empty if input is empty
//! - **Type Promotion:** Integer collection → Integer result, Decimal → Decimal, mixed → Decimal
//! - **Quantity Handling:** All quantities must have compatible units (same dimension)
//!
//! ## Examples
//! - `{ 1, 2, 3, 4 }.sum()` → `10`
//! - `{ 1.5, 2.5, 3.0 }.sum()` → `7.0`
//! - `{ 1, 2.5, 3 }.sum()` → `6.5` (promoted to Decimal)
//! - `{ 10 'mg', 20 'mg' }.sum()` → `30 'mg'`
//! - `{ 100 'cm', 1 'm' }.sum()` → `200 'cm'` (UCUM conversion)

use std::sync::Arc;

use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::quantity_utils::convert_quantity;
use rust_decimal::Decimal;

/// Sum function evaluator
pub struct SumFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SumFunctionEvaluator {
    /// Create a new sum function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "sum".to_string(),
                description: "Returns the sum of all numeric items in the collection".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Integer | Decimal | Quantity".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Custom,
                empty_propagation: EmptyPropagation::Custom,
                deterministic: true,
                category: FunctionCategory::Aggregate,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Add two values, handling type promotion and UCUM conversion
    fn add_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        match (left, right) {
            // Integer + Integer = Integer
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                Some(FhirPathValue::integer(l + r))
            }

            // Decimal + Decimal = Decimal
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some(FhirPathValue::decimal(*l + *r))
            }

            // Integer + Decimal = Decimal (type promotion)
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(FhirPathValue::decimal(left_decimal + *r))
            }

            // Decimal + Integer = Decimal (type promotion)
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(FhirPathValue::decimal(*l + right_decimal))
            }

            // Quantity + Quantity with same or compatible units
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    code: lc,
                    system: ls,
                    ucum_unit: lu_ucum,
                    calendar_unit: lc_unit,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    code: rc,
                    ucum_unit: ru_ucum,
                    calendar_unit: rc_unit,
                    ..
                },
            ) => {
                if lu == ru {
                    // Same units - direct addition
                    Some(FhirPathValue::quantity_with_components(
                        *lv + *rv,
                        lu.clone(),
                        lc.clone(),
                        ls.clone(),
                    ))
                } else if lu_ucum.is_some() && ru_ucum.is_some() {
                    // Different units but both have UCUM - attempt conversion
                    // Get target unit (left) - prefer code, fallback to unit string
                    let target_unit = lc.as_deref().or(lu.as_deref())?;

                    // Get source unit (right) - prefer code, fallback to unit string
                    let source_unit = rc.as_ref().or(ru.as_ref())?;

                    // Try to convert right quantity to left unit
                    let converted = convert_quantity(
                        *rv,
                        &Some(source_unit.clone()),
                        &None, // calendar_unit - we know both are UCUM units
                        target_unit,
                    )
                    .ok()?;

                    // Perform addition in left unit
                    Some(FhirPathValue::quantity_with_components(
                        *lv + converted.value,
                        lu.clone(),
                        lc.clone(),
                        ls.clone(),
                    ))
                } else if lc_unit.is_some() && rc_unit.is_some() {
                    // Calendar units - convert if compatible
                    let target_unit = lc.as_deref().or(lu.as_deref())?;
                    let source_unit = rc.as_ref().or(ru.as_ref())?;

                    let converted =
                        convert_quantity(*rv, &Some(source_unit.clone()), rc_unit, target_unit)
                            .ok()?;

                    Some(FhirPathValue::quantity_with_components(
                        *lv + converted.value,
                        lu.clone(),
                        lc.clone(),
                        ls.clone(),
                    ))
                } else {
                    // Different units without UCUM or calendar - incompatible
                    None
                }
            }

            // All other combinations are incompatible
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for SumFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        // Empty collection returns empty
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Filter out empty values from the collection
        let values: Vec<&FhirPathValue> = input
            .iter()
            .filter(|v| !matches!(v, FhirPathValue::Empty))
            .collect();

        if values.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Single value - return it directly
        if values.len() == 1 {
            return Ok(EvaluationResult {
                value: Collection::single((*values[0]).clone()),
            });
        }

        // Start with the first value as accumulator
        let mut accumulator = (*values[0]).clone();

        // Add each subsequent value to the accumulator
        for value in values.iter().skip(1) {
            match self.add_values(&accumulator, value) {
                Some(result) => accumulator = result,
                None => {
                    // Type mismatch or incompatible units - return empty
                    return Ok(EvaluationResult {
                        value: Collection::empty(),
                    });
                }
            }
        }

        Ok(EvaluationResult {
            value: Collection::single(accumulator),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_sum_integers() {
        let evaluator = SumFunctionEvaluator::create();

        // {1, 2, 3, 4}.sum() = 10
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
            FhirPathValue::integer(3),
            FhirPathValue::integer(4),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(10)));
    }

    #[tokio::test]
    async fn test_sum_decimals() {
        let evaluator = SumFunctionEvaluator::create();

        // {1.5, 2.5, 3.0}.sum() = 7.0
        let input = vec![
            FhirPathValue::decimal(dec!(1.5)),
            FhirPathValue::decimal(dec!(2.5)),
            FhirPathValue::decimal(dec!(3.0)),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first(),
            Some(&FhirPathValue::decimal(dec!(7.0)))
        );
    }

    #[tokio::test]
    async fn test_sum_mixed_integer_decimal() {
        let evaluator = SumFunctionEvaluator::create();

        // {1, 2.5, 3}.sum() = 6.5 (promoted to Decimal)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::decimal(dec!(2.5)),
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first(),
            Some(&FhirPathValue::decimal(dec!(6.5)))
        );
    }

    #[tokio::test]
    async fn test_sum_empty_collection() {
        let evaluator = SumFunctionEvaluator::create();

        // {}.sum() = {}
        let input = vec![];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_sum_single_value() {
        let evaluator = SumFunctionEvaluator::create();

        // {42}.sum() = 42
        let input = vec![FhirPathValue::integer(42)];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(42)));
    }

    #[tokio::test]
    async fn test_sum_with_empty_values() {
        let evaluator = SumFunctionEvaluator::create();

        // {1, {}, 3}.sum() = 4 (skip empty values)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::Empty,
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(4)));
    }

    #[tokio::test]
    async fn test_sum_type_mismatch() {
        let evaluator = SumFunctionEvaluator::create();

        // {1, 'string', 3}.sum() = {} (type mismatch)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::string("string"),
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_sum_quantities_same_unit() {
        let evaluator = SumFunctionEvaluator::create();

        // {10 'mg', 20 'mg', 30 'mg'}.sum() = 60 'mg'
        let input = vec![
            FhirPathValue::quantity_with_components(
                dec!(10),
                Some("mg".to_string()),
                Some("mg".to_string()),
                Some("http://unitsofmeasure.org".to_string()),
            ),
            FhirPathValue::quantity_with_components(
                dec!(20),
                Some("mg".to_string()),
                Some("mg".to_string()),
                Some("http://unitsofmeasure.org".to_string()),
            ),
            FhirPathValue::quantity_with_components(
                dec!(30),
                Some("mg".to_string()),
                Some("mg".to_string()),
                Some("http://unitsofmeasure.org".to_string()),
            ),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);

        if let Some(FhirPathValue::Quantity { value, unit, .. }) = result.value.first() {
            assert_eq!(*value, dec!(60));
            assert_eq!(unit, &Some("mg".to_string()));
        } else {
            panic!("Expected Quantity result");
        }
    }
}
