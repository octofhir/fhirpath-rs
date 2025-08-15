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

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Extension function - finds extensions by URL
#[derive(Debug, Clone)]
pub struct ExtensionFunction;

impl ExtensionFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("extension", OperationType::Function)
            .description("Finds extensions with the specified URL")
            .parameter("url", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Any)
            .example("Patient.extension('http://hl7.org/fhir/StructureDefinition/patient-birthTime')")
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
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ExtensionFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len()
            });
        }

        let url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => return Err(FhirPathError::TypeError {
                message: "extension() url argument must be a string".to_string()
            }),
        };

        match &context.input {
            FhirPathValue::JsonValue(json) => {
                // Look for extension array
                if let Some(extensions) = json.get_property("extension") {
                    if let Some(ext_array) = extensions.as_json().as_array() {
                        let mut matching_extensions = Vec::new();

                        for ext in ext_array {
                            if let Some(ext_obj) = ext.as_object() {
                                if let Some(ext_url) = ext_obj.get("url") {
                                    if let Some(ext_url_str) = ext_url.as_str() {
                                        if ext_url_str == url.as_ref() {
                                            matching_extensions.push(FhirPathValue::resource_from_json(ext.clone()));
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
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            },
            FhirPathValue::Collection(c) => {
                let mut all_matching = Vec::new();

                for item in c.iter() {
                    let item_context = EvaluationContext::new(item.clone(), context.registry.clone(), context.model_provider.clone());
                    let result = self.evaluate(args, &item_context).await?;

                    match result {
                        FhirPathValue::Collection(items) => {
                            all_matching.extend(items.iter().cloned());
                        },
                        FhirPathValue::Empty => {},
                        single_item => all_matching.push(single_item),
                    }
                }

                if all_matching.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(all_matching))
                }
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // extension() can be sync since it only does JSON traversal
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len()
            }));
        }

        let url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => return Some(Err(FhirPathError::TypeError {
                message: "extension() url argument must be a string".to_string()
            })),
        };

        match &context.input {
            FhirPathValue::JsonValue(json) => {
                if let Some(extensions) = json.get_property("extension") {
                    if let Some(ext_array) = extensions.as_json().as_array() {
                        let mut matching_extensions = Vec::new();

                        for ext in ext_array {
                            if let Some(ext_obj) = ext.as_object() {
                                if let Some(ext_url) = ext_obj.get("url") {
                                    if let Some(ext_url_str) = ext_url.as_str() {
                                        if ext_url_str == url.as_ref() {
                                            matching_extensions.push(FhirPathValue::resource_from_json(ext.clone()));
                                        }
                                    }
                                }
                            }
                        }

                        if matching_extensions.is_empty() {
                            Some(Ok(FhirPathValue::Empty))
                        } else if matching_extensions.len() == 1 {
                            Some(Ok(matching_extensions.into_iter().next().unwrap()))
                        } else {
                            Some(Ok(FhirPathValue::collection(matching_extensions)))
                        }
                    } else {
                        Some(Ok(FhirPathValue::Empty))
                    }
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            },
            FhirPathValue::Resource(resource) => {
                let json = resource.as_json();
                if let Some(extensions) = json.get("extension") {
                    if let Some(ext_array) = extensions.as_array() {
                        let mut matching_extensions = Vec::new();

                        for ext in ext_array {
                            if let Some(ext_obj) = ext.as_object() {
                                if let Some(ext_url) = ext_obj.get("url") {
                                    if let Some(ext_url_str) = ext_url.as_str() {
                                        if ext_url_str == url.as_ref() {
                                            matching_extensions.push(FhirPathValue::resource_from_json(ext.clone()));
                                        }
                                    }
                                }
                            }
                        }

                        if matching_extensions.is_empty() {
                            Some(Ok(FhirPathValue::Empty))
                        } else if matching_extensions.len() == 1 {
                            Some(Ok(matching_extensions.into_iter().next().unwrap()))
                        } else {
                            Some(Ok(FhirPathValue::collection(matching_extensions)))
                        }
                    } else {
                        Some(Ok(FhirPathValue::Empty))
                    }
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            },
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            _ => Some(Ok(FhirPathValue::Empty)),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_extension_function() {
        let func = ExtensionFunction::new();

        // Test with extension present
        let patient = json!({
            "resourceType": "Patient",
            "extension": [
                {
                    "url": "http://example.org/birthTime",
                    "valueDateTime": "1995-06-15T12:30:00Z"
                },
                {
                    "url": "http://example.org/other",
                    "valueString": "other value"
                }
            ]
        });

        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::resource_from_json(patient), registry, model_provider)
        };
        let args = vec![FhirPathValue::String("http://example.org/birthTime".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();

        // The extension function returns a Resource, not a JsonValue
        match result {
            FhirPathValue::Resource(resource) => {
                let json_value = resource.as_json();
                if let Some(ext) = json_value.as_object() {
                    assert_eq!(ext.get("url").unwrap().as_str().unwrap(), "http://example.org/birthTime");
                    assert_eq!(ext.get("valueDateTime").unwrap().as_str().unwrap(), "1995-06-15T12:30:00Z");
                } else {
                    panic!("Expected extension object");
                }
            }
            FhirPathValue::Collection(c) => {
                assert_eq!(c.len(), 1);
                let first_value = c.first().unwrap();
                match first_value {
                    FhirPathValue::Resource(resource) => {
                        let json_value = resource.as_json();
                        if let Some(ext) = json_value.as_object() {
                            assert_eq!(ext.get("url").unwrap().as_str().unwrap(), "http://example.org/birthTime");
                            assert_eq!(ext.get("valueDateTime").unwrap().as_str().unwrap(), "1995-06-15T12:30:00Z");
                        } else {
                            panic!("Expected extension object");
                        }
                    }
                    _ => panic!("Expected resource in collection"),
                }
            }
            FhirPathValue::Empty => panic!("Expected resource result from extension()"),
            _ => panic!("Expected resource result from extension()"),
        }

        // Test with non-existent extension
        let args = vec![FhirPathValue::String("http://example.org/nonexistent".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_extension_no_extensions() {
        let func = ExtensionFunction::new();

        let patient = json!({
            "resourceType": "Patient",
            "name": [{"family": "Doe"}]
        });

        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::resource_from_json(patient), registry, model_provider)
        };
        let args = vec![FhirPathValue::String("http://example.org/any".into())];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_extension_sync() {
        let func = ExtensionFunction::new();

        let patient = json!({
            "extension": [
                {
                    "url": "http://example.org/test",
                    "valueString": "test value"
                }
            ]
        });

        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::resource_from_json(patient), registry, model_provider)
        };
        let args = vec![FhirPathValue::String("http://example.org/test".into())];
        let result = func.try_evaluate_sync(&args, &ctx).unwrap().unwrap();

        match result {
            FhirPathValue::Resource(resource) => {
                let json_value = resource.as_json();
                if let Some(ext) = json_value.as_object() {
                    assert_eq!(ext.get("valueString").unwrap().as_str().unwrap(), "test value");
                } else {
                    panic!("Expected extension object");
                }
            }
            FhirPathValue::Collection(c) => {
                assert_eq!(c.len(), 1);
                let first_value = c.first().unwrap();
                match first_value {
                    FhirPathValue::Resource(resource) => {
                        let json_value = resource.as_json();
                        if let Some(ext) = json_value.as_object() {
                            assert_eq!(ext.get("valueString").unwrap().as_str().unwrap(), "test value");
                        } else {
                            panic!("Expected extension object");
                        }
                    }
                    _ => panic!("Expected resource in collection"),
                }
            }
            FhirPathValue::Empty => panic!("Expected resource result from extension()"),
            _ => panic!("Expected resource result from extension()"),
        }
    }

    #[tokio::test]
    async fn test_extension_invalid_args() {
        let func = ExtensionFunction::new();
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::resource_from_json(json!({})), registry, model_provider);

        // No arguments
        let result = func.evaluate(&[], &ctx).await;
        assert!(result.is_err());

        // Wrong argument type
        let args = vec![FhirPathValue::Integer(123)];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}
