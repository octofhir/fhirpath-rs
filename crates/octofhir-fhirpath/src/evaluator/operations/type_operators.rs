//! Type operators (is/as) implementation
//!
//! Implements FHIRPath type checking and casting operations with comprehensive
//! ModelProvider integration and FHIR type hierarchy support.

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::{Collection, FhirPathType, FhirPathValue, Result, TypeSignature};
use crate::evaluator::operator_registry::{
    Associativity, EmptyPropagation, OperationEvaluator, OperatorMetadata, OperatorSignature,
};
use crate::evaluator::{EvaluationContext, EvaluationResult};

/// "is" operator evaluator for type checking
pub struct IsOperatorEvaluator {
    metadata: OperatorMetadata,
}

impl Default for IsOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
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

    /// Enhanced type compatibility checking with namespace support
    async fn check_type_compatibility(
        &self,
        value_type: &crate::core::model_provider::TypeInfo,
        target_type: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        // Handle namespace prefixes (System.Boolean, FHIR.Patient)
        let (target_namespace, target_type_name) = if target_type.contains('.') {
            let parts: Vec<&str> = target_type.split('.').collect();
            (Some(parts[0]), parts[1])
        } else {
            (None, target_type)
        };

        // Check direct type match
        if value_type.type_name == target_type_name {
            return Ok(true);
        }

        // Use ModelProvider.of_type for schema-driven type checking
        if context
            .model_provider()
            .of_type(value_type, target_type_name)
            .is_some()
        {
            return Ok(true);
        }

        // Check namespace compatibility if specified
        if let Some(target_ns) = target_namespace
            && let Some(value_ns) = &value_type.namespace
            && value_ns != target_ns
        {
            return Ok(false);
        }

        // Check inheritance hierarchy using ModelProvider
        if let Some(value_name) = &value_type.name
            && context
                .model_provider()
                .is_type_derived_from(value_name, target_type_name)
        {
            return Ok(true);
        }

        // Check type_name derivation as well
        if context
            .model_provider()
            .is_type_derived_from(&value_type.type_name, target_type_name)
        {
            return Ok(true);
        }

        Ok(false)
    }
}

#[async_trait]
impl OperationEvaluator for IsOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Spec: If left operand is empty, result is empty (not false)
        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Spec: If right operand is empty or invalid, throw error
        if right.is_empty() {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "Type identifier required for 'is' operator",
            ));
        }

        // Spec: If left operand has more than one item, throw error
        if left.len() > 1 {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0063,
                "'is' operator requires single item collection",
            ));
        }

        let value = left.first().unwrap();

        // Extract type name from right operand (should be a string literal representing type identifier)
        let type_name = if let Some(FhirPathValue::String(type_str, _, _)) = right.first() {
            type_str.as_str()
        } else {
            // If right operand is not a string, this is likely a parser issue
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "Invalid type identifier for 'is' operator",
            ));
        };

        // Get type info for the value
        let value_type_info = value.type_info();

        // Enhanced type checking with namespace support and hierarchy
        let is_of_type = self
            .check_type_compatibility(value_type_info, type_name, context)
            .await?;

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

impl Default for AsOperatorEvaluator {
    fn default() -> Self {
        Self::new()
    }
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

    /// Enhanced type compatibility checking with namespace support
    async fn check_type_compatibility(
        &self,
        value_type: &crate::core::model_provider::TypeInfo,
        target_type: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        // Handle namespace prefixes (System.Boolean, FHIR.Patient)
        let (target_namespace, target_type_name) = if target_type.contains('.') {
            let parts: Vec<&str> = target_type.split('.').collect();
            (Some(parts[0]), parts[1])
        } else {
            (None, target_type)
        };

        // Check direct type match
        if value_type.type_name == target_type_name {
            return Ok(true);
        }

        // Use ModelProvider.of_type for schema-driven type checking
        if context
            .model_provider()
            .of_type(value_type, target_type_name)
            .is_some()
        {
            return Ok(true);
        }

        // Check namespace compatibility if specified
        if let Some(target_ns) = target_namespace
            && let Some(value_ns) = &value_type.namespace
            && value_ns != target_ns
        {
            return Ok(false);
        }

        // Check inheritance hierarchy using ModelProvider
        if let Some(value_name) = &value_type.name
            && context
                .model_provider()
                .is_type_derived_from(value_name, target_type_name)
        {
            return Ok(true);
        }

        // Check type_name derivation as well
        if context
            .model_provider()
            .is_type_derived_from(&value_type.type_name, target_type_name)
        {
            return Ok(true);
        }

        Ok(false)
    }
}

#[async_trait]
impl OperationEvaluator for AsOperatorEvaluator {
    async fn evaluate(
        &self,
        __input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        left: Vec<FhirPathValue>,
        right: Vec<FhirPathValue>,
    ) -> Result<EvaluationResult> {
        // Spec: If left operand is empty, result is empty
        if left.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Spec: If right operand is empty, result is empty
        if right.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Spec: If left operand has more than one item, throw error
        if left.len() > 1 {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0063,
                "'as' operator requires single item collection",
            ));
        }

        let value = left.first().unwrap();

        let type_name = if let Some(FhirPathValue::String(type_str, _, _)) = right.first() {
            type_str.as_str()
        } else {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "Invalid type identifier for 'as' operator",
            ));
        };

        // Get type info for the value
        let value_type_info = value.type_info();

        // Enhanced type casting with proper type conversion
        if self
            .check_type_compatibility(value_type_info, type_name, context)
            .await?
        {
            // Get target type info from ModelProvider
            let target_type_info = context
                .model_provider()
                .get_type(type_name)
                .await
                .map_err(|e| {
                    crate::core::FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        format!("ModelProvider error getting type '{type_name}': {e}"),
                    )
                })?
                .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                    type_name: type_name.to_string(),
                    singleton: Some(true),
                    namespace: None,
                    name: Some(type_name.to_string()),
                    is_empty: Some(false),
                });

            // Return the value with updated type info
            let cast_value = value.with_type_info(target_type_info);
            Ok(EvaluationResult {
                value: Collection::single(cast_value),
            })
        } else {
            Ok(EvaluationResult {
                value: Collection::empty(),
            })
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
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::boolean(true)];
        let right = vec![FhirPathValue::string("Boolean".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(true));
    }

    #[tokio::test]
    async fn test_is_operator_wrong_type() {
        let evaluator = IsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string("boolean".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_boolean(), Some(false));
    }

    #[tokio::test]
    async fn test_as_operator_valid_cast() {
        let evaluator = AsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string("Integer".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert_eq!(result.value.len(), 1);
        assert_eq!(result.value.first().unwrap().as_integer(), Some(42));
    }

    #[tokio::test]
    async fn test_as_operator_invalid_cast() {
        let evaluator = AsOperatorEvaluator::new();
        let context = EvaluationContext::new(
            Collection::empty(),
            std::sync::Arc::new(crate::core::types::test_utils::create_test_model_provider()),
            None,
            None,
            None,
        )
        .await;

        let left = vec![FhirPathValue::integer(42)];
        let right = vec![FhirPathValue::string("boolean".to_string())];

        let result = evaluator
            .evaluate(vec![], &context, left, right)
            .await
            .unwrap();

        assert!(result.value.is_empty());
    }
}
