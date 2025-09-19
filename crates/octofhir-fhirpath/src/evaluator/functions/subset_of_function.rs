//! SubsetOf function implementation
//!
//! The subsetOf function returns true if the input collection is a subset of the other collection.
//! Syntax: collection.subsetOf(other)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};use crate::evaluator::EvaluationResult;

/// SubsetOf function evaluator
pub struct SubsetOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SubsetOfFunctionEvaluator {
    /// Create a new subsetOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "subsetOf".to_string(),
                description: "Returns true if the input collection is a subset of the other collection.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "other".to_string(),
                        parameter_type: vec!["Collection".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The collection to check against".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Subsetting,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for SubsetOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "subsetOf function requires exactly one argument".to_string(),
            ));
        }

        // Handle empty other argument - propagate empty collections
        if args[0].is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(false)),
            });
        }

        let other = &args[0];

        // Empty set is a subset of any set
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // Check if every item in input exists in other
        let is_subset = input.iter().all(|input_item| {
            other.iter().any(|other_item| values_equal(input_item, other_item))
        });

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(is_subset)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Helper function to compare two FhirPathValues for equality using FHIR semantics
fn values_equal(a: &FhirPathValue, b: &FhirPathValue) -> bool {
    // Handle same pointer case (optimization)
    if std::ptr::eq(a, b) {
        return true;
    }

    match (a, b) {
        // Primitive types
        (FhirPathValue::String(s1, _, _), FhirPathValue::String(s2, _, _)) => s1 == s2,
        (FhirPathValue::Integer(i1, _, _), FhirPathValue::Integer(i2, _, _)) => i1 == i2,
        (FhirPathValue::Decimal(d1, _, _), FhirPathValue::Decimal(d2, _, _)) => d1 == d2,
        (FhirPathValue::Boolean(b1, _, _), FhirPathValue::Boolean(b2, _, _)) => b1 == b2,
        (FhirPathValue::Date(d1, _, _), FhirPathValue::Date(d2, _, _)) => d1 == d2,
        (FhirPathValue::DateTime(dt1, _, _), FhirPathValue::DateTime(dt2, _, _)) => dt1 == dt2,
        (FhirPathValue::Time(t1, _, _), FhirPathValue::Time(t2, _, _)) => t1 == t2,

        // FHIR Resources - leverage Arc<JsonValue> for efficient nested object comparison
        (FhirPathValue::Resource(json1, type1, _), FhirPathValue::Resource(json2, type2, _)) => {
            // Fast path: if Arc pointers are the same, objects are identical
            if std::sync::Arc::ptr_eq(json1, json2) {
                return true;
            }
            // Resources are equal if they have the same type and JSON content
            // JsonValue already implements proper equality for nested structures
            type1 == type2 && **json1 == **json2
        }

        // Cross-type numeric comparisons
        (FhirPathValue::Integer(i, _, _), FhirPathValue::Decimal(d, _, _)) => {
            *d == rust_decimal::Decimal::from(*i)
        }
        (FhirPathValue::Decimal(d, _, _), FhirPathValue::Integer(i, _, _)) => {
            *d == rust_decimal::Decimal::from(*i)
        }

        // String cross-type comparisons
        (FhirPathValue::String(s, _, _), other) | (other, FhirPathValue::String(s, _, _)) => {
            match other {
                FhirPathValue::Integer(i, _, _) => s == &i.to_string(),
                FhirPathValue::Decimal(d, _, _) => s == &d.to_string(),
                FhirPathValue::Boolean(b, _, _) => s == &b.to_string(),
                _ => false,
            }
        }

        // Different types are not equal
        _ => false,
    }
}
