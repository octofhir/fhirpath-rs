//! Union (|) operator implementation
//!
//! Implements FHIRPath collection union operator that combines two collections.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Union operator evaluator
pub struct UnionOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for UnionOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl UnionOperatorEvaluator {
    /// Create a new union operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_union_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl OperationEvaluator for UnionOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Union combines both collections and removes duplicates
        let mut result_values = left;
        result_values.extend(right);

        // Remove duplicates while preserving order
        let mut unique_values = Vec::new();
        for value in result_values {
            let is_duplicate = unique_values.iter().any(|existing| {
                match (existing, &value) {
                    // Use the FhirPath equality semantics
                    (FhirPathValue::Integer(a, _, _), FhirPathValue::Integer(b, _, _)) => a == b,
                    (FhirPathValue::Decimal(a, _, _), FhirPathValue::Decimal(b, _, _)) => a == b,
                    (FhirPathValue::String(a, _, _), FhirPathValue::String(b, _, _)) => a == b,
                    (FhirPathValue::Boolean(a, _, _), FhirPathValue::Boolean(b, _, _)) => a == b,
                    (FhirPathValue::Date(a, _, _), FhirPathValue::Date(b, _, _)) => a == b,
                    (FhirPathValue::DateTime(a, _, _), FhirPathValue::DateTime(b, _, _)) => a == b,
                    (FhirPathValue::Time(a, _, _), FhirPathValue::Time(b, _, _)) => a == b,
                    (
                        FhirPathValue::Quantity {
                            value: v1,
                            unit: u1,
                            ..
                        },
                        FhirPathValue::Quantity {
                            value: v2,
                            unit: u2,
                            ..
                        },
                    ) => v1 == v2 && u1 == u2,
                    // For different types, they are not equal
                    _ => false,
                }
            });
            if !is_duplicate {
                unique_values.push(value);
            }
        }

        Ok(EvaluationResult {
            value: Collection::from(unique_values),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the union operator
fn create_union_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::Any,
    );

    OperatorMetadata {
        name: "|".to_string(),
        description: "Collection union operation that combines two collections".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::NoPropagation, // Union doesn't propagate empty
        deterministic: true,
        precedence: 7, // FHIRPath union precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_union_basic() {
        let evaluator = UnionOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(1), FhirPathValue::integer(2)];
        let right = vec![FhirPathValue::integer(3), FhirPathValue::integer(4)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 4);
        assert_eq!(result.value.get(0).unwrap().as_integer(), Some(1));
        assert_eq!(result.value.get(1).unwrap().as_integer(), Some(2));
        assert_eq!(result.value.get(2).unwrap().as_integer(), Some(3));
        assert_eq!(result.value.get(3).unwrap().as_integer(), Some(4));
    }

    #[tokio::test]
    async fn test_union_with_empty() {
        let evaluator = UnionOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(1)];
        let right = vec![];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(1));
    }
}
