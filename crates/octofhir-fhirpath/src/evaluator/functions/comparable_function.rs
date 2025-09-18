//! Comparable function implementation
//!
//! The comparable function checks if two values can be compared.
//! Syntax: value1.comparable(value2)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};
use crate::evaluator::quantity_utils;

/// Comparable function evaluator
pub struct ComparableFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ComparableFunctionEvaluator {
    /// Create a new comparable function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
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
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Logic,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Check if two units are comparable (same dimension) using UCUM library
    fn are_units_comparable(&self, left_unit: &Option<String>, right_unit: &Option<String>) -> bool {
        match (left_unit, right_unit) {
            (Some(left), Some(right)) => {
                // Use the UCUM library to check if units are comparable
                quantity_utils::are_ucum_units_comparable(left, right)
                    .unwrap_or(false) // If there's an error (e.g., invalid unit), consider them not comparable
            }
            // If both units are None, they are comparable (both dimensionless)
            (None, None) => true,
            // If only one unit is missing, consider them not comparable
            _ => false,
        }
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

            // Quantities are comparable if they have compatible units
            (
                FhirPathValue::Quantity { unit: left_unit, .. },
                FhirPathValue::Quantity { unit: right_unit, .. },
            ) => self.are_units_comparable(left_unit, right_unit),

            // Different types are generally not comparable
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for ComparableFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("comparable function expects 1 argument, got {}", args.len()),
            ));
        }

        // Evaluate the other value
        let other_result = evaluator.evaluate(&args[0], context).await?;

        let mut results = Vec::new();

        for left_value in &input {
            for right_value in other_result.value.iter() {
                let is_comparable = self.are_comparable(left_value, right_value);
                results.push(FhirPathValue::boolean(is_comparable));
            }
        }

        // If either input or other is empty, return empty
        if input.is_empty() || other_result.value.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
