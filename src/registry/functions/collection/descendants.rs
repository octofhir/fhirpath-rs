//! descendants() function implementation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::FunctionSignature;

/// descendants() function - returns all descendants of nodes in the collection
pub struct DescendantsFunction;

impl FhirPathFunction for DescendantsFunction {
    fn name(&self) -> &str {
        "descendants"
    }
    fn human_friendly_name(&self) -> &str {
        "Descendants"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "descendants",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn documentation(&self) -> &str {
        "Returns a collection with all descendant nodes of all items in the input collection, in document order and without duplicates. Descendant nodes include the children, grandchildren, and all subsequent generations of child nodes."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = context.input.clone().to_collection();
        let mut result = Vec::new();

        fn collect_descendants(value: &FhirPathValue, result: &mut Vec<FhirPathValue>) {
            match value {
                FhirPathValue::Resource(resource) => {
                    // Collect all nested values from the resource
                    for (_key, field_value) in resource.properties() {
                        // Convert JSON Value to FhirPathValue
                        let fhir_path_value = value_to_fhir_path_value(field_value);

                        // Add the field value itself
                        match &fhir_path_value {
                            FhirPathValue::Empty => {} // Skip empty values
                            _ => {
                                result.push(fhir_path_value.clone());
                                // Recursively collect descendants
                                collect_descendants(&fhir_path_value, result);
                            }
                        }
                    }
                }
                FhirPathValue::Collection(items) => {
                    for item in items.iter() {
                        result.push(item.clone());
                        collect_descendants(item, result);
                    }
                }
                _ => {} // Primitives have no descendants
            }
        }

        fn value_to_fhir_path_value(value: &serde_json::Value) -> FhirPathValue {
            use crate::model::FhirResource;

            match value {
                serde_json::Value::Array(arr) => {
                    let mut collection = Vec::new();
                    for item in arr {
                        collection.push(value_to_fhir_path_value(item));
                    }
                    FhirPathValue::collection(collection)
                }
                serde_json::Value::Object(_) => {
                    let resource = FhirResource::from_json(value.clone());
                    FhirPathValue::Resource(resource)
                }
                serde_json::Value::String(s) => FhirPathValue::String(s.clone()),
                serde_json::Value::Number(n) => {
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
                serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
                serde_json::Value::Null => FhirPathValue::Empty,
            }
        }

        for item in items.iter() {
            collect_descendants(item, &mut result);
        }

        Ok(FhirPathValue::collection(result))
    }
}
