//! HasValue function implementation
//!
//! The hasValue function checks if a value is not empty/null.
//! Syntax: value.hasValue()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionEvaluator, PureFunctionEvaluator, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy,
};use crate::evaluator::EvaluationResult;

/// HasValue function evaluator
pub struct HasValueFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl HasValueFunctionEvaluator {
    /// Create a new hasValue function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "hasValue".to_string(),
                description: "Checks if a value is not empty/null".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Existence,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for HasValueFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "hasValue function takes no arguments".to_string(),
            ));
        }

        // hasValue returns true if the input collection is not empty
        // and contains at least one non-null value
        let has_value = !input.is_empty()
            && input.iter().any(|v| {
                // Check if the value is not null/empty
                match v {
                    // For primitive types, we consider them to have value if they exist
                    FhirPathValue::String(s, _, _) => !s.is_empty(),
                    FhirPathValue::Boolean(_, _, _) => true,
                    FhirPathValue::Integer(_, _, _) => true,
                    FhirPathValue::Decimal(_, _, _) => true,
                    FhirPathValue::Date(_, _, _) => true,
                    FhirPathValue::DateTime(_, _, _) => true,
                    FhirPathValue::Time(_, _, _) => true,
                    FhirPathValue::Quantity { .. } => true,
                    FhirPathValue::Resource(_, _, _) => true,
                    _ => false,
                }
            });

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(has_value)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}