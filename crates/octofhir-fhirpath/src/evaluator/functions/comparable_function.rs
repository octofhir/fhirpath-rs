//! Comparable function implementation
//!
//! The comparable function checks if two values can be compared.
//! Syntax: value1.comparable(value2)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::quantity_utils;

/// Comparable function evaluator
pub struct ComparableFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ComparableFunctionEvaluator {
    /// Create a new comparable function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "comparable".to_string(),
                description: "Checks if two values can be compared".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "other".to_string(),
                        parameter_type: vec!["Any".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The value to compare with".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Logic,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Check if two FhirPath values are comparable
    fn are_comparable(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        match (left, right) {
            // Same types are generally comparable
            (FhirPathValue::String(_, _, _), FhirPathValue::String(_, _, _)) => true,
            (FhirPathValue::Boolean(_, _, _), FhirPathValue::Boolean(_, _, _)) => true,
            (FhirPathValue::Integer(_, _, _), FhirPathValue::Integer(_, _, _)) => true,
            (FhirPathValue::Decimal(_, _, _), FhirPathValue::Decimal(_, _, _)) => true,
            (FhirPathValue::Date(_, _, _), FhirPathValue::Date(_, _, _)) => true,
            (FhirPathValue::DateTime(_, _, _), FhirPathValue::DateTime(_, _, _)) => true,
            (FhirPathValue::Time(_, _, _), FhirPathValue::Time(_, _, _)) => true,

            // Numeric types are comparable with each other
            (FhirPathValue::Integer(_, _, _), FhirPathValue::Decimal(_, _, _)) => true,
            (FhirPathValue::Decimal(_, _, _), FhirPathValue::Integer(_, _, _)) => true,

            // Quantities are comparable when their units are commensurable; a number is a
            // quantity with the unit '1' (`2 '1'.comparable(3)`).
            (FhirPathValue::Quantity { .. }, _) | (_, FhirPathValue::Quantity { .. }) => {
                let left = quantity_utils::number_as_quantity(left);
                let right = quantity_utils::number_as_quantity(right);
                match (left.as_ref(), right.as_ref()) {
                    (
                        FhirPathValue::Quantity {
                            value: lv,
                            unit: lu,
                            code: lc,
                            calendar_unit: lcal,
                            ..
                        },
                        FhirPathValue::Quantity {
                            value: rv,
                            unit: ru,
                            code: rc,
                            calendar_unit: rcal,
                            ..
                        },
                    ) => quantity_utils::align_quantities(
                        *lv,
                        lc.as_deref().or(lu.as_deref()),
                        *lcal,
                        *rv,
                        rc.as_deref().or(ru.as_deref()),
                        *rcal,
                        quantity_utils::QuantityComparison::Ordering,
                    )
                    .is_some(),
                    _ => false,
                }
            }

            // Different types are generally not comparable
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ComparableFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("comparable function expects 1 argument, got {}", args.len()),
            ));
        }

        // Handle empty input or arguments - propagate empty collections
        if input.is_empty() || args[0].is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let mut results = Vec::new();

        for left_value in &input {
            for right_value in &args[0] {
                let is_comparable = self.are_comparable(left_value, right_value);
                results.push(FhirPathValue::boolean(is_comparable));
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
