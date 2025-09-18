//! Greater than (>) operator implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

pub struct GreaterThanOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl GreaterThanOperatorEvaluator {
    pub fn new() -> Self {
        let signature = TypeSignature::polymorphic(
            vec![FhirPathType::Any, FhirPathType::Any],
            FhirPathType::Boolean,
        );

        Self {
            metadata: OperatorMetadata {
                name: ">".to_string(),
                description: "Greater than comparison for ordered types".to_string(),
                signature: OperatorSignature {
                    signature,
                    overloads: vec![
                        TypeSignature::new(
                            vec![FhirPathType::Integer, FhirPathType::Integer],
                            FhirPathType::Boolean,
                        ),
                        TypeSignature::new(
                            vec![FhirPathType::Decimal, FhirPathType::Decimal],
                            FhirPathType::Boolean,
                        ),
                        TypeSignature::new(
                            vec![FhirPathType::Integer, FhirPathType::Decimal],
                            FhirPathType::Boolean,
                        ),
                        TypeSignature::new(
                            vec![FhirPathType::Decimal, FhirPathType::Integer],
                            FhirPathType::Boolean,
                        ),
                    ],
                },
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                precedence: 6,
                associativity: Associativity::Left,
            },
        }
    }

    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    fn compare_values(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<bool> {
        match (left, right) {
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Integer(r, _, _)) => Some(l > r),
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Decimal(r, _, _)) => Some(l > r),
            (FhirPathValue::Integer(l, _, _), FhirPathValue::Decimal(r, _, _)) => {
                let left_decimal = Decimal::from(*l);
                Some(left_decimal > *r)
            }
            (FhirPathValue::Decimal(l, _, _), FhirPathValue::Integer(r, _, _)) => {
                let right_decimal = Decimal::from(*r);
                Some(*l > right_decimal)
            }
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => Some(l > r),
            (FhirPathValue::Date(l, _, _), FhirPathValue::Date(r, _, _)) => Some(l > r),
            (FhirPathValue::DateTime(l, _, _), FhirPathValue::DateTime(r, _, _)) => Some(l > r),
            (FhirPathValue::Time(l, _, _), FhirPathValue::Time(r, _, _)) => Some(l > r),
            _ => None,
        }
    }
}

#[async_trait]
impl OperationEvaluator for GreaterThanOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        if left.is_empty() || right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        match self.compare_values(left.first().unwrap(), right.first().unwrap()) {
            Some(result) => Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(result)),
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
