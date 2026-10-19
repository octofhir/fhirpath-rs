//! HasValue function implementation
//!
//! The hasValue function checks if a value is not empty/null.
//! Syntax: value.hasValue()

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

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
        input: Collection,
        _args: Vec<Collection>,
    ) -> Result<EvaluationResult> {
        self.evaluate_sync(input, _args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, _args: Vec<Collection>) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "hasValue function takes no arguments".to_string(),
            ));
        }

        // FHIR: true if the input is a single FHIR primitive that has a value.
        // A complex type (an element or resource node, or a Quantity) never
        // has a value, even when it has content; nor does a primitive that
        // carries only extensions.
        let has_value = input.len() == 1
            && match input.first().unwrap() {
                FhirPathValue::String(s, _, _) => !s.is_empty(),
                FhirPathValue::Boolean(_, _, _)
                | FhirPathValue::Integer(_, _, _)
                | FhirPathValue::Decimal(_, _, _)
                | FhirPathValue::Date(_, _, _)
                | FhirPathValue::DateTime(_, _, _)
                | FhirPathValue::Time(_, _, _) => true,
                _ => false,
            };

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(has_value)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
