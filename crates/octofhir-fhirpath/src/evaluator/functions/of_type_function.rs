//! ofType function implementation
//!
//! The ofType function allows filtering collections by type using comprehensive
//! ModelProvider integration for schema-driven type checking.

use async_trait::async_trait;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// ofType function evaluator for filtering collections by type
pub struct OfTypeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl OfTypeFunctionEvaluator {
    /// Create a new ofType function evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl FunctionEvaluator for OfTypeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        // Spec: ofType(type) filters the input collection to only items of the specified type

        // Check argument count
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "ofType function requires exactly one argument",
            ));
        }

        // Evaluate the type argument
        let type_result = evaluator.evaluate(&args[0], context).await?;
        let type_values = type_result.value.values();

        if type_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "ofType function requires a single type identifier",
            ));
        }

        // Extract type name from argument (should be a string literal representing type identifier)
        let type_name = if let Some(FhirPathValue::String(type_str, _, _)) = type_values.first() {
            type_str.as_str()
        } else {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "ofType function requires a string type identifier",
            ));
        };

        // Filter the input collection based on type matching
        let mut filtered_items = Vec::new();

        for item in input {
            // Get type info for the item
            let item_type_info = item.type_info();

            // Use ModelProvider.of_type for schema-driven type checking
            if context
                .model_provider()
                .of_type(&item_type_info, type_name)
                .is_some()
            {
                filtered_items.push(item);
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(filtered_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Create metadata for the ofType function
fn create_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "ofType".to_string(),
        description: "Filter collection to items of specified type".to_string(),
        signature: FunctionSignature {
            input_type: "Collection".to_string(),
            parameters: vec![FunctionParameter {
                name: "type".to_string(),
                parameter_type: vec!["String".to_string()],
                optional: false,
                is_expression: false,
                description: "Type identifier to filter by".to_string(),
                default_value: None,
            }],
            return_type: "Collection".to_string(),
            polymorphic: true,
            min_params: 1,
            max_params: Some(1),
        },
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::FilteringProjection,
        requires_terminology: false,
        requires_model: true, // Requires ModelProvider for type checking
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;
    use crate::evaluator::AsyncNodeEvaluator;
    use crate::parser::ExpressionNode;

    async fn create_test_evaluator() -> AsyncNodeEvaluator<'static> {
        // This is a stub for tests - in real usage this would be properly constructed
        unsafe { std::mem::zeroed() }
    }

    #[tokio::test]
    async fn test_of_type_metadata() {
        let evaluator = OfTypeFunctionEvaluator::new();
        let metadata = evaluator.metadata();

        assert_eq!(metadata.name, "ofType");
        assert_eq!(metadata.signature.parameters.len(), 1);
        assert_eq!(metadata.signature.parameters[0].name, "type");
    }
}
