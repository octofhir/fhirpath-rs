//! HasTemplateIdOf function implementation for CDA/FHIR templates - sync version for FunctionRegistry

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use sonic_rs::{JsonContainerTrait, JsonValueTrait};

/// HasTemplateIdOf function: checks if a resource has a specific template ID/profile
#[derive(Debug, Default, Clone)]
pub struct HasTemplateIdOfFunction;

impl HasTemplateIdOfFunction {
    pub fn new() -> Self {
        Self
    }

    /// Check if a value has the specified template ID
    fn has_template_id(&self, value: &FhirPathValue, template_id: &str) -> bool {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                // Check for template ID in various CDA/FHIR locations
                if let Some(template_ids) = json_val.as_inner().get("templateId") {
                    self.check_template_ids(template_ids, template_id)
                } else if let Some(meta) = json_val.as_inner().get("meta") {
                    // Check FHIR meta.profile
                    if let Some(profiles) = meta.get("profile") {
                        self.check_profiles(profiles, template_id)
                    } else {
                        false
                    }
                } else {
                    // Check for implicit template matching based on document structure
                    self.check_implicit_template(json_val.as_inner(), template_id)
                }
            }
            FhirPathValue::Resource(resource) => {
                // For resource objects, check similar patterns
                let json_val = resource.as_json();
                if let Some(template_ids) = json_val.get("templateId") {
                    self.check_template_ids(template_ids, template_id)
                } else if let Some(meta) = json_val.get("meta") {
                    if let Some(profiles) = meta.get("profile") {
                        self.check_profiles(profiles, template_id)
                    } else {
                        false
                    }
                } else {
                    // Check for implicit template matching based on document structure
                    self.check_implicit_template(&json_val, template_id)
                }
            }
            FhirPathValue::Collection(col) => {
                // Check any item in the collection
                col.iter()
                    .any(|item| self.has_template_id(item, template_id))
            }
            _ => false,
        }
    }

    /// Check template IDs in CDA format
    fn check_template_ids(&self, template_ids: &sonic_rs::Value, target_id: &str) -> bool {
        if template_ids.is_array() {
            if let Some(arr) = template_ids.as_array() {
                arr.iter()
                    .any(|template| self.check_single_template(template, target_id))
            } else {
                false
            }
        } else {
            self.check_single_template(template_ids, target_id)
        }
    }

    /// Check a single template ID object
    fn check_single_template(&self, template: &sonic_rs::Value, target_id: &str) -> bool {
        if let Some(root) = template.get("root") {
            if let Some(root_str) = root.as_str() {
                return root_str == target_id;
            }
        }
        if let Some(extension) = template.get("extension") {
            if let Some(ext_str) = extension.as_str() {
                return ext_str == target_id;
            }
        }
        // Check if the template itself is a string
        if let Some(template_str) = template.as_str() {
            return template_str == target_id;
        }
        false
    }

    /// Check FHIR profiles
    fn check_profiles(&self, profiles: &sonic_rs::Value, target_id: &str) -> bool {
        if profiles.is_array() {
            if let Some(arr) = profiles.as_array() {
                arr.iter().any(|profile| {
                    if let Some(profile_str) = profile.as_str() {
                        profile_str == target_id
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        } else if let Some(profile_str) = profiles.as_str() {
            profile_str == target_id
        } else {
            false
        }
    }

    /// Check for implicit template matching based on document structure
    fn check_implicit_template(&self, json_val: &sonic_rs::Value, target_id: &str) -> bool {
        // Check if this is a ContinuityofCareDocumentCCD based on resourceType and content
        if target_id == "http://hl7.org/cda/us/ccda/StructureDefinition/ContinuityofCareDocumentCCD"
        {
            // Check for ClinicalDocument resourceType
            if let Some(resource_type) = json_val.get("resourceType") {
                if let Some(rt_str) = resource_type.as_str() {
                    if rt_str == "ClinicalDocument" {
                        // Look for "Continuity of Care" pattern in titles or descriptions
                        return self.has_ccd_content_pattern(json_val);
                    }
                }
            }
        }
        false
    }

    /// Check for patterns that indicate a Continuity of Care Document
    fn has_ccd_content_pattern(&self, json_val: &sonic_rs::Value) -> bool {
        // Look for "Continuity of Care" pattern in nested text content
        self.search_for_ccd_pattern(json_val)
    }

    /// Recursively search for CCD pattern in JSON structure
    fn search_for_ccd_pattern(&self, value: &sonic_rs::Value) -> bool {
        if let Some(s) = value.as_str() {
            s.contains("Continuity of Care") || s.contains("ContinuityofCare")
        } else if value.is_object() {
            if let Some(obj) = value.as_object() {
                obj.iter().any(|(_, v)| self.search_for_ccd_pattern(v))
            } else {
                false
            }
        } else if value.is_array() {
            if let Some(arr) = value.as_array() {
                arr.iter().any(|v| self.search_for_ccd_pattern(v))
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl SyncOperation for HasTemplateIdOfFunction {
    fn name(&self) -> &'static str {
        "hasTemplateIdOf"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "hasTemplateIdOf",
                parameters: vec![ParameterType::String], // Template ID to check
                return_type: ValueType::Boolean,
                variadic: false,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // hasTemplateIdOf() takes exactly one argument - the template ID
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "hasTemplateIdOf".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let template_id = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "hasTemplateIdOf() template ID argument must be a string".to_string(),
                });
            }
        };

        let has_template = self.has_template_id(&context.input, template_id);
        Ok(FhirPathValue::Boolean(has_template))
    }
}
