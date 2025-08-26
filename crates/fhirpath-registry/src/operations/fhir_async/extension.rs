//! Extension function - async implementation for FunctionRegistry

use crate::signature::{FunctionSignature, ParameterType, ValueType};
use crate::traits::{AsyncOperation, EvaluationContext};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use sonic_rs::{JsonContainerTrait, JsonValueTrait};

/// Extension function - finds extensions by URL
#[derive(Debug, Default, Clone)]
pub struct ExtensionFunction;

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
                            if let Some(url_str) = ext_url.as_str() {
                                if url_str == url {
                                    matching_extensions.push(FhirPathValue::from(ext.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check underscore elements for primitive type extensions
        // For example, "_active": {"extension": [...]} for boolean extensions
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key.starts_with('_') {
                    if let Some(underscore_extensions) = value.get("extension") {
                        if let Some(ext_array) = underscore_extensions.as_array() {
                            for ext in ext_array {
                                if let Some(ext_obj) = ext.as_object() {
                                    if let Some(ext_url) = ext_obj.get(&"url") {
                                        if let Some(url_str) = ext_url.as_str() {
                                            if url_str == url {
                                                matching_extensions
                                                    .push(FhirPathValue::from(ext.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Recursively search nested objects and arrays
        if let Some(obj) = json.as_object() {
            for (_, value) in obj {
                let nested_result = self.find_extensions_in_json(value, url)?;
                if let FhirPathValue::Collection(ref col) = nested_result {
                    matching_extensions.extend(col.iter().cloned());
                }
            }
        } else if let Some(arr) = json.as_array() {
            for item in arr {
                let nested_result = self.find_extensions_in_json(item, url)?;
                if let FhirPathValue::Collection(ref col) = nested_result {
                    matching_extensions.extend(col.iter().cloned());
                }
            }
        }

        Ok(FhirPathValue::Collection(
            octofhir_fhirpath_model::Collection::from(matching_extensions),
        ))
    }
}

#[async_trait]
impl AsyncOperation for ExtensionFunction {
    fn name(&self) -> &'static str {
        "extension"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "extension",
                parameters: vec![ParameterType::String],
                return_type: ValueType::Collection,
                variadic: false,
            });
        &SIGNATURE
    }

    async fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "extension".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get URL argument
        let url = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "extension() url argument must be a string".to_string(),
                });
            }
        };

        // Search for extensions in the current context
        match &context.input {
            FhirPathValue::JsonValue(json_val) => {
                self.find_extensions_in_json(json_val.as_inner(), url)
            }
            FhirPathValue::Resource(resource) => {
                // Convert resource to JSON for extension search
                let json = resource.as_json();
                self.find_extensions_in_json(&json, url)
            }
            FhirPathValue::Collection(col) => {
                let mut all_extensions = Vec::new();

                for item in col.iter() {
                    let extensions_result = match item {
                        FhirPathValue::JsonValue(json_val) => {
                            self.find_extensions_in_json(json_val.as_inner(), url)?
                        }
                        FhirPathValue::Resource(resource) => {
                            let json = resource.as_json();
                            self.find_extensions_in_json(&json, url)?
                        }
                        _ => FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(
                            vec![],
                        )),
                    };

                    if let FhirPathValue::Collection(ref ext_col) = extensions_result {
                        all_extensions.extend(ext_col.iter().cloned());
                    }
                }

                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(all_extensions),
                ))
            }
            _ => {
                // Non-object types don't have extensions
                Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ))
            }
        }
    }
}
