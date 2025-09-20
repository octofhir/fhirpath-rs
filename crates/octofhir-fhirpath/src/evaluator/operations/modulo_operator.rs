//! Modulo (mod) operator implementation
//!
//! Implements FHIRPath modulo operation for integer types.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Modulo operator evaluator
pub struct ModuloOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for ModuloOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuloOperatorEvaluator {
    /// Create a new modulo operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_modulo_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform modulo operation on two FhirPathValues
    fn modulo_values(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Result<Option<FhirPathValue>> {
        match (left, right) {
            // Integer modulo
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                if *r == 0 {
                    return Ok(None);
                }
                Ok(Some(FhirPathValue::integer(l % r)))
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                if r.is_zero() {
                    return Ok(None);
                }
                Ok(Some(FhirPathValue::decimal(l % r)))
            }

            _ => Ok(None),
        }
    }
}

#[async_trait]
impl OperationEvaluator for ModuloOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
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

        // For arithmetic, we use the first elements (singleton evaluation)
        let left_value = left.first().unwrap();
        let right_value = right.first().unwrap();

        match self.modulo_values(left_value, right_value)? {
            Some(result) => Ok(EvaluationResult {
                value: Collection::single(result),
            }),
            None => Ok(EvaluationResult {
                value: Collection::empty(),
            }),
        }
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the modulo operator
fn create_modulo_metadata() -> OperatorMetadata {
    let signature = TypeSignature::new(
        vec![FhirPathType::Integer, FhirPathType::Integer],
        FhirPathType::Integer,
    );

    OperatorMetadata {
        name: "mod".to_string(),
        description: "Modulo operation for integers".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 8, // FHIRPath multiplication/division/modulo precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_modulo_integers() {
        let evaluator = ModuloOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(17)];
        let right = vec![FhirPathValue::integer(5)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(2));
    }

    #[tokio::test]
    async fn test_modulo_by_zero() {
        let evaluator = ModuloOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(10)];
        let right = vec![FhirPathValue::integer(0)];

        let result = evaluator.evaluate(vec![], &context, left, right).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Modulo by zero"));
    }
}
