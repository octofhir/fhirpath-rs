//! String concatenation (&) operator implementation
//!
//! Implements FHIRPath string concatenation operator.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// String concatenation operator evaluator
pub struct ConcatenateOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for ConcatenateOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcatenateOperatorEvaluator {
    /// Create a new concatenation operator evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_concatenate_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn OperationEvaluator> {
        Arc::new(Self::new())
    }

    /// Perform concatenation on two FhirPathValues
    #[allow(dead_code)]
    fn concatenate_values(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<FhirPathValue> {
        match (left, right) {
            // String concatenation
            (FhirPathValue::String(l, _, _), FhirPathValue::String(r, _, _)) => {
                Some(FhirPathValue::string(format!("{l}{r}")))
            }

            // Convert other types to string and concatenate
            (left_val, right_val) => {
                let left_str = self.to_string_representation(left_val);
                let right_str = self.to_string_representation(right_val);

                if let (Some(l), Some(r)) = (left_str, right_str) {
                    Some(FhirPathValue::string(format!("{l}{r}")))
                } else {
                    None
                }
            }
        }
    }

    /// Convert FhirPathValue to string representation for concatenation
    fn to_string_representation(&self, value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::String(s, _, _) => Some(s.clone()),
            FhirPathValue::Integer(i, _, _) => Some(i.to_string()),
            FhirPathValue::Decimal(d, _, _) => Some(d.to_string()),
            FhirPathValue::Boolean(b, _, _) => Some(b.to_string()),
            FhirPathValue::Date(date, _, _) => Some(date.to_string()),
            FhirPathValue::DateTime(datetime, _, _) => Some(datetime.to_string()),
            FhirPathValue::Time(time, _, _) => Some(time.to_string()),
            FhirPathValue::Quantity { value, unit, .. } => {
                if let Some(unit_str) = unit {
                    Some(format!("{value} {unit_str}"))
                } else {
                    Some(value.to_string())
                }
            }
            // Other types cannot be converted to string for concatenation
            _ => None,
        }
    }
}

#[async_trait]
impl OperationEvaluator for ConcatenateOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Handle empty collections - treat as empty strings
        let left_str = if left.is_empty() {
            String::new()
        } else if left.len() == 1 {
            // Single value concatenation
            if let Some(s) = self.to_string_representation(left.first().unwrap()) {
                s
            } else {
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
        } else {
            // Multiple values - this should error for concatenation
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::FP0051,
                "Concatenation operator cannot be applied to collections with multiple items",
            ));
        };

        let right_str = if right.is_empty() {
            String::new()
        } else if right.len() == 1 {
            // Single value concatenation
            if let Some(s) = self.to_string_representation(right.first().unwrap()) {
                s
            } else {
                return Ok(EvaluationResult {
                    value: Collection::empty(),
                });
            }
        } else {
            // Multiple values - this should error for concatenation
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::FP0051,
                "Concatenation operator cannot be applied to collections with multiple items",
            ));
        };

        let result = FhirPathValue::string(format!("{left_str}{right_str}"));
        Ok(EvaluationResult {
            value: Collection::single(result),
        })
    }

    fn metadata(&self) -> &OperatorMetadata {
        &self.metadata
    }
}

/// Create metadata for the concatenation operator
fn create_concatenate_metadata() -> OperatorMetadata {
    let signature = TypeSignature::polymorphic(
        vec![FhirPathType::Any, FhirPathType::Any],
        FhirPathType::String, // Always returns String
    );

    OperatorMetadata {
        name: "&".to_string(),
        description: "String concatenation operator".to_string(),
        signature: OperatorSignature {
            signature,
            overloads: vec![
                TypeSignature::new(
                    vec![FhirPathType::String, FhirPathType::String],
                    FhirPathType::String,
                ),
                TypeSignature::new(
                    vec![FhirPathType::String, FhirPathType::Integer],
                    FhirPathType::String,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Integer, FhirPathType::String],
                    FhirPathType::String,
                ),
                TypeSignature::new(
                    vec![FhirPathType::String, FhirPathType::Decimal],
                    FhirPathType::String,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Decimal, FhirPathType::String],
                    FhirPathType::String,
                ),
                TypeSignature::new(
                    vec![FhirPathType::String, FhirPathType::Boolean],
                    FhirPathType::String,
                ),
                TypeSignature::new(
                    vec![FhirPathType::Boolean, FhirPathType::String],
                    FhirPathType::String,
                ),
            ],
        },
        empty_propagation: EmptyPropagation::Custom,
        deterministic: true,
        precedence: 6, // FHIRPath concatenation precedence
        associativity: Associativity::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[tokio::test]
    async fn test_concatenate_strings() {
        let evaluator = ConcatenateOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![FhirPathValue::string(" World".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_string(),
            Some("Hello World".to_string())
        );
    }

    #[tokio::test]
    async fn test_concatenate_string_integer() {
        let evaluator = ConcatenateOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Count: ".to_string())];
        let right = vec![FhirPathValue::integer(42)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_string(),
            Some("Count: 42".to_string())
        );
    }

    #[tokio::test]
    async fn test_concatenate_integer_string() {
        let evaluator = ConcatenateOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string(" items".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_string(),
            Some("42 items".to_string())
        );
    }

    #[tokio::test]
    async fn test_concatenate_boolean_string() {
        let evaluator = ConcatenateOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        let left = vec![FhirPathValue::string("Active: ".to_string())];
        let right = vec![FhirPathValue::boolean(true)];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_string(),
            Some("Active: true".to_string())
        );
    }

    #[tokio::test]
    async fn test_concatenate_with_empty() {
        let evaluator = ConcatenateOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
        )
        .await;

        // Test string with empty collection
        let left = vec![FhirPathValue::string("Hello".to_string())];
        let right = vec![]; // Empty collection

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_string(),
            Some("Hello".to_string())
        );

        // Test empty collection with string
        let left = vec![]; // Empty collection
        let right = vec![FhirPathValue::string("World".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(
            result.value.first().unwrap().as_string(),
            Some("World".to_string())
        );
    }
}
