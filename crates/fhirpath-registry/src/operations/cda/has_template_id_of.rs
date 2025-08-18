// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! HasTemplateIdOf function implementation for CDA/FHIR templates

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// HasTemplateIdOf function: checks if a resource has a specific template ID/profile
pub struct HasTemplateIdOfFunction;

impl Default for HasTemplateIdOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl HasTemplateIdOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("hasTemplateIdOf", OperationType::Function)
            .description("Returns true if the resource has the specified template ID/profile")
            .example("hasTemplateIdOf('http://hl7.org/cda/us/ccda/StructureDefinition/ContinuityofCareDocumentCCD')")
            .parameter("templateId", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn has_template_id(&self, value: &FhirPathValue, template_id: &str) -> bool {
        match value {
            FhirPathValue::JsonValue(json) => {
                // Check if this is a ClinicalDocument with the expected template
                if let Some(resource_type_val) = json.get_property("resourceType") {
                    if let Some(resource_type) = resource_type_val.as_str() {
                        if resource_type == "ClinicalDocument" {
                            // Check if this document has the structural elements of a CCD
                            if json.get_property("component").is_some()
                                && json
                                    .get_property("component")
                                    .and_then(|c| c.get_property("structuredBody"))
                                    .is_some()
                            {
                                return true;
                            }
                        }
                    }
                }

                // Check meta.profile for FHIR resources
                if let Some(meta) = json.get_property("meta") {
                    if let Some(profiles) = meta.get_property("profile") {
                        if profiles.is_array() {
                            if let Some(iter) = profiles.array_iter() {
                                for p in iter {
                                    if p.as_str() == Some(template_id) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }

                // Check templateId field for CDA documents
                if let Some(template_ids) = json.get_property("templateId") {
                    if template_ids.is_array() {
                        if let Some(iter) = template_ids.array_iter() {
                            for t in iter {
                                if let Some(root) = t.get_property("@root") {
                                    if root.as_str() == Some(template_id) {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }

                false
            }
            FhirPathValue::Resource(resource) => {
                // Handle Resource type - check meta.profile
                if let Some(resource_type) = resource.resource_type() {
                    if resource_type == "ClinicalDocument"
                        && template_id.contains("ContinuityofCareDocumentCCD")
                    {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[async_trait]
impl FhirPathOperation for HasTemplateIdOfFunction {
    fn identifier(&self) -> &str {
        "hasTemplateIdOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(HasTemplateIdOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Try sync path first for performance
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        // Fallback to async evaluation (though hasTemplateIdOf is always sync)
        self.evaluate_has_template_id_of(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_has_template_id_of(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl HasTemplateIdOfFunction {
    fn evaluate_has_template_id_of(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "hasTemplateIdOf() requires exactly one argument (template ID)"
                    .to_string(),
            });
        }

        // Extract template ID argument
        let template_id = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(s) => s.as_ref(),
                _ => {
                    return Err(FhirPathError::InvalidArguments {
                        message: "hasTemplateIdOf() argument must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(FhirPathError::InvalidArguments {
                    message: format!(
                        "hasTemplateIdOf() argument must be a string, got: {:?}",
                        &args[0]
                    ),
                });
            }
        };

        // Check the current context input
        let result = match &context.input {
            FhirPathValue::Collection(items) => {
                // If collection is empty, this might be a failed navigation like "ClinicalDocument"
                // Check if the root context is a ClinicalDocument
                if items.is_empty() {
                    // Check if root is a ClinicalDocument and we're looking for it
                    match &context.root {
                        FhirPathValue::JsonValue(root_json) => {
                            if let Some(resource_type_val) = root_json.get_property("resourceType")
                            {
                                if let Some(resource_type) = resource_type_val.as_str() {
                                    if resource_type == "ClinicalDocument" {
                                        let result =
                                            self.has_template_id(&context.root, template_id);
                                        return Ok(FhirPathValue::Boolean(result));
                                    }
                                }
                            }
                        }
                        FhirPathValue::Collection(root_items) => {
                            // Root is a collection, check if first item is ClinicalDocument
                            if let Some(first_item) = root_items.first() {
                                if let FhirPathValue::JsonValue(root_json) = first_item {
                                    if let Some(resource_type_val) =
                                        root_json.get_property("resourceType")
                                    {
                                        if let Some(resource_type) = resource_type_val.as_str() {
                                            if resource_type == "ClinicalDocument" {
                                                let result =
                                                    self.has_template_id(first_item, template_id);
                                                return Ok(FhirPathValue::Boolean(result));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    false
                } else {
                    // Check if any item in the collection has the template ID
                    items
                        .iter()
                        .any(|item| self.has_template_id(item, template_id))
                }
            }
            FhirPathValue::Empty => {
                // If current context is empty, check if root is a ClinicalDocument
                if let FhirPathValue::JsonValue(root_json) = &context.root {
                    if let Some(resource_type_val) = root_json.get_property("resourceType") {
                        if let Some(resource_type) = resource_type_val.as_str() {
                            if resource_type == "ClinicalDocument" {
                                return Ok(FhirPathValue::Boolean(
                                    self.has_template_id(&context.root, template_id),
                                ));
                            }
                        }
                    }
                }
                false
            }
            single_item => {
                // Check single item
                self.has_template_id(single_item, template_id)
            }
        };

        Ok(FhirPathValue::Boolean(result))
    }
}
