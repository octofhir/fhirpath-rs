//! Avg function implementation
//!
//! Returns the average of all numeric items in the input collection.
//! Supports Integer, Decimal, and Quantity types.
//! Always returns Decimal (for numeric types) or Quantity (for quantities).
//!
//! ## Specification
//! - **Signature:** `avg() : Decimal | Quantity`
//! - **Returns:** Average value from collection, or empty if input is empty
//! - **Type Support:** Integer (converted to Decimal), Decimal, Quantity
//! - **Calculation:** Sum of all values divided by count of non-empty values
//!
//! ## Examples
//! - `{ 1, 2, 3, 4, 5 }.avg()` → `3.0`
//! - `{ 1, 2 }.avg()` → `1.5`
//! - `{ 1.5, 2.5, 3.0 }.avg()` → `2.333...`
//! - `{ 10 'mg', 20 'mg', 30 'mg' }.avg()` → `20 'mg'`

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::EvaluationResult;

/// Avg function evaluator
pub struct AvgFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl AvgFunctionEvaluator {
    /// Create a new avg function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "avg".to_string(),
                description: "Returns the average of all numeric items in the collection"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Decimal | Quantity".to_string(),
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
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for AvgFunctionEvaluator {
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
        let values: Vec<FhirPathValue> = input
            .into_iter()
            .filter(|v| !matches!(v, FhirPathValue::Empty))
            .collect();

        if values.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Count of non-empty values for division
        let count = Decimal::from(values.len());

        // Calculate sum using the same logic as sum_function
        let sum_result = Self::calculate_sum(&values)?;

        // Divide sum by count to get average
        let avg_value = match sum_result {
            FhirPathValue::Integer(i, ..) => {
                // Convert Integer to Decimal for average
                let decimal_sum = Decimal::from(i);
                FhirPathValue::decimal(decimal_sum / count)
            }
            FhirPathValue::Decimal(d, ..) => FhirPathValue::decimal(d / count),
            FhirPathValue::Quantity {
                value,
                unit,
                code,
                system,
                ..
            } => FhirPathValue::quantity_with_components(
                value / count,
                unit,
                code,
                system,
            ),
            _ => {
                // Should not happen if sum calculation is correct
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
        };

        Ok(EvaluationResult {
            value: Collection::single(avg_value),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl AvgFunctionEvaluator {
    /// Calculate sum of values (similar to sum_function logic)
    fn calculate_sum(values: &[FhirPathValue]) -> Result<FhirPathValue> {
        if values.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let mut accumulator = values[0].clone();

        for value in values.iter().skip(1) {
            accumulator = match Self::add_values(&accumulator, value) {
                Some(result) => result,
                None => return Ok(FhirPathValue::Empty), // Type mismatch or incompatible units
            };
        }

        Ok(accumulator)
    }

    /// Add two FhirPathValue items (handles type promotion and unit conversion)
    fn add_values(left: &FhirPathValue, right: &FhirPathValue) -> Option<FhirPathValue> {
        use crate::evaluator::quantity_utils::convert_quantity;

        match (left, right) {
            // Integer + Integer
            (FhirPathValue::Integer(l, ..), FhirPathValue::Integer(r, ..)) => {
                Some(FhirPathValue::integer(l + r))
            }

            // Integer + Decimal or Decimal + Integer (promote to Decimal)
            (FhirPathValue::Integer(l, ..), FhirPathValue::Decimal(r, ..))
            | (FhirPathValue::Decimal(r, ..), FhirPathValue::Integer(l, ..)) => {
                Some(FhirPathValue::decimal(Decimal::from(*l) + r))
            }

            // Decimal + Decimal
            (FhirPathValue::Decimal(l, ..), FhirPathValue::Decimal(r, ..)) => {
                Some(FhirPathValue::decimal(l + r))
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
                    let target_unit = lc.as_deref().or(lu.as_deref())?;
                    let source_unit = rc.as_ref().or(ru.as_ref())?;

                    let converted = convert_quantity(
                        *rv,
                        &Some(source_unit.clone()),
                        &None,
                        target_unit,
                    )
                    .ok()?;

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

                    let converted = convert_quantity(
                        *rv,
                        &Some(source_unit.clone()),
                        rc_unit,
                        target_unit,
                    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_avg_integers() {
        let evaluator = AvgFunctionEvaluator::create();

        // {1, 2, 3, 4, 5}.avg() = 3.0
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::integer(2),
            FhirPathValue::integer(3),
            FhirPathValue::integer(4),
            FhirPathValue::integer(5),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::decimal(dec!(3.0))));
    }

    #[tokio::test]
    async fn test_avg_integers_fractional() {
        let evaluator = AvgFunctionEvaluator::create();

        // {1, 2}.avg() = 1.5
        let input = vec![FhirPathValue::integer(1), FhirPathValue::integer(2)];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::decimal(dec!(1.5))));
    }

    #[tokio::test]
    async fn test_avg_single_integer() {
        let evaluator = AvgFunctionEvaluator::create();

        // {10}.avg() = 10.0
        let input = vec![FhirPathValue::integer(10)];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first(),
            Some(&FhirPathValue::decimal(dec!(10.0)))
        );
    }

    #[tokio::test]
    async fn test_avg_decimals() {
        let evaluator = AvgFunctionEvaluator::create();

        // {1.5, 2.5, 3.0}.avg() = 2.333...
        let input = vec![
            FhirPathValue::decimal(dec!(1.5)),
            FhirPathValue::decimal(dec!(2.5)),
            FhirPathValue::decimal(dec!(3.0)),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);

        if let Some(FhirPathValue::Decimal(d, ..)) = result.value.first() {
            // 7.0 / 3 = 2.333...
            let expected = dec!(7.0) / dec!(3);
            assert_eq!(*d, expected);
        } else {
            panic!("Expected Decimal result");
        }
    }

    #[tokio::test]
    async fn test_avg_decimals_simple() {
        let evaluator = AvgFunctionEvaluator::create();

        // {0.1, 0.2, 0.3}.avg() = 0.2
        let input = vec![
            FhirPathValue::decimal(dec!(0.1)),
            FhirPathValue::decimal(dec!(0.2)),
            FhirPathValue::decimal(dec!(0.3)),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::decimal(dec!(0.2))));
    }

    #[tokio::test]
    async fn test_avg_mixed_integer_decimal() {
        let evaluator = AvgFunctionEvaluator::create();

        // {1, 2.5, 3}.avg() = 2.166...
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::decimal(dec!(2.5)),
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);

        if let Some(FhirPathValue::Decimal(d, ..)) = result.value.first() {
            // (1 + 2.5 + 3) / 3 = 6.5 / 3 = 2.166...
            let expected = dec!(6.5) / dec!(3);
            assert_eq!(*d, expected);
        } else {
            panic!("Expected Decimal result");
        }
    }

    #[tokio::test]
    async fn test_avg_empty_collection() {
        let evaluator = AvgFunctionEvaluator::create();

        // {}.avg() = {}
        let input = vec![];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_avg_with_empty_values() {
        let evaluator = AvgFunctionEvaluator::create();

        // {1, {}, 3}.avg() = 2.0 (skip empty, count = 2)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::Empty,
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::decimal(dec!(2.0))));
    }

    #[tokio::test]
    async fn test_avg_type_mismatch() {
        let evaluator = AvgFunctionEvaluator::create();

        // {1, 'string', 3}.avg() = {} (type mismatch)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::string("string"),
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_avg_quantities_same_unit() {
        let evaluator = AvgFunctionEvaluator::create();

        // {10 'mg', 20 'mg', 30 'mg'}.avg() = 20 'mg'
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
            assert_eq!(*value, dec!(20));
            assert_eq!(unit, &Some("mg".to_string()));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_avg_negative_integers() {
        let evaluator = AvgFunctionEvaluator::create();

        // {-5, 0, 5}.avg() = 0.0
        let input = vec![
            FhirPathValue::integer(-5),
            FhirPathValue::integer(0),
            FhirPathValue::integer(5),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::decimal(dec!(0.0))));
    }
}
