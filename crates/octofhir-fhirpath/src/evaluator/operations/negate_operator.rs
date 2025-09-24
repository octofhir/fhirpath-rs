//! Negate (-) unary operator implementation
//!
//! Implements FHIRPath arithmetic negation for numeric types.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// Negate unary operator evaluator
pub struct NegateOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl NegateOperatorEvaluator {
    /// Create a new negate operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_negate_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform negation on a FhirPathValue
    fn negate_value(&self, value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            // Integer negation
            FhirPathValue::Integer(i, type_info, primitive) => Ok(FhirPathValue::Integer(
                -i,
                type_info.clone(),
                primitive.clone(),
            )),

            // Decimal negation
            FhirPathValue::Decimal(d, type_info, primitive) => Ok(FhirPathValue::Decimal(
                -*d,
                type_info.clone(),
                primitive.clone(),
            )),

            // Quantity negation
            FhirPathValue::Quantity {
                value,
                unit,
                code,
                system,
                ucum_unit,
                calendar_unit,
                primitive_element,
                type_info,
            } => Ok(FhirPathValue::Quantity {
                value: -*value,
                unit: unit.clone(),
                code: code.clone(),
                system: system.clone(),
                ucum_unit: ucum_unit.clone(),
                calendar_unit: *calendar_unit,
                type_info: type_info.clone(),
                primitive_element: primitive_element.clone(),
            }),

            _ => Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0055,
                format!("Cannot negate value of type {}", value.type_name()),
            )),
        }
    }
}

#[async_trait]
impl OperationEvaluator for NegateOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        _right: Vec<FhirPathValue>, // Empty for unary operations
    ) -> Result<EvaluationResult> {
        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let mut results = Vec::new();

        for value in left {
            let negated = self.negate_value(&value)?;
            results.push(negated);
        }

        Ok(EvaluationResult {
            value: Collection::from(results),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the negate operator
fn create_negate_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any],
        FhirPathType::Any, // Return type depends on operand
    );

    OperatorMetadata {
        name: "-".to_string(),
        description: "Arithmetic negation operator".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                TypeSignature::new(vec![FhirPathType::Integer], FhirPathType::Integer),
                TypeSignature::new(vec![FhirPathType::Decimal], FhirPathType::Decimal),
                TypeSignature::new(vec![FhirPathType::Quantity], FhirPathType::Quantity),
            ],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 20, // High precedence for unary operators
        associativity: Associativity::Right,
    }
}

impl Default for NegateOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
