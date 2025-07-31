//! extension() function - retrieves extensions with a given URL from an element

use crate::model::{FhirPathValue, FhirResource, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use serde_json::Value;

/// extension(url) function - retrieves extensions with a given URL from an element
pub struct ExtensionFunction;

impl FhirPathFunction for ExtensionFunction {
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

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Get the URL parameter
        let url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "URL parameter must be a string".to_string(),
                });
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
            FhirPathValue::String(_) | FhirPathValue::Date(_) | FhirPathValue::DateTime(_) => {
                // For primitive values, we need to look at the root context to find
                // the corresponding _field extension. This requires context awareness
                // that's not yet implemented in the current architecture.
                //
                // TODO: Implement proper context-aware extension lookup for primitives
                // This would require knowing the field path and parent resource
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
            FhirPathValue::Resource(resource)
        }
        Value::String(s) => FhirPathValue::String(s.clone()),
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
