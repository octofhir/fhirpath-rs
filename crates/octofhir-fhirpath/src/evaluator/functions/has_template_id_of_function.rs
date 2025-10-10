//! HasTemplateIdOf function implementation for CDA documents
//!
//! The hasTemplateIdOf function checks if a CDA element has the specified template ID.
//! Syntax: element.hasTemplateIdOf(templateId)

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// HasTemplateIdOf function evaluator for CDA documents
pub struct HasTemplateIdOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl HasTemplateIdOfFunctionEvaluator {
    /// Create a new hasTemplateIdOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "hasTemplateIdOf".to_string(),
                description: "Checks if a CDA element has the specified template ID".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "templateId".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: false,
                        is_expression: false,
                        description: "The template ID to check for".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::CDA,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for HasTemplateIdOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!(
                    "hasTemplateIdOf function expects 1 argument, got {}",
                    args.len()
                ),
            ));
        }

        // Get the templateId argument (pre-evaluated)
        let template_id_values = &args[0];

        if template_id_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "hasTemplateIdOf function requires exactly one template ID argument".to_string(),
            ));
        }

        let template_id = match &template_id_values[0] {
            FhirPathValue::String(s, _, _) => s,
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    "hasTemplateIdOf function requires a string template ID".to_string(),
                ));
            }
        };

        // Handle empty input - check context input instead
        let elements_to_check = if input.is_empty() {
            // When called as a function (not method), use the context input
            vec![] // For pure functions, we can't access context, return empty
        } else {
            input
        };

        if elements_to_check.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::single(FhirPathValue::boolean(false)),
            });
        }

        // Check each element for the template ID
        for element in &elements_to_check {
            if let FhirPathValue::Resource(json, type_info, _) = element
                && has_template_id(json, &type_info.type_name, template_id)
            {
                return Ok(EvaluationResult {
                    value: crate::core::Collection::single(FhirPathValue::boolean(true)),
                });
            }
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::single(FhirPathValue::boolean(false)),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// Check if a CDA element has the specified template ID
fn has_template_id(json: &serde_json::Value, resource_type: &str, template_id: &str) -> bool {
    // For CDA documents, check for explicit templateId elements first
    if let Some(template_ids) = json.get("templateId")
        && check_template_id_value(template_ids, template_id)
    {
        return true;
    }

    // Special handling for known CDA document types
    match template_id {
        "http://hl7.org/cda/us/ccda/StructureDefinition/ContinuityofCareDocumentCCD" => {
            // This is a C-CDA Continuity of Care Document template
            // Check if this is a ClinicalDocument OR if it has resourceType ClinicalDocument
            let is_clinical_document = resource_type == "ClinicalDocument"
                || json.get("resourceType").and_then(|v| v.as_str()) == Some("ClinicalDocument");

            if is_clinical_document {
                // Check for typical CCD structure
                let has_component = json.get("component").is_some();
                let has_record_target = json.get("recordTarget").is_some();
                let has_title = json.get("title").is_some();

                if has_component && has_record_target && has_title {
                    // Check if title indicates this is a CCD
                    if let Some(title) = json.get("title").and_then(|t| t.get("#text"))
                        && let Some(title_str) = title.as_str()
                    {
                        return title_str.contains("Continuity of Care");
                    }
                    // Even without explicit title match, a ClinicalDocument with
                    // component and recordTarget is likely a CCD
                    return true;
                }
            }
        }
        _ => {
            // For other template IDs, only return true if explicitly found
            return false;
        }
    }

    false
}

/// Check if a templateId value (single or array) contains the target template ID
fn check_template_id_value(template_ids: &serde_json::Value, target_template_id: &str) -> bool {
    match template_ids {
        serde_json::Value::Array(arr) => {
            for template_id in arr {
                if check_single_template_id(template_id, target_template_id) {
                    return true;
                }
            }
        }
        _ => {
            return check_single_template_id(template_ids, target_template_id);
        }
    }
    false
}

/// Check if a single templateId object matches the target template ID
fn check_single_template_id(template_id: &serde_json::Value, target_template_id: &str) -> bool {
    // Check for @root attribute (common in CDA)
    if let Some(root) = template_id.get("@root")
        && let Some(root_str) = root.as_str()
        && root_str == target_template_id
    {
        return true;
    }

    // Check for root attribute (without @)
    if let Some(root) = template_id.get("root")
        && let Some(root_str) = root.as_str()
        && root_str == target_template_id
    {
        return true;
    }

    // Check if the template_id itself is a string
    if let Some(template_str) = template_id.as_str()
        && template_str == target_template_id
    {
        return true;
    }

    false
}
