//! Equality (=) operator implementation
//!
//! Implements FHIRPath equality comparison with type-aware semantics.
//! The equality operator performs type-specific comparison and returns empty
//! if either operand is empty.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};
use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::quantity_utils;
use crate::evaluator::{EvaluationContext, EvaluationResult};
use rust_decimal::Decimal;

/// Equality operator evaluator
pub struct EqualsOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl EqualsOperatorEvaluator {
    /// Create a new equality operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_equals_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Extract quantity information from a FHIR Quantity resource
    fn extract_quantity_from_resource(&self, json: &serde_json::Value) -> Option<FhirPathValue> {
        let value = json.get("value")?.as_f64()?;
        let unit = json.get("unit").and_then(|u| u.as_str()).unwrap_or("");
        let system = json.get("system").and_then(|s| s.as_str()).unwrap_or("");
        let code = json.get("code").and_then(|c| c.as_str()).unwrap_or("");

        // Prefer code over unit if available
        let unit_str = if !code.is_empty() { code } else { unit };

        Some(FhirPathValue::quantity(
            rust_decimal::Decimal::from_f64_retain(value)?,
            if unit_str.is_empty() { None } else { Some(unit_str.to_string()) },
        ))
    }

    /// Try to parse a string as a temporal value
    fn try_parse_string_as_temporal(&self, s: &str) -> Option<FhirPathValue> {
        // Try parsing as Date first
        if let Some(date) = PrecisionDate::parse(s) {
            return Some(FhirPathValue::date(date));
        }

        // Try parsing as DateTime
        if let Some(datetime) = PrecisionDateTime::parse(s) {
            return Some(FhirPathValue::datetime(datetime));
        }

        // Try parsing as Time
        if let Some(time) = PrecisionTime::parse(s) {
            return Some(FhirPathValue::time(time));
        }

        None
    }

    /// Check if a value is a temporal type
    fn is_temporal(&self, value: &FhirPathValue) -> bool {
        matches!(
            value,
            FhirPathValue::Date(_, _, _)
                | FhirPathValue::DateTime(_, _, _)
                | FhirPathValue::Time(_, _, _)
        )
    }

    /// Compare two FhirPathValues for equality with automatic string-to-temporal conversion
    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        // Handle string-to-temporal conversions
        match (left, right) {
            // String vs temporal types - try to parse string as temporal
            (FhirPathValue::String(s, _, _), temporal) if self.is_temporal(temporal) => {
                if let Some(parsed) = self.try_parse_string_as_temporal(s) {
                    return self.compare_values_direct(&parsed, temporal);
                }
                // Fall through to regular comparison if parsing fails
            }
            (temporal, FhirPathValue::String(s, _, _)) if self.is_temporal(temporal) => {
                if let Some(parsed) = self.try_parse_string_as_temporal(s) {
                    return self.compare_values_direct(temporal, &parsed);
                }
                // Fall through to regular comparison if parsing fails
            }
            _ => {}
        }

        // Regular comparison without conversion
        self.compare_values_direct(left, right)
    }

    /// Compare two FhirPathValues for equality without auto-conversion
    fn compare_values_direct(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            // Boolean equality
            (FhirPathValue::Boolean(l, _, _), FhirPathValue::Boolean(r, _, _)) => Some(l == r),

            // String equality
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => Some(l == r),

            // Integer equality
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l == r),

            // Decimal equality
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                Some((l - r).abs() < Decimal::new(1, 10)) // Small epsilon for decimal comparison
            }

            // Integer vs Decimal comparison
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some((left_decimal - r).abs() < Decimal::new(1, 10))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some((l - right_decimal).abs() < Decimal::new(1, 10))
            }

            // Date equality
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => Some(l == r),

            // DateTime equality
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => {
                // Check if both have the same timezone specification
                if l.tz_specified != r.tz_specified {
                    // Different timezone specifications - comparison returns empty
                    return None;
                }
                Some(l == r)
            }

            // Time equality
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l == r),

            // Cross-type temporal comparisons should return empty (None)
            (FhirPathValue::Date(_, _, _), FhirPathValue::DateTime(_, _, _)) => None,
            (FhirPathValue::DateTime(_, _, _), FhirPathValue::Date(_, _, _)) => None,
            (FhirPathValue::Date(_, _, _), FhirPathValue::Time(_, _, _)) => None,
            (FhirPathValue::Time(_, _, _), FhirPathValue::Date(_, _, _)) => None,
            (FhirPathValue::DateTime(_, _, _), FhirPathValue::Time(_, _, _)) => None,
            (FhirPathValue::Time(_, _, _), FhirPathValue::DateTime(_, _, _)) => None,

            // Quantity equality (considering units)
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
                // Use the quantity utilities for exact equality comparison
                match quantity_utils::are_quantities_equal(*lv, lu, lc, *rv, ru, rc) {
                    Ok(result) => Some(result),
                    Err(_) => Some(false), // Conversion failed, not equal
                }
            }
            // FHIR.Quantity (Resource) vs Quantity comparison
            (FhirPathValue::Resource(json, type_info, _), quantity @ FhirPathValue::Quantity { .. }) => {
                if type_info.type_name == "Quantity" {
                    // Try to extract Quantity information from the FHIR resource
                    if let Some(fhir_quantity) = self.extract_quantity_from_resource(json) {
                        return self.compare_values_direct(&fhir_quantity, quantity);
                    }
                }
                Some(false)
            }
            (quantity @ FhirPathValue::Quantity { .. }, FhirPathValue::Resource(json, type_info, _)) => {
                if type_info.type_name == "Quantity" {
                    // Try to extract Quantity information from the FHIR resource
                    if let Some(fhir_quantity) = self.extract_quantity_from_resource(json) {
                        return self.compare_values_direct(quantity, &fhir_quantity);
                    }
                }
                Some(false)
            }

            // Resource equality - compare JSON content and type
            (
                FhirPathValue::Resource(json1, type1, _),
                FhirPathValue::Resource(json2, type2, _),
            ) => {
                // Fast path: if Arc pointers are the same, objects are identical
                if std::sync::Arc::ptr_eq(json1, json2) {
                    return Some(true);
                }
                // Resources are equal if they have the same type and JSON content
                // JsonValue already implements proper equality for nested structures
                Some(type1 == type2 && **json1 == **json2)
            }

            // Different types are not equal
            _ => Some(false),
        }
    }
}

#[async_trait]
impl OperationEvaluator for EqualsOperatorEvaluator {
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

        // Collection equality: collections are equal if they have the same size and all elements are equal
        if left.len() != right.len() {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // Check if all corresponding elements are equal
        for (left_val, right_val) in left.iter().zip(right.iter()) {
            match self.compare_values(left_val, right_val) {
                Some(false) => {
                    // Found unequal elements
                    return Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::boolean(false)),
                    });
                }
                None => {
                    // Comparison returned empty - collections are not equal
                    return Ok(EvaluationResult {
                        value: Collection::empty(),
                    });
                }
                Some(true) => {
                    // Elements are equal, continue checking
                    continue;
                }
            }
        }

        // All elements are equal
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::boolean(true)),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the equality operator
fn create_equals_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "=".to_string(),
        description: "Equality comparison with type-aware semantics".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 5, // FHIRPath equality precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_equals_boolean() {
        let evaluator = EqualsOperatorEvaluator::new();
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
    async fn test_equals_integer() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equals_integer_decimal() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::decimal(42.0)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_equals_different_types() {
        let evaluator = EqualsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("42".to_string())];
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_equals_empty_propagation() {
        let evaluator = EqualsOperatorEvaluator::new();
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

        assert!(result.value.is_empty());
    }
}
