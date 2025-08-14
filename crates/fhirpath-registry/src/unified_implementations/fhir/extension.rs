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

//! Unified extension() function implementation

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult, FunctionCategory},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, resource::FhirResource};
use serde_json::Value;

/// Unified extension() function implementation
/// 
/// Retrieves extensions with a given URL from an element. For each item in the collection,
/// if it is a Resource, returns any extensions with the given URL. For primitive types,
/// looks for extensions in the corresponding _fieldName elements in the root resource.
pub struct UnifiedExtensionFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedExtensionFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 required string parameter
        let signature = FunctionSignature::new(
            "extension",
            vec![ParameterInfo::required("url", TypeInfo::String)],
            TypeInfo::Collection(Box::new(TypeInfo::Resource("Extension".to_string()))),
        );

        let metadata = MetadataBuilder::new("extension", FunctionCategory::FhirSpecific)
            .display_name("Extension")
            .description("Retrieves extensions with a given URL from an element")
            .example("Patient.extension('http://example.org/birthTime')")
            .example("Patient.name.given.extension('http://example.org/original-text')")
            .signature(signature)
            .output_type(TypePattern::CollectionOf(Box::new(TypePattern::Resource)))
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .lsp_snippet("extension('${1:url}')")
            .keywords(vec!["extension", "url", "fhir", "metadata", "primitive"])
            .usage_pattern(
                "Get extensions by URL",
                "element.extension(url)",
                "Extension retrieval and metadata access"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedExtensionFunction {
    fn name(&self) -> &str {
        "extension"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate single argument (URL)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }

        // Get the URL parameter
        let url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => {
                // Return empty collection for invalid URL parameter
                return Ok(FhirPathValue::collection(vec![]));
            }
        };

        let mut results = Vec::new();

        // Process the input collection
        match &context.input {
            FhirPathValue::Resource(resource) => {
                // Check if the resource itself has an extension field
                if let Some(extensions_value) = resource.get_property("extension") {
                    let fhir_path_value = value_to_fhir_path_value(extensions_value);
                    extract_matching_extensions(&fhir_path_value, url, &mut results);
                }
            }
            FhirPathValue::String(_)
            | FhirPathValue::Date(_)
            | FhirPathValue::DateTime(_)
            | FhirPathValue::Boolean(_)
            | FhirPathValue::Integer(_)
            | FhirPathValue::Decimal(_) => {
                // For primitive values, we look at the root resource to find
                // the corresponding _field extension
                if let FhirPathValue::Resource(root_resource) = &context.root {
                    extract_primitive_extensions_from_root(
                        root_resource,
                        &context.input,
                        url,
                        &mut results,
                    );
                }
            }
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    if let FhirPathValue::Resource(resource) = item {
                        if let Some(extensions_value) = resource.get_property("extension") {
                            let fhir_path_value = value_to_fhir_path_value(extensions_value);
                            extract_matching_extensions(&fhir_path_value, url, &mut results);
                        }
                    }
                }
            }
            _ => {
                // Other types don't have extensions
            }
        }

        Ok(FhirPathValue::collection(results))
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate_sync(args, context)
    }
}

