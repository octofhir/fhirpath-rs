//! ConvertsToString function implementation
//!
//! The convertsToString function tests whether a value can be converted to a string.
//! Syntax: value.convertsToString()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::EvaluationResult;

/// ConvertsToString function evaluator
pub struct ConvertsToStringFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToStringFunctionEvaluator {
    /// Create a new convertsToString function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToString".to_string(),
                description: "Tests whether a value can be converted to a string".to_string(),
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
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Conversion,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToStringFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "convertsToString function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::String(_, _, _) => true,
                FhirPathValue::Boolean(_, _, _) => true,
                FhirPathValue::Integer(_, _, _) => true,
                FhirPathValue::Decimal(_, _, _) => true,
                FhirPathValue::Date(_, _, _) => true,
                FhirPathValue::DateTime(_, _, _) => true,
                FhirPathValue::Time(_, _, _) => true,
                FhirPathValue::Quantity { .. } => true,
                _ => false, // Other types cannot be converted to string
            };

            results.push(FhirPathValue::boolean(can_convert));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
