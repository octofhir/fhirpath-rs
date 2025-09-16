//! Type operators (is/as) implementation
//!
//! Implements FHIRPath type checking and casting operations.

use std::sync::Arc;
use async_trait::async_trait;

use crate::core::{FhirPathValue, FhirPathType, TypeSignature, Result, Collection};
use crate::evaluator::{EvaluationContext, EvaluationResult};
use crate::evaluator::operator_registry::{
    OperationEvaluator, OperatorMetadata, OperatorSignature,
    EmptyPropagation, Associativity
};

/// "is" operator evaluator for type checking
pub struct IsOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl IsOperatorEvaluator {
    /// Create a new "is" operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_is_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Check if a value is of the specified type
    fn check_type(&self, value: &FhirPathValue, type_name: &str) -> bool {
        match value {
            FhirPathValue::Boolean(_, _, _) => type_name == "boolean" || type_name == "System.Boolean",
            FhirPathValue::String(_, _, _) => type_name == "string" || type_name == "System.String",
            FhirPathValue::Integer(_, _, _) => type_name == "integer" || type_name == "System.Integer",
            FhirPathValue::Decimal(_, _, _) => type_name == "decimal" || type_name == "System.Decimal",
            FhirPathValue::Date(_, _, _) => type_name == "date" || type_name == "System.Date",
            FhirPathValue::DateTime(_, _, _) => type_name == "dateTime" || type_name == "System.DateTime",
            FhirPathValue::Time(_, _, _) => type_name == "time" || type_name == "System.Time",
            FhirPathValue::Quantity { .. } => type_name == "Quantity" || type_name == "System.Quantity",
            FhirPathValue::Resource(_, type_info, _) => {
                type_name == type_info.type_name ||
                type_name == format!("FHIR.{}", type_info.type_name)
            },
            _ => false,
        }
    }
}

#[async_trait]
impl OperationEvaluator for IsOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if left operand is empty, result is empty
        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // For type checking, we need the right operand to be a type identifier
        // This would normally be handled by parser, but for now we'll extract from string
        if right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        }

        let value = left.first().unwrap();

        // Extract type name from right operand (should be a string literal)
        let type_name = if let Some(FhirPathValue::String(type_str, _, _)) = right.first() {
            type_str.as_str()
        } else {
            // If right operand is not a string, this is likely a parser issue
            return Ok(EvaluationResult {
                value: Collection::single(FhirPathValue::boolean(false)),
            });
        };

        let is_of_type = self.check_type(value, type_name);

        Ok(EvaluationResult {
            value: Collection::single(FhirPathValue::boolean(is_of_type)),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// "as" operator evaluator for type casting
pub struct AsOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl AsOperatorEvaluator {
    /// Create a new "as" operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_as_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Cast a value to the specified type (returns value if cast is valid, empty otherwise)
    fn cast_type(&self, value: &FhirPathValue, type_name: &str) -> Option<FhirPathValue> {
        // For now, simple type checking - in a full implementation this would do actual conversion
        let is_evaluator = IsOperatorEvaluator::new();
        if is_evaluator.check_type(value, type_name) {
            Some(value.clone())
        } else {
            None
        }
    }
}

#[async_trait]
impl OperationEvaluator for AsOperatorEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Empty propagation: if left operand is empty, result is empty
        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Extract type name from right operand
        if right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        let value = left.first().unwrap();

        let type_name = if let Some(FhirPathValue::String(type_str, _, _)) = right.first() {
            type_str.as_str()
        } else {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        };

        match self.cast_type(value, type_name) {
            Some(cast_value) => Ok(EvaluationResult {
                value: Collection::single(cast_value),
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

/// Create metadata for the "is" operator
fn create_is_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::String], // Type name as string for now
        FhirPathType::Boolean,
    );

    OperatorMetadata {
        name: "is".to_string(),
        description: "Type checking operation".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 8, // FHIRPath type operator precedence
        associativity: Associativity::Left,
    }
}

/// Create metadata for the "as" operator
fn create_as_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::String], // Type name as string for now
        FhirPathType::Any,
    );

    OperatorMetadata {
        name: "as".to_string(),
        description: "Type casting operation".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![],
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        precedence: 8, // FHIRPath type operator precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_is_operator_boolean() {
        let evaluator = IsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::string("boolean".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_is_operator_wrong_type() {
        let evaluator = IsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string("boolean".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_as_operator_valid_cast() {
        let evaluator = AsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string("integer".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(42));
    }

    #[tokio::test]
    async fn test_as_operator_invalid_cast() {
        let evaluator = AsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::test_utils::create_test_model_provider()),
            None,
        ).await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string("boolean".to_string())];

        let result = evaluator.evaluate(vec![], &context, left, right).await.unwrap();

        assert!(result.value.is_empty());
    }
}