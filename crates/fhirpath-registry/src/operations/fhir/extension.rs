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

//! Extension function implementation - finds FHIR extensions by URL

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use sonic_rs::{JsonContainerTrait, JsonValueTrait};

/// Extension function - finds extensions by URL
#[derive(Debug, Clone)]
pub struct ExtensionFunction;

impl Default for ExtensionFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionFunction {
    pub fn new() -> Self {
        Self
    }

    /// Find extensions in JSON, checking both direct extensions and underscore elements
    fn find_extensions_in_json(&self, json: &sonic_rs::Value, url: &str) -> Result<FhirPathValue> {
        let mut matching_extensions = Vec::new();

        // First, check for direct extension array
        if let Some(extensions) = json.get("extension") {
            if let Some(ext_array) = extensions.as_array() {
                for ext in ext_array {
                    if let Some(ext_obj) = ext.as_object() {
                        if let Some(ext_url) = ext_obj.get(&"url") {
                            if let Some(ext_url_str) = ext_url.as_str() {
                                if ext_url_str == url {
                                    matching_extensions
                                        .push(FhirPathValue::resource_from_json(ext.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if matching_extensions.is_empty() {
            Ok(FhirPathValue::Empty)
        } else if matching_extensions.len() == 1 {
            Ok(matching_extensions.into_iter().next().unwrap())
        } else {
            Ok(FhirPathValue::collection(matching_extensions))
        }
    }

    /// Find extensions for primitive values by looking in the parent resource's underscore elements
    async fn find_primitive_extensions(
        &self,
        context: &EvaluationContext,
        url: &str,
    ) -> Result<FhirPathValue> {
        // For primitive values, we need to check the root resource for underscore elements
        // This is a FHIR-specific behavior where extensions for primitive values are stored
        // in parallel underscore elements

        // Get the root resource from context
        let root_resource = &context.root;

        if let FhirPathValue::JsonValue(root_json) = root_resource {
            let root_obj = root_json.as_sonic_value();

            // Look for all properties that start with underscore (primitive extensions)
            if let Some(root_map) = root_obj.as_object() {
                for (property_name, property_value) in root_map {
                    if property_name.starts_with('_') {
                        if let Some(extensions) = property_value.get("extension") {
                            if let Some(ext_array) = extensions.as_array() {
                                let mut matching_extensions = Vec::new();

                                for ext in ext_array {
                                    if let Some(ext_obj) = ext.as_object() {
                                        if let Some(ext_url) = ext_obj.get(&"url") {
                                            if let Some(ext_url_str) = ext_url.as_str() {
                                                if ext_url_str == url {
                                                    matching_extensions.push(
                                                        FhirPathValue::resource_from_json(
                                                            ext.clone(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                if !matching_extensions.is_empty() {
                                    return if matching_extensions.len() == 1 {
                                        Ok(matching_extensions.into_iter().next().unwrap())
                                    } else {
                                        Ok(FhirPathValue::collection(matching_extensions))
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(FhirPathValue::Empty)
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("extension", OperationType::Function)
            .description("Finds extensions with the specified URL")
            .parameter("url", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Any)
            .example(
                "Patient.extension('http://hl7.org/fhir/StructureDefinition/patient-birthTime')",
            )
            .example("extension('http://example.org/my-extension')")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ExtensionFunction {
    fn identifier(&self) -> &str {
        "extension"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ExtensionFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let url = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(coll) if coll.len() == 1 => {
                match coll.iter().next().unwrap() {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(FhirPathError::TypeError {
                            message: "extension() url argument must be a string".to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "extension() url argument must be a string".to_string(),
                });
            }
        };

        // Use the ModelProvider to find extensions, which knows about FHIR extension rules
        let extensions = context
            .model_provider
            .find_extensions_by_url(&context.input, &context.root, None, url)
            .await;

        match extensions.len() {
            0 => {
                // Try primitive extension lookup as fallback
                let primitive_result = self.find_primitive_extensions(context, url).await?;
                if !matches!(primitive_result, FhirPathValue::Empty) {
                    return Ok(primitive_result);
                }

                // Handle collection case by delegating to collection processing
                match &context.input {
                    FhirPathValue::Collection(c) => {
                        let mut all_matching = Vec::new();

                        for item in c.iter() {
                            let item_context = EvaluationContext::new(
                                item.clone(),
                                context.registry.clone(),
                                context.model_provider.clone(),
                            );
                            let result = self.evaluate(args, &item_context).await?;

                            match result {
                                FhirPathValue::Collection(items) => {
                                    all_matching.extend(items.iter().cloned());
                                }
                                FhirPathValue::Empty => {}
                                single_item => all_matching.push(single_item),
                            }
                        }

                        if all_matching.is_empty() {
                            Ok(FhirPathValue::Empty)
                        } else {
                            Ok(FhirPathValue::collection(all_matching))
                        }
                    }
                    _ => Ok(FhirPathValue::Empty),
                }
            }
            1 => Ok(extensions.into_iter().next().unwrap()),
            _ => Ok(FhirPathValue::collection(extensions)),
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // extension() requires ModelProvider which is async, so force async evaluation
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
