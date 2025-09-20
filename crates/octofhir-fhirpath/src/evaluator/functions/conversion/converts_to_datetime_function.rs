//! ConvertsToDateTime function implementation
//!
//! This function tests if a value can be converted to a DateTime.

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use std::sync::Arc;

/// ConvertsToDateTime function evaluator
pub struct ConvertsToDateTimeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToDateTimeFunctionEvaluator {
    /// Create a new convertsToDateTime function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToDateTime".to_string(),
                description: "Tests if the input can be converted to a DateTime".to_string(),
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
impl PureFunctionEvaluator for ConvertsToDateTimeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "convertsToDateTime function takes no arguments".to_string(),
            ));
        }

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::String(s, _, _) => {
                    // Test if string can be parsed as DateTime, Date, or partial datetime (which can be converted to DateTime)
                    use crate::core::temporal::{PrecisionDate, PrecisionDateTime};

                    // First try parsing as a full DateTime
                    if PrecisionDateTime::parse(s).is_some() {
                        true
                    } else if PrecisionDate::parse(s).is_some() {
                        // Date can be converted to DateTime
                        true
                    } else {
                        // Try parsing partial datetime strings like "2015-02-04T14" or "2015-02-04T14:28"
                        // These are essentially datetime strings with partial time information
                        s.contains('T') && s.len() >= 13 // Minimum for "2015-02-04T14"
                    }
                }
                FhirPathValue::DateTime(_, _, _) => true, // Already a DateTime
                FhirPathValue::Date(_, _, _) => true,     // Date can be converted to DateTime
                _ => false, // Other types cannot be converted to DateTime
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
