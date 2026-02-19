//! slice function implementation
//!
//! The slice function filters a collection based on a profile slice definition.
//! Syntax: element.slice(structure, name)

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// Slice function evaluator
pub struct SliceFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SliceFunctionEvaluator {
    /// Create a new slice function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "slice".to_string(),
                description: "Filters a collection based on a profile slice definition".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "structure".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "The canonical URL of the StructureDefinition / profile"
                                .to_string(),
                            default_value: None,
                        },
                        FunctionParameter {
                            name: "name".to_string(),
                            parameter_type: vec!["String".to_string()],
                            optional: false,
                            is_expression: false,
                            description: "The name of the slice".to_string(),
                            default_value: None,
                        },
                    ],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 2,
                    max_params: Some(2),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::FilteringProjection,
                requires_terminology: false,
                requires_model: true,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for SliceFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        if args.len() != 2 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "slice function requires exactly 2 arguments (structure, name)".to_string(),
            ));
        }

        let _structure_url = match args[0].first() {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "slice function first argument (structure) must be a string".to_string(),
                ));
            }
        };

        let slice_name = match args[1].first() {
            Some(FhirPathValue::String(s, _, _)) => s.clone(),
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0056,
                    "slice function second argument (name) must be a string".to_string(),
                ));
            }
        };

        if input.is_empty() {
            return Ok(EvaluationResult {
                value: Collection::empty(),
            });
        }

        // Basic slice filtering: match elements that have a profile/meta matching the slice
        // Full implementation requires FhirSchemaModelProvider with slicing discriminators
        // For now, implement basic pattern-based matching on common discriminator types
        let mut matched = Vec::new();

        for item in input {
            if let FhirPathValue::Resource(json, _, _) = &item {
                // Check if element matches by looking at common slice discriminator patterns:
                // 1. type discriminator (check resourceType or code/system)
                // 2. value discriminator (check specific field values)
                // 3. profile discriminator (check meta.profile)

                // Pattern: check if element has a "url" field matching the slice name
                // (common for extension slicing)
                if let Some(url) = json.get("url").and_then(|u| u.as_str())
                    && (url.ends_with(&slice_name) || url == slice_name)
                {
                    matched.push(item);
                    continue;
                }

                // Pattern: check meta.profile for profile-discriminated slices
                if let Some(profiles) = json
                    .get("meta")
                    .and_then(|m| m.get("profile"))
                    .and_then(|p| p.as_array())
                {
                    for profile in profiles {
                        if let Some(p) = profile.as_str()
                            && (p.ends_with(&slice_name) || p == slice_name)
                        {
                            matched.push(item.clone());
                            break;
                        }
                    }
                }
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(matched),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
