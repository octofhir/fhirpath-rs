//! checkModifiers function implementation
//!
//! The checkModifiers function checks that there are no modifier extensions
//! other than the ones explicitly allowed by the caller.
//! If unknown modifier extensions are found, an error is raised.
//! Syntax: resource.checkModifiers(modifier1, modifier2, ...)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// CheckModifiers function evaluator
pub struct CheckModifiersFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl CheckModifiersFunctionEvaluator {
    /// Create a new checkModifiers function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "checkModifiers".to_string(),
                description: "Checks that there are no modifier extensions other than the ones listed. Raises error if unknown modifiers found.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "modifier".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "URLs of allowed modifier extensions".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: None, // Variadic
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
impl PureFunctionEvaluator for CheckModifiersFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        // Collect allowed modifier extension URLs from all arguments
        let mut allowed_urls: Vec<String> = Vec::new();
        for arg_values in &args {
            for val in arg_values {
                if let FhirPathValue::String(url, _, _) = val {
                    allowed_urls.push(url.clone());
                }
            }
        }

        // Check each input element for modifier extensions
        for item in &input {
            if let FhirPathValue::Resource(json, _, _) = item
                && let Some(modifier_extensions) =
                    json.get("modifierExtension").and_then(|e| e.as_array())
            {
                for ext in modifier_extensions {
                    if let Some(url) = ext.get("url").and_then(|u| u.as_str())
                        && !allowed_urls.iter().any(|allowed| allowed == url)
                    {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0058,
                            format!(
                                "Unknown modifier extension '{}' is not in the allowed list",
                                url
                            ),
                        ));
                    }
                }
            }
        }

        // Return input unchanged if no unknown modifiers
        Ok(EvaluationResult { value: input })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
