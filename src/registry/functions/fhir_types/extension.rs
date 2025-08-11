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

//! extension() function - retrieves extensions with a given URL from an element

use crate::model::{FhirPathValue, FhirResource, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use serde_json::Value;

/// extension(url) function - retrieves extensions with a given URL from an element
pub struct ExtensionFunction;

#[async_trait]
impl AsyncFhirPathFunction for ExtensionFunction {
    fn name(&self) -> &str {
        "extension"
    }

    fn human_friendly_name(&self) -> &str {
        "Extension"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "extension",
                vec![ParameterInfo::required("url", TypeInfo::String)],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Get the URL parameter
        let url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => {
                // Per user request: return false instead of error for invalid URL parameter
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
                    // Try to find the field name by looking for primitive extensions
                    // We need to check all possible _fieldName patterns in the root resource
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
    let matches = match (json_value, fhir_value) {
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
    };

    // Debug output
    eprintln!("Comparing JSON {json_value:?} with FhirPath {fhir_value:?} -> {matches}");
    matches
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
