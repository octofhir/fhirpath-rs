//! ToBoolean function implementation
//!
//! The toBoolean function converts a value to a boolean.
//! Syntax: value.toBoolean()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ToBoolean function evaluator
pub struct ToBooleanFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToBooleanFunctionEvaluator {
    /// Create a new toBoolean function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toBoolean".to_string(),
                description: "Converts a value to a boolean".to_string(),
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
impl PureFunctionEvaluator for ToBooleanFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toBoolean function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            match &value {
                FhirPathValue::Boolean(b, _, _) => {
                    results.push(FhirPathValue::boolean(*b));
                }
                FhirPathValue::Integer(i, _, _) => {
                    match *i {
                        1 => results.push(FhirPathValue::boolean(true)),
                        0 => results.push(FhirPathValue::boolean(false)),
                        _ => { /* non-convertible → no result (empty) */ }
                    }
                }
                FhirPathValue::Decimal(d, _, _) => {
                    if d.is_zero() {
                        results.push(FhirPathValue::boolean(false));
                    } else if d == &rust_decimal::Decimal::ONE {
                        results.push(FhirPathValue::boolean(true));
                    } else {
                        // non-convertible → empty
                    }
                }
                FhirPathValue::String(s, _, _) => match s.trim().to_lowercase().as_str() {
                    "true" | "t" | "yes" | "y" | "1" | "1.0" => {
                        results.push(FhirPathValue::boolean(true));
                    }
                    "false" | "f" | "no" | "n" | "0" | "0.0" => {
                        results.push(FhirPathValue::boolean(false));
                    }
                    _ => { /* non-convertible → empty */ }
                },
                _ => { /* non-convertible type → empty */ }
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