fn extract_matching_extensions(
    extensions_value: &FhirPathValue,
    url: &str,
    results: &mut Vec<FhirPathValue>,
) {
    match extensions_value {
        FhirPathValue::Collection(extensions) => {
            for ext in extensions.iter() {
                if let FhirPathValue::Resource(ext_resource) = ext {
                    if let Some(url_value) = ext_resource.get_property("url") {
                        if let Some(ext_url) = url_value.as_str() {
                            if ext_url == url {
                                results.push(FhirPathValue::Resource(ext_resource.clone()));
                            }
                        }
                    }
                }
            }
        }
        FhirPathValue::Resource(single_ext) => {
            if let Some(url_value) = single_ext.get_property("url") {
                if let Some(ext_url) = url_value.as_str() {
                    if ext_url == url {
                        results.push(FhirPathValue::Resource(single_ext.clone()));
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_primitive_extensions_from_root(
    root_resource: &std::sync::Arc<FhirResource>,
    primitive_value: &FhirPathValue,
    target_url: &str,
    results: &mut Vec<FhirPathValue>,
) {
    // Get all properties from the root resource
    if let Some(root_json) = root_resource.as_json().as_object() {
        // Look for _fieldName patterns
        for (key, value) in root_json {
            if let Some(field_name) = key.strip_prefix('_') {
                // Remove the underscore

                // Check if this primitive field matches our current value
                if let Some(field_value) = root_json.get(field_name) {
                    if primitive_values_match(field_value, primitive_value) {
                        // Found a matching primitive field, extract extensions from the _field
                        if let Some(extensions_obj) = value.as_object() {
                            if let Some(extensions_array) = extensions_obj.get("extension") {
                                let fhir_path_value = value_to_fhir_path_value(extensions_array);
                                extract_matching_extensions(&fhir_path_value, target_url, results);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn primitive_values_match(json_value: &Value, fhir_value: &FhirPathValue) -> bool {
    match (json_value, fhir_value) {
        (Value::String(s), FhirPathValue::String(fs)) => s == fs.as_ref(),
        (Value::String(s), FhirPathValue::Date(fd)) => {
            // Compare string representation of date
            s == &fd.to_string()
        }
        (Value::String(s), FhirPathValue::DateTime(fdt)) => {
            // Compare string representation of datetime
            s == &fdt.to_string()
        }
        (Value::Bool(b), FhirPathValue::Boolean(fb)) => b == fb,
        (Value::Number(n), FhirPathValue::Integer(fi)) => {
            n.as_i64().map(|i| i == *fi).unwrap_or(false)
        }
        (Value::Number(n), FhirPathValue::Decimal(fd)) => n
            .as_f64()
            .and_then(rust_decimal::Decimal::from_f64_retain)
            .map(|d| d == *fd)
            .unwrap_or(false),
        _ => false,
    }
}

fn value_to_fhir_path_value(value: &Value) -> FhirPathValue {
    match value {
        Value::Array(arr) => {
            let mut collection = Vec::new();
            for item in arr {
                collection.push(value_to_fhir_path_value(item));
            }
            FhirPathValue::collection(collection)
        }
        Value::Object(_) => {
            let resource = FhirResource::from_json(value.clone());
            FhirPathValue::Resource(resource.into())
        }
        Value::String(s) => FhirPathValue::String(s.clone().into()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FhirPathValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                FhirPathValue::Decimal(
                    rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default(),
                )
            } else {
                FhirPathValue::Empty
            }
        }
        Value::Bool(b) => FhirPathValue::Boolean(*b),
        Value::Null => FhirPathValue::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::resource::FhirResource;
    use serde_json::json;

    #[tokio::test]
    async fn test_unified_extension_function() {
        let extension_func = UnifiedExtensionFunction::new();
        
        // Test with resource having extensions
        let patient_with_extension = json!({
            "resourceType": "Patient",
            "id": "patient1",
            "extension": [
                {
                    "url": "http://example.org/birthTime",
                    "valueTime": "14:35:00"
                },
                {
                    "url": "http://example.org/race",
                    "valueString": "Caucasian"
                }
            ],
            "name": [{"family": "Doe"}]
        });
        
        let patient_resource = FhirResource::from_json(patient_with_extension);
        let context = EvaluationContext::new(FhirPathValue::Resource(patient_resource.into()));
        
        // Test finding existing extension
        let args = vec![FhirPathValue::String("http://example.org/birthTime".into())];
        let result = extension_func.evaluate_sync(&args, &context).unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Resource(ext)) = items.get(0) {
                    assert_eq!(
                        ext.get_property("url").and_then(|v| v.as_str()),
                        Some("http://example.org/birthTime")
                    );
                    assert_eq!(
                        ext.get_property("valueTime").and_then(|v| v.as_str()),
                        Some("14:35:00")
                    );
                } else {
                    panic!("Expected Resource result");
                }
            },
            _ => panic!("Expected Collection result"),
        }
        
        // Test finding non-existent extension
        let args = vec![FhirPathValue::String("http://example.org/nonexistent".into())];
        let result = extension_func.evaluate_sync(&args, &context).unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 0);
            },
            _ => panic!("Expected empty Collection result"),
        }
        
        // Test metadata
        assert_eq!(extension_func.name(), "extension");
        assert_eq!(extension_func.execution_mode(), ExecutionMode::Sync);
        assert_eq!(extension_func.metadata().basic.display_name, "Extension");
        assert!(extension_func.metadata().basic.is_pure);
    }
    
    #[tokio::test]
    async fn test_extension_with_primitive_value() {
        let extension_func = UnifiedExtensionFunction::new();
        
        // Test with primitive extensions
        let patient_with_primitive_ext = json!({
            "resourceType": "Patient",
            "id": "patient1",
            "name": [
                {
                    "family": "Doe",
                    "given": ["John"]
                }
            ],
            "_name": [
                {
                    "_given": [
                        {
                            "extension": [
                                {
                                    "url": "http://example.org/original-text",
                                    "valueString": "Jonathan"
                                }
                            ]
                        }
                    ]
                }
            ]
        });
        
        let patient_resource = FhirResource::from_json(patient_with_primitive_ext.clone());
        
        // Test finding extension on primitive value
        let given_value = FhirPathValue::String("John".into());
        let mut context = EvaluationContext::new(given_value);
        context.root = FhirPathValue::Resource(patient_resource.into());
        
        let args = vec![FhirPathValue::String("http://example.org/original-text".into())];
        let result = extension_func.evaluate_sync(&args, &context).unwrap();
        
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let Some(FhirPathValue::Resource(ext)) = items.get(0) {
                    assert_eq!(
                        ext.get_property("url").and_then(|v| v.as_str()),
                        Some("http://example.org/original-text")
                    );
                    assert_eq!(
                        ext.get_property("valueString").and_then(|v| v.as_str()),
                        Some("Jonathan")
                    );
                } else {
                    panic!("Expected Resource result");
                }
            },
            _ => panic!("Expected Collection result"),
        }
    }
    
    #[tokio::test]
    async fn test_extension_invalid_arguments() {
        let extension_func = UnifiedExtensionFunction::new();
        
        let context = EvaluationContext::new(FhirPathValue::Empty);
        
        // Test with no arguments
        let result = extension_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        
        // Test with too many arguments
        let args = vec![
            FhirPathValue::String("url1".into()),
            FhirPathValue::String("url2".into())
        ];
        let result = extension_func.evaluate_sync(&args, &context);
        assert!(result.is_err());
        
        // Test with invalid argument type
        let args = vec![FhirPathValue::Integer(42)];
        let result = extension_func.evaluate_sync(&args, &context).unwrap();
        
        // Should return empty collection for invalid URL
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 0);
            },
            _ => panic!("Expected empty Collection result"),
        }
    }
}