//! `pathname([short])` (FHIRPath §1.5.10.2, STU)
//!
//! The path of each input item within the root resource, using only element names and
//! indexers (`Patient.name[0].given[1]`), such that evaluating it on the root resource gives
//! that item. Items that were computed rather than navigated, or that lie outside the root
//! resource, have no path and are left out. With `short = true`, indexers are kept only on
//! elements that are arrays in the instance.

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// pathname() function evaluator
pub struct PathnameFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl PathnameFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "pathname".to_string(),
                description: "The path of each input item within the root resource".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "short".to_string(),
                        parameter_type: vec!["Boolean".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "Leave out indexers on elements that are not arrays"
                            .to_string(),
                        default_value: Some("false".to_string()),
                    }],
                    return_type: "String".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::None,
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
impl PureFunctionEvaluator for PathnameFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        let short = match args.first() {
            None => false,
            Some(arg) if arg.is_empty() => false,
            Some(arg) => match (arg.len(), arg.first()) {
                (1, Some(FhirPathValue::Boolean(short, _, _))) => *short,
                _ => {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0057,
                        "pathname short argument must be a single Boolean".to_string(),
                    ));
                }
            },
        };
        let paths: Vec<FhirPathValue> = input
            .iter()
            .filter_map(|value| value.origin())
            .map(|origin| FhirPathValue::string(origin.path(short)))
            .collect();
        Ok(EvaluationResult {
            value: Collection::from(paths),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
