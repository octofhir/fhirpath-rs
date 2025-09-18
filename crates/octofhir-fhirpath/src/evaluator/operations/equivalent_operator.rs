//! Equivalent (~) operator implementation
//!
//! Implements FHIRPath equivalence comparison which is similar to equality
//! but has different handling of empty values and string case sensitivity.

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::core::model_provider::TypeInfo;
use crate::evaluator::quantity_utils;
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Equivalent operator evaluator
pub struct EquivalentOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl EquivalentOperatorEvaluator {
    /// Create a new equivalent operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_equivalent_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Get the decimal precision (number of digits after the decimal point)
    /// Trailing zeroes are ignored in determining precision according to FHIRPath spec
    fn get_decimal_precision(&self, decimal: &Decimal) -> u32 {
        let decimal_str = decimal.to_string();
        if let Some(dot_pos) = decimal_str.find('.') {
            let fractional_part = &decimal_str[dot_pos + 1..];
            // Remove trailing zeros to get actual precision
            let trimmed = fractional_part.trim_end_matches('0');
            trimmed.len() as u32
        } else {
            0
        }
    }

    /// Round a decimal to the specified number of decimal places
    fn round_decimal(&self, decimal: &Decimal, scale: u32) -> Decimal {
        decimal.round_dp(scale)
    }

    /// Compare a FHIR Resource (quantity) with a FHIRPath quantity
    fn compare_fhir_quantity_with_fhirpath_quantity(
        &self,
        fhir_json: &serde_json::Value,
        type_info: &TypeInfo,
        fhirpath_value: Decimal,
        fhirpath_unit: &Option<String>,
        fhirpath_calendar_unit: &Option<crate::core::CalendarUnit>,
    ) -> Option<bool> {
        // Check if this is a FHIR Quantity resource
        if type_info.type_name == "Quantity" || type_info.name.as_deref() == Some("Quantity") {
                // Extract value and code from FHIR quantity
                let fhir_value = fhir_json.get("value")?;
                let fhir_code = fhir_json.get("code")?.as_str()?;

                // Convert FHIR value to Decimal (could be integer or float)
                let fhir_decimal = if let Some(int_val) = fhir_value.as_i64() {
                    Decimal::from(int_val)
                } else if let Some(float_val) = fhir_value.as_f64() {
                    Decimal::from_f64(float_val)?
                } else {
                    return Some(false); // Invalid value type
                };

                // Use quantity utilities for comparison
                match crate::evaluator::quantity_utils::are_quantities_equivalent(
                    fhir_decimal,
                    &Some(fhir_code.to_string()),
                    &None, // FHIR quantities don't have calendar units
                    fhirpath_value,
                    fhirpath_unit,
                    fhirpath_calendar_unit,
                ) {
                    Ok(result) => Some(result),
                    Err(_) => Some(false),
                }
        } else {
            // Not a quantity, not equivalent
            Some(false)
        }
    }

    /// Compare two FhirPathValues for equivalence
    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            // Boolean equivalence
            (FhirPathValue::Boolean(l, _, _), FhirPathValue::Boolean(r, _, _)) => Some(l == r),

            // String equivalence (case-insensitive and whitespace-normalized per FHIRPath spec)
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => {
                // Normalize strings: trim and convert to lowercase
                let l_normalized = l.trim().to_lowercase();
                let r_normalized = r.trim().to_lowercase();
                Some(l_normalized == r_normalized)
            }

