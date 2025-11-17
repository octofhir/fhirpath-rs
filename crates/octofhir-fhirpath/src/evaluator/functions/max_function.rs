//! Max function implementation
//!
//! Returns the largest value in the input collection.
//! Supports Integer, Decimal, Quantity, Date, DateTime, Time, and String types.
//!
//! ## Specification
//! - **Signature:** `max() : Integer | Decimal | Quantity | Date | DateTime | Time | String`
//! - **Returns:** Maximum value from collection, or empty if input is empty
//! - **Type Support:** All items must be of the same comparable type
//! - **Comparison:** Uses standard comparison semantics for each type
//!
//! ## Examples
//! - `{ 3, 1, 4, 1, 5 }.max()` → `5`
//! - `{ 3.5, 1.2, 4.7 }.max()` → `4.7`
//! - `{ 'apple', 'banana', 'cherry' }.max()` → `'cherry'`
//! - `{ @2024-01-15, @2024-12-31 }.max()` → `@2024-12-31`
//! - `{ 30 'mg', 10 'mg', 20 'mg' }.max()` → `30 'mg'`

use std::cmp::Ordering;
use std::sync::Arc;

use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::EvaluationResult;

/// Max function evaluator
pub struct MaxFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl MaxFunctionEvaluator {
    /// Create a new max function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "max".to_string(),
                description: "Returns the largest value in the collection".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Integer | Decimal | Quantity | Date | DateTime | Time | String"
                        .to_string(),
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
impl PureFunctionEvaluator for MaxFunctionEvaluator {
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

        // Find the maximum value using partial_cmp
        let mut max_value = (*values[0]).clone();

        for value in values.iter().skip(1) {
            match max_value.partial_cmp(value) {
                Some(Ordering::Less) => {
                    // Current value is larger than max, update max
                    max_value = (*value).clone();
                }
                Some(Ordering::Greater) | Some(Ordering::Equal) => {
                    // max_value is still larger or equal, keep it
                }
                None => {
                    // Values are not comparable (e.g., different types or incompatible units)
                    return Ok(EvaluationResult {
                        value: Collection::empty(),
                    });
                }
            }
        }

        Ok(EvaluationResult {
            value: Collection::single(max_value),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::temporal::PrecisionDate;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_max_integers() {
        let evaluator = MaxFunctionEvaluator::create();

        // {3, 1, 4, 1, 5}.max() = 5
        let input = vec![
            FhirPathValue::integer(3),
            FhirPathValue::integer(1),
            FhirPathValue::integer(4),
            FhirPathValue::integer(1),
            FhirPathValue::integer(5),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(5)));
    }

    #[tokio::test]
    async fn test_max_negative_integers() {
        let evaluator = MaxFunctionEvaluator::create();

        // {-5, 0, 5}.max() = 5
        let input = vec![
            FhirPathValue::integer(-5),
            FhirPathValue::integer(0),
            FhirPathValue::integer(5),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(5)));
    }

    #[tokio::test]
    async fn test_max_decimals() {
        let evaluator = MaxFunctionEvaluator::create();

        // {3.5, 1.2, 4.7}.max() = 4.7
        let input = vec![
            FhirPathValue::decimal(dec!(3.5)),
            FhirPathValue::decimal(dec!(1.2)),
            FhirPathValue::decimal(dec!(4.7)),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first(),
            Some(&FhirPathValue::decimal(dec!(4.7)))
        );
    }

    #[tokio::test]
    async fn test_max_negative_decimals() {
        let evaluator = MaxFunctionEvaluator::create();

        // {-1.5, -2.5}.max() = -1.5
        let input = vec![
            FhirPathValue::decimal(dec!(-1.5)),
            FhirPathValue::decimal(dec!(-2.5)),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first(),
            Some(&FhirPathValue::decimal(dec!(-1.5)))
        );
    }

    #[tokio::test]
    async fn test_max_strings() {
        let evaluator = MaxFunctionEvaluator::create();

        // {'apple', 'banana', 'cherry'}.max() = 'cherry'
        let input = vec![
            FhirPathValue::string("apple"),
            FhirPathValue::string("banana"),
            FhirPathValue::string("cherry"),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first(),
            Some(&FhirPathValue::string("cherry"))
        );
    }

    #[tokio::test]
    async fn test_max_empty_collection() {
        let evaluator = MaxFunctionEvaluator::create();

        // {}.max() = {}
        let input = vec![];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_max_single_value() {
        let evaluator = MaxFunctionEvaluator::create();

        // {42}.max() = 42
        let input = vec![FhirPathValue::integer(42)];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(42)));
    }

    #[tokio::test]
    async fn test_max_with_empty_values() {
        let evaluator = MaxFunctionEvaluator::create();

        // {1, {}, 3}.max() = 3 (skip empty values)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::Empty,
            FhirPathValue::integer(3),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first(), Some(&FhirPathValue::integer(3)));
    }

    #[tokio::test]
    async fn test_max_type_mismatch() {
        let evaluator = MaxFunctionEvaluator::create();

        // {1, 'string'}.max() = {} (type mismatch)
        let input = vec![
            FhirPathValue::integer(1),
            FhirPathValue::string("string"),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert!(result.value.is_empty());
    }

    #[tokio::test]
    async fn test_max_quantities_same_unit() {
        let evaluator = MaxFunctionEvaluator::create();

        // {30 'mg', 10 'mg', 20 'mg'}.max() = 30 'mg'
        let input = vec![
            FhirPathValue::quantity_with_components(
                dec!(30),
                Some("mg".to_string()),
                Some("mg".to_string()),
                Some("http://unitsofmeasure.org".to_string()),
            ),
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
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);

        if let Some(FhirPathValue::Quantity { value, unit, .. }) = result.value.first() {
            assert_eq!(*value, dec!(30));
            assert_eq!(unit, &Some("mg".to_string()));
        } else {
            panic!("Expected Quantity result");
        }
    }

    #[tokio::test]
    async fn test_max_dates() {
        let evaluator = MaxFunctionEvaluator::create();

        // {@2024-01-15, @2024-01-01, @2024-12-31}.max() = @2024-12-31
        let input = vec![
            FhirPathValue::date(PrecisionDate::from_date(
                NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            )),
            FhirPathValue::date(PrecisionDate::from_date(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )),
            FhirPathValue::date(PrecisionDate::from_date(
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            )),
        ];
        let result = evaluator.evaluate(input, vec![]).await.unwrap();
        assert_eq!(result.value.len(), 1);

        if let Some(FhirPathValue::Date(date, _, _)) = result.value.first() {
            assert_eq!(date.to_string(), "2024-12-31");
        } else {
            panic!("Expected Date result");
        }
    }
}
