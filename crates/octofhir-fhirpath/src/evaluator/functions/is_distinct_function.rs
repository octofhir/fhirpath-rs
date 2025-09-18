//! isDistinct function implementation
//!
//! Returns true if all items in the collection are distinct (no duplicates)

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

pub struct IsDistinctFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl IsDistinctFunctionEvaluator {
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "isDistinct".to_string(),
                description: "Returns true if all items in the collection are distinct (no duplicates)".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Logic,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    fn values_equal(&self, a: &FhirPathValue, b: &FhirPathValue) -> bool {
        match (a, b) {
            (FhirPathValue::String(s1, _, _), FhirPathValue::String(s2, _, _)) => s1 == s2,
            (FhirPathValue::Integer(i1, _, _), FhirPathValue::Integer(i2, _, _)) => i1 == i2,
            (FhirPathValue::Decimal(d1, _, _), FhirPathValue::Decimal(d2, _, _)) => d1 == d2,
            (FhirPathValue::Boolean(b1, _, _), FhirPathValue::Boolean(b2, _, _)) => b1 == b2,
            (FhirPathValue::Date(d1, _, _), FhirPathValue::Date(d2, _, _)) => d1 == d2,
            (FhirPathValue::DateTime(dt1, _, _), FhirPathValue::DateTime(dt2, _, _)) => dt1 == dt2,
            (FhirPathValue::Time(t1, _, _), FhirPathValue::Time(t2, _, _)) => t1 == t2,
            // Cross-type comparisons
            (FhirPathValue::String(s, _, _), other) | (other, FhirPathValue::String(s, _, _)) => {
                // Compare string representation
                match other {
                    FhirPathValue::Integer(i, _, _) => s == &i.to_string(),
                    FhirPathValue::Decimal(d, _, _) => s == &d.to_string(),
                    FhirPathValue::Boolean(b, _, _) => s == &b.to_string(),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for IsDistinctFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::FP0053,
                "isDistinct function takes no arguments".to_string(),
            ));
        }

        // Empty collection is considered distinct
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // Single item is always distinct
        if input.len() == 1 {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(true)),
            });
        }

        // Check for duplicates by comparing each item with all subsequent items
        for i in 0..input.len() {
            for j in (i + 1)..input.len() {
                if self.values_equal(&input[i], &input[j]) {
                    // Found a duplicate
                    return Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::boolean(false)),
                    });
                }
            }
        }

        // No duplicates found
        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::boolean(true)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}