            // Integer equivalence
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l == r),

            // Decimal equivalence
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                // According to FHIRPath spec: values must be equal, comparison is done on values
                // rounded to the precision of the least precise operand
                let left_precision = self.get_decimal_precision(l);
                let right_precision = self.get_decimal_precision(r);
                let min_precision = left_precision.min(right_precision);

                let left_rounded = self.round_decimal(l, min_precision);
                let right_rounded = self.round_decimal(r, min_precision);

                Some(left_rounded == right_rounded)
            }

            // Integer vs Decimal comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                // Integer has precision 0, so compare rounded decimal to precision 0
                let right_precision = self.get_decimal_precision(r);
                let min_precision = 0_u32.min(right_precision);

                let left_rounded = self.round_decimal(&left_decimal, min_precision);
                let right_rounded = self.round_decimal(r, min_precision);

                Some(left_rounded == right_rounded)
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                // Integer has precision 0, so compare rounded decimal to precision 0
                let left_precision = self.get_decimal_precision(l);
                let min_precision = left_precision.min(0_u32);

                let left_rounded = self.round_decimal(l, min_precision);
                let right_rounded = self.round_decimal(&right_decimal, min_precision);

                Some(left_rounded == right_rounded)
            }

            // Date equivalence (considering precision)
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => {
                // For equivalence, dates should match considering their precision
                // This is a simplified implementation - the full spec requires more complex precision handling
                Some(l == r)
            }

            // DateTime equivalence (considering precision and timezone normalization)
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                // For equivalence, datetimes should match considering their precision
                // This is a simplified implementation - the full spec requires timezone normalization
                Some(l == r)
            }

            // Time equivalence
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l == r),

            // Quantity equivalence (with unit normalization)
            (
                FhirPathValue::Quantity {
                    value: lv,
                    unit: lu,
                    calendar_unit: lc,
                    ..
                },
                FhirPathValue::Quantity {
                    value: rv,
                    unit: ru,
                    calendar_unit: rc,
                    ..
                },
            ) => {
                // Use the quantity utilities for proper unit conversion
                match quantity_utils::are_quantities_equivalent(*lv, lu, lc, *rv, ru, rc) {
                    Ok(result) => Some(result),
                    Err(_) => Some(false), // Conversion failed, not equivalent
                }
            }

            // FHIR Resource (Quantity) vs FHIRPath Quantity equivalence
            (FhirPathValue::Resource(json, type_info, _), FhirPathValue::Quantity { value: rv, unit: ru, calendar_unit: rc, .. }) => {
                self.compare_fhir_quantity_with_fhirpath_quantity(json, type_info, *rv, ru, rc)
            }
            (FhirPathValue::Quantity { value: lv, unit: lu, calendar_unit: lc, .. }, FhirPathValue::Resource(json, type_info, _)) => {
                self.compare_fhir_quantity_with_fhirpath_quantity(json, type_info, *lv, lu, lc)
            }

            // Collection equivalence (recursive)
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.len() != r.len() {
                    return Some(false);
                }

                // For equivalence, collections should be compared order-independently
                // Check if each item in left collection has an equivalent in right collection
                for l_item in l.iter() {
                    let mut found_equivalent = false;
                    for r_item in r.iter() {
                        if let Some(true) = self.compare_values(l_item, r_item) {
                            found_equivalent = true;
                            break;
                        }
                    }
                    if !found_equivalent {
                        return Some(false);
                    }
                }

                // Check if each item in right collection has an equivalent in left collection
                for r_item in r.iter() {
                    let mut found_equivalent = false;
                    for l_item in l.iter() {
                        if let Some(true) = self.compare_values(l_item, r_item) {
                            found_equivalent = true;
                            break;
                        }
                    }
                    if !found_equivalent {
                        return Some(false);
                    }
                }

                Some(true)
            }

            // Resource equivalence (compare JSON objects)
            (FhirPathValue::Resource(l_json, l_type, _), FhirPathValue::Resource(r_json, r_type, _)) => {
                // Resources are equivalent if they have the same type and the same JSON content
                if l_type.type_name == r_type.type_name {
                    Some(l_json == r_json)
                } else {
                    Some(false)
                }
            }

            // Different types - for equivalence, this depends on the specific types
            // Some types can be equivalent (e.g., integer and decimal), others cannot
            _ => Some(false),
        }
    }
}

#[async_trait]
impl OperationEvaluator for EquivalentOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Equivalence has different empty handling than equality:
        // - If both are empty, result is true
        // - If one is empty and other is not, result is false
        // - If both are non-empty, compare values

        match (left.is_empty(), right.is_empty()) {
            (true, true) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            }),
            (true, false) | (false, true) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            }),
            (false, false) => {
                // Both sides have values, compare them
                // For collections, we need to compare all elements, not just the first
                if left.len() != right.len() {
                    // Different collection sizes are not equivalent
                    return Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::boolean(false)),
                    });
                }

                // For equivalence, collections should be compared order-independently
                // Check if each item in left collection has an equivalent in right collection
                for left_val in left.iter() {
                    let mut found_equivalent = false;
                    for right_val in right.iter() {
                        if let Some(true) = self.compare_values(left_val, right_val) {
                            found_equivalent = true;
                            break;
                        }
                    }
                    if !found_equivalent {
                        return Ok(EvaluationResult {
                            value: Collection::single(FhirPathValue::boolean(false)),
                        });
                    }
                }

                // Check if each item in right collection has an equivalent in left collection
                for right_val in right.iter() {
                    let mut found_equivalent = false;
                    for left_val in left.iter() {
                        if let Some(true) = self.compare_values(left_val, right_val) {
                            found_equivalent = true;
                            break;
                        }
                    }
                    if !found_equivalent {
                        return Ok(EvaluationResult {
                            value: Collection::single(FhirPathValue::boolean(false)),
                        });
                    }
                }

                // All elements have equivalents
                Ok(EvaluationResult {
                    value: Collection::single(FhirPathValue::boolean(true)),
                })
            }
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the equivalent operator
fn create_equivalent_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "~".to_string(),
        description: "Equivalence comparison with normalization and special empty handling"
            .to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Custom, // Equivalence has special empty handling
        deterministic: true,
        precedence: 5, // FHIRPath equivalence precedence (same as equality)
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_equivalent_boolean() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_strings_case_insensitive() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![FhirPathValue::string("HELLO".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_strings_whitespace_normalized() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("  hello  ".to_string())];
        let right = vec![FhirPathValue::string("hello".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_both_empty() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![];
        let right = vec![];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equivalent_one_empty() {
        let evaluator = EquivalentOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }
}
