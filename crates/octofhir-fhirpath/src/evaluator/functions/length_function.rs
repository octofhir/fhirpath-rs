//! Length function implementation
//!
//! The length function returns the number of items in a collection.
//! Syntax: collection.length()

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Length function evaluator
pub struct LengthFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl LengthFunctionEvaluator {
    /// Create a new length function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "length".to_string(),
                description: "Returns the number of items in a collection".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![],
                    return_type: "Integer".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for LengthFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        _args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !_args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "length function takes no arguments".to_string(),
            ));
        }

        // length() has different behavior based on input:
        // - For collections: returns the number of items in the collection
        // - For single string: returns the number of characters in the string

        if input.len() == 1 {
            match &input[0] {
                FhirPathValue::String(s, _, _) => {
                    // For a single string, return its character length
                    let string_length = s.chars().count() as i64;
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::integer(
                            string_length,
                        )),
                    });
                }
                _ => {
                    // For a single non-string value, return 1 (collection length)
                    return Ok(EvaluationResult {
                        value: crate::core::Collection::single(FhirPathValue::integer(1)),
                    });
                }
            }
        }

        // For collections with multiple items, return the collection size
        let collection_length = input.len() as i64;

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::integer(collection_length)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
